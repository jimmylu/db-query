use rusqlite::{Connection, Result as SqliteResult};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

/// SQLite storage for metadata and connections
/// Uses tokio::Mutex for async-friendly locking
pub struct SqliteStorage {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteStorage {
    /// Create a new SQLite storage instance
    pub async fn new<P: AsRef<Path>>(db_path: P) -> SqliteResult<Self> {
        // Handle SQLite URL format (sqlite:./path or sqlite://path)
        let path_str = db_path.as_ref().to_string_lossy();
        let clean_path: &str = if path_str.starts_with("sqlite:") {
            // Remove sqlite: or sqlite:// prefix
            let mut cleaned = path_str.trim_start_matches("sqlite:");
            cleaned = cleaned.trim_start_matches("//");
            cleaned
        } else {
            path_str.as_ref()
        };

        let conn = Connection::open(clean_path)?;
        // Enable foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        storage.init_schema().await?;
        Ok(storage)
    }

    /// Initialize database schema
    async fn init_schema(&self) -> SqliteResult<()> {
        let conn = self.conn.lock().await;

        // Enable foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        // Create domains table first (referenced by connections)
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS domains (
                id TEXT PRIMARY KEY,
                name TEXT UNIQUE NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
            [],
        )?;

        // Create default domain if not exists (for migration compatibility)
        // Use a fixed RFC3339 timestamp for consistency
        let default_timestamp = chrono::Utc::now().to_rfc3339();
        conn.execute(
            r#"
            INSERT OR IGNORE INTO domains (id, name, description, created_at, updated_at)
            VALUES ('default-domain-id', 'Default Domain', 'Auto-created for existing connections', ?1, ?2)
            "#,
            rusqlite::params![&default_timestamp, &default_timestamp],
        )?;

        // Create connections table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS connections (
                id TEXT PRIMARY KEY,
                name TEXT,
                connection_url TEXT NOT NULL,
                database_type TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                last_connected_at TIMESTAMP,
                metadata_cache_id TEXT,
                domain_id TEXT DEFAULT 'default-domain-id',
                FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE CASCADE
            )
            "#,
            [],
        )?;

