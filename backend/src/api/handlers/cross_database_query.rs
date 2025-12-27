// Cross-Database Query Handler
//
// API endpoint for executing cross-database JOIN and UNION queries using DataFusion's
// federated execution engine.

use axum::{extract::State, Json};
use std::collections::HashMap;

use crate::api::handlers::connection::AppState;
use crate::api::middleware::AppError;
use crate::models::CrossDatabaseQueryRequest;
use crate::services::database::{create_adapter, DatabaseAdapter, DatabaseType};
use crate::services::datafusion::{CrossDatabaseQueryPlanner, DataFusionFederatedExecutor};

/// Execute cross-database query (JOIN or UNION across multiple databases)
///
/// # Request Body
///
/// ```json
/// {
///   "query": "SELECT u.username, t.title FROM db1.users u JOIN db2.todos t ON u.id = t.user_id",
///   "connection_ids": ["mysql-conn-id", "pg-conn-id"],
///   "database_aliases": {
///     "db1": "mysql-conn-id",
///     "db2": "pg-conn-id"
///   },
///   "timeout_secs": 60,
///   "apply_limit": true,
///   "limit_value": 100
/// }
/// ```
///
/// # Response
///
/// Returns merged results with execution details:
/// - Original query
/// - Sub-queries executed per database
/// - Merged results as JSON
/// - Execution time and row count
pub async fn execute_cross_database_query(
    State(state): State<AppState>,
    Json(payload): Json<CrossDatabaseQueryRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    tracing::info!(
        "Executing cross-database query across {} databases",
        payload.connection_ids.len()
    );

    // Validate request
    payload
        .validate()
        .map_err(|e| AppError::Validation(e))?;

    // Get all connections and create adapters
    let mut adapters: HashMap<String, Box<dyn DatabaseAdapter>> = HashMap::new();

    for conn_id in &payload.connection_ids {
        tracing::debug!("Loading connection: {}", conn_id);

        // Get connection from storage
        let connection = state
            .storage
            .get_connection(conn_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("Connection {} not found", conn_id)))?;

        // Verify connection is active
        if !matches!(connection.status, crate::models::ConnectionStatus::Connected) {
            return Err(AppError::Connection(format!(
                "Connection {} is not active (status: {:?})",
                conn_id, connection.status
            )));
        }

        // Create database adapter
        let db_type = DatabaseType::from_str(&connection.database_type)?;
        let adapter = create_adapter(
            db_type,
            &connection.connection_url,
            state.pool_manager.clone(),
        )
        .await?;

        adapters.insert(conn_id.clone(), adapter);
    }

    tracing::info!("Created {} database adapters", adapters.len());

    // Create query planner
    let planner = CrossDatabaseQueryPlanner::from_request(&payload);

    // Generate execution plan
    let plan = planner
        .plan_query(&payload)
        .map_err(|e| {
            tracing::error!("Query planning failed: {}", e);
            e
        })?;

    tracing::info!(
        "Generated execution plan: {} sub-queries, merge strategy: {:?}",
        plan.sub_queries.len(),
        plan.merge_strategy
    );

    // Create federated executor
    let executor = DataFusionFederatedExecutor::new();

    // Execute cross-database query
    let result = executor
        .execute_cross_database_query(plan, adapters)
        .await
        .map_err(|e| {
            tracing::error!("Cross-database query execution failed: {}", e);
            e
        })?;

    tracing::info!(
        "Cross-database query completed: {} rows in {}ms",
        result.row_count,
        result.execution_time_ms
    );

    Ok(Json(serde_json::json!({
        "query": result,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CrossDatabaseQueryRequest;

    #[test]
    fn test_cross_database_query_request_validation() {
        let mut aliases = HashMap::new();
        aliases.insert("db1".to_string(), "conn-1".to_string());
        aliases.insert("db2".to_string(), "conn-2".to_string());

        let request = CrossDatabaseQueryRequest {
            query: "SELECT * FROM db1.users u JOIN db2.todos t ON u.id = t.user_id".to_string(),
            connection_ids: vec!["conn-1".to_string(), "conn-2".to_string()],
            database_aliases: Some(aliases),
            timeout_secs: Some(60),
            apply_limit: Some(true),
            limit_value: Some(100),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_cross_database_query_request_invalid() {
        // Empty query
        let request = CrossDatabaseQueryRequest {
            query: "".to_string(),
            connection_ids: vec!["conn-1".to_string()],
            database_aliases: None,
            timeout_secs: None,
            apply_limit: None,
            limit_value: None,
        };

        assert!(request.validate().is_err());

        // No connections
        let request = CrossDatabaseQueryRequest {
            query: "SELECT * FROM users".to_string(),
            connection_ids: vec![],
            database_aliases: None,
            timeout_secs: None,
            apply_limit: None,
            limit_value: None,
        };

        assert!(request.validate().is_err());
    }
}
