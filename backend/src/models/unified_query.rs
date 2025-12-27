// Unified Query Request Model
//
// This model represents a query request that can be executed against any database type
// using DataFusion's unified SQL semantics. The query will be automatically translated
// to the target database's dialect.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Database type enumeration for unified query execution
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    /// PostgreSQL database
    PostgreSQL,
    /// MySQL database
    MySQL,
    /// Apache Doris database
    Doris,
    /// Apache Druid database
    Druid,
}

impl DatabaseType {
    /// Get the string representation of the database type
    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseType::PostgreSQL => "postgresql",
            DatabaseType::MySQL => "mysql",
            DatabaseType::Doris => "doris",
            DatabaseType::Druid => "druid",
        }
    }

    /// Parse database type from string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "postgresql" | "postgres" => Ok(DatabaseType::PostgreSQL),
            "mysql" => Ok(DatabaseType::MySQL),
            "doris" => Ok(DatabaseType::Doris),
            "druid" => Ok(DatabaseType::Druid),
            _ => Err(format!("Unsupported database type: {}", s)),
        }
    }
}

/// Unified query request model
///
/// This represents a query that will be executed using DataFusion's unified SQL semantics.
/// The query is written in DataFusion SQL and will be automatically translated to the
/// target database's dialect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedQueryRequest {
    /// The SQL query in DataFusion syntax
    pub query: String,

    /// The target database type for execution
    pub database_type: DatabaseType,

    /// Optional timeout in seconds (defaults to 30)
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Whether to apply automatic LIMIT if not present (defaults to true)
    #[serde(default = "default_apply_limit")]
    pub apply_limit: bool,

    /// The LIMIT value to apply if apply_limit is true (defaults to 1000)
    #[serde(default = "default_limit_value")]
    pub limit_value: usize,
}

fn default_timeout() -> u64 {
    30
}

fn default_apply_limit() -> bool {
    true
}

fn default_limit_value() -> usize {
    1000
}

impl UnifiedQueryRequest {
    /// Create a new unified query request
    pub fn new(query: String, database_type: DatabaseType) -> Self {
        Self {
            query,
            database_type,
            timeout_secs: default_timeout(),
            apply_limit: default_apply_limit(),
            limit_value: default_limit_value(),
        }
    }

    /// Create a query request with custom timeout
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Create a query request with custom limit settings
    pub fn with_limit(mut self, apply_limit: bool, limit_value: usize) -> Self {
        self.apply_limit = apply_limit;
        self.limit_value = limit_value;
        self
    }
}

/// Unified query response model
///
/// This represents the result of executing a unified SQL query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedQueryResponse {
    /// The original query in DataFusion SQL syntax
    pub original_query: String,

    /// The translated query in the target database's dialect
    pub translated_query: String,

    /// The target database type
    pub database_type: DatabaseType,

    /// Query results as JSON objects
    pub results: Vec<serde_json::Value>,

    /// Number of rows returned
    pub row_count: usize,

    /// Execution time in milliseconds
    pub execution_time_ms: u128,

    /// Whether a LIMIT was automatically applied
    pub limit_applied: bool,

    /// Timestamp when the query was executed
    pub executed_at: DateTime<Utc>,
}

impl UnifiedQueryResponse {
    /// Create a new unified query response
    pub fn new(
        original_query: String,
        translated_query: String,
        database_type: DatabaseType,
        results: Vec<serde_json::Value>,
        execution_time_ms: u128,
        limit_applied: bool,
    ) -> Self {
        let row_count = results.len();

        Self {
            original_query,
            translated_query,
            database_type,
            results,
            row_count,
            execution_time_ms,
            limit_applied,
            executed_at: Utc::now(),
        }
    }
}

/// Simplified request for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleUnifiedQueryRequest {
    /// The SQL query
    pub query: String,
}

impl From<SimpleUnifiedQueryRequest> for String {
    fn from(req: SimpleUnifiedQueryRequest) -> String {
        req.query
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_type_str_conversion() {
        assert_eq!(DatabaseType::PostgreSQL.as_str(), "postgresql");
        assert_eq!(DatabaseType::MySQL.as_str(), "mysql");
        assert_eq!(DatabaseType::Doris.as_str(), "doris");
        assert_eq!(DatabaseType::Druid.as_str(), "druid");
    }

    #[test]
    fn test_database_type_parsing() {
        assert_eq!(
            DatabaseType::from_str("postgresql").unwrap(),
            DatabaseType::PostgreSQL
        );
        assert_eq!(
            DatabaseType::from_str("postgres").unwrap(),
            DatabaseType::PostgreSQL
        );
        assert_eq!(
            DatabaseType::from_str("mysql").unwrap(),
            DatabaseType::MySQL
        );
        assert_eq!(
            DatabaseType::from_str("MySQL").unwrap(),
            DatabaseType::MySQL
        );

        assert!(DatabaseType::from_str("unknown").is_err());
    }

    #[test]
    fn test_unified_query_request_defaults() {
        let req = UnifiedQueryRequest::new(
            "SELECT * FROM users".to_string(),
            DatabaseType::PostgreSQL,
        );

        assert_eq!(req.timeout_secs, 30);
        assert!(req.apply_limit);
        assert_eq!(req.limit_value, 1000);
    }

    #[test]
    fn test_unified_query_request_builder() {
        let req = UnifiedQueryRequest::new(
            "SELECT * FROM users".to_string(),
            DatabaseType::MySQL,
        )
        .with_timeout(60)
        .with_limit(false, 0);

        assert_eq!(req.timeout_secs, 60);
        assert!(!req.apply_limit);
        assert_eq!(req.limit_value, 0);
    }

    #[test]
    fn test_unified_query_response_creation() {
        let response = UnifiedQueryResponse::new(
            "SELECT * FROM users".to_string(),
            "SELECT * FROM users LIMIT 1000".to_string(),
            DatabaseType::PostgreSQL,
            vec![serde_json::json!({"id": 1, "name": "Alice"})],
            150,
            true,
        );

        assert_eq!(response.row_count, 1);
        assert_eq!(response.execution_time_ms, 150);
        assert!(response.limit_applied);
        assert_eq!(response.database_type, DatabaseType::PostgreSQL);
    }
}
