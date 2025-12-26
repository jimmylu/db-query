// DataFusion QueryExecutor
//
// Handles query parsing, logical plan generation, and execution with timeout support.

use datafusion::prelude::*;
use datafusion::logical_expr::LogicalPlan;
use datafusion::arrow::array::RecordBatch;
use datafusion::arrow::datatypes::SchemaRef;
use std::time::Duration;
use std::sync::Arc;
use anyhow::{Result, Context, anyhow};
use tokio::time::timeout;

/// Query execution result containing record batches
pub struct QueryExecutionResult {
    /// Schema of the result set
    pub schema: SchemaRef,
    /// Result data as record batches
    pub batches: Vec<RecordBatch>,
    /// Number of rows returned
    pub row_count: usize,
    /// Execution time in milliseconds
    pub execution_time_ms: u128,
}

impl QueryExecutionResult {
    /// Create a new result from record batches
    pub fn from_batches(schema: SchemaRef, batches: Vec<RecordBatch>, execution_time_ms: u128) -> Self {
        let row_count = batches.iter().map(|batch| batch.num_rows()).sum();

        Self {
            schema,
            batches,
            row_count,
            execution_time_ms,
        }
    }
}

/// Executes DataFusion queries with timeout and error handling
pub struct DataFusionQueryExecutor {
    /// Session context for query execution
    ctx: SessionContext,
    /// Default timeout for queries
    default_timeout: Duration,
}

impl DataFusionQueryExecutor {
    /// Create a new QueryExecutor with a session context
    ///
    /// # Arguments
    /// * `ctx` - DataFusion session context
    /// * `default_timeout` - Default timeout for query execution
    pub fn new(ctx: SessionContext, default_timeout: Duration) -> Self {
        Self {
            ctx,
            default_timeout,
        }
    }

    /// Execute a SQL query with default timeout
    ///
    /// # Arguments
    /// * `sql` - SQL query string
    ///
    /// # Returns
    /// Query execution result containing schema and data
    ///
    /// # Errors
    /// Returns error if parsing fails, execution fails, or timeout occurs
    pub async fn execute_query(&self, sql: &str) -> Result<QueryExecutionResult> {
        self.execute_query_with_timeout(sql, self.default_timeout).await
    }

    /// Execute a SQL query with custom timeout
    ///
    /// # Arguments
    /// * `sql` - SQL query string
    /// * `timeout_duration` - Maximum time to wait for query completion
    ///
    /// # Returns
    /// Query execution result containing schema and data
    ///
    /// # Errors
    /// Returns error if parsing fails, execution fails, or timeout occurs
    pub async fn execute_query_with_timeout(
        &self,
        sql: &str,
        timeout_duration: Duration,
    ) -> Result<QueryExecutionResult> {
        let start_time = std::time::Instant::now();

        // Wrap the entire execution in a timeout
        let result = timeout(timeout_duration, async {
            // 1. Parse SQL and create logical plan
            let logical_plan = self.parse_sql(sql).await?;

            // 2. Execute the query
            let batches = self.execute_plan(&logical_plan).await?;

            Ok::<_, anyhow::Error>(batches)
        })
        .await
        .map_err(|_| anyhow!("Query execution timeout after {:?}", timeout_duration))??;

        let execution_time = start_time.elapsed();

        // Extract schema from first batch or create empty schema
        let schema = if let Some(first_batch) = result.first() {
            first_batch.schema()
        } else {
            // Empty result set - create empty schema
            Arc::new(datafusion::arrow::datatypes::Schema::empty())
        };

        Ok(QueryExecutionResult::from_batches(
            schema,
            result,
            execution_time.as_millis(),
        ))
    }

    /// Parse SQL query into a logical plan
    ///
    /// # Arguments
    /// * `sql` - SQL query string
    ///
    /// # Returns
    /// DataFusion logical plan
    ///
    /// # Errors
    /// Returns error if SQL parsing fails
    async fn parse_sql(&self, sql: &str) -> Result<LogicalPlan> {
        let df = self
            .ctx
            .sql(sql)
            .await
            .context("Failed to parse SQL query")?;

        Ok(df.logical_plan().clone())
    }

