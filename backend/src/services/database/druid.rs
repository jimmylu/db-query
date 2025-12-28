// Apache Druid adapter using HTTP REST API
// Druid is a real-time analytics database optimized for OLAP queries
use crate::models::{DatabaseConnection, DatabaseMetadata, Table, Column};
use crate::api::middleware::AppError;
use crate::services::database::adapter::{DatabaseAdapter, QueryResult};
use reqwest::Client;
use url::Url;
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use std::time::Instant;

pub struct DruidAdapter {
    base_url: String,
    client: Client,
}

#[derive(Debug, Serialize)]
struct DruidSqlRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct DruidSqlResponse {
    #[serde(default)]
    columns: Vec<DruidColumn>,
    #[serde(default)]
    rows: Vec<Vec<Value>>,
}

#[derive(Debug, Deserialize, Clone)]
struct DruidColumn {
    name: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]  // Used for deserializing Druid API response
    data_type: String,
}

// DruidDatasource struct removed - not needed since we get datasources as Vec<String>

impl DruidAdapter {
    pub fn new(connection_url: &str) -> Result<Self, AppError> {
        // Validate Druid URL format
        let url = Url::parse(connection_url)
            .map_err(|e| AppError::Validation(format!("Invalid Druid URL: {}", e)))?;

        // Convert druid:// scheme to http:// for REST API
        let base_url = if url.scheme() == "druid" {
            // druid://host:port → http://host:port
            format!("http://{}:{}",
                url.host_str().unwrap_or("localhost"),
                url.port().unwrap_or(8888))
        } else if url.scheme() == "http" || url.scheme() == "https" {
            // Use as-is, but remove path
            format!("{}://{}{}",
                url.scheme(),
                url.host_str().unwrap_or("localhost"),
                url.port().map(|p| format!(":{}", p)).unwrap_or_default())
        } else {
            return Err(AppError::Validation(
                "URL must use druid://, http://, or https:// scheme for Druid".to_string()
            ));
        };

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            base_url,
            client,
        })
    }

    /// Execute SQL query via Druid SQL API
    async fn execute_sql(&self, sql: &str, timeout_secs: u64) -> Result<DruidSqlResponse, AppError> {
        let sql_endpoint = format!("{}/druid/v2/sql", self.base_url);

        let request = DruidSqlRequest {
            query: sql.to_string(),
            context: Some(json!({
                "sqlTimeZone": "UTC",
                "useCache": true,
            })),
        };

        let response = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            self.client
                .post(&sql_endpoint)
                .json(&request)
                .send()
        )
        .await
        .map_err(|_| AppError::Database(format!("Query timeout after {} seconds", timeout_secs)))?
        .map_err(|e| AppError::Database(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::Database(format!(
                "Druid SQL query failed ({}): {}",
                status, error_body
            )));
        }

        response
            .json::<DruidSqlResponse>()
            .await
            .map_err(|e| AppError::Database(format!("Failed to parse Druid response: {}", e)))
    }

    /// Get list of datasources (equivalent to tables in Druid)
    async fn get_datasources(&self) -> Result<Vec<String>, AppError> {
        let datasources_endpoint = format!("{}/druid/coordinator/v1/datasources", self.base_url);

        let response = self.client
            .get(&datasources_endpoint)
            .send()
            .await
            .map_err(|e| AppError::Database(format!("Failed to get datasources: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Database(format!(
                "Failed to get datasources: HTTP {}",
                response.status()
            )));
        }

        let datasources: Vec<String> = response
            .json()
            .await
            .map_err(|e| AppError::Database(format!("Failed to parse datasources response: {}", e)))?;

        Ok(datasources)
    }

    /// Get schema for a specific datasource
    async fn get_datasource_schema(&self, datasource: &str) -> Result<Vec<Column>, AppError> {
        // Use INFORMATION_SCHEMA to get column information
        let sql = format!(
            "SELECT COLUMN_NAME, DATA_TYPE FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_NAME = '{}'",
            datasource
        );

        let response = self.execute_sql(&sql, 30).await?;

        let columns = response.rows.iter().filter_map(|row| {
            if row.len() >= 2 {
                Some(Column {
                    name: row[0].as_str().unwrap_or("unknown").to_string(),
                    data_type: row[1].as_str().unwrap_or("STRING").to_string(),
                    is_nullable: true, // Druid columns are generally nullable
                    is_primary_key: false, // Druid doesn't have primary keys
                    is_foreign_key: false, // Druid doesn't have foreign keys
                    default_value: None,
                    max_length: None,
                    description: None,
                })
            } else {
                None
            }
        }).collect();

        Ok(columns)
    }
}

