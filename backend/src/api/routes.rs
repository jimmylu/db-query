use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use std::sync::Arc;

use crate::api::handlers::{connection, metadata, query};
use crate::api::handlers::connection::AppState;
use crate::storage::SqliteStorage;
use crate::config::Config;
use crate::services::ConnectionPoolManager;

/// Create the main application router (deprecated - use create_router_with_state)
/// This is kept for backward compatibility but requires state to work properly
pub fn create_router() -> Router<()> {
    Router::new()
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
}

/// Create router with application state
pub fn create_router_with_state(storage: Arc<SqliteStorage>, config: Config) -> Router {
    // Initialize connection pool manager
    let pool_manager = Arc::new(ConnectionPoolManager::new());

    let state = AppState {
        storage,
        config,
        pool_manager,
    };

    Router::new()
        .route("/health", get(health_check))
        .route(
            "/api/connections",
            get(connection::list_connections).post(connection::create_connection),
        )
        .route(
            "/api/connections/{id}",
            get(connection::get_connection).delete(connection::delete_connection),
        )
        .route(
            "/api/connections/{id}/metadata",
            get(metadata::get_metadata),
        )
        .route(
            "/api/connections/{id}/query",
            post(query::execute_query),
        )
        .route(
            "/api/connections/{id}/nl-query",
            post(query::execute_natural_language_query),
        )
        .route(
            "/api/connections/{id}/unified-query",
            post(query::execute_unified_query),
        )
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}