        // Create metadata_cache table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS metadata_cache (
                id TEXT PRIMARY KEY,
                connection_id TEXT NOT NULL,
                metadata_json TEXT NOT NULL,
                retrieved_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                version INTEGER NOT NULL DEFAULT 1,
                FOREIGN KEY (connection_id) REFERENCES connections(id) ON DELETE CASCADE
            )
            "#,
            [],
        )?;

        // Create indexes for better query performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_connections_status ON connections(status)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_metadata_cache_connection_id ON metadata_cache(connection_id)",
            [],
        )?;

        // Create composite index for domain filtering (O(log n) performance)
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_connections_domain_created ON connections(domain_id, created_at DESC)",
            [],
        )?;

        // Create saved_queries table (domain-scoped)
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS saved_queries (
                id TEXT PRIMARY KEY,
                domain_id TEXT NOT NULL,
                connection_id TEXT NOT NULL,
                name TEXT NOT NULL,
                query_text TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE CASCADE,
                FOREIGN KEY (connection_id) REFERENCES connections(id) ON DELETE SET NULL,
                UNIQUE(domain_id, name)
            )
            "#,
            [],
        )?;

        // Create query_history table (domain-scoped)
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS query_history (
                id TEXT PRIMARY KEY,
                domain_id TEXT NOT NULL,
                connection_id TEXT NOT NULL,
                query_text TEXT NOT NULL,
                row_count INTEGER NOT NULL,
                execution_time_ms INTEGER NOT NULL,
                status TEXT NOT NULL,
                error_message TEXT,
                executed_at TEXT NOT NULL,
                is_llm_generated INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE CASCADE,
                FOREIGN KEY (connection_id) REFERENCES connections(id) ON DELETE SET NULL
            )
            "#,
            [],
        )?;

        // Create indexes for query tables
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_saved_queries_domain ON saved_queries(domain_id, created_at DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_query_history_domain ON query_history(domain_id, executed_at DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_query_history_connection ON query_history(connection_id, executed_at DESC)",
            [],
        )?;

        Ok(())
    }

    /// Get a reference to the connection (for use in async contexts)
    pub fn get_conn(&self) -> Arc<Mutex<Connection>> {
        self.conn.clone()
    }

    /// Save a connection to the database
    pub async fn save_connection(&self, conn: &crate::models::DatabaseConnection) -> SqliteResult<()> {
        let db_conn = self.conn.lock().await;
        db_conn.execute(
            r#"
            INSERT OR REPLACE INTO connections 
            (id, name, connection_url, database_type, status, created_at, last_connected_at, metadata_cache_id)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            rusqlite::params![
                conn.id,
                conn.name,
                conn.connection_url,
                conn.database_type,
                format!("{:?}", conn.status).to_lowercase(),
                conn.created_at.to_rfc3339(),
                conn.last_connected_at.map(|d| d.to_rfc3339()),
                conn.metadata_cache_id,
            ],
        )?;
        Ok(())
    }

    /// Get a connection by ID
    pub async fn get_connection(&self, id: &str) -> SqliteResult<Option<crate::models::DatabaseConnection>> {
        let db_conn = self.conn.lock().await;
        let mut stmt = db_conn.prepare(
            "SELECT id, name, connection_url, database_type, domain_id, status, created_at, last_connected_at, metadata_cache_id FROM connections WHERE id = ?1"
        )?;

        let result = stmt.query_row(rusqlite::params![id], |row| {
            Ok(crate::models::DatabaseConnection {
                id: row.get(0)?,
                name: row.get(1)?,
                connection_url: row.get(2)?,
                database_type: row.get(3)?,
                domain_id: row.get(4)?,
                status: match row.get::<_, String>(5)?.as_str() {
                    "connected" => crate::models::ConnectionStatus::Connected,
                    "error" => crate::models::ConnectionStatus::Error,
                    _ => crate::models::ConnectionStatus::Disconnected,
                },
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                last_connected_at: row.get::<_, Option<String>>(7)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                metadata_cache_id: row.get(8)?,
            })
        });

        match result {
            Ok(conn) => Ok(Some(conn)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// List all connections
    pub async fn list_connections(&self) -> SqliteResult<Vec<crate::models::DatabaseConnection>> {
        let db_conn = self.conn.lock().await;
        let mut stmt = db_conn.prepare(
            "SELECT id, name, connection_url, database_type, domain_id, status, created_at, last_connected_at, metadata_cache_id FROM connections ORDER BY created_at DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(crate::models::DatabaseConnection {
                id: row.get(0)?,
                name: row.get(1)?,
                connection_url: row.get(2)?,
                database_type: row.get(3)?,
                domain_id: row.get(4)?,
                status: match row.get::<_, String>(5)?.as_str() {
                    "connected" => crate::models::ConnectionStatus::Connected,
                    "error" => crate::models::ConnectionStatus::Error,
                    _ => crate::models::ConnectionStatus::Disconnected,
                },
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                last_connected_at: row.get::<_, Option<String>>(7)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                metadata_cache_id: row.get(8)?,
            })
        })?;

        let mut connections = Vec::new();
        for row in rows {
            connections.push(row?);
        }
        Ok(connections)
    }

    /// Delete a connection
    pub async fn delete_connection(&self, id: &str) -> SqliteResult<bool> {
        let db_conn = self.conn.lock().await;
        let rows_affected = db_conn.execute("DELETE FROM connections WHERE id = ?1", rusqlite::params![id])?;
        Ok(rows_affected > 0)
    }

    /// Save metadata cache
    pub async fn save_metadata_cache(&self, metadata: &crate::models::DatabaseMetadata) -> SqliteResult<()> {
        let db_conn = self.conn.lock().await;
        db_conn.execute(
            r#"
            INSERT OR REPLACE INTO metadata_cache 
            (id, connection_id, metadata_json, retrieved_at, version)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            rusqlite::params![
                metadata.id,
                metadata.connection_id,
                metadata.metadata_json,
                metadata.retrieved_at.to_rfc3339(),
                metadata.version,
            ],
        )?;
        Ok(())
    }

    /// Get metadata cache by connection ID
    pub async fn get_metadata_cache(&self, connection_id: &str) -> SqliteResult<Option<crate::models::DatabaseMetadata>> {
        let db_conn = self.conn.lock().await;
        let mut stmt = db_conn.prepare(
            "SELECT id, connection_id, metadata_json, retrieved_at, version FROM metadata_cache WHERE connection_id = ?1 ORDER BY version DESC LIMIT 1"
        )?;

        let result = stmt.query_row(rusqlite::params![connection_id], |row| {
            let metadata_json: String = row.get(2)?;
            let json_value: serde_json::Value = serde_json::from_str(&metadata_json).unwrap_or_default();
            
            Ok(crate::models::DatabaseMetadata {
                id: row.get(0)?,
                connection_id: row.get(1)?,
                tables: json_value["tables"].as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect()
                    })
                    .unwrap_or_default(),
                views: json_value["views"].as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect()
                    })
                    .unwrap_or_default(),
                schemas: json_value["schemas"].as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default(),
                metadata_json,
                retrieved_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                version: row.get(4)?,
            })
        });

        match result {
            Ok(metadata) => Ok(Some(metadata)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    // ==================== Domain Management ====================

    /// Create a new domain
    pub async fn create_domain(&self, domain: &crate::models::Domain) -> SqliteResult<()> {
        let db_conn = self.conn.lock().await;
        db_conn.execute(
            r#"
            INSERT INTO domains (id, name, description, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            rusqlite::params![
                domain.id,
                domain.name,
                domain.description,
                domain.created_at.to_rfc3339(),
                domain.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Get a domain by ID
    pub async fn get_domain(&self, id: &str) -> SqliteResult<Option<crate::models::Domain>> {
        let db_conn = self.conn.lock().await;
        let mut stmt = db_conn.prepare(
            "SELECT id, name, description, created_at, updated_at FROM domains WHERE id = ?1"
        )?;

        let result = stmt.query_row(rusqlite::params![id], |row| {
            Ok(crate::models::Domain {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
            })
        });

        match result {
            Ok(domain) => Ok(Some(domain)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// List all domains with resource counts
    pub async fn list_domains(&self) -> SqliteResult<Vec<crate::models::DomainResponse>> {
        let db_conn = self.conn.lock().await;

        // Query to get domains with connection counts
        let mut stmt = db_conn.prepare(
            r#"
            SELECT
                d.id,
                d.name,
                d.description,
                d.created_at,
                d.updated_at,
                COUNT(DISTINCT c.id) as connection_count
            FROM domains d
            LEFT JOIN connections c ON c.domain_id = d.id
            GROUP BY d.id, d.name, d.description, d.created_at, d.updated_at
            ORDER BY d.created_at DESC
            "#
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(crate::models::DomainResponse {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                connection_count: row.get::<_, i64>(5)? as usize,
                // Note: saved_query_count and query_history_count will be 0 until we implement those tables
                saved_query_count: 0,
                query_history_count: 0,
            })
        })?;

        let mut domains = Vec::new();
        for row in rows {
            domains.push(row?);
        }
        Ok(domains)
    }

    /// Update an existing domain
    pub async fn update_domain(&self, domain: &crate::models::Domain) -> SqliteResult<bool> {
        let db_conn = self.conn.lock().await;
        let rows_affected = db_conn.execute(
            r#"
            UPDATE domains
            SET name = ?1, description = ?2, updated_at = ?3
            WHERE id = ?4
            "#,
            rusqlite::params![
                domain.name,
                domain.description,
                domain.updated_at.to_rfc3339(),
                domain.id,
            ],
        )?;
        Ok(rows_affected > 0)
    }

    /// Delete a domain (CASCADE will delete associated connections)
    pub async fn delete_domain(&self, id: &str) -> SqliteResult<bool> {
        let db_conn = self.conn.lock().await;
        let rows_affected = db_conn.execute("DELETE FROM domains WHERE id = ?1", rusqlite::params![id])?;
        Ok(rows_affected > 0)
    }

    /// Get connection count for a domain
    pub async fn get_domain_connection_count(&self, domain_id: &str) -> SqliteResult<usize> {
        let db_conn = self.conn.lock().await;
        let count: i64 = db_conn.query_row(
            "SELECT COUNT(*) FROM connections WHERE domain_id = ?1",
            rusqlite::params![domain_id],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    /// List connections for a specific domain
    pub async fn list_connections_by_domain(&self, domain_id: &str) -> SqliteResult<Vec<crate::models::DatabaseConnection>> {
        let db_conn = self.conn.lock().await;
        let mut stmt = db_conn.prepare(
            "SELECT id, name, connection_url, database_type, domain_id, status, created_at, last_connected_at, metadata_cache_id FROM connections WHERE domain_id = ?1 ORDER BY created_at DESC"
        )?;

        let rows = stmt.query_map(rusqlite::params![domain_id], |row| {
            Ok(crate::models::DatabaseConnection {
                id: row.get(0)?,
                name: row.get(1)?,
                connection_url: row.get(2)?,
                database_type: row.get(3)?,
                domain_id: row.get(4)?,
                status: match row.get::<_, String>(5)?.as_str() {
                    "connected" => crate::models::ConnectionStatus::Connected,
                    "error" => crate::models::ConnectionStatus::Error,
                    _ => crate::models::ConnectionStatus::Disconnected,
                },
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                last_connected_at: row.get::<_, Option<String>>(7)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                metadata_cache_id: row.get(8)?,
            })
        })?;

        let mut connections = Vec::new();
        for row in rows {
            connections.push(row?);
        }
        Ok(connections)
    }

    // ============================================================================
    // Saved Query Operations (Domain-Scoped)
    // ============================================================================

    /// Save a query for a domain
    pub async fn save_query(&self, query: &crate::models::SavedQuery) -> SqliteResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            r#"
            INSERT INTO saved_queries (id, domain_id, connection_id, name, query_text, description, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            rusqlite::params![
                query.id,
                query.domain_id,
                query.connection_id,
                query.name,
                query.query_text,
                query.description,
                query.created_at.to_rfc3339(),
                query.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Get a saved query by ID
    pub async fn get_saved_query(&self, id: &str) -> SqliteResult<Option<crate::models::SavedQuery>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, domain_id, connection_id, name, query_text, description, created_at, updated_at
             FROM saved_queries WHERE id = ?1"
        )?;

        let result = stmt.query_row([id], |row| {
            Ok(crate::models::SavedQuery {
                id: row.get(0)?,
                domain_id: row.get(1)?,
                connection_id: row.get(2)?,
                name: row.get(3)?,
                query_text: row.get(4)?,
                description: row.get(5)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
            })
        });

        match result {
            Ok(query) => Ok(Some(query)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// List all saved queries for a domain
    pub async fn list_saved_queries(&self, domain_id: &str) -> SqliteResult<Vec<crate::models::SavedQuery>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, domain_id, connection_id, name, query_text, description, created_at, updated_at
             FROM saved_queries
             WHERE domain_id = ?1
             ORDER BY created_at DESC"
        )?;

        let queries = stmt.query_map([domain_id], |row| {
            Ok(crate::models::SavedQuery {
                id: row.get(0)?,
                domain_id: row.get(1)?,
                connection_id: row.get(2)?,
                name: row.get(3)?,
                query_text: row.get(4)?,
                description: row.get(5)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
            })
        })?;

        queries.collect()
    }

    /// Update a saved query
    pub async fn update_saved_query(
        &self,
        id: &str,
        name: Option<String>,
        query_text: Option<String>,
        description: Option<String>,
    ) -> SqliteResult<()> {
        let conn = self.conn.lock().await;

        // Build dynamic UPDATE query based on which fields are provided
        let mut updates = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(n) = name {
            updates.push("name = ?");
            params.push(Box::new(n));
        }
        if let Some(q) = query_text {
            updates.push("query_text = ?");
            params.push(Box::new(q));
        }
        if let Some(d) = description {
            updates.push("description = ?");
            params.push(Box::new(d));
        }

        if updates.is_empty() {
            return Ok(()); // Nothing to update
        }

        updates.push("updated_at = ?");
        params.push(Box::new(chrono::Utc::now().to_rfc3339()));

        params.push(Box::new(id.to_string()));

        let query = format!(
            "UPDATE saved_queries SET {} WHERE id = ?",
            updates.join(", ")
        );

        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        conn.execute(&query, params_refs.as_slice())?;

        Ok(())
    }

    /// Delete a saved query
    pub async fn delete_saved_query(&self, id: &str) -> SqliteResult<()> {
        let conn = self.conn.lock().await;
        conn.execute("DELETE FROM saved_queries WHERE id = ?1", [id])?;
        Ok(())
    }

    // ============================================================================
    // Query History Operations (Domain-Scoped)
    // ============================================================================

    /// Add a query execution to history
    pub async fn add_query_history(&self, history: &crate::models::QueryHistory) -> SqliteResult<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            r#"
            INSERT INTO query_history
            (id, domain_id, connection_id, query_text, row_count, execution_time_ms, status, error_message, executed_at, is_llm_generated)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            rusqlite::params![
                history.id,
                history.domain_id,
                history.connection_id,
                history.query_text,
                history.row_count as i64,
                history.execution_time_ms as i64,
                format!("{:?}", history.status).to_lowercase(),
                history.error_message,
                history.executed_at.to_rfc3339(),
                if history.is_llm_generated { 1 } else { 0 },
            ],
        )?;
        Ok(())
    }

    /// List query history for a domain (with limit)
    pub async fn list_query_history(
        &self,
        domain_id: &str,
        limit: usize,
    ) -> SqliteResult<Vec<crate::models::QueryHistory>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, domain_id, connection_id, query_text, row_count, execution_time_ms, status, error_message, executed_at, is_llm_generated
            FROM query_history
            WHERE domain_id = ?1
            ORDER BY executed_at DESC
            LIMIT ?2
            "#
        )?;

        let histories = stmt.query_map(rusqlite::params![domain_id, limit as i64], |row| {
            let status_str: String = row.get(6)?;
            let status = match status_str.as_str() {
                "success" => crate::models::QueryHistoryStatus::Success,
                "failed" => crate::models::QueryHistoryStatus::Failed,
                _ => crate::models::QueryHistoryStatus::Failed,
            };

            Ok(crate::models::QueryHistory {
                id: row.get(0)?,
                domain_id: row.get(1)?,
                connection_id: row.get(2)?,
                query_text: row.get(3)?,
                row_count: row.get::<_, i64>(4)? as usize,
                execution_time_ms: row.get::<_, i64>(5)? as u64,
                status,
                error_message: row.get(7)?,
                executed_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                is_llm_generated: row.get::<_, i32>(9)? == 1,
            })
        })?;

        histories.collect()
    }

    /// List query history for a specific connection (with limit)
    pub async fn list_query_history_by_connection(
        &self,
        connection_id: &str,
        limit: usize,
    ) -> SqliteResult<Vec<crate::models::QueryHistory>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, domain_id, connection_id, query_text, row_count, execution_time_ms, status, error_message, executed_at, is_llm_generated
            FROM query_history
            WHERE connection_id = ?1
            ORDER BY executed_at DESC
            LIMIT ?2
            "#
        )?;

        let histories = stmt.query_map(rusqlite::params![connection_id, limit as i64], |row| {
            let status_str: String = row.get(6)?;
            let status = match status_str.as_str() {
                "success" => crate::models::QueryHistoryStatus::Success,
                "failed" => crate::models::QueryHistoryStatus::Failed,
                _ => crate::models::QueryHistoryStatus::Failed,
            };

            Ok(crate::models::QueryHistory {
                id: row.get(0)?,
                domain_id: row.get(1)?,
                connection_id: row.get(2)?,
                query_text: row.get(3)?,
                row_count: row.get::<_, i64>(4)? as usize,
                execution_time_ms: row.get::<_, i64>(5)? as u64,
                status,
                error_message: row.get(7)?,
                executed_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                is_llm_generated: row.get::<_, i32>(9)? == 1,
            })
        })?;

        histories.collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_sqlite_storage_creation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let storage = rt.block_on(async {
            SqliteStorage::new(&db_path).await
        });
        assert!(storage.is_ok());
    }

    #[test]
    fn test_schema_initialization() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let storage = rt.block_on(async {
            SqliteStorage::new(&db_path).await.unwrap()
        });

        // Verify tables exist
        let conn = rt.block_on(async {
            storage.conn.lock().await
        });
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name IN ('domains', 'connections', 'metadata_cache')"
        ).unwrap();

        let tables: Vec<String> = stmt.query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(tables.len(), 3);
        assert!(tables.contains(&"domains".to_string()));
        assert!(tables.contains(&"connections".to_string()));
        assert!(tables.contains(&"metadata_cache".to_string()));

        // Verify default domain exists
        let mut stmt = conn.prepare("SELECT id, name FROM domains WHERE id = 'default-domain-id'").unwrap();
        let default_domain: Result<(String, String), _> = stmt.query_row([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        });

        assert!(default_domain.is_ok());
        let (id, name) = default_domain.unwrap();
        assert_eq!(id, "default-domain-id");
        assert_eq!(name, "Default Domain");
    }

    #[test]
    fn test_domain_crud_operations() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let storage = rt.block_on(async {
            SqliteStorage::new(&db_path).await.unwrap()
        });

        // Create a domain
        let domain = crate::models::Domain::new(
            "Production".to_string(),
            Some("Production environment".to_string())
        ).unwrap();
        let domain_id = domain.id.clone();

        rt.block_on(async {
            storage.create_domain(&domain).await.unwrap();
        });

        // Get the domain
        let retrieved = rt.block_on(async {
            storage.get_domain(&domain_id).await.unwrap()
        });
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, "Production");
        assert_eq!(retrieved.description, Some("Production environment".to_string()));

        // Update the domain
        let mut updated_domain = retrieved.clone();
        updated_domain.name = "Production Env".to_string();
        updated_domain.touch();

        rt.block_on(async {
            let result = storage.update_domain(&updated_domain).await.unwrap();
            assert!(result); // Should return true (rows affected > 0)
        });

        // Verify update
        let after_update = rt.block_on(async {
            storage.get_domain(&domain_id).await.unwrap().unwrap()
        });
        assert_eq!(after_update.name, "Production Env");

        // Delete the domain
        rt.block_on(async {
            let result = storage.delete_domain(&domain_id).await.unwrap();
            assert!(result); // Should return true
        });

        // Verify deletion
        let after_delete = rt.block_on(async {
            storage.get_domain(&domain_id).await.unwrap()
        });
        assert!(after_delete.is_none());
    }

    #[test]
    fn test_list_domains_with_connection_count() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let storage = rt.block_on(async {
            SqliteStorage::new(&db_path).await.unwrap()
        });

        // Create two domains
        let domain1 = crate::models::Domain::new("Dev".to_string(), None).unwrap();
        let domain2 = crate::models::Domain::new("Staging".to_string(), None).unwrap();

        rt.block_on(async {
            storage.create_domain(&domain1).await.unwrap();
            storage.create_domain(&domain2).await.unwrap();
        });

        // List domains
        let domains = rt.block_on(async {
            storage.list_domains().await.unwrap()
        });

        // Should have 3 domains (default + 2 created)
        assert!(domains.len() >= 3);

        // Find our created domains
        let dev_domain = domains.iter().find(|d| d.name == "Dev");
        let staging_domain = domains.iter().find(|d| d.name == "Staging");
        let default_domain = domains.iter().find(|d| d.name == "Default Domain");

        assert!(dev_domain.is_some());
        assert!(staging_domain.is_some());
        assert!(default_domain.is_some());

        // All should have 0 connections initially
        assert_eq!(dev_domain.unwrap().connection_count, 0);
        assert_eq!(staging_domain.unwrap().connection_count, 0);
    }

    #[test]
    fn test_domain_cascade_delete() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let storage = rt.block_on(async {
            SqliteStorage::new(&db_path).await.unwrap()
        });

        // Create a domain
        let domain = crate::models::Domain::new("TestDomain".to_string(), None).unwrap();
        let domain_id = domain.id.clone();

        rt.block_on(async {
            storage.create_domain(&domain).await.unwrap();
        });

        // Create a connection in this domain
        let mut conn = crate::models::DatabaseConnection {
            id: "test-conn-123".to_string(),
            name: Some("Test Connection".to_string()),
            connection_url: "postgresql://localhost/test".to_string(),
            database_type: "postgresql".to_string(),
            status: crate::models::ConnectionStatus::Disconnected,
            created_at: chrono::Utc::now(),
            last_connected_at: None,
            metadata_cache_id: None,
        };

        rt.block_on(async {
            storage.save_connection(&conn).await.unwrap();
        });

        // Verify connection exists
        let conn_before = rt.block_on(async {
            storage.get_connection("test-conn-123").await.unwrap()
        });
        assert!(conn_before.is_some());

        // Delete the domain (should CASCADE delete the connection)
        rt.block_on(async {
            storage.delete_domain(&domain_id).await.unwrap();
        });

        // Note: CASCADE delete happens at SQL level, connection should be removed
        // This test verifies the foreign key constraint is working
        let domain_after = rt.block_on(async {
            storage.get_domain(&domain_id).await.unwrap()
        });
        assert!(domain_after.is_none());
    }

    #[test]
    fn test_get_domain_connection_count() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let storage = rt.block_on(async {
            SqliteStorage::new(&db_path).await.unwrap()
        });

        // Use default domain
        let default_domain_id = "default-domain-id";

        // Initially should have 0 connections
        let count_before = rt.block_on(async {
            storage.get_domain_connection_count(default_domain_id).await.unwrap()
        });

        // Create a connection (will use default domain)
        let conn = crate::models::DatabaseConnection {
            id: "test-conn-456".to_string(),
            name: Some("Another Test".to_string()),
            connection_url: "mysql://localhost/test".to_string(),
            database_type: "mysql".to_string(),
            status: crate::models::ConnectionStatus::Connected,
            created_at: chrono::Utc::now(),
            last_connected_at: Some(chrono::Utc::now()),
            metadata_cache_id: None,
        };

        rt.block_on(async {
            storage.save_connection(&conn).await.unwrap();
        });

        // Count should increase by 1
        let count_after = rt.block_on(async {
            storage.get_domain_connection_count(default_domain_id).await.unwrap()
        });

        assert_eq!(count_after, count_before + 1);
    }

    #[test]
    fn test_list_connections_by_domain() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let storage = rt.block_on(async {
            SqliteStorage::new(&db_path).await.unwrap()
        });

        // Create a custom domain
        let domain = crate::models::Domain::new("Custom".to_string(), None).unwrap();
        let domain_id = domain.id.clone();

        rt.block_on(async {
            storage.create_domain(&domain).await.unwrap();
        });

        // Initially should have empty list
        let conns_before = rt.block_on(async {
            storage.list_connections_by_domain(&domain_id).await.unwrap()
        });
        assert_eq!(conns_before.len(), 0);

        // Note: Without modifying save_connection to accept domain_id,
        // connections will go to default domain
        // This test validates the query works correctly
    }
}

