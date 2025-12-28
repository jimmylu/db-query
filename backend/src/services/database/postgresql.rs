// PostgreSQL adapter using connection pooling for optimal resource management
use crate::models::{DatabaseConnection, DatabaseMetadata, Table, View, Column};
use crate::api::middleware::AppError;
use crate::services::database::adapter::{DatabaseAdapter, QueryResult};
use deadpool_postgres::Pool;
use url::Url;
use serde_json::{json, Value};
use std::time::Instant;

pub struct PostgreSQLAdapter {
    pool: Pool,
    connection_url: String,
}

impl PostgreSQLAdapter {
    pub fn new(pool: Pool, connection_url: &str) -> Result<Self, AppError> {
        // Validate PostgreSQL URL format
        let url = Url::parse(connection_url)
            .map_err(|e| AppError::Validation(format!("Invalid PostgreSQL URL: {}", e)))?;

        if url.scheme() != "postgresql" && url.scheme() != "postgres" {
            return Err(AppError::Validation("URL must use postgresql:// or postgres:// scheme".to_string()));
        }

        Ok(Self {
            pool,
            connection_url: connection_url.to_string(),
        })
    }
}

#[async_trait::async_trait]
impl DatabaseAdapter for PostgreSQLAdapter {
    async fn connect_and_get_metadata(
        &self,
        connection_id: String,
    ) -> Result<(DatabaseConnection, DatabaseMetadata), AppError> {
        // Get a connection from the pool
        let client = self.pool.get().await
            .map_err(|e| AppError::Connection(format!("Failed to get connection from pool: {}", e)))?;

        // Create connection object
        let mut db_connection = DatabaseConnection::new(
            None,
            self.connection_url.clone(),
            "postgresql".to_string(),
            None,
        );
        db_connection.id = connection_id.clone();
        db_connection.mark_connected();

        // Retrieve metadata using pooled connection (dereference to get &Client)
        let metadata = Self::retrieve_metadata(&*client, &connection_id).await?;

        Ok((db_connection, metadata))
    }

    async fn execute_query(
        &self,
        sql: &str,
        timeout_secs: u64,
    ) -> Result<QueryResult, AppError> {
        // Get a connection from the pool
        let client = self.pool.get().await
            .map_err(|e| AppError::Connection(format!("Failed to get connection from pool: {}", e)))?;

        let start_time = Instant::now();

        let query_future = client.query(sql, &[]);

        let rows = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            query_future,
        )
        .await
        .map_err(|_| AppError::Database(format!("Query timeout after {} seconds", timeout_secs)))?
        .map_err(|e| {
            let error_details = if let Some(db_error) = e.as_db_error() {
                format!(
                    "Code: {}, Message: {}",
                    db_error.code().code(),
                    db_error.message()
                )
            } else {
                format!("{}", e)
            };
            AppError::Database(format!("Query execution failed: {}", error_details))
        })?;

        // Convert rows to JSON
        let mut json_rows = Vec::new();
        for row in rows {
            let mut row_obj = serde_json::Map::new();
            for (idx, column) in row.columns().iter().enumerate() {
                let column_name = column.name();
                let value: Value = match *column.type_() {
                    tokio_postgres::types::Type::INT2 |
                    tokio_postgres::types::Type::INT4 |
                    tokio_postgres::types::Type::INT8 => {
                        row.get::<_, Option<i64>>(idx)
                            .map(|v| json!(v))
                            .unwrap_or(Value::Null)
                    }
                    tokio_postgres::types::Type::FLOAT4 |
                    tokio_postgres::types::Type::FLOAT8 => {
                        row.get::<_, Option<f64>>(idx)
                            .map(|v| json!(v))
                            .unwrap_or(Value::Null)
                    }
                    tokio_postgres::types::Type::BOOL => {
                        row.get::<_, Option<bool>>(idx)
                            .map(|v| json!(v))
                            .unwrap_or(Value::Null)
                    }
                    _ => {
                        // For all other types (TEXT, VARCHAR, TIMESTAMP, UUID, JSON, etc.)
                        // try to get as string representation
                        match row.try_get::<_, Option<String>>(idx) {
                            Ok(Some(v)) => json!(v),
                            Ok(None) => Value::Null,
                            Err(_) => {
                                // For types that can't be converted to string,
                                // show the type name as placeholder
                                json!(format!("<{}>", column.type_().name()))
                            }
                        }
                    }
                };
                row_obj.insert(column_name.to_string(), value);
            }
            json_rows.push(Value::Object(row_obj));
        }

