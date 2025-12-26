use deadpool_postgres::{Config as PoolConfig, ManagerConfig, Pool, RecyclingMethod};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_postgres::NoTls;

use crate::api::middleware::AppError;

/// Connection pool manager that maintains pools for multiple database connections
/// Each database connection URL gets its own dedicated pool for optimal resource management
pub struct ConnectionPoolManager {
    pools: Arc<RwLock<HashMap<String, Pool>>>,
    max_pool_size: usize,
    min_idle: Option<usize>,
}

impl ConnectionPoolManager {
    /// Create a new connection pool manager with default settings
    pub fn new() -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            max_pool_size: 16,
            min_idle: Some(2),
        }
    }

    /// Create a connection pool manager with custom pool settings
    pub fn with_config(max_pool_size: usize, min_idle: Option<usize>) -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            max_pool_size,
            min_idle,
        }
    }

    /// Get or create a connection pool for the given connection URL
    /// This method is safe to call concurrently from multiple tasks
    pub async fn get_or_create_pool(&self, connection_url: &str) -> Result<Pool, AppError> {
        // Fast path: check if pool already exists (read lock)
        {
            let pools = self.pools.read().await;
            if let Some(pool) = pools.get(connection_url) {
                tracing::debug!("Using existing connection pool for: {}", Self::mask_credentials(connection_url));
                return Ok(pool.clone());
            }
        }

        // Slow path: create new pool (write lock)
        let mut pools = self.pools.write().await;

        // Double-check in case another task created the pool while we were waiting
        if let Some(pool) = pools.get(connection_url) {
            tracing::debug!("Pool created by another task for: {}", Self::mask_credentials(connection_url));
            return Ok(pool.clone());
        }

        tracing::info!(
            "Creating new connection pool for: {} (max_size: {}, min_idle: {:?})",
            Self::mask_credentials(connection_url),
            self.max_pool_size,
            self.min_idle
        );

        // Create pool configuration
        let mut cfg = PoolConfig::new();
        cfg.url = Some(connection_url.to_string());
        cfg.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });

        // Create the pool
        let pool = cfg
            .create_pool(Some(deadpool_postgres::Runtime::Tokio1), NoTls)
            .map_err(|e| {
                tracing::error!("Failed to create connection pool: {}", e);
                AppError::Connection(format!("Failed to create connection pool: {}", e))
            })?;

        // Configure pool size
        pool.resize(self.max_pool_size);

        // Store the pool
        pools.insert(connection_url.to_string(), pool.clone());

        tracing::info!(
            "Successfully created connection pool for: {}",
            Self::mask_credentials(connection_url)
        );

        Ok(pool)
    }

    /// Remove a connection pool (useful when a connection is deleted)
    pub async fn remove_pool(&self, connection_url: &str) -> bool {
        let mut pools = self.pools.write().await;
        let removed = pools.remove(connection_url).is_some();

        if removed {
            tracing::info!(
                "Removed connection pool for: {}",
                Self::mask_credentials(connection_url)
            );
        }

        removed
    }

    /// Get the number of active pools
    pub async fn pool_count(&self) -> usize {
        let pools = self.pools.read().await;
        pools.len()
    }

    /// Get pool statistics for a given connection URL
    pub async fn get_pool_status(&self, connection_url: &str) -> Option<PoolStatus> {
        let pools = self.pools.read().await;
        pools.get(connection_url).map(|pool| {
            let status = pool.status();
            PoolStatus {
                size: status.size,
                available: status.available,
                max_size: status.max_size,
            }
        })
    }

    /// Mask credentials in connection URL for safe logging
    fn mask_credentials(url: &str) -> String {
        if let Ok(parsed_url) = url::Url::parse(url) {
            let mut masked = parsed_url.clone();
            if parsed_url.password().is_some() {
                let _ = masked.set_password(Some("***"));
            }
            masked.to_string()
        } else {
            "[invalid-url]".to_string()
        }
    }
}

impl Default for ConnectionPoolManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Connection pool status information
#[derive(Debug, Clone)]
pub struct PoolStatus {
    pub size: usize,
    pub available: usize,
    pub max_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_manager_creation() {
        let manager = ConnectionPoolManager::new();
        assert_eq!(manager.pool_count().await, 0);
    }

    #[tokio::test]
    async fn test_pool_manager_custom_config() {
        let manager = ConnectionPoolManager::with_config(32, Some(4));
        assert_eq!(manager.max_pool_size, 32);
        assert_eq!(manager.min_idle, Some(4));
    }

    #[test]
    fn test_mask_credentials() {
        let url = "postgresql://user:secret@localhost:5432/db";
        let masked = ConnectionPoolManager::mask_credentials(url);
        assert!(masked.contains("***"));
        assert!(!masked.contains("secret"));
    }

    #[tokio::test]
    async fn test_remove_pool() {
        let manager = ConnectionPoolManager::new();
        // Removing non-existent pool should return false
        assert!(!manager.remove_pool("postgresql://localhost/test").await);
    }
}