#[async_trait::async_trait]
impl DatabaseAdapter for DruidAdapter {
    async fn connect_and_get_metadata(
        &self,
        connection_id: String,
    ) -> Result<(DatabaseConnection, DatabaseMetadata), AppError> {
        // Test connection by getting datasources
        let datasources = self.get_datasources().await?;

        // Create connection object
        let mut db_connection = DatabaseConnection::new(
            None,
            self.base_url.clone(),
            "druid".to_string(),
            None,
        );
        db_connection.id = connection_id.clone();
        db_connection.mark_connected();

        // Get metadata for each datasource
        let mut tables = Vec::new();
        for datasource in &datasources {
            match self.get_datasource_schema(datasource).await {
                Ok(columns) => {
                    tables.push(Table {
                        name: datasource.clone(),
                        schema: Some("druid".to_string()), // Druid doesn't have traditional schemas
                        columns,
                        row_count: None,
                        description: Some(format!("Druid datasource: {}", datasource)),
                    });
                }
                Err(e) => {
                    eprintln!("Warning: Failed to get schema for datasource {}: {}", datasource, e);
                    // Continue with other datasources
                }
            }
        }

        let metadata = DatabaseMetadata::new(
            connection_id,
            tables,
            vec![], // Druid doesn't have views
            vec!["druid".to_string()], // Single logical schema
        );

        Ok((db_connection, metadata))
    }

    async fn execute_query(
        &self,
        sql: &str,
        timeout_secs: u64,
    ) -> Result<QueryResult, AppError> {
        let start_time = Instant::now();

        let druid_response = self.execute_sql(sql, timeout_secs).await?;

        // Convert Druid response to standard QueryResult format
        let mut json_rows = Vec::new();

        for row_values in druid_response.rows {
            let mut row_obj = serde_json::Map::new();

            for (idx, column) in druid_response.columns.iter().enumerate() {
                let value = row_values.get(idx).cloned().unwrap_or(Value::Null);
                row_obj.insert(column.name.clone(), value);
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
        "druid"
    }

    fn dialect_name(&self) -> &str {
        "generic" // Druid has its own SQL dialect
    }

    fn supports_datafusion_execution(&self) -> bool {
        true // Druid supports DataFusion-style queries
    }

    async fn execute_datafusion_query(
        &self,
        datafusion_sql: &str,
        timeout_secs: u64,
    ) -> Result<(datafusion::arrow::datatypes::SchemaRef, Vec<datafusion::arrow::record_batch::RecordBatch>), AppError> {
        use datafusion::arrow::datatypes::{DataType, Field, Schema, TimeUnit};
        use datafusion::arrow::array::{
            ArrayRef, Int64Builder, Float64Builder, StringBuilder,
            BooleanBuilder, Date32Builder, TimestampMicrosecondBuilder,
        };
        use datafusion::arrow::record_batch::RecordBatch;
        use std::sync::Arc;

        // Execute SQL query via Druid SQL API
        let response = self.execute_sql(datafusion_sql, timeout_secs).await?;

        if response.columns.is_empty() || response.rows.is_empty() {
            // Return empty result
            let schema = Arc::new(Schema::empty());
            let batch = RecordBatch::new_empty(schema.clone());
            return Ok((schema, vec![batch]));
        }

        // Build Arrow schema from Druid column metadata
        let mut fields = Vec::new();
        for col in &response.columns {
            let data_type = Self::druid_type_to_arrow(&col.data_type);
            fields.push(Field::new(&col.name, data_type, true)); // nullable=true
        }

        let schema = Arc::new(Schema::new(fields));

        // Convert Druid response rows to Arrow RecordBatch
        let batch = Self::convert_druid_rows_to_arrow(&response, schema.clone())?;

        Ok((schema, vec![batch]))
    }

    async fn test_connection(&self) -> Result<(), AppError> {
        // Test connection by getting status
        let status_endpoint = format!("{}/status", self.base_url);

        let response = self.client
            .get(&status_endpoint)
            .send()
            .await
            .map_err(|e| AppError::Connection(format!("Connection test failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Connection(format!(
                "Connection test failed: HTTP {}",
                response.status()
            )));
        }

        Ok(())
    }
}

impl DruidAdapter {
    /// Map Druid data type to Arrow DataType
    fn druid_type_to_arrow(druid_type: &str) -> datafusion::arrow::datatypes::DataType {
        use datafusion::arrow::datatypes::{DataType, TimeUnit};

        match druid_type.to_uppercase().as_str() {
            "LONG" | "BIGINT" => DataType::Int64,
            "FLOAT" => DataType::Float32,
            "DOUBLE" => DataType::Float64,
            "STRING" | "VARCHAR" => DataType::Utf8,
            "TIMESTAMP" => DataType::Timestamp(TimeUnit::Millisecond, None),
            "DATE" => DataType::Date32,
            "BOOLEAN" => DataType::Boolean,
            _ => DataType::Utf8, // Default to string for unknown types
        }
    }

