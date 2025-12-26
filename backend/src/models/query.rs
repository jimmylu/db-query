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

