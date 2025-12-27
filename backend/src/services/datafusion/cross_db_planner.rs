// Cross-Database Query Planner
//
// Parses cross-database SQL queries and creates execution plans for federated query execution.

use crate::api::middleware::AppError;
use crate::models::cross_database_query::{
    CrossDatabaseExecutionPlan, CrossDatabaseQueryRequest, JoinCondition, MergeStrategy, SubQuery,
};
use sqlparser::ast::{Statement, TableFactor, SetExpr};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use std::collections::HashMap;

/// Cross-Database Query Planner
///
/// Parses cross-database SQL queries and generates execution plans.
pub struct CrossDatabaseQueryPlanner {
    /// Map of table qualifiers to connection IDs
    /// Example: {"mysql_conn" => "connection-id-1", "pg_conn" => "connection-id-2"}
    connection_map: HashMap<String, String>,
}

impl CrossDatabaseQueryPlanner {
    /// Create a new planner with connection mappings
    ///
    /// # Arguments
    ///
    /// * `connection_ids` - List of connection IDs involved in the query
    ///
    /// Uses connection IDs as qualifiers (not recommended for UUIDs)
    pub fn new(connection_ids: Vec<String>) -> Self {
        let mut connection_map = HashMap::new();

        // Use connection ID itself as the qualifier
        for id in connection_ids {
            connection_map.insert(id.clone(), id);
        }

        Self { connection_map }
    }

    /// Create a new planner with custom aliases
    ///
    /// # Arguments
    ///
    /// * `aliases` - Map of aliases to connection IDs (e.g., {"db1": "uuid-1", "db2": "uuid-2"})
    ///
    /// Recommended for better query readability
    pub fn with_aliases(aliases: HashMap<String, String>) -> Self {
        Self {
            connection_map: aliases,
        }
    }

    /// Create a new planner from a request
    ///
    /// Uses aliases if provided, otherwise falls back to connection IDs
    pub fn from_request(request: &CrossDatabaseQueryRequest) -> Self {
        if let Some(ref aliases) = request.database_aliases {
            Self::with_aliases(aliases.clone())
        } else {
            Self::new(request.connection_ids.clone())
        }
    }

    /// Plan a cross-database query
    ///
    /// Parses the query, identifies tables and their sources, decomposes into sub-queries,
    /// and generates an execution plan.
    pub fn plan_query(
        &self,
        request: &CrossDatabaseQueryRequest,
    ) -> Result<CrossDatabaseExecutionPlan, AppError> {
        // Validate request
        request.validate()
            .map_err(|e| AppError::Validation(e))?;

        // Parse SQL query
        let statements = Parser::parse_sql(&GenericDialect {}, &request.query)
            .map_err(|e| AppError::InvalidSql(format!("Failed to parse query: {}", e)))?;

        if statements.is_empty() {
            return Err(AppError::InvalidSql("Empty query".to_string()));
        }

        // Only support SELECT queries
        let statement = &statements[0];
        match statement {
            Statement::Query(query) => {
                self.plan_select_query(query, request)
            }
            _ => Err(AppError::InvalidSql(
                "Only SELECT queries are supported for cross-database execution".to_string(),
            )),
        }
    }

    /// Plan a SELECT query
    fn plan_select_query(
        &self,
        query: &sqlparser::ast::Query,
        request: &CrossDatabaseQueryRequest,
    ) -> Result<CrossDatabaseExecutionPlan, AppError> {
        // Check for UNION queries
        if let SetExpr::SetOperation { op, .. } = &*query.body {
            return self.plan_union_query(query, request, op);
        }

        // Regular SELECT query - check for JOINs
        if let SetExpr::Select(select) = &*query.body {
            let tables = self.extract_tables(select)?;

            if tables.is_empty() {
                return Err(AppError::InvalidSql("No tables found in query".to_string()));
            }

            // Check if query involves multiple databases
            let databases = self.identify_databases(&tables)?;

            if databases.len() == 1 {
                // Single database query - no cross-database execution needed
                return self.plan_single_database_query(&tables, request);
            }

            // Multiple databases - check for JOINs
            if select.from.len() > 0 && !select.from[0].joins.is_empty() {
                return self.plan_join_query(select, &tables, request);
            }

            // Multiple tables but no explicit JOIN - may still need federation
            return self.plan_join_query(select, &tables, request);
        }

        Err(AppError::InvalidSql(
            "Unsupported query structure for cross-database execution".to_string(),
        ))
    }

