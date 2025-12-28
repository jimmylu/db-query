use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::api::middleware::AppError;
use crate::api::handlers::connection::AppState;
use crate::models::{Domain, CreateDomainRequest, UpdateDomainRequest};

/// List all domains with resource counts
pub async fn list_domains(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let domains = state
        .storage
        .list_domains()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "domains": domains
    })))
}

/// Get a domain by ID
pub async fn get_domain(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let domain = state
        .storage
        .get_domain(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Domain with ID {} not found", id)))?;

    // Get connection count
    let connection_count = state
        .storage
        .get_domain_connection_count(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "domain": {
            "id": domain.id,
            "name": domain.name,
            "description": domain.description,
            "created_at": domain.created_at,
            "updated_at": domain.updated_at,
            "connection_count": connection_count,
            "saved_query_count": 0,
            "query_history_count": 0,
        }
    })))
}

/// Create a new domain
pub async fn create_domain(
    State(state): State<AppState>,
    Json(payload): Json<CreateDomainRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    // Create domain with validation
    let domain = Domain::new(payload.name, payload.description)
        .map_err(|e| AppError::Validation(e))?;

    // Save to storage
    state
        .storage
        .create_domain(&domain)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "domain": {
                "id": domain.id,
                "name": domain.name,
                "description": domain.description,
                "created_at": domain.created_at,
                "updated_at": domain.updated_at,
                "connection_count": 0,
                "saved_query_count": 0,
                "query_history_count": 0,
            }
        })),
    ))
}

/// Update an existing domain
pub async fn update_domain(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateDomainRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Get existing domain
    let mut domain = state
        .storage
        .get_domain(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Domain with ID {} not found", id)))?;

    // Update fields with validation
    if let Some(name) = payload.name {
        Domain::validate_name(&name).map_err(|e| AppError::Validation(e))?;
        domain.name = name;
    }

    if let Some(description) = payload.description {
        Domain::validate_description(&Some(description.clone())).map_err(|e| AppError::Validation(e))?;
        domain.description = Some(description);
    }

    // Touch updated_at timestamp
    domain.touch();

    // Save changes
    state
        .storage
        .update_domain(&domain)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Get connection count for response
    let connection_count = state
        .storage
        .get_domain_connection_count(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "domain": {
            "id": domain.id,
            "name": domain.name,
            "description": domain.description,
            "created_at": domain.created_at,
            "updated_at": domain.updated_at,
            "connection_count": connection_count,
            "saved_query_count": 0,
            "query_history_count": 0,
        }
    })))
}

/// Delete a domain (CASCADE will delete associated connections)
pub async fn delete_domain(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    // Prevent deletion of default domain
    if id == "default-domain-id" {
        return Err(AppError::Validation(
            "Cannot delete the default domain".to_string(),
        ));
    }

    let deleted = state
        .storage
        .delete_domain(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound(format!("Domain with ID {} not found", id)))
    }
}

/// List connections for a specific domain
pub async fn list_domain_connections(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Verify domain exists
    state
        .storage
        .get_domain(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Domain with ID {} not found", id)))?;

    // Get connections for domain
    let connections = state
        .storage
        .list_connections_by_domain(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "connections": connections
    })))
}
