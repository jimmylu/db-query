// DataFusion SessionManager
//
// Manages the lifecycle of DataFusion SessionContext instances and integrates
// with the existing connection pool infrastructure.

use datafusion::prelude::*;
use datafusion::execution::SessionStateBuilder;
use datafusion::common::Result as DataFusionResult;
use std::sync::Arc;
use anyhow::{Result, Context};

/// Configuration for DataFusion sessions
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Batch size for query execution
    pub batch_size: usize,
    /// Number of partitions for parallel execution
    pub target_partitions: usize,
    /// Enable query optimization
    pub enable_optimization: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            batch_size: 8192,
            target_partitions: num_cpus::get(),
            enable_optimization: true,
        }
    }
}

/// Manages DataFusion SessionContext lifecycle
///
/// The SessionManager creates and configures DataFusion sessions that integrate
/// with existing database connection pools. Each session maintains its own catalog
/// registry for registered tables.
///
/// # Example
/// ```rust,ignore
/// let manager = DataFusionSessionManager::new(SessionConfig::default());
/// let session = manager.create_session()?;
/// let df = session.sql("SELECT * FROM users").await?;
/// ```
pub struct DataFusionSessionManager {
    config: SessionConfig,
}

impl DataFusionSessionManager {
    /// Create a new SessionManager with the given configuration
    pub fn new(config: SessionConfig) -> Self {
        Self { config }
    }

    /// Create a new SessionManager with default configuration
    pub fn default_config() -> Self {
        Self::new(SessionConfig::default())
    }

    /// Create a new DataFusion SessionContext
    ///
    /// This factory method creates a configured SessionContext that can be used
    /// for query execution. The session inherits configuration from the manager.
    ///
    /// # Returns
    /// A configured `SessionContext` ready for query execution
    ///
    /// # Example
    /// ```rust,ignore
    /// let session = manager.create_session()?;
    /// session.register_table("my_table", table_provider)?;
    /// let results = session.sql("SELECT * FROM my_table").await?;
    /// ```
    pub fn create_session(&self) -> Result<SessionContext> {
        // Create DataFusion configuration
        let config = datafusion::execution::SessionConfig::new()
            .with_batch_size(self.config.batch_size)
            .with_target_partitions(self.config.target_partitions);

        // Create and return the session context
        let ctx = SessionContext::new_with_config(config);

        Ok(ctx)
    }

    /// Create a SessionContext with a custom RuntimeEnv
    ///
    /// This allows for advanced configuration such as custom object stores,
    /// memory limits, and disk managers.
    ///
    /// # Arguments
    /// * `runtime_env` - Custom runtime environment
    ///
    /// # Returns
    /// A configured `SessionContext` with custom runtime
    pub fn create_session_with_runtime(
        &self,
        runtime_env: Arc<datafusion::execution::runtime_env::RuntimeEnv>,
    ) -> Result<SessionContext> {
        let config = datafusion::execution::SessionConfig::new()
            .with_batch_size(self.config.batch_size)
            .with_target_partitions(self.config.target_partitions);

        let state = SessionStateBuilder::new()
            .with_config(config)
            .with_runtime_env(runtime_env)
            .build();

        Ok(SessionContext::new_with_state(state))
    }

    /// Get the current configuration
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }

    /// Update the configuration
    ///
    /// Note: This only affects newly created sessions, not existing ones.
    pub fn update_config(&mut self, config: SessionConfig) {
        self.config = config;
    }
}

/// Session factory that integrates with connection pools
///
/// This struct provides a bridge between the existing connection pool infrastructure
/// and DataFusion sessions. It allows creating sessions that have access to specific
/// database connections.
pub struct SessionFactory {
    manager: Arc<DataFusionSessionManager>,
}

impl SessionFactory {
    /// Create a new SessionFactory
    pub fn new(manager: Arc<DataFusionSessionManager>) -> Self {
        Self { manager }
    }

    /// Create a session for query execution
    ///
    /// This method creates a new SessionContext that can be used to execute queries.
    /// The session is configured according to the manager's settings.
    pub async fn create_session(&self) -> Result<SessionContext> {
        self.manager
            .create_session()
            .context("Failed to create DataFusion session")
    }

    /// Create a session with specific memory limit
    ///
    /// # Arguments
    /// * `memory_limit` - Maximum memory in bytes for this session
    pub async fn create_session_with_memory_limit(&self, memory_limit: usize) -> Result<SessionContext> {
        // Create a custom runtime environment with memory limit
        let runtime_config = datafusion::execution::runtime_env::RuntimeConfig::new()
            .with_memory_limit(memory_limit, 1.0); // 100% memory pool

        let runtime_env = Arc::new(
            datafusion::execution::runtime_env::RuntimeEnv::new(runtime_config)
                .context("Failed to create runtime environment")?
        );

        self.manager
            .create_session_with_runtime(runtime_env)
            .context("Failed to create session with memory limit")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SessionConfig::default();
        assert_eq!(config.batch_size, 8192);
        assert!(config.enable_optimization);
    }

    #[test]
    fn test_session_manager_creation() {
        let manager = DataFusionSessionManager::default_config();
        let session = manager.create_session();
        assert!(session.is_ok());
    }

    #[tokio::test]
    async fn test_session_factory() {
        let manager = Arc::new(DataFusionSessionManager::default_config());
        let factory = SessionFactory::new(manager);
        let session = factory.create_session().await;
        assert!(session.is_ok());
    }

    #[tokio::test]
    async fn test_session_with_memory_limit() {
        let manager = Arc::new(DataFusionSessionManager::default_config());
        let factory = SessionFactory::new(manager);

        // Create session with 100MB memory limit
        let session = factory.create_session_with_memory_limit(100 * 1024 * 1024).await;
        assert!(session.is_ok());
    }
}
