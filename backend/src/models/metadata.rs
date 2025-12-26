use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMetadata {
    pub id: String,
    pub connection_id: String,
    pub tables: Vec<Table>,
    pub views: Vec<View>,
    pub schemas: Vec<String>,
    pub metadata_json: String,
    pub retrieved_at: chrono::DateTime<chrono::Utc>,
    pub version: i32,
}

impl DatabaseMetadata {
    pub fn new(connection_id: String, tables: Vec<Table>, views: Vec<View>, schemas: Vec<String>) -> Self {
        let json_value = serde_json::json!({
            "tables": &tables,
            "views": &views,
            "schemas": &schemas,
        });
        let metadata_json = serde_json::to_string(&json_value).unwrap_or_default();

        Self {
            id: Uuid::new_v4().to_string(),
            connection_id,
            tables,
            views,
            schemas,
            metadata_json,
            retrieved_at: chrono::Utc::now(),
            version: 1,
        }
    }

    pub fn increment_version(&mut self) {
        self.version += 1;
        self.retrieved_at = chrono::Utc::now();
        // Update metadata_json as well
        let json_value = serde_json::json!({
            "tables": &self.tables,
            "views": &self.views,
            "schemas": &self.schemas,
        });
        self.metadata_json = serde_json::to_string(&json_value).unwrap_or_default();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub schema: Option<String>,
    pub columns: Vec<Column>,
    pub row_count: Option<i64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct View {
    pub name: String,
    pub schema: Option<String>,
    pub columns: Vec<Column>,
    pub definition: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub is_foreign_key: bool,
    pub default_value: Option<String>,
    pub max_length: Option<i32>,
    pub description: Option<String>,
}

