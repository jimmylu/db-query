// DataFusion Federated Executor
//
// Executes cross-database queries by coordinating sub-queries across multiple databases
// and merging results using DataFusion's in-memory execution engine.

use crate::api::middleware::AppError;
use crate::models::cross_database_query::{
    CrossDatabaseExecutionPlan, CrossDatabaseQueryResponse, MergeStrategy, SubQuery,
    SubQueryExecution,
};
use crate::services::database::adapter::DatabaseAdapter;
use crate::services::datafusion::{DataFusionSessionManager, SessionConfig};
use datafusion::arrow::array::{ArrayRef, RecordBatch, StringArray, Int64Array, Float64Array, Array};
use datafusion::arrow::datatypes::{Schema, Field, DataType};
use datafusion::execution::context::SessionContext;
use datafusion::prelude::*;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::timeout;
use std::time::Duration;

/// Result from executing a sub-query
struct SubQueryResult {
    connection_id: String,
    database_type: String,
    query: String,
    rows: Vec<serde_json::Value>,
    execution_time_ms: u128,
}

/// DataFusion Federated Executor
///
/// Executes cross-database queries by:
/// 1. Running sub-queries in parallel against each database
/// 2. Converting results to Arrow RecordBatches
/// 3. Merging results using DataFusion's join/union operators
pub struct DataFusionFederatedExecutor {
    session_manager: DataFusionSessionManager,
}

impl DataFusionFederatedExecutor {
    /// Create a new federated executor
    pub fn new() -> Self {
        Self {
            session_manager: DataFusionSessionManager::new(SessionConfig::default()),
        }
    }

    /// Execute a cross-database query
    ///
    /// # Arguments
    ///
    /// * `plan` - Execution plan from the planner
    /// * `adapters` - Map of connection IDs to database adapters
    ///
    /// # Returns
    ///
    /// Complete query response with merged results
    pub async fn execute_cross_database_query(
        &self,
        plan: CrossDatabaseExecutionPlan,
        adapters: std::collections::HashMap<String, Box<dyn DatabaseAdapter>>,
    ) -> Result<CrossDatabaseQueryResponse, AppError> {
        let start_time = Instant::now();

        // Validate that we have adapters for all connections
        for sub_query in &plan.sub_queries {
            if !adapters.contains_key(&sub_query.connection_id) {
                return Err(AppError::Validation(format!(
                    "No adapter found for connection: {}",
                    sub_query.connection_id
                )));
            }
        }

        // Execute sub-queries in parallel
        let sub_results = self.execute_sub_queries_parallel(plan.sub_queries.clone(), adapters, plan.timeout_secs).await?;

        // Merge results based on strategy
        let merged_results = match plan.merge_strategy {
            MergeStrategy::None => {
                // Single database query - return results directly
                if sub_results.is_empty() {
                    vec![]
                } else {
                    sub_results[0].rows.clone()
                }
            }
            MergeStrategy::InnerJoin { ref conditions } => {
                self.merge_with_join(&sub_results, conditions, plan.apply_limit, plan.limit_value).await?
            }
            MergeStrategy::LeftJoin { ref conditions } => {
                self.merge_with_join(&sub_results, conditions, plan.apply_limit, plan.limit_value).await?
            }
            MergeStrategy::Union { all } => {
                self.merge_with_union(&sub_results, all, plan.apply_limit, plan.limit_value).await?
            }
        };

        let execution_time_ms = start_time.elapsed().as_millis();

        // Build response
        let sub_query_executions: Vec<SubQueryExecution> = sub_results
            .iter()
            .map(|r| SubQueryExecution {
                connection_id: r.connection_id.clone(),
                database_type: r.database_type.clone(),
                query: r.query.clone(),
                row_count: r.rows.len(),
                execution_time_ms: r.execution_time_ms,
            })
            .collect();

        Ok(CrossDatabaseQueryResponse::new(
            plan.original_query,
            sub_query_executions,
            merged_results,
            execution_time_ms,
            plan.apply_limit,
        ))
    }

