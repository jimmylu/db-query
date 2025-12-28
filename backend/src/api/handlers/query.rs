use axum::{
    extract::{Path, State},
    Json,
};

use crate::api::middleware::AppError;
use crate::api::handlers::connection::AppState;
use crate::models::{
    Query, QueryRequest, NaturalLanguageQueryRequest, UnifiedQueryRequest,
    DatabaseType as ModelDatabaseType, SavedQuery, QueryHistory,
    CreateSavedQueryRequest, UpdateSavedQueryRequest,
};
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

    // Log query history (if connection has domain_id)
    if let Some(domain_id) = &connection.domain_id {
        let history = match &result.status {
            crate::models::QueryStatus::Completed => {
                QueryHistory::new(
                    domain_id.clone(),
                    id.clone(),
                    sanitized_query.to_string(),
                    result.row_count.unwrap_or(0),
                    result.execution_time_ms.unwrap_or(0),
                    false,
                )
            }
            crate::models::QueryStatus::Failed => {
                QueryHistory::new_failed(
                    domain_id.clone(),
                    id.clone(),
                    sanitized_query.to_string(),
                    result.error_message.clone().unwrap_or_else(|| "Unknown error".to_string()),
                    false,
                )
            }
            _ => {
                // Don't log pending/executing states
                return Ok(Json(serde_json::json!({
                    "query": result,
                })));
            }
        };

        // Log to history (ignore errors to not block query response)
        if let Err(e) = state.storage.add_query_history(&history).await {
            tracing::warn!("Failed to log query history: {}", e);
        }
    }

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

    // Log query history (if connection has domain_id)
    if let Some(domain_id) = &connection.domain_id {
        let history = match &result.status {
            crate::models::QueryStatus::Completed => {
                QueryHistory::new(
                    domain_id.clone(),
                    id.clone(),
                    generated_sql.clone(),
                    result.row_count.unwrap_or(0),
                    result.execution_time_ms.unwrap_or(0),
                    true, // LLM-generated
                )
            }
            crate::models::QueryStatus::Failed => {
                QueryHistory::new_failed(
                    domain_id.clone(),
                    id.clone(),
                    generated_sql.clone(),
                    result.error_message.clone().unwrap_or_else(|| "Unknown error".to_string()),
                    true, // LLM-generated
                )
            }
            _ => {
                // Don't log pending/executing states
                return Ok(Json(serde_json::json!({
                    "query": result,
                    "generated_sql": generated_sql,
                })));
            }
        };

        // Log to history (ignore errors to not block query response)
        if let Err(e) = state.storage.add_query_history(&history).await {
            tracing::warn!("Failed to log query history: {}", e);
        }
    }

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

// ============================================================================
// Saved Query Handlers
// ============================================================================

