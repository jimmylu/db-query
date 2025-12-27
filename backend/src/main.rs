use axum::Router;
use std::net::SocketAddr;
use tracing::{info, error};
use tracing_subscriber;

mod api;
mod config;
mod models;
mod services;
mod storage;
mod validation;

use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Load configuration
    let config = Config::from_env().map_err(|e| {
        error!("Failed to load configuration: {}", e);
        e
    })?;

    info!("Starting server on {}", config.server_address());

    // Initialize SQLite storage
    let storage = std::sync::Arc::new(
        storage::SqliteStorage::new(&config.database.url)
            .await
            .map_err(|e| {
                error!("Failed to initialize database: {}", e);
                e
            })?
    );

    // Create router with state
    let app: Router = api::routes::create_router_with_state(storage, config.clone());

    // Start server
    let addr: SocketAddr = config.server_address().parse()?;
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
