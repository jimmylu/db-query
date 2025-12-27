// Apache Doris adapter using MySQL protocol compatibility
// Doris is a high-performance analytical database that uses MySQL wire protocol
use crate::models::{DatabaseConnection, DatabaseMetadata, Table, View, Column};
use crate::api::middleware::AppError;
use crate::services::database::adapter::{DatabaseAdapter, QueryResult};
use mysql_async::{Pool, OptsBuilder, Conn, Row, Value as MySqlValue, prelude::*};
use url::Url;
use serde_json::{json, Value};
use std::time::Instant;

pub struct DorisAdapter {
    pool: Pool,
    connection_url: String,
}

impl DorisAdapter {
    pub fn new(connection_url: &str) -> Result<Self, AppError> {
        // Validate Doris URL format
        // Doris uses MySQL protocol, so we accept both doris:// and mysql:// schemes
        let url = Url::parse(connection_url)
            .map_err(|e| AppError::Validation(format!("Invalid Doris URL: {}", e)))?;

        // Convert doris:// scheme to mysql:// for connection
        let mysql_url = if url.scheme() == "doris" {
            connection_url.replace("doris://", "mysql://")
        } else if url.scheme() == "mysql" {
            connection_url.to_string()
        } else {
            return Err(AppError::Validation(
                "URL must use doris:// or mysql:// scheme for Doris".to_string()
            ));
        };

        // Build MySQL connection options from URL
        let opts = OptsBuilder::from_opts(mysql_url.as_str());
        let pool = Pool::new(opts);

        Ok(Self {
            pool,
            connection_url: connection_url.to_string(),
        })
    }

    /// Get a connection from the pool
    async fn get_conn(&self) -> Result<Conn, AppError> {
        self.pool
            .get_conn()
            .await
            .map_err(|e| AppError::Connection(format!("Failed to get Doris connection from pool: {}", e)))
    }
}

#[async_trait::async_trait]
impl DatabaseAdapter for DorisAdapter {
    async fn connect_and_get_metadata(
        &self,
        connection_id: String,
    ) -> Result<(DatabaseConnection, DatabaseMetadata), AppError> {
        // Get a connection from the pool to test connectivity
        let mut conn = self.get_conn().await?;

        // Create connection object
        let mut db_connection = DatabaseConnection::new(
            None,
            self.connection_url.clone(),
            "doris".to_string(),
        );
        db_connection.id = connection_id.clone();
        db_connection.mark_connected();

        // Retrieve metadata using pooled connection
        let metadata = Self::retrieve_metadata(&mut conn, &connection_id).await?;

        Ok((db_connection, metadata))
    }

    async fn execute_query(
        &self,
        sql: &str,
        timeout_secs: u64,
    ) -> Result<QueryResult, AppError> {
        // Get a connection from the pool
        let mut conn = self.get_conn().await?;

        let start_time = Instant::now();

        // Execute query with timeout
        let rows: Vec<Row> = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            conn.query(sql),
        )
        .await
        .map_err(|_| AppError::Database(format!("Query timeout after {} seconds", timeout_secs)))?
        .map_err(|e| AppError::Database(format!("Query execution failed: {}", e)))?;

        // Convert rows to JSON
        let mut json_rows = Vec::new();
        for row in rows {
            let mut row_obj = serde_json::Map::new();
            let columns = row.columns_ref();

            for (idx, column) in columns.iter().enumerate() {
                let column_name = column.name_str();
                let value: Value = match row.get_opt::<MySqlValue, usize>(idx) {
                    Some(Ok(mysql_val)) => Self::mysql_value_to_json(mysql_val),
                    Some(Err(_)) => Value::Null,
                    None => Value::Null,
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
        "doris"
    }

    fn dialect_name(&self) -> &str {
        "mysql" // Doris uses MySQL-compatible dialect
    }

    fn supports_datafusion_execution(&self) -> bool {
        true // Doris supports DataFusion execution through MySQL protocol
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
        use std::time::Duration;

        // Create DataFusion session
        let session_manager = DataFusionSessionManager::new(SessionConfig::default());
        let _session = session_manager.create_session()
            .map_err(|e| AppError::Database(format!("Failed to create DataFusion session: {}", e)))?;

        // Create translator service
        let translator_service = DialectTranslationService::new();

        // Translate DataFusion SQL to MySQL/Doris dialect
        let translated_sql = translator_service
            .translate_query(datafusion_sql, DFDatabaseType::MySQL)
            .await
            .map_err(|e| AppError::Database(format!("Failed to translate SQL: {}", e)))?;

        // Execute the translated query against Doris
        let mut conn = self.get_conn().await?;

        let rows: Vec<Row> = tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            conn.query(&translated_sql),
        )
        .await
        .map_err(|_| AppError::Database(format!("Query timeout after {} seconds", timeout_secs)))?
        .map_err(|e| AppError::Database(format!("Query execution failed: {}", e)))?;

        // For now, return a not implemented error as full Arrow conversion needs more work
        // This can be implemented similar to PostgreSQL adapter's Arrow conversion
        Err(AppError::NotImplemented(
            "Full Doris to Arrow conversion not yet implemented. Use execute_query instead.".to_string()
        ))
    }

    async fn test_connection(&self) -> Result<(), AppError> {
        // Test connection by executing a simple query
        let mut conn = self.get_conn().await?;
        let _: Vec<Row> = conn.query("SELECT 1")
            .await
            .map_err(|e| AppError::Connection(format!("Connection test failed: {}", e)))?;
        Ok(())
    }
}

impl DorisAdapter {
    /// Retrieve database metadata (tables, views, schemas)
    async fn retrieve_metadata(
        conn: &mut Conn,
        connection_id: &str,
    ) -> Result<DatabaseMetadata, AppError> {
        // Get schemas
        let schemas = Self::get_schemas(conn).await?;

        // Get tables
        let tables = Self::get_tables(conn).await?;

        // Get views
        let views = Self::get_views(conn).await?;

        Ok(DatabaseMetadata::new(
            connection_id.to_string(),
            tables,
            views,
            schemas,
        ))
    }

