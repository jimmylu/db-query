// Dialect Translation Service
//
// Provides high-level service for translating queries between SQL dialects.
// Coordinates dialect translators and provides caching for translations.

use std::sync::Arc;
use std::collections::HashMap;
use anyhow::{Result, anyhow, Context};

use super::dialect::{DialectTranslator, PostgreSQLDialectTranslator, MySQLDialectTranslator, GenericDialectTranslator};

/// Database types supported by the translation service
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    Doris,
    Druid,
}

impl DatabaseType {
    /// Parse database type from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "postgresql" | "postgres" | "pg" => Ok(DatabaseType::PostgreSQL),
            "mysql" | "mariadb" => Ok(DatabaseType::MySQL),
            "doris" | "apache doris" => Ok(DatabaseType::Doris),
            "druid" | "apache druid" => Ok(DatabaseType::Druid),
            _ => Err(anyhow!("Unsupported database type: {}", s)),
        }
    }

    /// Get the database type name
    pub fn as_str(&self) -> &str {
        match self {
            DatabaseType::PostgreSQL => "PostgreSQL",
            DatabaseType::MySQL => "MySQL",
            DatabaseType::Doris => "Doris",
            DatabaseType::Druid => "Druid",
        }
    }
}

/// Service for managing SQL dialect translations
///
/// The DialectTranslationService coordinates translation between DataFusion SQL
/// and various database dialects. It maintains a registry of translators and
/// provides caching for performance.
///
/// # Example
/// ```rust,ignore
/// let service = DialectTranslationService::new();
/// let translated = service.translate_query(
///     "SELECT * FROM users WHERE created_at >= CURRENT_DATE - INTERVAL '7 days'",
///     DatabaseType::MySQL
/// ).await?;
/// // Result: "SELECT * FROM `users` WHERE created_at >= CURDATE() - INTERVAL 7 DAY"
/// ```
pub struct DialectTranslationService {
    /// Registry of dialect translators by database type
    translators: HashMap<DatabaseType, Arc<dyn DialectTranslator>>,
    /// Optional translation cache (query -> translated query)
    cache: Option<Arc<tokio::sync::RwLock<HashMap<(String, DatabaseType), String>>>>,
}

impl DialectTranslationService {
    /// Create a new translation service with default translators
    pub fn new() -> Self {
        let mut translators: HashMap<DatabaseType, Arc<dyn DialectTranslator>> = HashMap::new();

        // Register default translators
        translators.insert(
            DatabaseType::PostgreSQL,
            Arc::new(PostgreSQLDialectTranslator::new()),
        );
        translators.insert(
            DatabaseType::MySQL,
            Arc::new(MySQLDialectTranslator::new()),
        );
        // Doris and Druid use generic translator for now
        translators.insert(
            DatabaseType::Doris,
            Arc::new(GenericDialectTranslator::new()),
        );
        translators.insert(
            DatabaseType::Druid,
            Arc::new(GenericDialectTranslator::new()),
        );

        Self {
            translators,
            cache: None,
        }
    }

    /// Create a new translation service with caching enabled
    pub fn with_cache() -> Self {
        let mut service = Self::new();
        service.cache = Some(Arc::new(tokio::sync::RwLock::new(HashMap::new())));
        service
    }

    /// Register a custom dialect translator
    ///
    /// This allows adding support for new database types or overriding
    /// default translators.
    ///
    /// # Arguments
    /// * `db_type` - Database type to register translator for
    /// * `translator` - Translator implementation
    pub fn register_translator(
        &mut self,
        db_type: DatabaseType,
        translator: Arc<dyn DialectTranslator>,
    ) {
        self.translators.insert(db_type, translator);
    }

    /// Translate a DataFusion SQL query to a target database dialect
    ///
    /// # Arguments
    /// * `datafusion_sql` - Query in DataFusion/standard SQL syntax
    /// * `target_db` - Target database type
    ///
    /// # Returns
    /// Translated SQL query that can be executed on the target database
    ///
    /// # Errors
    /// Returns error if translation fails or database type not supported
    pub async fn translate_query(
        &self,
        datafusion_sql: &str,
        target_db: DatabaseType,
    ) -> Result<String> {
        // Check cache first if enabled
        if let Some(cache) = &self.cache {
            let cache_key = (datafusion_sql.to_string(), target_db);
            let cache_read = cache.read().await;
            if let Some(cached_result) = cache_read.get(&cache_key) {
                tracing::debug!("Cache hit for query translation to {}", target_db.as_str());
                return Ok(cached_result.clone());
            }
        }

        // Get translator for target database
        let translator = self
            .translators
            .get(&target_db)
            .ok_or_else(|| anyhow!("No translator registered for {:?}", target_db))?;

        // Perform translation
        let translated = translator
            .translate(datafusion_sql)
            .await
            .context(format!("Failed to translate query to {} dialect", target_db.as_str()))?;

        // Cache result if caching is enabled
        if let Some(cache) = &self.cache {
            let cache_key = (datafusion_sql.to_string(), target_db);
            let mut cache_write = cache.write().await;
            cache_write.insert(cache_key, translated.clone());
            tracing::debug!("Cached translation result for {}", target_db.as_str());
        }

        Ok(translated)
    }

