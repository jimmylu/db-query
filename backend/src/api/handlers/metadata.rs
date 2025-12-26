use axum::{
    extract::{Path, Query, State},
    Json,
};
use std::collections::HashMap;

use crate::api::middleware::AppError;
use crate::services::{DbService, MetadataCacheService};
use crate::api::handlers::connection::AppState;

/// Get database metadata
pub async fn get_metadata(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
    tracing::info!("Getting metadata for connection: {}", id);
    
    // Check if connection exists
    let connection = state
        .storage
        .get_connection(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Connection {} not found", id)))?;

    // Check if refresh is requested
    let refresh = params
        .get("refresh")
        .map(|v| v == "true")
        .unwrap_or(false);

    let cache_service = MetadataCacheService::new(state.storage.clone());

    if refresh {
        tracing::info!("Force refreshing metadata for connection: {}", id);
        // Force refresh: retrieve fresh metadata from database
        let (_, metadata) = DbService::connect_and_get_metadata(
            id.clone(),
            &connection.connection_url,
            &connection.database_type,
            state.pool_manager.clone(),
        )
        .await?;
        
        tracing::info!("Metadata refreshed. Found {} tables and {} views", metadata.tables.len(), metadata.views.len());

        // Convert to JSON using LLM service
        let llm_service = crate::services::LlmService::new(&state.config);
        let metadata_json = llm_service.convert_metadata_to_json(&metadata).await?;
        
        let mut metadata_with_json = metadata;
        metadata_with_json.metadata_json = metadata_json;
        metadata_with_json.increment_version();

        // Save to cache
        cache_service.save_metadata(&metadata_with_json).await?;

        Ok(Json(serde_json::json!({
            "metadata": metadata_with_json,
            "cached": false
        })))
    } else {
        // Try to get from cache
        match cache_service.get_cached_metadata(&id).await? {
            Some(metadata) => Ok(Json(serde_json::json!({
                "metadata": metadata,
                "cached": true
            }))),
            None => {
                // No cache, retrieve fresh
                let (_, metadata) = DbService::connect_and_get_metadata(
                    id.clone(),
                    &connection.connection_url,
                    &connection.database_type,
                    state.pool_manager.clone(),
                )
                .await?;

                // Convert to JSON
                let llm_service = crate::services::LlmService::new(&state.config);
                let metadata_json = llm_service.convert_metadata_to_json(&metadata).await?;
                
                let mut metadata_with_json = metadata;
                metadata_with_json.metadata_json = metadata_json;

                // Save to cache
                cache_service.save_metadata(&metadata_with_json).await?;

                Ok(Json(serde_json::json!({
                    "metadata": metadata_with_json,
                    "cached": false
                })))
            }
        }
    }
}

