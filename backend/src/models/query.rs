use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::unified_query::DatabaseType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub id: String,
    pub connection_id: String,
    pub query_text: String,
    pub is_llm_generated: bool,
    pub status: QueryStatus,
    pub results: Option<Vec<serde_json::Value>>,
    pub row_count: Option<usize>,
    pub execution_time_ms: Option<u64>,
    pub error_message: Option<String>,
    pub executed_at: Option<DateTime<Utc>>,
    pub limit_applied: bool,
    /// Database type for unified query execution (optional for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_type: Option<DatabaseType>,
    /// Original DataFusion SQL query (if using unified query execution)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_query: Option<String>,
    /// Translated query in target dialect (if using unified query execution)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translated_query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum QueryStatus {
    Pending,
    Executing,
    Completed,
    Failed,
}

impl Query {
    pub fn new(connection_id: String, query_text: String, is_llm_generated: bool) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            connection_id,
            query_text,
            is_llm_generated,
            status: QueryStatus::Pending,
            results: None,
            row_count: None,
            execution_time_ms: None,
            error_message: None,
            executed_at: None,
            limit_applied: false,
            database_type: None,
            original_query: None,
            translated_query: None,
        }
    }

    pub fn mark_executing(&mut self) {
        self.status = QueryStatus::Executing;
    }

    pub fn mark_completed(&mut self, results: Vec<serde_json::Value>, execution_time_ms: u64) {
        self.status = QueryStatus::Completed;
        self.results = Some(results.clone());
        self.row_count = Some(results.len());
        self.execution_time_ms = Some(execution_time_ms);
        self.executed_at = Some(Utc::now());
    }

    pub fn mark_failed(&mut self, error_message: String) {
        self.status = QueryStatus::Failed;
        self.error_message = Some(error_message);
        self.executed_at = Some(Utc::now());
    }
}

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub query: String,
}

#[derive(Debug, Deserialize)]
pub struct NaturalLanguageQueryRequest {
    pub question: String,
}

// ============================================================================
// Saved Query Models (Domain-Scoped)
// ============================================================================

/// SavedQuery - User-saved queries scoped to a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedQuery {
    pub id: String,
    pub domain_id: String,
    pub connection_id: String,
    pub name: String,
    pub query_text: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SavedQuery {
    pub fn new(
        domain_id: String,
        connection_id: String,
        name: String,
        query_text: String,
        description: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            domain_id,
            connection_id,
            name,
            query_text,
            description,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateSavedQueryRequest {
    pub connection_id: String,
    pub name: String,
    pub query_text: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSavedQueryRequest {
    pub name: Option<String>,
    pub query_text: Option<String>,
    pub description: Option<String>,
}

// ============================================================================
// Query History Models (Domain-Scoped)
// ============================================================================

/// QueryHistory - Execution history of queries scoped to a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryHistory {
    pub id: String,
    pub domain_id: String,
    pub connection_id: String,
    pub query_text: String,
    pub row_count: usize,
    pub execution_time_ms: u64,
    pub status: QueryHistoryStatus,
    pub error_message: Option<String>,
    pub executed_at: DateTime<Utc>,
    pub is_llm_generated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum QueryHistoryStatus {
    Success,
    Failed,
}

impl QueryHistory {
    pub fn new(
        domain_id: String,
        connection_id: String,
        query_text: String,
        row_count: usize,
        execution_time_ms: u64,
        is_llm_generated: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            domain_id,
            connection_id,
            query_text,
            row_count,
            execution_time_ms,
            status: QueryHistoryStatus::Success,
            error_message: None,
            executed_at: Utc::now(),
            is_llm_generated,
        }
    }

    pub fn new_failed(
        domain_id: String,
        connection_id: String,
        query_text: String,
        error_message: String,
        is_llm_generated: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            domain_id,
            connection_id,
            query_text,
            row_count: 0,
            execution_time_ms: 0,
            status: QueryHistoryStatus::Failed,
            error_message: Some(error_message),
            executed_at: Utc::now(),
            is_llm_generated,
        }
    }
}