    /// Get list of schemas/databases (excluding system databases)
    async fn get_schemas(conn: &mut Conn) -> Result<Vec<String>, AppError> {
        let query = r#"
            SELECT SCHEMA_NAME
            FROM INFORMATION_SCHEMA.SCHEMATA
            WHERE SCHEMA_NAME NOT IN ('information_schema', '__internal_schema', '_statistics_')
            ORDER BY SCHEMA_NAME
        "#;

        let rows: Vec<Row> = conn.query(query)
            .await
            .map_err(|e| AppError::Database(format!("Failed to get schemas: {}", e)))?;

        Ok(rows
            .iter()
            .filter_map(|row| row.get::<String, usize>(0))
            .collect())
    }

    /// Get list of tables with their metadata
    async fn get_tables(conn: &mut Conn) -> Result<Vec<Table>, AppError> {
        let query = r#"
            SELECT
                TABLE_SCHEMA,
                TABLE_NAME,
                TABLE_TYPE
            FROM INFORMATION_SCHEMA.TABLES
            WHERE TABLE_SCHEMA NOT IN ('information_schema', '__internal_schema', '_statistics_')
                AND TABLE_TYPE = 'BASE TABLE'
            ORDER BY TABLE_SCHEMA, TABLE_NAME
        "#;

        let rows: Vec<Row> = conn.query(query)
            .await
            .map_err(|e| AppError::Database(format!("Failed to get tables: {}", e)))?;

        let mut tables = Vec::new();
        for row in rows {
            let schema: String = row.get(0).unwrap_or_default();
            let name: String = row.get(1).unwrap_or_default();

            let columns = Self::get_table_columns(conn, &schema, &name).await?;
            tables.push(Table {
                name,
                schema: Some(schema),
                columns,
                row_count: None,
                description: None,
            });
        }

        Ok(tables)
    }

    /// Get list of views with their metadata
    async fn get_views(conn: &mut Conn) -> Result<Vec<View>, AppError> {
        let query = r#"
            SELECT
                TABLE_SCHEMA,
                TABLE_NAME
            FROM INFORMATION_SCHEMA.VIEWS
            WHERE TABLE_SCHEMA NOT IN ('information_schema', '__internal_schema', '_statistics_')
            ORDER BY TABLE_SCHEMA, TABLE_NAME
        "#;

        let rows: Vec<Row> = conn.query(query)
            .await
            .map_err(|e| AppError::Database(format!("Failed to get views: {}", e)))?;

        let mut views = Vec::new();
        for row in rows {
            let schema: String = row.get(0).unwrap_or_default();
            let name: String = row.get(1).unwrap_or_default();

            let columns = Self::get_table_columns(conn, &schema, &name).await?;
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

    /// Get columns for a specific table or view
    async fn get_table_columns(
        conn: &mut Conn,
        schema: &str,
        table_name: &str,
    ) -> Result<Vec<Column>, AppError> {
        let query = r#"
            SELECT
                c.COLUMN_NAME,
                c.DATA_TYPE,
                c.IS_NULLABLE,
                c.COLUMN_DEFAULT,
                c.COLUMN_KEY,
                c.CHARACTER_MAXIMUM_LENGTH
            FROM INFORMATION_SCHEMA.COLUMNS c
            WHERE c.TABLE_SCHEMA = ? AND c.TABLE_NAME = ?
            ORDER BY c.ORDINAL_POSITION
        "#;

        let rows: Vec<Row> = conn.exec(query, (schema, table_name))
            .await
            .map_err(|e| AppError::Database(format!("Failed to get columns: {}", e)))?;

        Ok(rows
            .iter()
            .map(|row| {
                let column_key: String = row.get(4).unwrap_or_default();

                Column {
                    name: row.get(0).unwrap_or_default(),
                    data_type: row.get(1).unwrap_or_default(),
                    is_nullable: row.get::<String, usize>(2).unwrap_or_default() == "YES",
                    default_value: row.get(3),
                    is_primary_key: column_key == "PRI",
                    is_foreign_key: column_key == "MUL" || column_key == "FOR",
                    max_length: row.get::<Option<u64>, usize>(5)
                        .and_then(|opt_val| opt_val.and_then(|v| i32::try_from(v).ok())),
                    description: None,
                }
            })
            .collect())
    }

    /// Convert MySQL value to JSON
    fn mysql_value_to_json(value: MySqlValue) -> Value {
        match value {
            MySqlValue::NULL => Value::Null,
            MySqlValue::Bytes(bytes) => {
                // Try to convert to string
                String::from_utf8(bytes)
                    .map(|s| json!(s))
                    .unwrap_or(Value::Null)
            }
            MySqlValue::Int(i) => json!(i),
            MySqlValue::UInt(u) => json!(u),
            MySqlValue::Float(f) => json!(f),
            MySqlValue::Double(d) => json!(d),
            MySqlValue::Date(year, month, day, hour, minute, second, _) => {
                json!(format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                    year, month, day, hour, minute, second
                ))
            }
            MySqlValue::Time(_, _, _, _, _, _) => {
                // Convert to string representation
                json!(format!("{:?}", value))
            }
        }
    }
}
