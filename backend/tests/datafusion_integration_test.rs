// Integration tests for DataFusion semantic layer
//
// These tests verify that all Phase 2 components work together correctly.

use db_query_backend::services::datafusion::*;
use datafusion::prelude::*;
use std::sync::Arc;
use std::time::Duration;

/// Test that SessionManager can create functional sessions
#[tokio::test]
async fn test_session_manager_integration() {
    let manager = DataFusionSessionManager::default_config();
    let session = manager.create_session().expect("Failed to create session");

    // Execute a simple query
    let df = session
        .sql("SELECT 1 as num, 'test' as text")
        .await
        .expect("Failed to execute SQL");

    let results = df.collect().await.expect("Failed to collect results");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].num_rows(), 1);
}

/// Test QueryExecutor basic functionality
#[tokio::test]
async fn test_query_executor_basic() {
    let ctx = SessionContext::new();
    let executor = DataFusionQueryExecutor::new(ctx, Duration::from_secs(30));

    let result = executor
        .execute_query("SELECT 42 as answer, 'hello' as greeting")
        .await
        .expect("Query execution failed");

    assert_eq!(result.row_count, 1);
    assert_eq!(result.batches.len(), 1);
}

/// Test ResultConverter with various data types
#[tokio::test]
async fn test_result_converter_integration() {
    let ctx = SessionContext::new();
    let executor = DataFusionQueryExecutor::new(ctx, Duration::from_secs(30));

    // Query with multiple data types
    let sql = "SELECT
        123 as int_col,
        45.67 as float_col,
        'test' as string_col,
        true as bool_col";

    let result = executor.execute_query(sql).await.expect("Query failed");

    // Convert to QueryResult format
    let query_result = DataFusionResultConverter::convert_to_query_result(
        result.schema,
        result.batches,
    )
    .expect("Conversion failed");

    assert_eq!(query_result.row_count, 1);
    assert_eq!(query_result.columns.len(), 4);
    assert_eq!(query_result.columns[0], "int_col");
    assert_eq!(query_result.columns[1], "float_col");
    assert_eq!(query_result.columns[2], "string_col");
    assert_eq!(query_result.columns[3], "bool_col");
}

/// Test dialect translator service
#[tokio::test]
async fn test_dialect_translator_service() {
    let service = DialectTranslationService::new();

    // Test PostgreSQL translation (should be pass-through)
    let pg_sql = "SELECT * FROM users WHERE active = true";
    let pg_result = service
        .translate_query(pg_sql, DatabaseType::PostgreSQL)
        .await
        .expect("PostgreSQL translation failed");
    assert!(pg_result.contains("users"));

    // Test MySQL translation (identifier quoting)
    let mysql_sql = r#"SELECT "id" FROM "users""#;
    let mysql_result = service
        .translate_query(mysql_sql, DatabaseType::MySQL)
        .await
        .expect("MySQL translation failed");
    assert!(mysql_result.contains("`"), "MySQL should use backticks");
}

/// Test SessionManager with QueryExecutor
#[tokio::test]
async fn test_session_with_executor() {
    let manager = Arc::new(DataFusionSessionManager::default_config());
    let factory = SessionFactory::new(manager);

    let session = factory
        .create_session()
        .await
        .expect("Failed to create session");

    let executor = DataFusionQueryExecutor::new(session, Duration::from_secs(30));

    // Execute a query with aggregation
    let result = executor
        .execute_query("SELECT COUNT(*) as count FROM (SELECT 1 UNION ALL SELECT 2 UNION ALL SELECT 3)")
        .await
        .expect("Aggregation query failed");

    assert_eq!(result.row_count, 1);
}

/// Test query timeout functionality
#[tokio::test]
async fn test_query_timeout_integration() {
    let ctx = SessionContext::new();
    let executor = DataFusionQueryExecutor::new(ctx, Duration::from_millis(1));

    // This query should execute successfully despite short timeout
    // (it's very simple)
    let result = executor.execute_query("SELECT 1").await;

    // We don't assert failure because the query might complete in < 1ms
    // The test just verifies timeout mechanism doesn't break normal operation
    match result {
        Ok(_) => println!("Query completed within timeout"),
        Err(e) => {
            println!("Query timed out (expected): {}", e);
            assert!(e.to_string().contains("timeout") || e.to_string().contains("Timeout"));
        }
    }
}

/// Test batch translation
#[tokio::test]
async fn test_batch_translation() {
    let service = DialectTranslationService::new();

    let queries = vec![
        "SELECT * FROM users",
        "SELECT * FROM orders",
        "SELECT * FROM products",
    ];

    let results = service
        .translate_batch(queries, DatabaseType::PostgreSQL)
        .await
        .expect("Batch translation failed");

    assert_eq!(results.len(), 3);
    assert!(results[0].contains("users"));
    assert!(results[1].contains("orders"));
    assert!(results[2].contains("products"));
}

/// Test caching in translation service
#[tokio::test]
async fn test_translation_caching() {
    let service = DialectTranslationService::with_cache();

    let sql = "SELECT * FROM test_table";

    // First call - cache miss
    let result1 = service
        .translate_query(sql, DatabaseType::PostgreSQL)
        .await
        .expect("Translation failed");

    let cache_size = service.cache_size().await;
    assert_eq!(cache_size, Some(1));

    // Second call - cache hit
    let result2 = service
        .translate_query(sql, DatabaseType::PostgreSQL)
        .await
        .expect("Translation failed");

    assert_eq!(result1, result2);

    // Clear cache
    service.clear_cache().await;
    let cache_size = service.cache_size().await;
    assert_eq!(cache_size, Some(0));
}

/// Test EXPLAIN functionality
#[tokio::test]
async fn test_explain_query() {
    let ctx = SessionContext::new();
    let executor = DataFusionQueryExecutor::new(ctx, Duration::from_secs(30));

    let plan = executor
        .explain_query("SELECT 1 as num", false)
        .await
        .expect("EXPLAIN failed");

    assert!(!plan.is_empty());
    println!("Query plan:\n{}", plan);
}

/// Test multiple queries in sequence
#[tokio::test]
async fn test_sequential_queries() {
    let ctx = SessionContext::new();
    let executor = DataFusionQueryExecutor::new(ctx, Duration::from_secs(30));

    // Execute multiple queries
    for i in 1..=5 {
        let sql = format!("SELECT {} as num", i);
        let result = executor
            .execute_query(&sql)
            .await
            .expect("Query failed");

        assert_eq!(result.row_count, 1);
    }
}
