use crate::models::{Query, UnifiedQueryRequest, UnifiedQueryResponse, DatabaseType};
use crate::api::middleware::AppError;
use crate::validation::SqlValidator;
use crate::services::database::DatabaseAdapter;
use crate::services::datafusion::{
    DialectTranslationService,
    DatabaseType as DFDatabaseType,
};
use tokio_postgres::Client;
use std::time::Instant;

pub struct QueryService {
    dialect_translator: DialectTranslationService,
}

impl QueryService {
    pub fn new() -> Self {
        Self {
            dialect_translator: DialectTranslationService::with_cache(),
        }
    }

    /// Execute a unified SQL query using DataFusion semantic layer
    ///
    /// This method accepts DataFusion SQL syntax and automatically translates
    /// it to the target database's dialect before execution.
    ///
    /// # Arguments
    /// * `request` - Unified query request with DataFusion SQL
    /// * `adapter` - Database adapter for the target database
    ///
    /// # Returns
    /// UnifiedQueryResponse with original query, translated query, and results
    pub async fn execute_unified_query(
        &self,
        request: UnifiedQueryRequest,
        adapter: Box<dyn DatabaseAdapter>,
    ) -> Result<UnifiedQueryResponse, AppError> {
        let start_time = Instant::now();

        // Validate SQL (SELECT-only check)
        SqlValidator::validate_select_only(&request.query)
            .map_err(|e| AppError::InvalidSql(e.to_string()))?;

        // Apply LIMIT if needed
        let datafusion_sql = if request.apply_limit {
            SqlValidator::ensure_limit(&request.query, request.limit_value as u64)
                .map_err(|e| AppError::InvalidSql(e.to_string()))?
        } else {
            request.query.clone()
        };

        // Convert DatabaseType to DFDatabaseType
        let df_db_type = Self::convert_database_type(request.database_type)?;

        // Translate to target dialect
        let translated_sql = self.dialect_translator
            .translate_query(&datafusion_sql, df_db_type)
            .await
            .map_err(|e| AppError::Database(format!("Dialect translation failed: {}", e)))?;

        tracing::info!(
            "Translated query from DataFusion to {}: {} -> {}",
            adapter.dialect_name(),
            datafusion_sql,
            translated_sql
        );

        // Execute the translated query
        let query_result = adapter
            .execute_query(&translated_sql, request.timeout_secs)
            .await?;

        let execution_time_ms = start_time.elapsed().as_millis();

        // Build response
        let response = UnifiedQueryResponse::new(
            datafusion_sql,
            translated_sql,
            request.database_type,
            query_result.rows,
            execution_time_ms,
            request.apply_limit,
        );

        Ok(response)
    }

    /// Convert DatabaseType to DataFusion DatabaseType
    fn convert_database_type(db_type: DatabaseType) -> Result<DFDatabaseType, AppError> {
        match db_type {
            DatabaseType::PostgreSQL => Ok(DFDatabaseType::PostgreSQL),
            DatabaseType::MySQL => Ok(DFDatabaseType::MySQL),
            DatabaseType::Doris => Ok(DFDatabaseType::Doris),
            DatabaseType::Druid => Ok(DFDatabaseType::Druid),
        }
    }

    /// Execute a SQL query using a database adapter (with connection pooling)
    pub async fn execute_query_with_adapter(
        &self,
        mut query: Query,
        adapter: Box<dyn DatabaseAdapter>,
    ) -> Result<Query, AppError> {
        let start_time = Instant::now();
        query.mark_executing();

        // Validate and prepare SQL (SELECT-only check and LIMIT enforcement)
        let (prepared_sql, limit_applied) = SqlValidator::validate_and_prepare(&query.query_text, 1000)
            .map_err(|e| {
                query.mark_failed(e.to_string());
                e
            })?;

        query.limit_applied = limit_applied;

        // Execute query using the adapter (which uses connection pool internally)
        let query_result = adapter.execute_query(&prepared_sql, 30).await
            .map_err(|e| {
                query.mark_failed(e.to_string());
                e
            })?;

        // Convert adapter QueryResult to our Query model
        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        query.mark_completed(query_result.rows, execution_time_ms);

        Ok(query)
    }

