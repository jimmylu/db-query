// DataFusion DialectTranslator
//
// Defines the trait and implementations for translating DataFusion SQL
// to database-specific SQL dialects (PostgreSQL, MySQL, etc.)

use anyhow::{Result, Context, anyhow};
use sqlparser::ast::Statement;
use sqlparser::dialect::{PostgreSqlDialect, MySqlDialect, GenericDialect};
use sqlparser::parser::Parser;
use async_trait::async_trait;

/// Trait for translating DataFusion SQL to database-specific dialects
///
/// Each database type implements this trait to handle dialect-specific
/// transformations like function names, identifier quoting, and syntax differences.
#[async_trait]
pub trait DialectTranslator: Send + Sync {
    /// Get the name of this dialect (e.g., "PostgreSQL", "MySQL")
    fn dialect_name(&self) -> &str;

    /// Translate DataFusion SQL to the target database dialect
    ///
    /// # Arguments
    /// * `datafusion_sql` - SQL query in DataFusion/standard SQL syntax
    ///
    /// # Returns
    /// The translated SQL query that can be executed on the target database
    ///
    /// # Errors
    /// Returns error if SQL cannot be parsed or translation fails
    async fn translate(&self, datafusion_sql: &str) -> Result<String>;

    /// Check if a specific SQL feature is supported in this dialect
    fn supports_feature(&self, feature: SqlFeature) -> bool;
}

/// SQL features that may differ across dialects
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlFeature {
    /// String concatenation with || operator
    ConcatOperator,
    /// CONCAT() function
    ConcatFunction,
    /// INTERVAL syntax
    IntervalSyntax,
    /// RETURNING clause
    ReturningClause,
    /// WITH (Common Table Expressions)
    CommonTableExpressions,
    /// Double-quoted identifiers
    DoubleQuotedIdentifiers,
    /// Backtick-quoted identifiers
    BacktickIdentifiers,
}

/// PostgreSQL dialect translator
pub struct PostgreSQLDialectTranslator {
    dialect: PostgreSqlDialect,
}

impl PostgreSQLDialectTranslator {
    pub fn new() -> Self {
        Self {
            dialect: PostgreSqlDialect {},
        }
    }
}

impl Default for PostgreSQLDialectTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DialectTranslator for PostgreSQLDialectTranslator {
    fn dialect_name(&self) -> &str {
        "PostgreSQL"
    }

    async fn translate(&self, datafusion_sql: &str) -> Result<String> {
        // For PostgreSQL, DataFusion SQL is already very compatible
        // Just validate the SQL can be parsed
        let statements = Parser::parse_sql(&self.dialect, datafusion_sql)
            .context("Failed to parse SQL for PostgreSQL")?;

        if statements.is_empty() {
            return Err(anyhow!("Empty SQL statement"));
        }

        // PostgreSQL-specific transformations
        let mut translated = datafusion_sql.to_string();

        // Handle any PostgreSQL-specific syntax needs
        // (Currently DataFusion SQL is already PostgreSQL-compatible)

        Ok(translated)
    }

    fn supports_feature(&self, feature: SqlFeature) -> bool {
        match feature {
            SqlFeature::ConcatOperator => true,
            SqlFeature::ConcatFunction => true,
            SqlFeature::IntervalSyntax => true,
            SqlFeature::ReturningClause => true,
            SqlFeature::CommonTableExpressions => true,
            SqlFeature::DoubleQuotedIdentifiers => true,
            SqlFeature::BacktickIdentifiers => false,
        }
    }
}

/// MySQL dialect translator
pub struct MySQLDialectTranslator {
    dialect: MySqlDialect,
}

impl MySQLDialectTranslator {
    pub fn new() -> Self {
        Self {
            dialect: MySqlDialect {},
        }
    }

    /// Translate identifier quoting from double quotes to backticks
    fn translate_identifiers(&self, sql: &str) -> String {
        // Simple replacement of double quotes with backticks for identifiers
        // This is a basic implementation; a full parser-based approach would be more robust
        let mut result = String::new();
        let mut in_string = false;
        let mut in_identifier = false;
        let mut prev_char = ' ';

        for ch in sql.chars() {
            match ch {
                '\'' => {
                    // Single quote - string literal
                    if !in_identifier {
                        in_string = !in_string;
                    }
                    result.push(ch);
                }
                '"' if !in_string => {
                    // Double quote - identifier (convert to backtick)
                    in_identifier = !in_identifier;
                    result.push('`');
                }
                _ => {
                    result.push(ch);
                }
            }
            prev_char = ch;
        }

        result
    }

    /// Translate CONCAT operator (||) to CONCAT function
    fn translate_concat_operator(&self, sql: &str) -> String {
        // This is a simplified transformation
        // A real implementation would need to parse the SQL properly
        // For now, we rely on DataFusion to handle this
        sql.to_string()
    }

    /// Translate INTERVAL syntax
    fn translate_interval(&self, sql: &str) -> String {
        // PostgreSQL: INTERVAL '7 days'
        // MySQL: INTERVAL 7 DAY

        let mut result = sql.to_string();

        // Simple pattern matching for common interval patterns
        // Real implementation would use AST transformation
        result = result.replace("INTERVAL '1 day'", "INTERVAL 1 DAY");
        result = result.replace("INTERVAL '7 days'", "INTERVAL 7 DAY");
        result = result.replace("INTERVAL '30 days'", "INTERVAL 30 DAY");
        result = result.replace("INTERVAL '1 hour'", "INTERVAL 1 HOUR");
        result = result.replace("INTERVAL '1 month'", "INTERVAL 1 MONTH");
        result = result.replace("INTERVAL '1 year'", "INTERVAL 1 YEAR");

        result
    }

