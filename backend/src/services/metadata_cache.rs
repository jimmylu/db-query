use crate::models::DatabaseMetadata;
use crate::storage::SqliteStorage;
use crate::api::middleware::AppError;

/// Metadata cache service for storing and retrieving cached metadata
pub struct MetadataCacheService {
    storage: std::sync::Arc<SqliteStorage>,
}

impl MetadataCacheService {
    pub fn new(storage: std::sync::Arc<SqliteStorage>) -> Self {
        Self { storage }
    }

    /// Get cached metadata for a connection
    pub async fn get_cached_metadata(
        &self,
        connection_id: &str,
    ) -> Result<Option<DatabaseMetadata>, AppError> {
        self.storage
            .get_metadata_cache(connection_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    /// Save metadata to cache
    pub async fn save_metadata(&self, metadata: &DatabaseMetadata) -> Result<(), AppError> {
        self.storage
            .save_metadata_cache(metadata)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    /// Check if cached metadata exists and is fresh
    pub async fn has_fresh_cache(&self, connection_id: &str) -> Result<bool, AppError> {
        match self.get_cached_metadata(connection_id).await? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }
}

