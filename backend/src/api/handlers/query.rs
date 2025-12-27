use axum::{
    extract::{Path, State},
    Json,
};

use crate::api::middleware::AppError;
use crate::api::handlers::connection::AppState;
use crate::models::{Query, QueryRequest, NaturalLanguageQueryRequest, UnifiedQueryRequest, DatabaseType as ModelDatabaseType};
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

/// Execute unified SQL query using DataFusion semantic layer
///
/// This endpoint accepts DataFusion SQL syntax and automatically translates
/// it to the target database's dialect before execution.
///
/// # Endpoint
/// POST /api/connections/{id}/unified-query
///
/// # Request Body
/// ```json
/// {
///   "query": "SELECT * FROM users WHERE created_at >= CURRENT_DATE - INTERVAL '7' DAY",
///   "database_type": "postgresql",
///   "timeout_secs": 30,
///   "apply_limit": true,
///   "limit_value": 1000
/// }
/// ```
///
/// # Response
/// Returns UnifiedQueryResponse with original query, translated query, and results
pub async fn execute_unified_query(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UnifiedQueryRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    tracing::info!(
        "Executing unified SQL query for connection {} with database type {:?}",
        id,
        payload.database_type
    );

    // Validate query
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

    // Verify database type matches
    let expected_db_type = DatabaseType::from_str(&connection.database_type)?;
    let requested_db_type = convert_model_db_type_to_service(payload.database_type)?;

    if expected_db_type != requested_db_type {
        return Err(AppError::Validation(format!(
            "Database type mismatch: connection is {}, but request specified {}",
            connection.database_type,
            payload.database_type.as_str()
        )));
    }

    // Create database adapter with connection pool
    let adapter = create_adapter(
        expected_db_type,
        &connection.connection_url,
        state.pool_manager.clone(),
    ).await?;

    // Create unified query request
    let unified_request = UnifiedQueryRequest {
        query: sanitized_query.to_string(),
        database_type: payload.database_type,
        timeout_secs: payload.timeout_secs,
        apply_limit: payload.apply_limit,
        limit_value: payload.limit_value,
    };

    // Execute unified query using QueryService
    let query_service = QueryService::new();
    let result = query_service
        .execute_unified_query(unified_request, adapter)
        .await?;

    Ok(Json(serde_json::json!(result)))
}

/// Helper function to convert model DatabaseType to service DatabaseType
fn convert_model_db_type_to_service(db_type: ModelDatabaseType) -> Result<DatabaseType, AppError> {
    match db_type {
        ModelDatabaseType::PostgreSQL => Ok(DatabaseType::PostgreSQL),
        ModelDatabaseType::MySQL => Ok(DatabaseType::MySQL),
        ModelDatabaseType::Doris => Ok(DatabaseType::Doris),
        ModelDatabaseType::Druid => Ok(DatabaseType::Druid),
    }
}

/// Execute cross-database query
///
/// Allows querying across multiple databases with JOINs and UNIONs
pub async fn execute_cross_database_query(
    State(state): State<AppState>,
    Json(payload): Json<crate::models::CrossDatabaseQueryRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    tracing::info!(
        "Executing cross-database query across {} connections",
        payload.connection_ids.len()
    );

    // Validate request
    payload
        .validate()
        .map_err(|e| AppError::Validation(e))?;

    // Create planner (automatically uses aliases if provided)
    use crate::services::datafusion::CrossDatabaseQueryPlanner;
    let planner = CrossDatabaseQueryPlanner::from_request(&payload);

    // Generate execution plan
    let plan = planner.plan_query(&payload)?;

    // Get adapters for all connections
    let mut adapters = std::collections::HashMap::new();

    for conn_id in &payload.connection_ids {
        // Get connection from storage
        let connection = state
            .storage
            .get_connection(conn_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("Connection {} not found", conn_id)))?;

        // Create adapter
        let db_type = DatabaseType::from_str(&connection.database_type)?;
        let adapter = create_adapter(
            db_type,
            &connection.connection_url,
            state.pool_manager.clone(),
        )
        .await?;

        adapters.insert(conn_id.clone(), adapter);
    }

    // Execute cross-database query
    use crate::services::datafusion::DataFusionFederatedExecutor;
    let executor = DataFusionFederatedExecutor::new();
    let result = executor
        .execute_cross_database_query(plan, adapters)
        .await?;

    Ok(Json(serde_json::json!(result)))
}
