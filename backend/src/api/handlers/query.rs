use axum::{
    extract::{Path, State},
    Json,
};

use crate::api::middleware::AppError;
use crate::api::handlers::connection::AppState;
use crate::models::{Query, QueryRequest, NaturalLanguageQueryRequest};
use crate::services::{QueryService, LlmService, MetadataCacheService};
use crate::services::database::{DatabaseType, create_adapter};

/// Execute SQL query using connection pooling
pub async fn execute_query(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<QueryRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    tracing::info!("Executing SQL query for connection: {}", id);

    // Sanitize SQL query input
    let sanitized_query = payload.query.trim();
    if sanitized_query.is_empty() {
        return Err(AppError::Validation("SQL query cannot be empty".to_string()));
    }

    // Get connection from storage
    let connection = state
        .storage
        .get_connection(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Connection {} not found", id)))?;

    // Create database adapter with connection pool
    let db_type = DatabaseType::from_str(&connection.database_type)?;
    let adapter = create_adapter(
        db_type,
        &connection.connection_url,
        state.pool_manager.clone(),
    ).await?;

    // Execute query using QueryService (validation will happen there)
    let query_service = QueryService::new();
    let query = Query::new(id.clone(), sanitized_query.to_string(), false);
    let result = query_service.execute_query_with_adapter(query, adapter).await?;

    Ok(Json(serde_json::json!({
        "query": result,
    })))
}

/// Execute natural language query using connection pooling
pub async fn execute_natural_language_query(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<NaturalLanguageQueryRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    tracing::info!("Executing natural language query for connection: {}", id);

    // Validate question
    let question = payload.question.trim();
    if question.is_empty() {
        return Err(AppError::Validation("Question cannot be empty".to_string()));
    }

    // Get connection
    let connection = state
        .storage
        .get_connection(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Connection {} not found", id)))?;

    // Get metadata for LLM context
    let cache_service = MetadataCacheService::new(state.storage.clone());
    let metadata = cache_service
        .get_cached_metadata(&id)
        .await?
        .ok_or_else(|| AppError::NotFound("Database metadata not found. Please connect to the database first.".to_string()))?;

    // Generate SQL from natural language using LLM
    tracing::info!("Generating SQL from natural language question: {}", question);
    let llm_service = LlmService::new(&state.config);
    let generated_sql = llm_service
        .generate_sql_from_natural_language(question, &metadata, &connection.database_type)
        .await?;

    tracing::info!("Generated SQL from natural language: {}", generated_sql);

    // Create database adapter with connection pool
    let db_type = DatabaseType::from_str(&connection.database_type)?;
    let adapter = create_adapter(
        db_type,
        &connection.connection_url,
        state.pool_manager.clone(),
    ).await?;

    // Create query object (marked as LLM-generated)
    let query = Query::new(id.clone(), generated_sql.clone(), true);

    // Execute query using QueryService
    let query_service = QueryService::new();
    let result = query_service.execute_query_with_adapter(query, adapter).await?;

    Ok(Json(serde_json::json!({
        "query": result,
        "generated_sql": generated_sql,
    })))
}

