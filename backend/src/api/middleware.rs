use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Application error types
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Invalid SQL: {0}")]
    InvalidSql(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("LLM service error: {0}")]
    LlmService(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

/// Error response format
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ErrorDetail {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_detail) = match self {
            AppError::Database(msg) => {
                // Provide actionable suggestions for database errors
                let enhanced_msg = if msg.contains("TABLE_NOT_FOUND") || msg.contains("does not exist") {
                    format!("{} Try refreshing the metadata or check if the table name is correct.", msg)
                } else if msg.contains("timeout") {
                    format!("{} Consider simplifying your query or checking database performance.", msg)
                } else {
                    msg
                };
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorDetail::new("DATABASE_ERROR", enhanced_msg),
                )
            },
            AppError::Connection(msg) => {
                // Connection errors already include suggestions
                (
                    StatusCode::BAD_REQUEST,
                    ErrorDetail::new("CONNECTION_ERROR", msg),
                )
            },
            AppError::InvalidSql(msg) => {
                let enhanced_msg = format!("{} Only SELECT queries are allowed. Please check your SQL syntax.", msg);
                (
                    StatusCode::BAD_REQUEST,
                    ErrorDetail::new("INVALID_SQL", enhanced_msg),
                )
            },
            AppError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorDetail::new("VALIDATION_ERROR", msg),
            ),
            AppError::LlmService(msg) => {
                let enhanced_msg = if msg.contains("not yet implemented") || msg.contains("not configured") {
                    format!("{} Please configure LLM_GATEWAY_URL environment variable to use natural language queries.", msg)
                } else {
                    msg
                };
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorDetail::new("LLM_SERVICE_ERROR", enhanced_msg),
                )
            },
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                ErrorDetail::new("NOT_FOUND", msg),
            ),
            AppError::NotImplemented(msg) => (
                StatusCode::NOT_IMPLEMENTED,
                ErrorDetail::new("NOT_IMPLEMENTED", msg),
            ),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorDetail::new("INTERNAL_ERROR", msg),
            ),
        };

        let body = Json(ErrorResponse {
            error: error_detail,
        });

        (status, body).into_response()
    }
}

/// Convert anyhow::Error to AppError
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

/// Convert rusqlite::Error to AppError
impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response_format() {
        let error = AppError::NotFound("Connection not found".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_error_detail_creation() {
        let detail = ErrorDetail::new("TEST_CODE", "Test message");
        assert_eq!(detail.code, "TEST_CODE");
        assert_eq!(detail.message, "Test message");
        assert!(detail.details.is_none());
    }
}

