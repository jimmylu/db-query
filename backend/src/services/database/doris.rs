// Apache Doris adapter using DataFusion as intermediate semantic layer
use crate::models::{DatabaseConnection, DatabaseMetadata};
use crate::api::middleware::AppError;
use crate::services::database::adapter::{DatabaseAdapter, QueryResult};
use url::Url;

pub struct DorisAdapter {
    connection_url: String,
}

impl DorisAdapter {
    pub fn new(connection_url: &str) -> Result<Self, AppError> {
        // Validate Doris URL format
        // Doris typically uses MySQL protocol, so URL format is similar
        let url = Url::parse(connection_url)
            .map_err(|e| AppError::Validation(format!("Invalid Doris URL: {}", e)))?;
        
        if url.scheme() != "doris" && url.scheme() != "mysql" {
            return Err(AppError::Validation("URL must use doris:// or mysql:// scheme for Doris".to_string()));
        }

        Ok(Self {
            connection_url: connection_url.to_string(),
        })
    }
}

#[async_trait::async_trait]
impl DatabaseAdapter for DorisAdapter {
    async fn connect_and_get_metadata(
        &self,
        connection_id: String,
    ) -> Result<(DatabaseConnection, DatabaseMetadata), AppError> {
        // TODO: Implement Doris connection and metadata retrieval
        // Doris uses MySQL protocol, so we can reuse MySQL adapter logic
        Err(AppError::Internal("Doris adapter not yet implemented. Please use PostgreSQL for now.".to_string()))
    }

    async fn execute_query(
        &self,
        sql: &str,
        timeout_secs: u64,
    ) -> Result<QueryResult, AppError> {
        // TODO: Implement Doris query execution using DataFusion
        // DataFusion can connect to Doris through MySQL protocol
        Err(AppError::Internal("Doris query execution not yet implemented. Please use PostgreSQL for now.".to_string()))
    }

    fn database_type(&self) -> &str {
        "doris"
    }

    fn dialect_name(&self) -> &str {
        "mysql" // Doris uses MySQL-compatible dialect
    }

    fn supports_datafusion_execution(&self) -> bool {
        false // Not yet implemented
    }

    async fn execute_datafusion_query(
        &self,
        _datafusion_sql: &str,
        _timeout_secs: u64,
    ) -> Result<(datafusion::arrow::datatypes::SchemaRef, Vec<datafusion::arrow::record_batch::RecordBatch>), AppError> {
        Err(AppError::NotImplemented("Doris DataFusion execution not yet implemented. This will be added in Phase 5.".to_string()))
    }

    async fn test_connection(&self) -> Result<(), AppError> {
        // TODO: Implement Doris connection test
        Err(AppError::Internal("Doris connection test not yet implemented.".to_string()))
    }
}


