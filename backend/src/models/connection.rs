use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConnection {
    pub id: String,
    pub name: Option<String>,
    pub connection_url: String,
    pub database_type: String,
    pub status: ConnectionStatus,
    pub created_at: DateTime<Utc>,
    pub last_connected_at: Option<DateTime<Utc>>,
    pub metadata_cache_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Error,
}

impl DatabaseConnection {
    pub fn new(
        name: Option<String>,
        connection_url: String,
        database_type: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            connection_url,
            database_type,
            status: ConnectionStatus::Disconnected,
            created_at: Utc::now(),
            last_connected_at: None,
            metadata_cache_id: None,
        }
    }

    pub fn mark_connected(&mut self) {
        self.status = ConnectionStatus::Connected;
        self.last_connected_at = Some(Utc::now());
    }

    pub fn mark_disconnected(&mut self) {
        self.status = ConnectionStatus::Disconnected;
    }

    pub fn mark_error(&mut self) {
        self.status = ConnectionStatus::Error;
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateConnectionRequest {
    pub name: Option<String>,
    pub connection_url: String,
    #[serde(default = "default_database_type")]
    pub database_type: String,
}

fn default_database_type() -> String {
    "postgresql".to_string()
}

