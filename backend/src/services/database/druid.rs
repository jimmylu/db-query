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
        // For now, execute directly as Druid SQL
        // Full Arrow conversion can be added later
        let _result = self.execute_query(datafusion_sql, timeout_secs).await?;

        Err(AppError::NotImplemented(
            "Full Druid to Arrow conversion not yet implemented. Use execute_query instead.".to_string()
        ))
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
