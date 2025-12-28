use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

use crate::api::middleware::AppError;
use crate::models::{CreateConnectionRequest, DatabaseConnection};
use crate::services::{DbService, MetadataCacheService, ConnectionPoolManager};
use crate::services::LlmService;
use crate::storage::SqliteStorage;
use crate::config::Config;

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<SqliteStorage>,
    pub config: Config,
    pub pool_manager: Arc<ConnectionPoolManager>,
}

/// List all connections
pub async fn list_connections(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let connections = state
        .storage
        .list_connections()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "connections": connections
    })))
}

/// Create a new database connection
pub async fn create_connection(
    State(state): State<AppState>,
    Json(payload): Json<CreateConnectionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    // Validate connection URL
    if payload.connection_url.is_empty() {
        return Err(AppError::Validation("Connection URL cannot be empty".to_string()));
    }

    // Validate connection URL format based on database type
    let db_type_lower = payload.database_type.to_lowercase();
    match db_type_lower.as_str() {
        "postgresql" | "postgres" => {
            if !payload.connection_url.starts_with("postgresql://") && !payload.connection_url.starts_with("postgres://") {
                return Err(AppError::Validation(
                    "Invalid PostgreSQL URL format. Must start with 'postgresql://' or 'postgres://'. Example: postgresql://user:password@host:port/database".to_string()
                ));
            }
        }
        "mysql" => {
            if !payload.connection_url.starts_with("mysql://") && !payload.connection_url.starts_with("mariadb://") {
                return Err(AppError::Validation(
                    "Invalid MySQL URL format. Must start with 'mysql://' or 'mariadb://'. Example: mysql://user:password@host:port/database".to_string()
                ));
            }
        }
        "doris" => {
            if !payload.connection_url.starts_with("doris://") && !payload.connection_url.starts_with("mysql://") {
                return Err(AppError::Validation(
                    "Invalid Doris URL format. Must start with 'doris://' or 'mysql://'. Example: doris://user:password@host:port/database".to_string()
                ));
            }
        }
        "druid" => {
            if !payload.connection_url.starts_with("druid://") && !payload.connection_url.starts_with("http://") && !payload.connection_url.starts_with("https://") {
                return Err(AppError::Validation(
                    "Invalid Druid URL format. Must start with 'druid://', 'http://', or 'https://'. Example: druid://host:port or http://host:port".to_string()
                ));
            }
        }
        _ => {
            return Err(AppError::Validation(
                format!("Unsupported database type: {}. Supported types: postgresql, mysql, doris, druid", payload.database_type)
            ));
        }
    }

    // Parse URL to validate format
    if let Err(e) = url::Url::parse(&payload.connection_url) {
        return Err(AppError::Validation(format!(
            "Invalid connection URL format: {}. Example: postgresql://user:password@host:port/database",
            e
        )));
    }

    // Create connection object
    let connection_id = uuid::Uuid::new_v4().to_string();
    let mut connection = DatabaseConnection::new(
        payload.name,
        payload.connection_url.clone(),
        payload.database_type.clone(),
        payload.domain_id.clone(),
    );
    connection.id = connection_id.clone();

    // Connect to database and retrieve metadata
    tracing::info!("Connecting to database and retrieving metadata for connection: {}", connection_id);
    let (mut db_connection, metadata) = DbService::connect_and_get_metadata(
        connection_id.clone(),
        &payload.connection_url,
        &payload.database_type,
        state.pool_manager.clone(),
    )
    .await?;

    // Convert metadata to JSON using LLM service
    let llm_service = LlmService::new(&state.config);
    let metadata_json = llm_service.convert_metadata_to_json(&metadata).await?;
    
    // Update metadata with JSON
    let mut metadata_with_json = metadata;
    metadata_with_json.metadata_json = metadata_json;

    // IMPORTANT: Save connection FIRST (before metadata_cache due to foreign key constraint)
    // Save connection without metadata_cache_id first
    state
        .storage
        .save_connection(&db_connection)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Now save metadata cache (connection_id foreign key will be valid)
    let cache_service = MetadataCacheService::new(state.storage.clone());
    cache_service.save_metadata(&metadata_with_json).await?;

    // Update connection with metadata cache ID
    db_connection.metadata_cache_id = Some(metadata_with_json.id.clone());
    state
        .storage
        .save_connection(&db_connection)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "connection": db_connection,
            "metadata": metadata_with_json
        })),
    ))
}

/// Get connection details
pub async fn get_connection(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let connection = state
        .storage
        .get_connection(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Connection {} not found", id)))?;

    Ok(Json(serde_json::json!(connection)))
}

/// Delete a connection
pub async fn delete_connection(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    tracing::info!("Deleting connection: {}", id);

    let deleted = state
        .storage
        .delete_connection(&id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete connection {}: {}", id, e);
            AppError::Database(e.to_string())
        })?;

    if deleted {
        tracing::info!("Connection deleted successfully: {}", id);
        Ok(StatusCode::NO_CONTENT)
    } else {
        tracing::warn!("Connection not found for deletion: {}", id);
        Err(AppError::NotFound(format!("Connection {} not found", id)))
    }
}