    /// Extract table references from a SELECT statement
    fn extract_tables(&self, select: &sqlparser::ast::Select) -> Result<Vec<(String, Option<String>, String)>, AppError> {
        let mut tables = Vec::new();

        for table_with_joins in &select.from {
            // Main table
            if let TableFactor::Table { name, alias, .. } = &table_with_joins.relation {
                let (qualifier, table_name) = self.parse_table_name(name)?;
                let alias_name = alias.as_ref().map(|a| a.name.value.clone());
                tables.push((qualifier, alias_name, table_name));
            }

            // Joined tables
            for join in &table_with_joins.joins {
                if let TableFactor::Table { name, alias, .. } = &join.relation {
                    let (qualifier, table_name) = self.parse_table_name(name)?;
                    let alias_name = alias.as_ref().map(|a| a.name.value.clone());
                    tables.push((qualifier, alias_name, table_name));
                }
            }
        }

        Ok(tables)
    }

    /// Parse table name to extract qualifier (connection ID) and table name
    ///
    /// Examples:
    /// - "mysql_conn.users" => ("mysql_conn", "users")
    /// - "users" => (first_connection_id, "users")
    fn parse_table_name(
        &self,
        name: &sqlparser::ast::ObjectName,
    ) -> Result<(String, String), AppError> {
        let parts: Vec<String> = name.0.iter().map(|i| i.to_string()).collect();

        match parts.len() {
            1 => {
                // Unqualified table name - use first connection ID
                let first_conn_id = self.connection_map.keys().next()
                    .ok_or_else(|| AppError::Validation("No connection IDs available".to_string()))?
                    .clone();
                Ok((first_conn_id, parts[0].clone()))
            }
            2 => {
                // Qualified table name: qualifier.table
                let qualifier = &parts[0];
                let table_name = &parts[1];

                // Verify qualifier exists in connection map
                if !self.connection_map.contains_key(qualifier) {
                    return Err(AppError::Validation(format!(
                        "Unknown database qualifier '{}'. Available: {:?}",
                        qualifier,
                        self.connection_map.keys().collect::<Vec<_>>()
                    )));
                }

                Ok((qualifier.clone(), table_name.clone()))
            }
            _ => Err(AppError::InvalidSql(format!(
                "Invalid table name format: {}. Expected 'table' or 'db.table'",
                parts.join(".")
            ))),
        }
    }

    /// Identify which databases are involved in the query
    fn identify_databases(
        &self,
        tables: &[(String, Option<String>, String)],
    ) -> Result<HashMap<String, Vec<String>>, AppError> {
        let mut databases: HashMap<String, Vec<String>> = HashMap::new();

        for (qualifier, _alias, table_name) in tables {
            let conn_id = self.connection_map.get(qualifier)
                .ok_or_else(|| AppError::Validation(format!("Unknown qualifier: {}", qualifier)))?;

            databases
                .entry(conn_id.clone())
                .or_insert_with(Vec::new)
                .push(table_name.clone());
        }

        Ok(databases)
    }

