// Database abstraction layer for multi-database support
pub mod adapter;
pub mod postgresql;
pub mod mysql;
pub mod doris;
pub mod druid;

pub use adapter::DatabaseAdapter;
pub use postgresql::PostgreSQLAdapter;
pub use mysql::MySQLAdapter;
pub use doris::DorisAdapter;
pub use druid::DruidAdapter;

use crate::api::middleware::AppError;
use crate::services::ConnectionPoolManager;
use std::sync::Arc;

/// Database type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    Doris,
    Druid,
}

impl DatabaseType {
    pub fn from_str(s: &str) -> Result<Self, AppError> {
        match s.to_lowercase().as_str() {
            "postgresql" | "postgres" => Ok(DatabaseType::PostgreSQL),
            "mysql" => Ok(DatabaseType::MySQL),
            "doris" => Ok(DatabaseType::Doris),
            "druid" => Ok(DatabaseType::Druid),
            _ => Err(AppError::Validation(format!("Unsupported database type: {}", s))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseType::PostgreSQL => "postgresql",
            DatabaseType::MySQL => "mysql",
            DatabaseType::Doris => "doris",
            DatabaseType::Druid => "druid",
        }
    }
}

/// Factory function to create appropriate database adapter
/// For PostgreSQL, uses connection pooling for optimal resource management
pub async fn create_adapter(
    db_type: DatabaseType,
    connection_url: &str,
    pool_manager: Arc<ConnectionPoolManager>,
) -> Result<Box<dyn DatabaseAdapter>, AppError> {
    match db_type {
        DatabaseType::PostgreSQL => {
            // Get or create connection pool for this database
            let pool = pool_manager.get_or_create_pool(connection_url).await?;
            Ok(Box::new(PostgreSQLAdapter::new(pool, connection_url)?))
        },
        DatabaseType::MySQL => Ok(Box::new(MySQLAdapter::new(connection_url)?)),
        DatabaseType::Doris => Ok(Box::new(DorisAdapter::new(connection_url)?)),
        DatabaseType::Druid => Ok(Box::new(DruidAdapter::new(connection_url)?)),
    }
}