    /// Execute a logical plan and collect results
    ///
    /// # Arguments
    /// * `plan` - DataFusion logical plan
    ///
    /// # Returns
    /// Vector of record batches containing query results
    ///
    /// # Errors
    /// Returns error if execution fails
    async fn execute_plan(&self, plan: &LogicalPlan) -> Result<Vec<RecordBatch>> {
        // Create DataFrame from logical plan
        let df = DataFrame::new(self.ctx.state(), plan.clone());

        // Execute and collect results
        let batches = df
            .collect()
            .await
            .context("Failed to execute query plan")?;

        Ok(batches)
    }

    /// Explain query execution plan
    ///
    /// Returns the logical and physical execution plans for a query.
    /// Useful for debugging and optimization.
    ///
    /// # Arguments
    /// * `sql` - SQL query string
    /// * `analyze` - If true, actually execute the query and include runtime metrics
    ///
    /// # Returns
    /// Query plan as string
    pub async fn explain_query(&self, sql: &str, analyze: bool) -> Result<String> {
        let explain_sql = if analyze {
            format!("EXPLAIN ANALYZE {}", sql)
        } else {
            format!("EXPLAIN {}", sql)
        };

        let result = self.execute_query(&explain_sql).await?;

        // Convert result batches to string representation
        let mut output = String::new();
        for batch in &result.batches {
            output.push_str(&format!("{:?}\n", batch));
        }

        Ok(output)
    }

    /// Get the session context
    pub fn session_context(&self) -> &SessionContext {
        &self.ctx
    }

    /// Update default timeout
    pub fn set_default_timeout(&mut self, timeout: Duration) {
        self.default_timeout = timeout;
    }

    /// Get current default timeout
    pub fn default_timeout(&self) -> Duration {
        self.default_timeout
    }
}

/// Builder for QueryExecutor
pub struct QueryExecutorBuilder {
    ctx: Option<SessionContext>,
    timeout: Duration,
}

impl QueryExecutorBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            ctx: None,
            timeout: Duration::from_secs(30), // Default 30 seconds
        }
    }

    /// Set the session context
    pub fn with_context(mut self, ctx: SessionContext) -> Self {
        self.ctx = Some(ctx);
        self
    }

    /// Set the default timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Build the QueryExecutor
    pub fn build(self) -> Result<DataFusionQueryExecutor> {
        let ctx = self.ctx.ok_or_else(|| anyhow!("SessionContext is required"))?;

        Ok(DataFusionQueryExecutor::new(ctx, self.timeout))
    }
}

impl Default for QueryExecutorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_executor_creation() {
        let ctx = SessionContext::new();
        let executor = DataFusionQueryExecutor::new(ctx, Duration::from_secs(30));

        assert_eq!(executor.default_timeout(), Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_query_executor_builder() {
        let ctx = SessionContext::new();
        let executor = QueryExecutorBuilder::new()
            .with_context(ctx)
            .with_timeout(Duration::from_secs(60))
            .build();

        assert!(executor.is_ok());
        let executor = executor.unwrap();
        assert_eq!(executor.default_timeout(), Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_simple_query_execution() {
        let ctx = SessionContext::new();
        let executor = DataFusionQueryExecutor::new(ctx, Duration::from_secs(30));

        // Simple query that should always work
        let sql = "SELECT 1 as num, 'hello' as text";
        let result = executor.execute_query(sql).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.row_count, 1);
        assert_eq!(result.batches.len(), 1);
    }

    #[tokio::test]
    async fn test_query_timeout() {
        let ctx = SessionContext::new();
        let executor = DataFusionQueryExecutor::new(ctx, Duration::from_millis(1));

        // This query should timeout with very short timeout
        let sql = "SELECT 1";
        let result = executor.execute_query(sql).await;

        // May or may not timeout depending on system speed, but should not crash
        match result {
            Ok(_) => {}, // Query was fast enough
            Err(e) => {
                // Should be a timeout error
                assert!(e.to_string().contains("timeout") || e.to_string().contains("Timeout"));
            }
        }
    }

    #[tokio::test]
    async fn test_explain_query() {
        let ctx = SessionContext::new();
        let executor = DataFusionQueryExecutor::new(ctx, Duration::from_secs(30));

        let sql = "SELECT 1 as num";
        let result = executor.explain_query(sql, false).await;

        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(!plan.is_empty());
    }
}