    /// Plan a single database query (no cross-database execution needed)
    fn plan_single_database_query(
        &self,
        tables: &[(String, Option<String>, String)],
        request: &CrossDatabaseQueryRequest,
    ) -> Result<CrossDatabaseExecutionPlan, AppError> {
        let (qualifier, _alias, _table_name) = &tables[0];
        let connection_id = self.connection_map.get(qualifier)
            .ok_or_else(|| AppError::Validation(format!("Unknown qualifier: {}", qualifier)))?
            .clone();

        // Strip qualifiers from query for single-database execution
        let query_without_qualifiers = self.strip_qualifiers(&request.query)?;

        let sub_query = SubQuery {
            connection_id: connection_id.clone(),
            database_type: "unknown".to_string(), // Will be filled by executor
            query: query_without_qualifiers,
            tables: tables.iter().map(|(_, _, t)| t.clone()).collect(),
            result_alias: "result".to_string(),
        };

        Ok(CrossDatabaseExecutionPlan {
            original_query: request.query.clone(),
            sub_queries: vec![sub_query],
            merge_strategy: MergeStrategy::None,
            timeout_secs: request.timeout_secs.unwrap_or(60),
            apply_limit: request.apply_limit.unwrap_or(true),
            limit_value: request.limit_value.unwrap_or(1000),
        })
    }

    /// Plan a JOIN query across databases
    fn plan_join_query(
        &self,
        select: &sqlparser::ast::Select,
        tables: &[(String, Option<String>, String)],
        request: &CrossDatabaseQueryRequest,
    ) -> Result<CrossDatabaseExecutionPlan, AppError> {
        let databases = self.identify_databases(tables)?;

        // Extract JOIN conditions from the SQL
        let join_conditions = self.extract_join_conditions(select, tables)?;

        // Build a map of table alias to connection ID
        let mut table_to_conn: HashMap<String, String> = HashMap::new();
        for (qualifier, table_alias, table_name) in tables {
            let conn_id = self.connection_map.get(qualifier).unwrap().clone();
            let key = table_alias.clone().unwrap_or_else(|| table_name.clone());
            table_to_conn.insert(key, conn_id);
        }

        // Generate sub-queries for each database
        let mut sub_queries = Vec::new();
        for (conn_id, table_names) in databases {
            // For cross-database JOINs, we need to fetch all data from each table
            // TODO: Implement predicate pushdown for optimization
            let query = format!("SELECT * FROM {}", table_names.join(", "));

            // Find the table alias for this sub-query
            let result_alias = tables
                .iter()
                .find(|(q, _, t)| {
                    let cid = self.connection_map.get(q).unwrap();
                    cid == &conn_id && table_names.contains(t)
                })
                .and_then(|(_, a, t)| a.clone().or_else(|| Some(t.clone())))
                .unwrap_or_else(|| format!("result_{}", conn_id));

            sub_queries.push(SubQuery {
                connection_id: conn_id.clone(),
                database_type: "unknown".to_string(),
                query,
                tables: table_names,
                result_alias,
            });
        }

        // Determine merge strategy based on join type
        let merge_strategy = if !join_conditions.is_empty() {
            MergeStrategy::InnerJoin {
                conditions: join_conditions,
            }
        } else {
            // If no join conditions found, try to infer from query
            tracing::warn!("No explicit JOIN conditions found, using placeholder");
            MergeStrategy::InnerJoin { conditions: vec![] }
        };

        Ok(CrossDatabaseExecutionPlan {
            original_query: request.query.clone(),
            sub_queries,
            merge_strategy,
            timeout_secs: request.timeout_secs.unwrap_or(60),
            apply_limit: request.apply_limit.unwrap_or(true),
            limit_value: request.limit_value.unwrap_or(1000),
        })
    }