    /// Execute a SQL query against a PostgreSQL database (legacy method for backward compatibility)
    /// This method is deprecated - use execute_query_with_adapter instead
    pub async fn execute_query(
        &self,
        mut query: Query,
        client: &Client,
    ) -> Result<Query, AppError> {
        let start_time = Instant::now();
        query.mark_executing();

        // Validate and prepare SQL (SELECT-only check and LIMIT enforcement)
        let (prepared_sql, limit_applied) = SqlValidator::validate_and_prepare(&query.query_text, 1000)
            .map_err(|e| {
                query.mark_failed(e.to_string());
                e
            })?;

        query.limit_applied = limit_applied;

        // Execute query with timeout
        let query_future = client.query(&prepared_sql, &[]);
        
        let rows = tokio::time::timeout(
            std::time::Duration::from_secs(30), // 30 second query timeout
            query_future,
        )
        .await
        .map_err(|_| {
            let error_msg = format!("Query execution timeout: Query did not complete within 30 seconds. The query may be too complex or the database may be slow. SQL: {}", prepared_sql);
            query.mark_failed(error_msg.clone());
            AppError::Database(error_msg)
        })?
        .map_err(|e| {
                // Extract detailed error information from tokio_postgres::Error
                let error_string = format!("{}", e);
                
                // Try to extract more details if it's a database error
                let error_details = if let Some(db_error) = e.as_db_error() {
                    let position_str = db_error.position()
                        .map(|p| format!("{:?}", p))
                        .unwrap_or_else(|| "".to_string());
                    
                    format!(
                        "Code: {}, Message: {}, Detail: {}, Hint: {}, Position: {}",
                        db_error.code().code(),
                        db_error.message(),
                        db_error.detail().unwrap_or(""),
                        db_error.hint().unwrap_or(""),
                        position_str
                    )
                } else {
                    error_string.clone()
                };
                
                // Determine error code based on PostgreSQL error code or message
                let error_code = if error_details.contains("42P01") || (error_details.contains("relation") && error_details.contains("does not exist")) {
                    "TABLE_NOT_FOUND"
                } else if error_details.contains("42703") || (error_details.contains("column") && error_details.contains("does not exist")) {
                    "COLUMN_NOT_FOUND"
                } else if error_details.contains("42601") || error_details.contains("syntax error") {
                    "SQL_SYNTAX_ERROR"
                } else if error_details.contains("42501") || error_details.contains("permission denied") {
                    "PERMISSION_DENIED"
                } else {
                    "QUERY_EXECUTION_ERROR"
                };
                
                let error_msg = format!("{}: {}. SQL: {}", error_code, error_details, prepared_sql);
                tracing::error!("Query execution error: {}", error_msg);
                query.mark_failed(error_msg.clone());
                AppError::Database(error_msg)
            })?;

        // Convert rows to JSON
        let results = Self::rows_to_json(rows)
            .map_err(|e| {
                let error_msg = format!("Failed to convert query results to JSON: {}", e);
                query.mark_failed(error_msg.clone());
                AppError::Database(error_msg)
            })?;

        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        query.mark_completed(results, execution_time_ms);

        Ok(query)
    }

    /// Convert PostgreSQL rows to JSON format
    fn rows_to_json(rows: Vec<tokio_postgres::Row>) -> Result<Vec<serde_json::Value>, AppError> {
        let mut results = Vec::new();

        for row in rows {
            let mut row_obj = serde_json::Map::new();
            
            for (idx, column) in row.columns().iter().enumerate() {
                let column_name = column.name();
                let value: serde_json::Value = match column.type_().name() {
                    "int4" | "int2" => {
                        if let Ok(v) = row.try_get::<_, i32>(idx) {
                            serde_json::Value::Number(serde_json::Number::from(v))
                        } else {
                            serde_json::Value::Null
                        }
                    }
                    "int8" => {
                        if let Ok(v) = row.try_get::<_, i64>(idx) {
                            serde_json::Value::Number(serde_json::Number::from(v))
                        } else {
                            serde_json::Value::Null
                        }
                    }
                    "float4" => {
                        if let Ok(v) = row.try_get::<_, f32>(idx) {
                            serde_json::Value::Number(
                                serde_json::Number::from_f64(v as f64)
                                    .unwrap_or_else(|| serde_json::Number::from(0))
                            )
                        } else {
                            serde_json::Value::Null
                        }
                    }
                    "float8" | "numeric" => {
                        if let Ok(v) = row.try_get::<_, f64>(idx) {
                            serde_json::Value::Number(
                                serde_json::Number::from_f64(v)
                                    .unwrap_or_else(|| serde_json::Number::from(0))
                            )
                        } else {
                            serde_json::Value::Null
                        }
                    }
                    "bool" => {
                        if let Ok(v) = row.try_get::<_, bool>(idx) {
                            serde_json::Value::Bool(v)
                        } else {
                            serde_json::Value::Null
                        }
                    }
                    "text" | "varchar" | "char" | "name" => {
                        if let Ok(v) = row.try_get::<_, String>(idx) {
                            serde_json::Value::String(v)
                        } else {
                            serde_json::Value::Null
                        }
                    }
                    "timestamp" | "timestamptz" | "date" | "time" | "timetz" => {
                        // Try as string (most reliable for timestamps)
                        if let Ok(v) = row.try_get::<_, String>(idx) {
                            serde_json::Value::String(v)
                        } else {
                            serde_json::Value::Null
                        }
                    }
                    "json" | "jsonb" => {
                        // Try as string first, then parse as JSON
                        if let Ok(v) = row.try_get::<_, String>(idx) {
                            serde_json::from_str(&v).unwrap_or(serde_json::Value::String(v))
                        } else {
                            serde_json::Value::Null
                        }
                    }
                    _ => {
                        // Try different types for unknown types
                        let type_name = column.type_().name();
                        
                        // Try as string first (most common fallback)
                        if let Ok(v) = row.try_get::<_, String>(idx) {
                            serde_json::Value::String(v)
                        }
                        // Try as i64
                        else if let Ok(v) = row.try_get::<_, i64>(idx) {
                            serde_json::Value::Number(serde_json::Number::from(v))
                        }
                        // Try as f64
                        else if let Ok(v) = row.try_get::<_, f64>(idx) {
                            serde_json::Value::Number(
                                serde_json::Number::from_f64(v)
                                    .unwrap_or_else(|| serde_json::Number::from(0))
                            )
                        }
                        // Try as bool
                        else if let Ok(v) = row.try_get::<_, bool>(idx) {
                            serde_json::Value::Bool(v)
                        }
                        // Last resort: NULL
                        else {
                            tracing::warn!("Unknown column type {} for column {}, using NULL", type_name, column_name);
                            serde_json::Value::Null
                        }
                    }
                };
                
                row_obj.insert(column_name.to_string(), value);
            }
            
            results.push(serde_json::Value::Object(row_obj));
        }

        Ok(results)
    }
}