    /// Translate date functions
    fn translate_date_functions(&self, sql: &str) -> String {
        let mut result = sql.to_string();

        // CURRENT_DATE -> CURDATE()
        result = result.replace("CURRENT_DATE", "CURDATE()");

        // CURRENT_TIMESTAMP -> NOW()
        result = result.replace("CURRENT_TIMESTAMP", "NOW()");

        result
    }
}

impl Default for MySQLDialectTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DialectTranslator for MySQLDialectTranslator {
    fn dialect_name(&self) -> &str {
        "MySQL"
    }

    async fn translate(&self, datafusion_sql: &str) -> Result<String> {
        // Validate SQL can be parsed
        let statements = Parser::parse_sql(&GenericDialect {}, datafusion_sql)
            .context("Failed to parse SQL for MySQL translation")?;

        if statements.is_empty() {
            return Err(anyhow!("Empty SQL statement"));
        }

        // Apply MySQL-specific transformations
        let mut translated = datafusion_sql.to_string();

        // 1. Translate identifier quoting: " -> `
        translated = self.translate_identifiers(&translated);

        // 2. Translate INTERVAL syntax
        translated = self.translate_interval(&translated);

        // 3. Translate date functions
        translated = self.translate_date_functions(&translated);

        // 4. CONCAT operator translation (if needed)
        // Note: DataFusion should handle this at the logical plan level

        Ok(translated)
    }

    fn supports_feature(&self, feature: SqlFeature) -> bool {
        match feature {
            SqlFeature::ConcatOperator => false, // MySQL uses CONCAT()
            SqlFeature::ConcatFunction => true,
            SqlFeature::IntervalSyntax => true,  // But different syntax
            SqlFeature::ReturningClause => false,
            SqlFeature::CommonTableExpressions => true, // MySQL 8.0+
            SqlFeature::DoubleQuotedIdentifiers => false, // MySQL uses backticks
            SqlFeature::BacktickIdentifiers => true,
        }
    }
}

/// Generic dialect translator (pass-through)
///
/// This translator is used for databases that are already compatible
/// with DataFusion's SQL syntax or when no translation is needed.
pub struct GenericDialectTranslator;

impl GenericDialectTranslator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GenericDialectTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DialectTranslator for GenericDialectTranslator {
    fn dialect_name(&self) -> &str {
        "Generic"
    }

    async fn translate(&self, datafusion_sql: &str) -> Result<String> {
        // Pass-through: no translation needed
        Ok(datafusion_sql.to_string())
    }

    fn supports_feature(&self, _feature: SqlFeature) -> bool {
        true // Assume all features supported
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_postgresql_translator() {
        let translator = PostgreSQLDialectTranslator::new();
        assert_eq!(translator.dialect_name(), "PostgreSQL");

        let sql = "SELECT * FROM users WHERE created_at >= CURRENT_DATE - INTERVAL '7 days'";
        let result = translator.translate(sql).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mysql_translator_identifiers() {
        let translator = MySQLDialectTranslator::new();
        assert_eq!(translator.dialect_name(), "MySQL");

        let sql = r#"SELECT "user_id", "name" FROM "users""#;
        let translated = translator.translate(sql).await.unwrap();

        // Should convert double quotes to backticks
        assert!(translated.contains("`user_id`"));
        assert!(translated.contains("`name`"));
        assert!(translated.contains("`users`"));
    }

    #[tokio::test]
    async fn test_mysql_translator_intervals() {
        let translator = MySQLDialectTranslator::new();

        let sql = "SELECT * FROM orders WHERE order_date >= CURRENT_DATE - INTERVAL '7 days'";
        let translated = translator.translate(sql).await.unwrap();

        // Should convert INTERVAL syntax
        assert!(translated.contains("INTERVAL 7 DAY"));
        assert!(translated.contains("CURDATE()"));
    }

    #[tokio::test]
    async fn test_mysql_translator_date_functions() {
        let translator = MySQLDialectTranslator::new();

        let sql = "SELECT CURRENT_DATE, CURRENT_TIMESTAMP FROM dual";
        let translated = translator.translate(sql).await.unwrap();

        // Should convert date functions
        assert!(translated.contains("CURDATE()"));
        assert!(translated.contains("NOW()"));
    }

    #[test]
    fn test_feature_support_postgresql() {
        let translator = PostgreSQLDialectTranslator::new();

        assert!(translator.supports_feature(SqlFeature::ConcatOperator));
        assert!(translator.supports_feature(SqlFeature::DoubleQuotedIdentifiers));
        assert!(!translator.supports_feature(SqlFeature::BacktickIdentifiers));
    }

    #[test]
    fn test_feature_support_mysql() {
        let translator = MySQLDialectTranslator::new();

        assert!(!translator.supports_feature(SqlFeature::ConcatOperator));
        assert!(translator.supports_feature(SqlFeature::ConcatFunction));
        assert!(translator.supports_feature(SqlFeature::BacktickIdentifiers));
        assert!(!translator.supports_feature(SqlFeature::DoubleQuotedIdentifiers));
    }

    #[tokio::test]
    async fn test_generic_translator() {
        let translator = GenericDialectTranslator::new();
        let sql = "SELECT * FROM table";
        let result = translator.translate(sql).await.unwrap();
        assert_eq!(result, sql);
    }
}