    /// Plan a UNION query across databases
    fn plan_union_query(
        &self,
        query: &sqlparser::ast::Query,
        request: &CrossDatabaseQueryRequest,
        op: &sqlparser::ast::SetOperator,
    ) -> Result<CrossDatabaseExecutionPlan, AppError> {
        use sqlparser::ast::SetOperator;

        // Determine if UNION ALL or UNION (distinct)
        let is_union_all = matches!(op, SetOperator::Union);

        // Extract individual SELECT queries from UNION
        let select_queries = self.extract_union_selects(query)?;

        if select_queries.is_empty() {
            return Err(AppError::InvalidSql("No SELECT statements found in UNION".to_string()));
        }

        // Generate sub-queries for each SELECT in the UNION
        let mut sub_queries = Vec::new();

        for (idx, select_sql) in select_queries.iter().enumerate() {
            // Parse each SELECT to identify tables
            let parsed = Parser::parse_sql(&GenericDialect {}, select_sql)
                .map_err(|e| AppError::InvalidSql(format!("Failed to parse UNION SELECT: {}", e)))?;

            if let Some(Statement::Query(q)) = parsed.first() {
                if let SetExpr::Select(select) = &*q.body {
                    let tables = self.extract_tables(select)?;

                    if tables.is_empty() {
                        continue;
                    }

                    // Get connection ID for this query
                    let (qualifier, _, _) = &tables[0];
                    let connection_id = self.connection_map.get(qualifier)
                        .ok_or_else(|| AppError::Validation(format!("Unknown qualifier: {}", qualifier)))?
                        .clone();

                    // Strip qualifiers from SELECT SQL
                    let stripped_sql = self.strip_qualifiers(select_sql)?;

                    sub_queries.push(SubQuery {
                        connection_id: connection_id.clone(),
                        database_type: "unknown".to_string(),
                        query: stripped_sql,
                        tables: tables.iter().map(|(_, _, t)| t.clone()).collect(),
                        result_alias: format!("union_part_{}", idx),
                    });
                }
            }
        }

        if sub_queries.is_empty() {
            return Err(AppError::InvalidSql("No valid sub-queries found in UNION".to_string()));
        }

        let merge_strategy = MergeStrategy::Union { all: is_union_all };

        Ok(CrossDatabaseExecutionPlan {
            original_query: request.query.clone(),
            sub_queries,
            merge_strategy,
            timeout_secs: request.timeout_secs.unwrap_or(60),
            apply_limit: request.apply_limit.unwrap_or(true),
            limit_value: request.limit_value.unwrap_or(1000),
        })
    }

    /// Extract individual SELECT statements from a UNION query
    fn extract_union_selects(&self, query: &sqlparser::ast::Query) -> Result<Vec<String>, AppError> {
        let mut selects: Vec<String> = Vec::new();

        // Traverse the SetOperation tree to extract all SELECT statements
        self.extract_set_operation_selects(&query.body, &mut selects)?;

        if selects.is_empty() {
            return Err(AppError::InvalidSql("No SELECT statements found in UNION".to_string()));
        }

        tracing::debug!("Extracted {} SELECT statements from UNION query", selects.len());
        Ok(selects)
    }

    /// Recursively extract SELECT statements from a SetOperation tree
    fn extract_set_operation_selects(
        &self,
        set_expr: &SetExpr,
        selects: &mut Vec<String>,
    ) -> Result<(), AppError> {
        match set_expr {
            SetExpr::Select(select) => {
                // Base case: this is a SELECT statement
                selects.push(format!("{}", select));
            }
            SetExpr::SetOperation { left, right, .. } => {
                // Recursive case: process both sides of the UNION
                self.extract_set_operation_selects(left, selects)?;
                self.extract_set_operation_selects(right, selects)?;
            }
            SetExpr::Query(query) => {
                // Nested query
                self.extract_set_operation_selects(&query.body, selects)?;
            }
            _ => {
                tracing::warn!("Unsupported SetExpr type in UNION decomposition");
            }
        }
        Ok(())
    }

    /// Generate a sub-query for a specific database
    fn generate_sub_query(&self, tables: &[String]) -> Result<String, AppError> {
        // For now, generate a simple SELECT * from each table
        // TODO: Implement proper query decomposition

        if tables.len() == 1 {
            Ok(format!("SELECT * FROM {}", tables[0]))
        } else {
            // Multiple tables - need to handle joins
            // For now, just select from first table
            Ok(format!("SELECT * FROM {}", tables[0]))
        }
    }

