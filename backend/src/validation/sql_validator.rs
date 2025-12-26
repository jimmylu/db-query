use sqlparser::ast::Statement;
use sqlparser::parser::Parser;
use sqlparser::dialect::PostgreSqlDialect;
use crate::api::middleware::AppError;

/// SQL validation service for ensuring queries are safe and valid
pub struct SqlValidator;

impl SqlValidator {
    /// Validate SQL query and ensure it's a SELECT statement
    pub fn validate_select_only(sql: &str) -> Result<String, AppError> {
        let dialect = PostgreSqlDialect {};
        let mut parser = Parser::new(&dialect).try_with_sql(sql)
            .map_err(|e| AppError::InvalidSql(format!("SQL parsing error: {}", e)))?;
        
        // Parse SQL statement
        let ast = parser.parse_statements()
            .map_err(|e| AppError::InvalidSql(format!("SQL parsing error: {}", e)))?;

        if ast.is_empty() {
            return Err(AppError::InvalidSql("Empty SQL query".to_string()));
        }

        // Check each statement
        for stmt in ast {
            match stmt {
                Statement::Query(_) => {
                    // Valid SELECT query
                }
                Statement::Insert { .. } => {
                    return Err(AppError::InvalidSql("INSERT statements are not allowed. Only SELECT queries are permitted.".to_string()));
                }
                Statement::Update { .. } => {
                    return Err(AppError::InvalidSql("UPDATE statements are not allowed. Only SELECT queries are permitted.".to_string()));
                }
                Statement::Delete { .. } => {
                    return Err(AppError::InvalidSql("DELETE statements are not allowed. Only SELECT queries are permitted.".to_string()));
                }
                Statement::Drop { .. } => {
                    return Err(AppError::InvalidSql("DROP statements are not allowed. Only SELECT queries are permitted.".to_string()));
                }
                Statement::CreateTable { .. } => {
                    return Err(AppError::InvalidSql("CREATE TABLE statements are not allowed. Only SELECT queries are permitted.".to_string()));
                }
                Statement::AlterTable { .. } => {
                    return Err(AppError::InvalidSql("ALTER TABLE statements are not allowed. Only SELECT queries are permitted.".to_string()));
                }
                _ => {
                    return Err(AppError::InvalidSql(format!("Only SELECT queries are permitted. Found: {:?}", stmt)));
                }
            }
        }

        Ok(sql.to_string())
    }

    /// Check if query has LIMIT clause and append if missing
    /// Uses AST parsing to properly detect LIMIT clauses, avoiding false positives
    pub fn ensure_limit(sql: &str, default_limit: u64) -> Result<String, AppError> {
        let dialect = PostgreSqlDialect {};
        let mut parser = Parser::new(&dialect).try_with_sql(sql)
            .map_err(|e| AppError::InvalidSql(format!("SQL parsing error: {}", e)))?;

        // Parse SQL statement
        let ast = parser.parse_statements()
            .map_err(|e| AppError::InvalidSql(format!("SQL parsing error: {}", e)))?;

        if ast.is_empty() {
            return Err(AppError::InvalidSql("Empty SQL query".to_string()));
        }

        // Get the first statement (we already validated it's a Query in validate_select_only)
        let stmt = &ast[0];

        // Check if LIMIT exists by parsing the AST
        let has_limit = Self::check_limit_in_statement(stmt);

        if has_limit {
            // LIMIT already exists, return original SQL
            Ok(sql.to_string())
        } else {
            // Append LIMIT clause
            let trimmed_sql = sql.trim_end_matches(';').trim();
            Ok(format!("{} LIMIT {}", trimmed_sql, default_limit))
        }
    }

    /// Check if a statement has a LIMIT clause using AST analysis
    fn check_limit_in_statement(stmt: &Statement) -> bool {
        match stmt {
            Statement::Query(query) => {
                // Check if the query has a LIMIT clause
                // Query has body, order_by, limit_clause, offset, etc
                // The limit_clause field is Option<LimitClause>
                query.limit_clause.is_some()
            }
            _ => false,
        }
    }

    /// Validate and prepare SQL query (validate SELECT-only and ensure LIMIT)
    pub fn validate_and_prepare(sql: &str, default_limit: u64) -> Result<(String, bool), AppError> {
        // First validate it's SELECT-only
        let validated_sql = Self::validate_select_only(sql)?;
        
        // Check if LIMIT was already present
        let original_has_limit = Self::has_limit(&validated_sql);
        
        // Ensure LIMIT exists
        let final_sql = Self::ensure_limit(&validated_sql, default_limit)?;
        
        let limit_applied = !original_has_limit;
        
        Ok((final_sql, limit_applied))
    }

