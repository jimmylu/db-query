use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use std::sync::Arc;

use crate::api::handlers::{connection, domain, metadata, query, cross_database_query};
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
        // Domain routes
        .route(
            "/api/domains",
            get(domain::list_domains).post(domain::create_domain),
        )
        .route(
            "/api/domains/{id}",
            get(domain::get_domain)
                .put(domain::update_domain)
                .delete(domain::delete_domain),
        )
        .route(
            "/api/domains/{id}/connections",
            get(domain::list_domain_connections),
        )
        // Connection routes
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
        .route(
            "/api/cross-database/query",
            post(cross_database_query::execute_cross_database_query),
        )
        // Saved query routes (domain-scoped)
        .route(
            "/api/domains/{domain_id}/queries/saved",
            get(query::list_saved_queries).post(query::create_saved_query),
        )
        .route(
            "/api/domains/{domain_id}/queries/saved/{query_id}",
            get(query::get_saved_query)
                .put(query::update_saved_query)
                .delete(query::delete_saved_query),
        )
        // Query history routes (domain-scoped)
        .route(
            "/api/domains/{domain_id}/queries/history",
            get(query::list_query_history),
        )
        .route(
            "/api/domains/{domain_id}/connections/{connection_id}/history",
            get(query::list_connection_query_history),
        )
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}