    /// Strip database qualifiers from a SQL query
    ///
    /// Converts "SELECT * FROM db1.users" to "SELECT * FROM users"
    /// This is needed when executing single-database queries
    fn strip_qualifiers(&self, sql: &str) -> Result<String, AppError> {
        // Parse the SQL query
        let statements = Parser::parse_sql(&GenericDialect {}, sql)
            .map_err(|e| AppError::InvalidSql(format!("Failed to parse query: {}", e)))?;

        if statements.is_empty() {
            return Err(AppError::InvalidSql("Empty query".to_string()));
        }

        // For now, use a simple string replacement approach
        // TODO: Implement proper AST rewriting for complex queries
        let mut result = sql.to_string();

        // Replace all qualified table names with unqualified names
        for (alias, _conn_id) in &self.connection_map {
            // Replace "alias.table" with "table"
            let pattern = format!("{}.", alias);
            result = result.replace(&pattern, "");
        }

        Ok(result)
    }

    /// Extract JOIN conditions from SELECT statement
    ///
    /// Parses JOIN ON clauses and converts them to JoinCondition structures
    /// ENHANCEMENT: Now supports multiple AND conditions in JOIN ON clauses
    fn extract_join_conditions(
        &self,
        select: &sqlparser::ast::Select,
        tables: &[(String, Option<String>, String)],
    ) -> Result<Vec<JoinCondition>, AppError> {
        use sqlparser::ast::{JoinConstraint, JoinOperator};

        let mut conditions = Vec::new();

        // Build a map of table names to their aliases for quick lookup
        let mut table_aliases: HashMap<String, String> = HashMap::new();
        for (_, alias, table_name) in tables {
            if let Some(ref a) = alias {
                table_aliases.insert(table_name.clone(), a.clone());
            } else {
                table_aliases.insert(table_name.clone(), table_name.clone());
            }
        }

        // Iterate through FROM clause tables with joins
        for table_with_joins in &select.from {
            for join in &table_with_joins.joins {
                // Extract join conditions based on join constraint
                match &join.join_operator {
                    JoinOperator::Inner(constraint) | JoinOperator::LeftOuter(constraint) | JoinOperator::RightOuter(constraint) => {
                        if let JoinConstraint::On(expr) = constraint {
                            // Parse the ON expression to extract ALL conditions (including AND chains)
                            self.parse_all_join_conditions(expr, &table_aliases, &mut conditions)?;
                        }
                    }
                    _ => {
                        tracing::debug!("Unsupported join type, skipping");
                    }
                }
            }
        }

        tracing::debug!("Extracted {} JOIN conditions", conditions.len());
        Ok(conditions)
    }

    /// Parse a JOIN expression to extract join conditions
    ///
    /// Handles expressions like: table1.col1 = table2.col2
    /// Also supports multiple conditions: table1.col1 = table2.col2 AND table1.col3 = table2.col4
    fn parse_join_expr(
        &self,
        expr: &sqlparser::ast::Expr,
        table_aliases: &HashMap<String, String>,
    ) -> Result<Option<JoinCondition>, AppError> {
        use sqlparser::ast::{BinaryOperator, Expr};

        match expr {
            Expr::BinaryOp { left, op, right } => {
                // Handle equality conditions: col1 = col2
                if matches!(op, BinaryOperator::Eq) {
                    // Extract left side (table.column)
                    if let (Some((left_table, left_col)), Some((right_table, right_col))) = (
                        self.extract_table_column(left, table_aliases)?,
                        self.extract_table_column(right, table_aliases)?,
                    ) {
                        return Ok(Some(JoinCondition {
                            left_alias: left_table,
                            left_column: left_col,
                            right_alias: right_table,
                            right_column: right_col,
                        }));
                    }
                }

                // Handle AND conditions: recursively extract both sides
                // ENHANCEMENT: Now returns the first valid condition found
                // Multiple conditions are extracted in extract_join_conditions
                if matches!(op, BinaryOperator::And) {
                    return self.parse_join_expr(left, table_aliases);
                }
            }
            _ => {
                tracing::debug!("Unsupported join expression type");
            }
        }

        Ok(None)
    }

