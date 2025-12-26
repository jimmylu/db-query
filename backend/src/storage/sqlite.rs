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
                metadata_cache_id TEXT
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
            "SELECT id, name, connection_url, database_type, status, created_at, last_connected_at, metadata_cache_id FROM connections WHERE id = ?1"
        )?;

        let result = stmt.query_row(rusqlite::params![id], |row| {
            Ok(crate::models::DatabaseConnection {
                id: row.get(0)?,
                name: row.get(1)?,
                connection_url: row.get(2)?,
                database_type: row.get(3)?,
                status: match row.get::<_, String>(4)?.as_str() {
                    "connected" => crate::models::ConnectionStatus::Connected,
                    "error" => crate::models::ConnectionStatus::Error,
                    _ => crate::models::ConnectionStatus::Disconnected,
                },
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                last_connected_at: row.get::<_, Option<String>>(6)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                metadata_cache_id: row.get(7)?,
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
            "SELECT id, name, connection_url, database_type, status, created_at, last_connected_at, metadata_cache_id FROM connections ORDER BY created_at DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(crate::models::DatabaseConnection {
                id: row.get(0)?,
                name: row.get(1)?,
                connection_url: row.get(2)?,
                database_type: row.get(3)?,
                status: match row.get::<_, String>(4)?.as_str() {
                    "connected" => crate::models::ConnectionStatus::Connected,
                    "error" => crate::models::ConnectionStatus::Error,
                    _ => crate::models::ConnectionStatus::Disconnected,
                },
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                last_connected_at: row.get::<_, Option<String>>(6)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                metadata_cache_id: row.get(7)?,
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
            "SELECT name FROM sqlite_master WHERE type='table' AND name IN ('connections', 'metadata_cache')"
        ).unwrap();

        let tables: Vec<String> = stmt.query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(tables.len(), 2);
        assert!(tables.contains(&"connections".to_string()));
        assert!(tables.contains(&"metadata_cache".to_string()));
    }
}