    /// Execute sub-queries in parallel
    async fn execute_sub_queries_parallel(
        &self,
        sub_queries: Vec<SubQuery>,
        adapters: std::collections::HashMap<String, Box<dyn DatabaseAdapter>>,
        timeout_secs: u64,
    ) -> Result<Vec<SubQueryResult>, AppError> {
        let mut tasks = Vec::new();

        for sub_query in sub_queries {
            let adapter = adapters.get(&sub_query.connection_id)
                .ok_or_else(|| AppError::Validation(format!("Adapter not found: {}", sub_query.connection_id)))?;

            // Clone necessary data for the task
            let conn_id = sub_query.connection_id.clone();
            let db_type = adapter.database_type().to_string(); // Get from adapter instead of sub_query
            let query = sub_query.query.clone();

            let start = Instant::now();

            // Execute the query with timeout
            let query_result = timeout(
                Duration::from_secs(timeout_secs),
                adapter.execute_query(&query, timeout_secs)
            )
            .await
            .map_err(|_| AppError::Database(format!("Sub-query timeout after {} seconds", timeout_secs)))?
            .map_err(|e| AppError::Database(format!("Sub-query execution failed: {}", e)))?;

            let execution_time = start.elapsed().as_millis();

            let result = SubQueryResult {
                connection_id: conn_id,
                database_type: db_type,
                query,
                rows: query_result.rows,
                execution_time_ms: execution_time,
            };

            tasks.push(result);
        }

        Ok(tasks)
    }

    /// Merge results using JOIN
    async fn merge_with_join(
        &self,
        sub_results: &[SubQueryResult],
        conditions: &[crate::models::cross_database_query::JoinCondition],
        apply_limit: bool,
        limit_value: u32,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        if sub_results.len() < 2 {
            return Err(AppError::Validation("JOIN requires at least 2 sub-queries".to_string()));
        }

        // Create DataFusion session
        let ctx = self.session_manager.create_session()
            .map_err(|e| AppError::Database(format!("Failed to create DataFusion session: {}", e)))?;

        // Register each sub-result as a temporary table
        for (idx, result) in sub_results.iter().enumerate() {
            if result.rows.is_empty() {
                tracing::warn!("Sub-query {} returned no rows, JOIN may return empty result", idx);
                continue;
            }

            let batch = self.json_to_record_batch(&result.rows)?;
            let table_name = format!("table_{}", idx);

            ctx.register_batch(&table_name, batch)
                .map_err(|e| AppError::Database(format!("Failed to register table {}: {}", table_name, e)))?;

            tracing::debug!("Registered table_{} with {} rows", idx, result.rows.len());
        }

        // Build JOIN SQL
        let join_sql = if !conditions.is_empty() {
            // Use explicit JOIN conditions
            self.build_join_sql(conditions, sub_results.len())
        } else {
            // Fallback: simple Cartesian product for testing
            tracing::warn!("No JOIN conditions provided, using Cartesian product");
            self.build_cartesian_product_sql(sub_results.len())
        };

        tracing::info!("Executing JOIN SQL: {}", join_sql);

        // Execute JOIN query using DataFusion
        let df = ctx.sql(&join_sql).await
            .map_err(|e| AppError::Database(format!("Failed to execute JOIN SQL: {}", e)))?;

        // Apply limit in SQL if requested
        let df = if apply_limit {
            df.limit(0, Some(limit_value as usize))
                .map_err(|e| AppError::Database(format!("Failed to apply LIMIT: {}", e)))?
        } else {
            df
        };

        // Execute and collect results
        let batches = df.collect().await
            .map_err(|e| AppError::Database(format!("Failed to collect JOIN results: {}", e)))?;

        // Convert RecordBatch results back to JSON
        let mut results = Vec::new();
        for batch in batches {
            let json_rows = self.record_batch_to_json(&batch)?;
            results.extend(json_rows);
        }

        tracing::info!("JOIN produced {} rows", results.len());

        Ok(results)
    }