/// Create a saved query for a domain
///
/// POST /api/domains/{domain_id}/queries/saved
pub async fn create_saved_query(
    State(state): State<AppState>,
    Path(domain_id): Path<String>,
    Json(payload): Json<CreateSavedQueryRequest>,
) -> Result<Json<SavedQuery>, AppError> {
    tracing::info!("Creating saved query '{}' for domain {}", payload.name, domain_id);

    // Validate inputs
    if payload.name.trim().is_empty() {
        return Err(AppError::Validation("Query name cannot be empty".to_string()));
    }
    if payload.query_text.trim().is_empty() {
        return Err(AppError::Validation("Query text cannot be empty".to_string()));
    }

    // Verify domain exists
    let domain = state
        .storage
        .get_domain(&domain_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Domain {} not found", domain_id)))?;

    // Verify connection exists and belongs to domain
    let connection = state
        .storage
        .get_connection(&payload.connection_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Connection {} not found", payload.connection_id)))?;

    if connection.domain_id.as_ref() != Some(&domain_id) {
        return Err(AppError::Validation(format!(
            "Connection {} does not belong to domain {}",
            payload.connection_id, domain_id
        )));
    }

    // Create saved query
    let saved_query = SavedQuery::new(
        domain.id,
        payload.connection_id,
        payload.name,
        payload.query_text,
        payload.description,
    );

    // Save to storage
    state
        .storage
        .save_query(&saved_query)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    tracing::info!("Saved query created with ID: {}", saved_query.id);
    Ok(Json(saved_query))
}

/// List all saved queries for a domain
///
/// GET /api/domains/{domain_id}/queries/saved
pub async fn list_saved_queries(
    State(state): State<AppState>,
    Path(domain_id): Path<String>,
) -> Result<Json<Vec<SavedQuery>>, AppError> {
    tracing::info!("Listing saved queries for domain {}", domain_id);

    // Verify domain exists
    state
        .storage
        .get_domain(&domain_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Domain {} not found", domain_id)))?;

    // Get saved queries
    let queries = state
        .storage
        .list_saved_queries(&domain_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    tracing::info!("Found {} saved queries for domain {}", queries.len(), domain_id);
    Ok(Json(queries))
}

/// Get a specific saved query
///
/// GET /api/domains/{domain_id}/queries/saved/{query_id}
pub async fn get_saved_query(
    State(state): State<AppState>,
    Path((domain_id, query_id)): Path<(String, String)>,
) -> Result<Json<SavedQuery>, AppError> {
    tracing::info!("Getting saved query {} for domain {}", query_id, domain_id);

    // Get saved query
    let query = state
        .storage
        .get_saved_query(&query_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Saved query {} not found", query_id)))?;

    // Verify query belongs to domain
    if query.domain_id != domain_id {
        return Err(AppError::NotFound(format!("Saved query {} not found in domain {}", query_id, domain_id)));
    }

    Ok(Json(query))
}

/// Update a saved query
///
/// PUT /api/domains/{domain_id}/queries/saved/{query_id}
pub async fn update_saved_query(
    State(state): State<AppState>,
    Path((domain_id, query_id)): Path<(String, String)>,
    Json(payload): Json<UpdateSavedQueryRequest>,
) -> Result<Json<SavedQuery>, AppError> {
    tracing::info!("Updating saved query {} for domain {}", query_id, domain_id);

    // Get existing query
    let query = state
        .storage
        .get_saved_query(&query_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Saved query {} not found", query_id)))?;

    // Verify query belongs to domain
    if query.domain_id != domain_id {
        return Err(AppError::NotFound(format!("Saved query {} not found in domain {}", query_id, domain_id)));
    }

    // Update query
    state
        .storage
        .update_saved_query(
            &query_id,
            payload.name.map(|s| s.to_string()),
            payload.query_text.map(|s| s.to_string()),
            payload.description.map(|s| s.to_string()),
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Get updated query
    let updated_query = state
        .storage
        .get_saved_query(&query_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::Database("Failed to retrieve updated query".to_string()))?;

    tracing::info!("Saved query {} updated successfully", query_id);
    Ok(Json(updated_query))
}

/// Delete a saved query
///
/// DELETE /api/domains/{domain_id}/queries/saved/{query_id}
pub async fn delete_saved_query(
    State(state): State<AppState>,
    Path((domain_id, query_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    tracing::info!("Deleting saved query {} from domain {}", query_id, domain_id);

    // Get existing query
    let query = state
        .storage
        .get_saved_query(&query_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Saved query {} not found", query_id)))?;

    // Verify query belongs to domain
    if query.domain_id != domain_id {
        return Err(AppError::NotFound(format!("Saved query {} not found in domain {}", query_id, domain_id)));
    }

    // Delete query
    state
        .storage
        .delete_saved_query(&query_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    tracing::info!("Saved query {} deleted successfully", query_id);
    Ok(Json(serde_json::json!({
        "message": "Saved query deleted successfully",
        "query_id": query_id
    })))
}

// ============================================================================
// Query History Handlers
// ============================================================================

/// List query history for a domain
///
/// GET /api/domains/{domain_id}/queries/history?limit=50
pub async fn list_query_history(
    State(state): State<AppState>,
    Path(domain_id): Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<QueryHistory>>, AppError> {
    tracing::info!("Listing query history for domain {}", domain_id);

    // Verify domain exists
    state
        .storage
        .get_domain(&domain_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Domain {} not found", domain_id)))?;

    // Parse limit parameter
    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(50);

    // Get query history
    let history = state
        .storage
        .list_query_history(&domain_id, limit)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    tracing::info!("Found {} query history entries for domain {}", history.len(), domain_id);
    Ok(Json(history))
}

/// List query history for a specific connection
///
/// GET /api/domains/{domain_id}/connections/{connection_id}/history?limit=50
pub async fn list_connection_query_history(
    State(state): State<AppState>,
    Path((domain_id, connection_id)): Path<(String, String)>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<QueryHistory>>, AppError> {
    tracing::info!("Listing query history for connection {} in domain {}", connection_id, domain_id);

    // Verify domain exists
    state
        .storage
        .get_domain(&domain_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Domain {} not found", domain_id)))?;

    // Verify connection exists and belongs to domain
    let connection = state
        .storage
        .get_connection(&connection_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Connection {} not found", connection_id)))?;

    if connection.domain_id.as_ref() != Some(&domain_id) {
        return Err(AppError::NotFound(format!(
            "Connection {} not found in domain {}",
            connection_id, domain_id
        )));
    }

    // Parse limit parameter
    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(50);

    // Get query history
    let history = state
        .storage
        .list_query_history_by_connection(&connection_id, limit)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    tracing::info!("Found {} query history entries for connection {}", history.len(), connection_id);
    Ok(Json(history))
}
