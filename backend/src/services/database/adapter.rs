// Database adapter trait for multi-database support
use crate::models::{DatabaseConnection, DatabaseMetadata};
use crate::api::middleware::AppError;
use serde_json::Value;
use datafusion::arrow::datatypes::SchemaRef;
use datafusion::arrow::record_batch::RecordBatch;

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

    /// Execute a DataFusion SQL query and return Arrow RecordBatches
    /// This method is used for unified SQL execution with automatic dialect translation.
    /// The query is in DataFusion SQL syntax and will be translated to the target dialect.
    ///
    /// # Arguments
    /// * `datafusion_sql` - SQL query in DataFusion syntax
    /// * `timeout_secs` - Timeout in seconds
    ///
    /// # Returns
    /// Tuple of (schema, batches) containing the query results
    async fn execute_datafusion_query(
        &self,
        datafusion_sql: &str,
        timeout_secs: u64,
    ) -> Result<(SchemaRef, Vec<RecordBatch>), AppError>;

    /// Get the database dialect name for DataFusion translation
    ///
    /// # Returns
    /// Dialect name (e.g., "postgresql", "mysql", "doris", "druid")
    fn dialect_name(&self) -> &str;

    /// Get database type
    fn database_type(&self) -> &str;

    /// Test connection
    async fn test_connection(&self) -> Result<(), AppError>;

    /// Check if this adapter supports DataFusion-based execution
    ///
    /// # Returns
    /// true if the adapter has implemented execute_datafusion_query
    fn supports_datafusion_execution(&self) -> bool {
        // Default to false for backward compatibility
        // Adapters that implement DataFusion execution should override this
        false
    }
}