    /// Build JOIN SQL from JOIN conditions
    fn build_join_sql(&self, conditions: &[crate::models::cross_database_query::JoinCondition], table_count: usize) -> String {
        if conditions.is_empty() || table_count < 2 {
            return "SELECT * FROM table_0".to_string();
        }

        // For now, support simple 2-table JOIN
        // TODO: Support multi-table JOINs with multiple conditions
        let first_cond = &conditions[0];

        format!(
            "SELECT * FROM table_0 INNER JOIN table_1 ON table_0.{} = table_1.{}",
            first_cond.left_column, first_cond.right_column
        )
    }

    /// Build Cartesian product SQL (fallback when no conditions)
    fn build_cartesian_product_sql(&self, table_count: usize) -> String {
        if table_count < 2 {
            return "SELECT * FROM table_0".to_string();
        }

        let mut sql = "SELECT * FROM table_0".to_string();
        for i in 1..table_count {
            sql.push_str(&format!(", table_{}", i));
        }
        sql
    }

    /// Merge results using UNION
    async fn merge_with_union(
        &self,
        sub_results: &[SubQueryResult],
        _all: bool,
        apply_limit: bool,
        limit_value: u32,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        // Create DataFusion session
        let ctx = self.session_manager.create_session()
            .map_err(|e| AppError::Database(format!("Failed to create DataFusion session: {}", e)))?;

        // Convert each sub-result to RecordBatch
        let mut all_batches = Vec::new();

        for (idx, result) in sub_results.iter().enumerate() {
            let batch = self.json_to_record_batch(&result.rows)?;

            // Register as temporary table
            let table_name = format!("temp_table_{}", idx);
            ctx.register_batch(&table_name, batch)
                .map_err(|e| AppError::Database(format!("Failed to register table: {}", e)))?;

            all_batches.push(table_name);
        }

        // Build UNION query - each table needs SELECT * FROM
        let select_statements: Vec<String> = all_batches
            .iter()
            .map(|table_name| format!("SELECT * FROM {}", table_name))
            .collect();

        let union_operator = if _all { " UNION ALL " } else { " UNION " };
        let union_query = select_statements.join(union_operator);

        tracing::debug!("Executing UNION query: {}", union_query);

        // Execute UNION
        let df = ctx.sql(&union_query).await
            .map_err(|e| AppError::Database(format!("Failed to execute UNION: {}", e)))?;

        // Apply limit if requested
        let df = if apply_limit {
            df.limit(0, Some(limit_value as usize))
                .map_err(|e| AppError::Database(format!("Failed to apply LIMIT: {}", e)))?
        } else {
            df
        };

        // Collect results
        let batches = df.collect().await
            .map_err(|e| AppError::Database(format!("Failed to collect results: {}", e)))?;

        // Convert back to JSON
        self.record_batches_to_json(&batches)
    }

    /// Convert JSON rows to Arrow RecordBatch
    fn json_to_record_batch(&self, rows: &[serde_json::Value]) -> Result<RecordBatch, AppError> {
        if rows.is_empty() {
            return Ok(RecordBatch::new_empty(Arc::new(Schema::empty())));
        }

        // Infer schema from first row
        let first_row = &rows[0];
        let obj = first_row.as_object()
            .ok_or_else(|| AppError::Database("Expected JSON object".to_string()))?;

        let mut fields = Vec::new();
        let mut column_names = Vec::new();

        for (key, value) in obj {
            column_names.push(key.clone());

            let data_type = match value {
                serde_json::Value::Number(n) if n.is_i64() => DataType::Int64,
                serde_json::Value::Number(n) if n.is_f64() => DataType::Float64,
                serde_json::Value::String(_) => DataType::Utf8,
                serde_json::Value::Bool(_) => DataType::Boolean,
                _ => DataType::Utf8, // Default to string
            };

            fields.push(Field::new(key, data_type, true));
        }

        let schema = Arc::new(Schema::new(fields));

        // Build arrays for each column
        let mut arrays: Vec<ArrayRef> = Vec::new();

        for (field_idx, field) in schema.fields().iter().enumerate() {
            let column_name = &column_names[field_idx];

            match field.data_type() {
                DataType::Int64 => {
                    let values: Vec<Option<i64>> = rows.iter().map(|row| {
                        row.get(column_name)
                            .and_then(|v| v.as_i64())
                    }).collect();
                    arrays.push(Arc::new(Int64Array::from(values)) as ArrayRef);
                }
                DataType::Float64 => {
                    let values: Vec<Option<f64>> = rows.iter().map(|row| {
                        row.get(column_name)
                            .and_then(|v| v.as_f64())
                    }).collect();
                    arrays.push(Arc::new(Float64Array::from(values)) as ArrayRef);
                }
                DataType::Utf8 => {
                    let values: Vec<Option<String>> = rows.iter().map(|row| {
                        row.get(column_name)
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    }).collect();
                    arrays.push(Arc::new(StringArray::from(values)) as ArrayRef);
                }
                _ => {
                    // Default: convert to string
                    let values: Vec<Option<String>> = rows.iter().map(|row| {
                        row.get(column_name)
                            .map(|v| v.to_string())
                    }).collect();
                    arrays.push(Arc::new(StringArray::from(values)) as ArrayRef);
                }
            }
        }

        RecordBatch::try_new(schema, arrays)
            .map_err(|e| AppError::Database(format!("Failed to create RecordBatch: {}", e)))
    }

