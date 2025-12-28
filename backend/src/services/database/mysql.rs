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

        // Translate DataFusion SQL to MySQL dialect
        let translated_sql = translator_service
            .translate_query(datafusion_sql, DFDatabaseType::MySQL)
            .await
            .map_err(|e| AppError::Database(format!("Failed to translate SQL: {}", e)))?;

        // Execute the translated query against MySQL
        let mut conn = self.get_conn().await?;

        let rows: Vec<Row> = tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            conn.query(&translated_sql),
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
        let fields: Vec<Field> = first_row.columns_ref().iter().map(|col| {
            let data_type = Self::mysql_column_to_arrow(col);
            Field::new(col.name_str().as_ref(), data_type, true)
        }).collect();

        let schema = Arc::new(Schema::new(fields.clone()));

        // Convert rows to Arrow RecordBatch
        let batch = Self::convert_mysql_rows_to_arrow(&rows, schema.clone())?;

        Ok((schema, vec![batch]))
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

    /// Convert MySQL column type to Arrow DataType
    fn mysql_column_to_arrow(col: &mysql_async::Column) -> datafusion::arrow::datatypes::DataType {
        use datafusion::arrow::datatypes::{DataType, TimeUnit};
        use mysql_async::consts::ColumnType;

        match col.column_type() {
            ColumnType::MYSQL_TYPE_TINY => DataType::Int8,
            ColumnType::MYSQL_TYPE_SHORT => DataType::Int16,
            ColumnType::MYSQL_TYPE_LONG | ColumnType::MYSQL_TYPE_INT24 => DataType::Int32,
            ColumnType::MYSQL_TYPE_LONGLONG => DataType::Int64,
            ColumnType::MYSQL_TYPE_FLOAT => DataType::Float32,
            ColumnType::MYSQL_TYPE_DOUBLE => DataType::Float64,
            ColumnType::MYSQL_TYPE_DECIMAL | ColumnType::MYSQL_TYPE_NEWDECIMAL => DataType::Utf8,
            ColumnType::MYSQL_TYPE_DATE => DataType::Date32,
            ColumnType::MYSQL_TYPE_DATETIME | ColumnType::MYSQL_TYPE_TIMESTAMP =>
                DataType::Timestamp(TimeUnit::Microsecond, None),
            ColumnType::MYSQL_TYPE_TIME => DataType::Utf8,
            ColumnType::MYSQL_TYPE_YEAR => DataType::Int32,
            ColumnType::MYSQL_TYPE_VARCHAR |
            ColumnType::MYSQL_TYPE_VAR_STRING |
            ColumnType::MYSQL_TYPE_STRING |
            ColumnType::MYSQL_TYPE_BLOB |
            ColumnType::MYSQL_TYPE_TINY_BLOB |
            ColumnType::MYSQL_TYPE_MEDIUM_BLOB |
            ColumnType::MYSQL_TYPE_LONG_BLOB => DataType::Utf8,
            _ => DataType::Utf8, // Default to string for unsupported types
        }
    }

    /// Convert MySQL rows to Arrow RecordBatch
    fn convert_mysql_rows_to_arrow(
        rows: &[Row],
        schema: datafusion::arrow::datatypes::SchemaRef,
    ) -> Result<datafusion::arrow::record_batch::RecordBatch, AppError> {
        use datafusion::arrow::array::*;
        use datafusion::arrow::datatypes::DataType;
        use datafusion::arrow::record_batch::RecordBatch;
        use std::sync::Arc;

        if rows.is_empty() {
            let empty_batch = RecordBatch::new_empty(schema);
            return Ok(empty_batch);
        }

        let mut columns: Vec<Arc<dyn Array>> = Vec::new();

        for (col_idx, field) in schema.fields().iter().enumerate() {
            let array: Arc<dyn Array> = match field.data_type() {
                DataType::Boolean => {
                    let mut builder = BooleanBuilder::new();
                    for row in rows {
                        let value: Option<bool> = row.get_opt(col_idx)
                            .and_then(|v| v.ok())
                            .and_then(|v| match v {
                                MySqlValue::Int(i) => Some(i != 0),
                                MySqlValue::UInt(u) => Some(u != 0),
                                _ => None,
                            });
                        builder.append_option(value);
                    }
                    Arc::new(builder.finish())
                }
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
                                    // Convert MySQL DATE to days since Unix epoch
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
                                    // Convert MySQL DATETIME/TIMESTAMP to microseconds since Unix epoch
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
                    // Default: convert to string
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

        RecordBatch::try_new(schema, columns)
            .map_err(|e| AppError::Database(format!("Failed to create RecordBatch: {}", e)))
    }
}