    /// Batch translate multiple queries
    ///
    /// Useful for translating a set of queries at once, potentially
    /// with better caching efficiency.
    ///
    /// # Arguments
    /// * `queries` - Vector of queries to translate
    /// * `target_db` - Target database type
    ///
    /// # Returns
    /// Vector of translated queries in the same order as input
    pub async fn translate_batch(
        &self,
        queries: Vec<&str>,
        target_db: DatabaseType,
    ) -> Result<Vec<String>> {
        let mut results = Vec::with_capacity(queries.len());

        for query in queries {
            let translated = self.translate_query(query, target_db).await?;
            results.push(translated);
        }

        Ok(results)
    }

    /// Get the translator for a specific database type
    ///
    /// Returns None if no translator is registered for the database type.
    pub fn get_translator(&self, db_type: DatabaseType) -> Option<Arc<dyn DialectTranslator>> {
        self.translators.get(&db_type).cloned()
    }

    /// List all supported database types
    pub fn supported_databases(&self) -> Vec<DatabaseType> {
        self.translators.keys().copied().collect()
    }

    /// Clear the translation cache
    ///
    /// Useful when dialect translators are updated or for memory management.
    pub async fn clear_cache(&self) {
        if let Some(cache) = &self.cache {
            let mut cache_write = cache.write().await;
            cache_write.clear();
            tracing::info!("Translation cache cleared");
        }
    }

    /// Get cache statistics
    ///
    /// Returns number of cached translations if caching is enabled.
    pub async fn cache_size(&self) -> Option<usize> {
        if let Some(cache) = &self.cache {
            let cache_read = cache.read().await;
            Some(cache_read.len())
        } else {
            None
        }
    }
}

impl Default for DialectTranslationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_type_parsing() {
        assert_eq!(
            DatabaseType::from_str("postgresql").unwrap(),
            DatabaseType::PostgreSQL
        );
        assert_eq!(
            DatabaseType::from_str("MySQL").unwrap(),
            DatabaseType::MySQL
        );
        assert_eq!(
            DatabaseType::from_str("doris").unwrap(),
            DatabaseType::Doris
        );
        assert!(DatabaseType::from_str("unknown").is_err());
    }

    #[tokio::test]
    async fn test_translation_service_creation() {
        let service = DialectTranslationService::new();
        let supported = service.supported_databases();

        assert!(supported.contains(&DatabaseType::PostgreSQL));
        assert!(supported.contains(&DatabaseType::MySQL));
        assert_eq!(supported.len(), 4);
    }

    #[tokio::test]
    async fn test_translate_to_postgresql() {
        let service = DialectTranslationService::new();
        let sql = "SELECT * FROM users WHERE active = true";

        let result = service.translate_query(sql, DatabaseType::PostgreSQL).await;
        assert!(result.is_ok());

        // PostgreSQL translator should accept this SQL as-is
        let translated = result.unwrap();
        assert!(translated.contains("SELECT"));
        assert!(translated.contains("users"));
    }

    #[tokio::test]
    async fn test_translate_to_mysql() {
        let service = DialectTranslationService::new();
        let sql = r#"SELECT "user_id" FROM "users""#;

        let result = service.translate_query(sql, DatabaseType::MySQL).await;
        assert!(result.is_ok());

        // MySQL translator should convert double quotes to backticks
        let translated = result.unwrap();
        assert!(translated.contains("`"));
    }

    #[tokio::test]
    async fn test_batch_translation() {
        let service = DialectTranslationService::new();
        let queries = vec![
            "SELECT * FROM users",
            "SELECT * FROM orders",
        ];

        let results = service.translate_batch(queries, DatabaseType::PostgreSQL).await;
        assert!(results.is_ok());

        let translated = results.unwrap();
        assert_eq!(translated.len(), 2);
    }

    #[tokio::test]
    async fn test_caching() {
        let service = DialectTranslationService::with_cache();
        let sql = "SELECT * FROM users";

        // First call - cache miss
        let result1 = service.translate_query(sql, DatabaseType::PostgreSQL).await;
        assert!(result1.is_ok());

        let cache_size = service.cache_size().await;
        assert_eq!(cache_size, Some(1));

        // Second call - cache hit
        let result2 = service.translate_query(sql, DatabaseType::PostgreSQL).await;
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());

        // Clear cache
        service.clear_cache().await;
        let cache_size = service.cache_size().await;
        assert_eq!(cache_size, Some(0));
    }

    #[tokio::test]
    async fn test_custom_translator_registration() {
        let mut service = DialectTranslationService::new();

        // Register a custom translator
        let custom_translator = Arc::new(GenericDialectTranslator::new());
        service.register_translator(DatabaseType::PostgreSQL, custom_translator);

        // Should still work
        let result = service.translate_query("SELECT 1", DatabaseType::PostgreSQL).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_database_type_as_str() {
        assert_eq!(DatabaseType::PostgreSQL.as_str(), "PostgreSQL");
        assert_eq!(DatabaseType::MySQL.as_str(), "MySQL");
        assert_eq!(DatabaseType::Doris.as_str(), "Doris");
        assert_eq!(DatabaseType::Druid.as_str(), "Druid");
    }
}
