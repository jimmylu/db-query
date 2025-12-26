use crate::models::{DatabaseConnection, DatabaseMetadata};
use crate::api::middleware::AppError;
use crate::services::database::{DatabaseType, create_adapter};
use crate::services::ConnectionPoolManager;
use std::sync::Arc;

/// Database service for connecting to external databases and retrieving metadata
/// Uses DataFusion as the intermediate semantic layer for multi-database support
/// Uses connection pooling for optimal resource management
pub struct DbService;

impl DbService {
    /// Connect to a database and retrieve metadata
    /// Supports multiple database types: PostgreSQL, MySQL, Doris, Druid
    /// Uses DataFusion as the intermediate semantic layer
    /// PostgreSQL connections are pooled for optimal performance
    pub async fn connect_and_get_metadata(
        connection_id: String,
        connection_url: &str,
        database_type: &str,
        pool_manager: Arc<ConnectionPoolManager>,
    ) -> Result<(DatabaseConnection, DatabaseMetadata), AppError> {
        tracing::info!("Connecting to {} database: {}", database_type, connection_url);

        // Determine database type and create appropriate adapter
        let db_type = DatabaseType::from_str(database_type)?;
        let adapter = create_adapter(db_type, connection_url, pool_manager).await?;

        // Use adapter to connect and retrieve metadata
        let (db_connection, metadata) = adapter
            .connect_and_get_metadata(connection_id)
            .await?;

        tracing::info!("Successfully connected to {} database", database_type);

        Ok((db_connection, metadata))
    }
}
