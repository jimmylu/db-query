// MySQL adapter using connection pooling for optimal resource management
use crate::models::{DatabaseConnection, DatabaseMetadata, Table, View, Column};
use crate::api::middleware::AppError;
use crate::services::database::adapter::{DatabaseAdapter, QueryResult};
use mysql_async::{Pool, OptsBuilder, Conn, Row, Value as MySqlValue, prelude::*};
use url::Url;
use serde_json::{json, Value};
use std::time::Instant;

pub struct MySQLAdapter {
    pool: Pool,
    connection_url: String,
}

impl MySQLAdapter {
    pub fn new(connection_url: &str) -> Result<Self, AppError> {
        // Validate MySQL URL format
        let url = Url::parse(connection_url)
            .map_err(|e| AppError::Validation(format!("Invalid MySQL URL: {}", e)))?;

        if url.scheme() != "mysql" && url.scheme() != "mariadb" {
            return Err(AppError::Validation("URL must use mysql:// or mariadb:// scheme".to_string()));
        }

        // Build MySQL connection options from URL
        let opts = OptsBuilder::from_opts(connection_url);
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
            .map_err(|e| AppError::Connection(format!("Failed to get MySQL connection from pool: {}", e)))
    }
}

#[async_trait::async_trait]
impl DatabaseAdapter for MySQLAdapter {
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
            "mysql".to_string(),
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
        "mysql"
    }

    fn dialect_name(&self) -> &str {
        "mysql"
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
            DialectTranslationService, DatabaseType as DFDatabaseType,
        };

        // Create translator service
        let mut translator_service = DialectTranslationService::new();

        // Translate DataFusion SQL to MySQL dialect
        let translated_sql = translator_service
            .translate_query(datafusion_sql, DFDatabaseType::MySQL)
            .await
            .map_err(|e| AppError::Database(format!("Failed to translate SQL: {}", e)))?;

        // Execute the translated query against MySQL
        // For now, return NotImplemented as full Arrow conversion is complex
        // The basic execute_query method should be used instead
        Err(AppError::NotImplemented("Full MySQL to Arrow conversion not yet implemented. Use execute_query instead.".to_string()))
    }

    async fn test_connection(&self) -> Result<(), AppError> {
        // Get a connection from the pool to test
        let _conn = self.get_conn().await?;
        Ok(())
    }
}

impl MySQLAdapter {
    /// Helper function to convert MySQL Value to JSON Value
    fn mysql_value_to_json(mysql_val: MySqlValue) -> Value {
        match mysql_val {
            MySqlValue::NULL => Value::Null,
            MySqlValue::Bytes(bytes) => {
                // Try to convert to UTF-8 string
                match String::from_utf8(bytes) {
                    Ok(s) => json!(s),
                    Err(_) => Value::Null,
                }
            }
            MySqlValue::Int(i) => json!(i),
            MySqlValue::UInt(u) => json!(u),
            MySqlValue::Float(f) => json!(f),
            MySqlValue::Double(d) => json!(d),
            MySqlValue::Date(y, m, d, h, min, s, _) => {
                json!(format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, m, d, h, min, s))
            }
            MySqlValue::Time(is_neg, d, h, m, s, _) => {
                let sign = if is_neg { "-" } else { "" };
                let total_hours = d * 24 + h as u32;
                json!(format!("{}{}:{:02}:{:02}", sign, total_hours, m, s))
            }
        }
    }

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

    async fn get_schemas(conn: &mut Conn) -> Result<Vec<String>, AppError> {
        let rows: Vec<String> = conn
            .query(
                "SELECT SCHEMA_NAME FROM information_schema.SCHEMATA
                 WHERE SCHEMA_NAME NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys')
                 ORDER BY SCHEMA_NAME"
            )
            .await
            .map_err(|e| AppError::Database(format!("Failed to get schemas: {}", e)))?;

        Ok(rows)
    }

    async fn get_tables(conn: &mut Conn) -> Result<Vec<Table>, AppError> {
        let rows: Vec<(String, String)> = conn
            .query(
                r#"
                SELECT
                    TABLE_SCHEMA,
                    TABLE_NAME
                FROM information_schema.TABLES
                WHERE TABLE_TYPE = 'BASE TABLE'
                  AND TABLE_SCHEMA NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys')
                ORDER BY TABLE_SCHEMA, TABLE_NAME
                "#
            )
            .await
            .map_err(|e| AppError::Database(format!("Failed to get tables: {}", e)))?;

        let mut tables = Vec::new();
        for (schema, name) in rows {
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

    async fn get_views(conn: &mut Conn) -> Result<Vec<View>, AppError> {
        let rows: Vec<(String, String)> = conn
            .query(
                r#"
                SELECT
                    TABLE_SCHEMA,
                    TABLE_NAME
                FROM information_schema.VIEWS
                WHERE TABLE_SCHEMA NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys')
                ORDER BY TABLE_SCHEMA, TABLE_NAME
                "#
            )
            .await
            .map_err(|e| AppError::Database(format!("Failed to get views: {}", e)))?;

        let mut views = Vec::new();
        for (schema, name) in rows {
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
                CASE WHEN c.COLUMN_KEY = 'PRI' THEN 1 ELSE 0 END as is_primary_key,
                CASE WHEN c.COLUMN_KEY = 'MUL' THEN 1 ELSE 0 END as is_foreign_key,
                c.CHARACTER_MAXIMUM_LENGTH
            FROM information_schema.COLUMNS c
            WHERE c.TABLE_SCHEMA = ? AND c.TABLE_NAME = ?
            ORDER BY c.ORDINAL_POSITION
        "#;

        let rows: Vec<(String, String, String, Option<String>, u8, u8, Option<u64>)> = conn
            .exec(query, (schema, table_name))
            .await
            .map_err(|e| AppError::Database(format!("Failed to get columns: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(|(name, data_type, is_nullable, default_value, is_pk, is_fk, max_length)| Column {
                name,
                data_type,
                is_nullable: is_nullable == "YES",
                default_value,
                is_primary_key: is_pk == 1,
                is_foreign_key: is_fk == 1,
                max_length: max_length.map(|v| v as i32),
                description: None,
            })
            .collect())
    }
}
