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
            None,
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
        use datafusion::arrow::datatypes::{DataType, Field, Schema, TimeUnit};
        use datafusion::arrow::array::{
            ArrayRef, Int8Builder, Int16Builder, Int32Builder, Int64Builder,
            Float32Builder, Float64Builder, StringBuilder,
            Date32Builder, TimestampMicrosecondBuilder,
        };
        use datafusion::arrow::record_batch::RecordBatch;
        use std::sync::Arc;
        use std::time::Duration;

        // Create DataFusion session (for potential future use)
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

        if rows.is_empty() {
            // Return empty result with minimal schema
            let schema = Arc::new(Schema::empty());
            let batch = RecordBatch::new_empty(schema.clone());
            return Ok((schema, vec![batch]));
        }

        // Build Arrow schema from MySQL result columns
        let first_row = &rows[0];
        let columns_ref = first_row.columns_ref();
        let mut fields = Vec::new();

        for column in columns_ref.iter() {
            let column_name = column.name_str().to_string();
            let data_type = Self::mysql_column_to_arrow(column);
            fields.push(Field::new(column_name, data_type, true)); // nullable=true
        }

        let schema = Arc::new(Schema::new(fields));

        // Convert rows to Arrow RecordBatch
        let batch = Self::convert_mysql_rows_to_arrow(&rows, schema.clone())?;

        Ok((schema, vec![batch]))
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

    /// Map MySQL column type to Arrow DataType
    fn mysql_column_to_arrow(col: &mysql_async::Column) -> datafusion::arrow::datatypes::DataType {
        use mysql_async::consts::ColumnType;
        use datafusion::arrow::datatypes::{DataType, TimeUnit};

        match col.column_type() {
            ColumnType::MYSQL_TYPE_TINY => DataType::Int8,
            ColumnType::MYSQL_TYPE_SHORT => DataType::Int16,
            ColumnType::MYSQL_TYPE_LONG => DataType::Int32,
            ColumnType::MYSQL_TYPE_LONGLONG => DataType::Int64,
            ColumnType::MYSQL_TYPE_FLOAT => DataType::Float32,
            ColumnType::MYSQL_TYPE_DOUBLE => DataType::Float64,
            ColumnType::MYSQL_TYPE_DATE => DataType::Date32,
            ColumnType::MYSQL_TYPE_DATETIME => DataType::Timestamp(TimeUnit::Microsecond, None),
            ColumnType::MYSQL_TYPE_TIMESTAMP => DataType::Timestamp(TimeUnit::Microsecond, None),
            _ => DataType::Utf8,
        }
    }

    /// Convert MySQL rows to Arrow RecordBatch
    fn convert_mysql_rows_to_arrow(
        rows: &[Row],
        schema: std::sync::Arc<datafusion::arrow::datatypes::Schema>,
    ) -> Result<datafusion::arrow::record_batch::RecordBatch, AppError> {
        use datafusion::arrow::datatypes::DataType;
        use datafusion::arrow::array::{
            Array, Int8Builder, Int16Builder, Int32Builder, Int64Builder,
            Float32Builder, Float64Builder, StringBuilder,
            Date32Builder, TimestampMicrosecondBuilder,
        };
        use std::sync::Arc;

        let num_rows = rows.len();
        let mut columns: Vec<Arc<dyn Array>> = Vec::new();

        for (col_idx, field) in schema.fields().iter().enumerate() {
            let array: Arc<dyn Array> = match field.data_type() {
                DataType::Int8 => {
                    let mut builder = Int8Builder::new();
                    for row in rows {
                        let value: Option<i8> = row.get_opt(col_idx)
                            .and_then(|v| v.ok())
                            .and_then(|v| match v {
                                MySqlValue::Int(i) => Some(i as i8),
                                MySqlValue::UInt(u) => Some(u as i8),
                                _ => None,
                            });
                        builder.append_option(value);
                    }
                    Arc::new(builder.finish())
                }
                DataType::Int16 => {
                    let mut builder = Int16Builder::new();
                    for row in rows {
                        let value: Option<i16> = row.get_opt(col_idx)
                            .and_then(|v| v.ok())
                            .and_then(|v| match v {
                                MySqlValue::Int(i) => Some(i as i16),
                                MySqlValue::UInt(u) => Some(u as i16),
                                _ => None,
                            });
                        builder.append_option(value);
                    }
                    Arc::new(builder.finish())
                }
                DataType::Int32 => {
                    let mut builder = Int32Builder::new();
                    for row in rows {
                        let value: Option<i32> = row.get_opt(col_idx)
                            .and_then(|v| v.ok())
                            .and_then(|v| match v {
                                MySqlValue::Int(i) => Some(i as i32),
                                MySqlValue::UInt(u) => Some(u as i32),
                                _ => None,
                            });
                        builder.append_option(value);
                    }
                    Arc::new(builder.finish())
                }
                DataType::Int64 => {
                    let mut builder = Int64Builder::new();
                    for row in rows {
                        let value: Option<i64> = row.get_opt(col_idx)
                            .and_then(|v| v.ok())
                            .and_then(|v| match v {
                                MySqlValue::Int(i) => Some(i),
                                MySqlValue::UInt(u) => Some(u as i64),
                                _ => None,
                            });
                        builder.append_option(value);
                    }
                    Arc::new(builder.finish())
                }
                DataType::Float32 => {
                    let mut builder = Float32Builder::new();
                    for row in rows {
                        let value: Option<f32> = row.get_opt(col_idx)
                            .and_then(|v| v.ok())
                            .and_then(|v| match v {
                                MySqlValue::Float(f) => Some(f),
                                MySqlValue::Double(d) => Some(d as f32),
                                _ => None,
                            });
                        builder.append_option(value);
                    }
                    Arc::new(builder.finish())
                }
                DataType::Float64 => {
                    let mut builder = Float64Builder::new();
                    for row in rows {
                        let value: Option<f64> = row.get_opt(col_idx)
                            .and_then(|v| v.ok())
                            .and_then(|v| match v {
                                MySqlValue::Double(d) => Some(d),
                                MySqlValue::Float(f) => Some(f as f64),
                                _ => None,
                            });
                        builder.append_option(value);
                    }
                    Arc::new(builder.finish())
                }
                DataType::Utf8 => {
                    let mut builder = StringBuilder::new();
                    for row in rows {
                        let value: Option<String> = row.get_opt(col_idx)
                            .and_then(|v| v.ok())
                            .map(|v| Self::mysql_value_to_json(v).to_string());
                        builder.append_option(value);
                    }
                    Arc::new(builder.finish())
                }
                DataType::Date32 => {
                    let mut builder = Date32Builder::new();
                    for row in rows {
                        let days: Option<i32> = row.get_opt(col_idx)
                            .and_then(|v| v.ok())
                            .and_then(|v| match v {
                                MySqlValue::Date(y, m, d, _, _, _, _) => {
                                    chrono::NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32)
                                        .map(|date| {
                                            let epoch = chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
                                            date.signed_duration_since(epoch).num_days() as i32
                                        })
                                }
                                _ => None,
                            });
                        builder.append_option(days);
                    }
                    Arc::new(builder.finish())
                }
                DataType::Timestamp(_, _) => {
                    let mut builder = TimestampMicrosecondBuilder::new();
                    for row in rows {
                        let micros: Option<i64> = row.get_opt(col_idx)
                            .and_then(|v| v.ok())
                            .and_then(|v| match v {
                                MySqlValue::Date(y, m, d, h, min, s, us) => {
                                    chrono::NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32)
                                        .and_then(|date| {
                                            date.and_hms_micro_opt(h as u32, min as u32, s as u32, us)
                                        })
                                        .map(|dt| dt.and_utc().timestamp_micros())
                                }
                                _ => None,
                            });
                        builder.append_option(micros);
                    }
                    Arc::new(builder.finish())
                }
                _ => {
                    let mut builder = StringBuilder::new();
                    for row in rows {
                        let value: Option<String> = row.get_opt(col_idx)
                            .and_then(|v| v.ok())
                            .map(|v| Self::mysql_value_to_json(v).to_string());
                        builder.append_option(value);
                    }
                    Arc::new(builder.finish())
                }
            };
            columns.push(array);
        }

        datafusion::arrow::record_batch::RecordBatch::try_new(schema, columns)
            .map_err(|e| AppError::Database(format!("Failed to create RecordBatch: {}", e)))
    }
}