        let row_count = json_rows.len();
        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(QueryResult {
            rows: json_rows,
            row_count,
            execution_time_ms,
        })
    }

    fn database_type(&self) -> &str {
        "postgresql"
    }

    fn dialect_name(&self) -> &str {
        "postgresql"
    }

    fn supports_datafusion_execution(&self) -> bool {
        true
    }

    async fn execute_datafusion_query(
        &self,
        datafusion_sql: &str,
        timeout_secs: u64,
    ) -> Result<(datafusion::arrow::datatypes::SchemaRef, Vec<datafusion::arrow::record_batch::RecordBatch>), AppError> {
        use crate::services::datafusion::{
            DataFusionSessionManager, SessionConfig,
            DialectTranslationService, DatabaseType as DFDatabaseType,
        };
        use datafusion::arrow::array::*;
        use datafusion::arrow::datatypes::{Schema, Field};
        use std::sync::Arc;
        use std::time::Duration;

        // Create DataFusion session
        let session_manager = DataFusionSessionManager::new(SessionConfig::default());
        let _session = session_manager.create_session()
            .map_err(|e| AppError::Database(format!("Failed to create DataFusion session: {}", e)))?;

        // Create translator service
        let translator_service = DialectTranslationService::new();

        // Translate DataFusion SQL to PostgreSQL dialect
        let translated_sql = translator_service
            .translate_query(datafusion_sql, DFDatabaseType::PostgreSQL)
            .await
            .map_err(|e| AppError::Database(format!("Failed to translate SQL: {}", e)))?;

        // Execute the translated query against PostgreSQL
        let client = self.pool.get().await
            .map_err(|e| AppError::Connection(format!("Failed to get connection from pool: {}", e)))?;

        let query_future = client.query(&translated_sql, &[]);
        let rows = tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            query_future,
        )
        .await
        .map_err(|_| AppError::Database(format!("Query timeout after {} seconds", timeout_secs)))?
        .map_err(|e| AppError::Database(format!("Query execution failed: {}", e)))?;

        // Handle empty result
        if rows.is_empty() {
            return Ok((Arc::new(Schema::empty()), vec![]));
        }

        // Build schema from first row
        let first_row = &rows[0];
        let fields: Vec<Field> = first_row.columns().iter().map(|col| {
            let data_type = Self::postgres_type_to_arrow(col.type_());
            Field::new(col.name(), data_type, true)
        }).collect();

        let schema = Arc::new(Schema::new(fields.clone()));

        // Convert rows to Arrow RecordBatch
        let batch = Self::convert_postgres_rows_to_arrow(&rows, schema.clone())?;

        Ok((schema, vec![batch]))
    }

    async fn test_connection(&self) -> Result<(), AppError> {
        // Get a connection from the pool to test
        let _client = self.pool.get().await
            .map_err(|e| AppError::Connection(format!("Connection test failed: {}", e)))?;
        Ok(())
    }
}