    /// Convert Druid response rows to Arrow RecordBatch
    fn convert_druid_rows_to_arrow(
        response: &DruidSqlResponse,
        schema: std::sync::Arc<datafusion::arrow::datatypes::Schema>,
    ) -> Result<datafusion::arrow::record_batch::RecordBatch, AppError> {
        use datafusion::arrow::datatypes::DataType;
        use datafusion::arrow::array::{
            Array, Int64Builder, Float32Builder, Float64Builder, StringBuilder,
            BooleanBuilder, Date32Builder, TimestampMillisecondBuilder,
        };
        use std::sync::Arc;

        let mut columns: Vec<Arc<dyn Array>> = Vec::new();

        for (col_idx, field) in schema.fields().iter().enumerate() {
            let array: Arc<dyn Array> = match field.data_type() {
                DataType::Int64 => {
                    let mut builder = Int64Builder::new();
                    for row in &response.rows {
                        let value = row.get(col_idx)
                            .and_then(|v| v.as_i64());
                        builder.append_option(value);
                    }
                    Arc::new(builder.finish())
                }
                DataType::Float32 => {
                    let mut builder = Float32Builder::new();
                    for row in &response.rows {
                        let value = row.get(col_idx)
                            .and_then(|v| v.as_f64())
                            .map(|f| f as f32);
                        builder.append_option(value);
                    }
                    Arc::new(builder.finish())
                }
                DataType::Float64 => {
                    let mut builder = Float64Builder::new();
                    for row in &response.rows {
                        let value = row.get(col_idx)
                            .and_then(|v| v.as_f64());
                        builder.append_option(value);
                    }
                    Arc::new(builder.finish())
                }
                DataType::Boolean => {
                    let mut builder = BooleanBuilder::new();
                    for row in &response.rows {
                        let value = row.get(col_idx)
                            .and_then(|v| v.as_bool());
                        builder.append_option(value);
                    }
                    Arc::new(builder.finish())
                }
                DataType::Date32 => {
                    let mut builder = Date32Builder::new();
                    for row in &response.rows {
                        let value = row.get(col_idx)
                            .and_then(|v| {
                                // Parse ISO date string to days since epoch
                                v.as_str().and_then(|s| {
                                    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                                        .ok()
                                        .map(|date| {
                                            let epoch = chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
                                            date.signed_duration_since(epoch).num_days() as i32
                                        })
                                })
                            });
                        builder.append_option(value);
                    }
                    Arc::new(builder.finish())
                }
                DataType::Timestamp(_, _) => {
                    let mut builder = TimestampMillisecondBuilder::new();
                    for row in &response.rows {
                        let value = row.get(col_idx)
                            .and_then(|v| v.as_i64());
                        builder.append_option(value);
                    }
                    Arc::new(builder.finish())
                }
                _ => {
                    // Default: convert to string
                    let mut builder = StringBuilder::new();
                    for row in &response.rows {
                        let value = row.get(col_idx)
                            .map(|v| v.to_string());
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