    /// Convert Arrow RecordBatches to JSON
    fn record_batches_to_json(&self, batches: &[RecordBatch]) -> Result<Vec<serde_json::Value>, AppError> {
        let mut results = Vec::new();

        for batch in batches {
            let schema = batch.schema();

            for row_idx in 0..batch.num_rows() {
                let mut row_obj = serde_json::Map::new();

                for (col_idx, field) in schema.fields().iter().enumerate() {
                    let column = batch.column(col_idx);
                    let column_name = field.name();

                    // Extract value based on type
                    let value = match field.data_type() {
                        DataType::Int64 => {
                            let array = column.as_any().downcast_ref::<Int64Array>()
                                .ok_or_else(|| AppError::Database("Type mismatch".to_string()))?;
                            if array.is_null(row_idx) {
                                serde_json::Value::Null
                            } else {
                                serde_json::json!(array.value(row_idx))
                            }
                        }
                        DataType::Float64 => {
                            let array = column.as_any().downcast_ref::<Float64Array>()
                                .ok_or_else(|| AppError::Database("Type mismatch".to_string()))?;
                            if array.is_null(row_idx) {
                                serde_json::Value::Null
                            } else {
                                serde_json::json!(array.value(row_idx))
                            }
                        }
                        DataType::Utf8 => {
                            let array = column.as_any().downcast_ref::<StringArray>()
                                .ok_or_else(|| AppError::Database("Type mismatch".to_string()))?;
                            if array.is_null(row_idx) {
                                serde_json::Value::Null
                            } else {
                                serde_json::json!(array.value(row_idx))
                            }
                        }
                        _ => serde_json::Value::Null,
                    };

                    row_obj.insert(column_name.clone(), value);
                }

                results.push(serde_json::Value::Object(row_obj));
            }
        }

        Ok(results)
    }

    /// Convert a single Arrow RecordBatch to JSON (convenience method)
    fn record_batch_to_json(&self, batch: &RecordBatch) -> Result<Vec<serde_json::Value>, AppError> {
        self.record_batches_to_json(&[batch.clone()])
    }
}

impl Default for DataFusionFederatedExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_to_record_batch() {
        let executor = DataFusionFederatedExecutor::new();

        let rows = vec![
            serde_json::json!({"id": 1, "name": "Alice"}),
            serde_json::json!({"id": 2, "name": "Bob"}),
        ];

        let batch = executor.json_to_record_batch(&rows).unwrap();

        assert_eq!(batch.num_rows(), 2);
        assert_eq!(batch.num_columns(), 2);
    }

    #[test]
    fn test_empty_json_to_record_batch() {
        let executor = DataFusionFederatedExecutor::new();
        let rows: Vec<serde_json::Value> = vec![];

        let batch = executor.json_to_record_batch(&rows).unwrap();

        assert_eq!(batch.num_rows(), 0);
    }
}