    /// Check if SQL has LIMIT clause using AST parsing
    fn has_limit(sql: &str) -> bool {
        let dialect = PostgreSqlDialect {};
        let mut parser = match Parser::new(&dialect).try_with_sql(sql) {
            Ok(p) => p,
            Err(_) => return false,
        };

        let ast = match parser.parse_statements() {
            Ok(statements) => statements,
            Err(_) => return false,
        };

        if ast.is_empty() {
            return false;
        }

        Self::check_limit_in_statement(&ast[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_select_only() {
        // Valid SELECT
        assert!(SqlValidator::validate_select_only("SELECT * FROM users").is_ok());
        
        // Invalid INSERT
        assert!(SqlValidator::validate_select_only("INSERT INTO users VALUES (1)").is_err());
        
        // Invalid UPDATE
        assert!(SqlValidator::validate_select_only("UPDATE users SET name = 'test'").is_err());
        
        // Invalid DELETE
        assert!(SqlValidator::validate_select_only("DELETE FROM users").is_err());
    }

    #[test]
    fn test_ensure_limit() {
        // Query without LIMIT
        let sql = "SELECT * FROM users";
        let result = SqlValidator::ensure_limit(sql, 1000).unwrap();
        assert!(result.contains("LIMIT 1000"));
        
        // Query with LIMIT
        let sql = "SELECT * FROM users LIMIT 100";
        let result = SqlValidator::ensure_limit(sql, 1000).unwrap();
        assert_eq!(result, sql);
    }

    #[test]
    fn test_validate_and_prepare() {
        // Valid query without LIMIT
        let (sql, limit_applied) = SqlValidator::validate_and_prepare("SELECT * FROM users", 1000).unwrap();
        assert!(sql.contains("LIMIT 1000"));
        assert!(limit_applied);

        // Valid query with LIMIT
        let (sql, limit_applied) = SqlValidator::validate_and_prepare("SELECT * FROM users LIMIT 50", 1000).unwrap();
        assert!(sql.contains("LIMIT 50"));
        assert!(!limit_applied);

        // Invalid query
        assert!(SqlValidator::validate_and_prepare("DELETE FROM users", 1000).is_err());
    }

    #[test]
    fn test_limit_detection_with_ast() {
        // Test case 1: Table name contains "limit" - should NOT be detected as having LIMIT
        let sql = "SELECT * FROM table_limit";
        let (result, limit_applied) = SqlValidator::validate_and_prepare(sql, 1000).unwrap();
        assert!(result.contains("LIMIT 1000"));
        assert!(limit_applied, "Should apply LIMIT when table name contains 'limit'");

        // Test case 2: Column name contains "limit" - should NOT be detected as having LIMIT
        let sql = "SELECT limit_value FROM users";
        let (result, limit_applied) = SqlValidator::validate_and_prepare(sql, 1000).unwrap();
        assert!(result.contains("LIMIT 1000"));
        assert!(limit_applied, "Should apply LIMIT when column name is 'limit_value'");

        // Test case 3: Comment contains "LIMIT" - should NOT be detected as having LIMIT
        let sql = "SELECT * FROM users /* LIMIT */";
        let (result, limit_applied) = SqlValidator::validate_and_prepare(sql, 1000).unwrap();
        assert!(result.contains("LIMIT 1000"));
        assert!(limit_applied, "Should apply LIMIT when 'LIMIT' is in comment");

        // Test case 4: Actual LIMIT clause - should be detected
        let sql = "SELECT * FROM users LIMIT 50";
        let (result, limit_applied) = SqlValidator::validate_and_prepare(sql, 1000).unwrap();
        assert!(result.contains("LIMIT 50"));
        assert!(!limit_applied, "Should NOT apply LIMIT when query already has LIMIT");

        // Test case 5: LIMIT with OFFSET - should be detected
        let sql = "SELECT * FROM users LIMIT 100 OFFSET 10";
        let (result, limit_applied) = SqlValidator::validate_and_prepare(sql, 1000).unwrap();
        assert!(result.contains("LIMIT 100"));
        assert!(!limit_applied, "Should NOT apply LIMIT when query has LIMIT with OFFSET");
    }

    #[test]
    fn test_has_limit_method() {
        // Test internal has_limit method
        assert!(!SqlValidator::has_limit("SELECT * FROM users"));
        assert!(SqlValidator::has_limit("SELECT * FROM users LIMIT 10"));
        assert!(!SqlValidator::has_limit("SELECT * FROM table_limit"));
        assert!(!SqlValidator::has_limit("SELECT limit_value FROM users"));
    }
}