    /// Parse all JOIN conditions from an expression (including AND chains)
    ///
    /// Extracts all equality conditions connected by AND
    fn parse_all_join_conditions(
        &self,
        expr: &sqlparser::ast::Expr,
        table_aliases: &HashMap<String, String>,
        conditions: &mut Vec<JoinCondition>,
    ) -> Result<(), AppError> {
        use sqlparser::ast::{BinaryOperator, Expr};

        match expr {
            Expr::BinaryOp { left, op, right } => {
                if matches!(op, BinaryOperator::And) {
                    // Recursively process both sides of AND
                    self.parse_all_join_conditions(left, table_aliases, conditions)?;
                    self.parse_all_join_conditions(right, table_aliases, conditions)?;
                } else if matches!(op, BinaryOperator::Eq) {
                    // Extract equality condition
                    if let (Some((left_table, left_col)), Some((right_table, right_col))) = (
                        self.extract_table_column(left, table_aliases)?,
                        self.extract_table_column(right, table_aliases)?,
                    ) {
                        conditions.push(JoinCondition {
                            left_alias: left_table,
                            left_column: left_col,
                            right_alias: right_table,
                            right_column: right_col,
                        });
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Extract table and column names from an expression
    ///
    /// Handles: table.column or column
    fn extract_table_column(
        &self,
        expr: &sqlparser::ast::Expr,
        table_aliases: &HashMap<String, String>,
    ) -> Result<Option<(String, String)>, AppError> {
        use sqlparser::ast::Expr;

        match expr {
            Expr::CompoundIdentifier(idents) if idents.len() == 2 => {
                // table.column format
                let table = idents[0].value.clone();
                let column = idents[1].value.clone();

                // Resolve table alias
                let resolved_table = table_aliases.get(&table).cloned().unwrap_or(table);

                Ok(Some((resolved_table, column)))
            }
            Expr::Identifier(ident) => {
                // Just column name - we'll need context to determine the table
                // For now, return None
                tracing::debug!("Found unqualified column: {}", ident.value);
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_database_query() {
        let conn_ids = vec!["conn1".to_string()];
        let planner = CrossDatabaseQueryPlanner::new(conn_ids);

        let request = CrossDatabaseQueryRequest::new(
            "SELECT * FROM conn1.users".to_string(),
            vec!["conn1".to_string()],
        );

        let plan = planner.plan_query(&request).unwrap();

        assert_eq!(plan.sub_queries.len(), 1);
        assert!(matches!(plan.merge_strategy, MergeStrategy::None));
    }

    #[test]
    fn test_cross_database_join() {
        let conn_ids = vec!["conn1".to_string(), "conn2".to_string()];
        let planner = CrossDatabaseQueryPlanner::new(conn_ids);

        let request = CrossDatabaseQueryRequest::new(
            "SELECT u.id, t.title FROM conn1.users u JOIN conn2.todos t ON u.id = t.user_id".to_string(),
            vec!["conn1".to_string(), "conn2".to_string()],
        );

        let plan = planner.plan_query(&request).unwrap();

        assert_eq!(plan.sub_queries.len(), 2);
        assert!(matches!(plan.merge_strategy, MergeStrategy::InnerJoin { .. }));
    }

    #[test]
    fn test_invalid_qualifier() {
        let conn_ids = vec!["conn1".to_string()];
        let planner = CrossDatabaseQueryPlanner::new(conn_ids);

        let request = CrossDatabaseQueryRequest::new(
            "SELECT * FROM unknown.users".to_string(),
            vec!["conn1".to_string()],
        );

        let result = planner.plan_query(&request);
        assert!(result.is_err());
    }
}
