// Database adapter trait for multi-database support
use crate::models::{DatabaseConnection, DatabaseMetadata};
use crate::api::middleware::AppError;
use serde_json::Value;

/// Database connection information
#[derive(Debug, Clone)]
pub struct DatabaseConnectionInfo {
    pub connection_url: String,
    pub database_type: String,
}

/// Query execution result
#[derive(Debug)]
pub struct QueryResult {
    pub rows: Vec<Value>,
    pub row_count: usize,
    pub execution_time_ms: u64,
}

/// Database adapter trait - abstraction layer for different database types
/// All adapters use DataFusion as the intermediate semantic layer
#[async_trait::async_trait]
pub trait DatabaseAdapter: Send + Sync {
    /// Connect to database and retrieve metadata
    async fn connect_and_get_metadata(
        &self,
        connection_id: String,
    ) -> Result<(DatabaseConnection, DatabaseMetadata), AppError>;

    /// Execute a SQL query using DataFusion
    /// The SQL is validated and converted to DataFusion's logical plan,
    /// then executed against the target database
    async fn execute_query(
        &self,
        sql: &str,
        timeout_secs: u64,
    ) -> Result<QueryResult, AppError>;

    /// Get database type
    fn database_type(&self) -> &str;

    /// Test connection
    async fn test_connection(&self) -> Result<(), AppError>;
}