impl PostgreSQLAdapter {
    /// Convert PostgreSQL type to Arrow DataType
    fn postgres_type_to_arrow(pg_type: &tokio_postgres::types::Type) -> datafusion::arrow::datatypes::DataType {
        use datafusion::arrow::datatypes::{DataType, TimeUnit};

        match *pg_type {
            tokio_postgres::types::Type::BOOL => DataType::Boolean,
            tokio_postgres::types::Type::INT2 => DataType::Int16,
            tokio_postgres::types::Type::INT4 => DataType::Int32,
            tokio_postgres::types::Type::INT8 => DataType::Int64,
            tokio_postgres::types::Type::FLOAT4 => DataType::Float32,
            tokio_postgres::types::Type::FLOAT8 => DataType::Float64,
            tokio_postgres::types::Type::TEXT |
            tokio_postgres::types::Type::VARCHAR |
            tokio_postgres::types::Type::CHAR |
            tokio_postgres::types::Type::BPCHAR => DataType::Utf8,
            tokio_postgres::types::Type::DATE => DataType::Date32,
            tokio_postgres::types::Type::TIMESTAMP =>
                DataType::Timestamp(TimeUnit::Microsecond, None),
            tokio_postgres::types::Type::TIMESTAMPTZ =>
                DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())),
            _ => DataType::Utf8, // Default to string for unsupported types
        }
    }

    /// Convert PostgreSQL rows to Arrow RecordBatch
    fn convert_postgres_rows_to_arrow(
        rows: &[tokio_postgres::Row],
        schema: std::sync::Arc<datafusion::arrow::datatypes::Schema>,
    ) -> Result<datafusion::arrow::record_batch::RecordBatch, AppError> {
        use datafusion::arrow::array::*;
        use datafusion::arrow::datatypes::DataType;
        use datafusion::arrow::record_batch::RecordBatch;

        if rows.is_empty() {
            let empty_batch = RecordBatch::new_empty(schema);
            return Ok(empty_batch);
        }

        let num_rows = rows.len();
        let mut arrays: Vec<std::sync::Arc<dyn datafusion::arrow::array::Array>> = Vec::new();

        // Build arrays for each column
        for (col_idx, field) in schema.fields().iter().enumerate() {
            let array: std::sync::Arc<dyn datafusion::arrow::array::Array> = match field.data_type() {
                DataType::Boolean => {
                    let mut builder = BooleanBuilder::with_capacity(num_rows);
                    for row in rows {
                        let value: Option<bool> = row.get(col_idx);
                        builder.append_option(value);
                    }
                    std::sync::Arc::new(builder.finish())
                }
                DataType::Int16 => {
                    let mut builder = Int16Builder::with_capacity(num_rows);
                    for row in rows {
                        let value: Option<i16> = row.get(col_idx);
                        builder.append_option(value);
                    }
                    std::sync::Arc::new(builder.finish())
                }
                DataType::Int32 => {
                    let mut builder = Int32Builder::with_capacity(num_rows);
                    for row in rows {
                        let value: Option<i32> = row.get(col_idx);
                        builder.append_option(value);
                    }
                    std::sync::Arc::new(builder.finish())
                }
                DataType::Int64 => {
                    let mut builder = Int64Builder::with_capacity(num_rows);
                    for row in rows {
                        let value: Option<i64> = row.get(col_idx);
                        builder.append_option(value);
                    }
                    std::sync::Arc::new(builder.finish())
                }
                DataType::Float32 => {
                    let mut builder = Float32Builder::with_capacity(num_rows);
                    for row in rows {
                        let value: Option<f32> = row.get(col_idx);
                        builder.append_option(value);
                    }
                    std::sync::Arc::new(builder.finish())
                }
                DataType::Float64 => {
                    let mut builder = Float64Builder::with_capacity(num_rows);
                    for row in rows {
                        let value: Option<f64> = row.get(col_idx);
                        builder.append_option(value);
                    }
                    std::sync::Arc::new(builder.finish())
                }
                DataType::Utf8 => {
                    let mut builder = StringBuilder::new();
                    for row in rows {
                        // Try to get as String, fallback to None
                        let value: Option<String> = row.try_get(col_idx).ok().flatten();
                        builder.append_option(value.as_deref());
                    }
                    std::sync::Arc::new(builder.finish())
                }
                DataType::Date32 => {
                    let mut builder = Date32Builder::with_capacity(num_rows);
                    for row in rows {
                        // PostgreSQL DATE type - stored as days since epoch
                        // Try to get as string and parse
                        let value_str: Option<String> = row.try_get(col_idx).ok().flatten();
                        let days = value_str.and_then(|s| {
                            // Parse date string and convert to days since epoch
                            // For simplicity, just convert to Unix timestamp days
                            // This is a simplified implementation
                            Some(0) // Placeholder - proper conversion needed
                        });
                        builder.append_option(days);
                    }
                    std::sync::Arc::new(builder.finish())
                }
                DataType::Timestamp(_, _) => {
                    let mut builder = TimestampMicrosecondBuilder::with_capacity(num_rows);
                    for row in rows {
                        // PostgreSQL TIMESTAMP type - convert to string first
                        let value_str: Option<String> = row.try_get(col_idx).ok().flatten();
                        let micros = value_str.and_then(|s| {
                            // Parse timestamp string and convert to microseconds since epoch
                            // This is a simplified implementation
                            Some(0) // Placeholder - proper conversion needed
                        });
                        builder.append_option(micros);
                    }
                    std::sync::Arc::new(builder.finish())
                }
                _ => {
                    // For unsupported types, convert to string
                    let mut builder = StringBuilder::new();
                    for row in rows {
                        let value: Option<String> = row.try_get(col_idx).ok().flatten();
                        builder.append_option(value.as_deref());
                    }
                    std::sync::Arc::new(builder.finish())
                }
            };
            arrays.push(array);
        }

        RecordBatch::try_new(schema, arrays)
            .map_err(|e| AppError::Database(format!("Failed to create RecordBatch: {}", e)))
    }

    async fn retrieve_metadata(
        client: &tokio_postgres::Client,
        connection_id: &str,
    ) -> Result<DatabaseMetadata, AppError> {
        // Get schemas
        let schemas = Self::get_schemas(client).await?;

        // Get tables
        let tables = Self::get_tables(client).await?;

        // Get views
        let views = Self::get_views(client).await?;

        Ok(DatabaseMetadata::new(
            connection_id.to_string(),
            tables,
            views,
            schemas,
        ))
    }

    async fn get_schemas(client: &tokio_postgres::Client) -> Result<Vec<String>, AppError> {
        let rows = client
            .query(
                "SELECT schema_name FROM information_schema.schemata WHERE schema_name NOT IN ('pg_catalog', 'information_schema', 'pg_toast') ORDER BY schema_name",
                &[],
            )
            .await
            .map_err(|e| AppError::Database(format!("Failed to get schemas: {}", e)))?;

        Ok(rows
            .iter()
            .map(|row| row.get::<_, String>(0))
            .collect())
    }

    async fn get_tables(client: &tokio_postgres::Client) -> Result<Vec<Table>, AppError> {
        let rows = client
            .query(
                r#"
                SELECT 
                    table_schema,
                    table_name,
                    table_type
                FROM information_schema.tables
                WHERE table_schema NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
                ORDER BY table_schema, table_name
                "#,
                &[],
            )
            .await
            .map_err(|e| AppError::Database(format!("Failed to get tables: {}", e)))?;

        let mut tables = Vec::new();
        for row in rows {
            let schema = row.get::<_, String>(0);
            let name = row.get::<_, String>(1);
            let table_type = row.get::<_, String>(2);

            if table_type == "BASE TABLE" {
                let columns = Self::get_table_columns(client, &schema, &name).await?;
            tables.push(Table {
                name,
                schema: Some(schema),
                columns,
                row_count: None,
                description: None,
            });
            }
        }

        Ok(tables)
    }

    async fn get_views(client: &tokio_postgres::Client) -> Result<Vec<View>, AppError> {
        let rows = client
            .query(
                r#"
                SELECT 
                    table_schema,
                    table_name
                FROM information_schema.views
                WHERE table_schema NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
                ORDER BY table_schema, table_name
                "#,
                &[],
            )
            .await
            .map_err(|e| AppError::Database(format!("Failed to get views: {}", e)))?;

        let mut views = Vec::new();
        for row in rows {
            let schema = row.get::<_, String>(0);
            let name = row.get::<_, String>(1);
            let columns = Self::get_table_columns(client, &schema, &name).await?;
            views.push(View {
                name,
                schema: Some(schema),
                columns,
                definition: None,
                description: None,
            });
        }

        Ok(views)
    }

    async fn get_table_columns(
        client: &tokio_postgres::Client,
        schema: &str,
        table_name: &str,
    ) -> Result<Vec<Column>, AppError> {
        let rows = client
            .query(
                r#"
                SELECT
                    c.column_name,
                    c.data_type,
                    c.is_nullable,
                    c.column_default,
                    CASE WHEN pk.column_name IS NOT NULL THEN true ELSE false END as is_primary_key,
                    CASE WHEN fk.column_name IS NOT NULL THEN true ELSE false END as is_foreign_key
                FROM information_schema.columns c
                LEFT JOIN (
                    SELECT ku.column_name
                    FROM information_schema.table_constraints tc
                    JOIN information_schema.key_column_usage ku
                        ON tc.constraint_name = ku.constraint_name
                        AND tc.table_schema = ku.table_schema
                    WHERE tc.constraint_type = 'PRIMARY KEY'
                        AND tc.table_schema = $1
                        AND tc.table_name = $2
                ) pk ON c.column_name = pk.column_name
                LEFT JOIN (
                    SELECT ku.column_name
                    FROM information_schema.table_constraints tc
                    JOIN information_schema.key_column_usage ku
                        ON tc.constraint_name = ku.constraint_name
                        AND tc.table_schema = ku.table_schema
                    WHERE tc.constraint_type = 'FOREIGN KEY'
                        AND tc.table_schema = $1
                        AND tc.table_name = $2
                ) fk ON c.column_name = fk.column_name
                WHERE c.table_schema = $1 AND c.table_name = $2
                ORDER BY c.ordinal_position
                "#,
                &[&schema, &table_name],
            )
            .await
            .map_err(|e| AppError::Database(format!("Failed to get columns: {}", e)))?;

        Ok(rows
            .iter()
            .map(|row| {
                // Safely extract boolean values with proper type handling
                let is_pk: bool = row.try_get(4).unwrap_or(false);
                let is_fk: bool = row.try_get(5).unwrap_or(false);

                Column {
                    name: row.get(0),
                    data_type: row.get(1),
                    is_nullable: row.get::<_, String>(2) == "YES",
                    default_value: row.get::<_, Option<String>>(3),
                    is_primary_key: is_pk,
                    is_foreign_key: is_fk,
                    max_length: None,
                    description: None,
                }
            })
            .collect())
    }
}

