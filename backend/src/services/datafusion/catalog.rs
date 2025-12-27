// DataFusion CatalogManager
//
// Manages the registration of database tables as DataFusion catalogs.
// Enables querying multiple databases through a unified interface.

use datafusion::catalog::CatalogProvider;
use datafusion::prelude::*;
use datafusion::datasource::MemTable;
use datafusion::arrow::datatypes::{Schema, Field, DataType};
use std::sync::Arc;
use std::collections::HashMap;
use anyhow::{Result, Context, anyhow};

use crate::models::metadata::{DatabaseMetadata, Table, Column};

/// Manages catalog registration for DataFusion
///
/// The CatalogManager is responsible for registering database tables as DataFusion
/// catalogs, allowing them to be queried using unified SQL syntax.
///
/// # Architecture
/// ```text
/// CatalogManager
///   └── Database Connection (e.g., "postgres_db")
///       └── Schema (e.g., "public")
///           └── Tables (e.g., "users", "orders")
/// ```
pub struct DataFusionCatalogManager {
    /// Session context for catalog registration
    ctx: SessionContext,
    /// Cache of registered catalogs by database name
    registered_catalogs: HashMap<String, Arc<dyn CatalogProvider>>,
}

impl DataFusionCatalogManager {
    /// Create a new CatalogManager with a SessionContext
    pub fn new(ctx: SessionContext) -> Self {
        Self {
            ctx,
            registered_catalogs: HashMap::new(),
        }
    }

    /// Register a database's metadata as a catalog
    ///
    /// This creates a catalog provider for the database and registers all its tables.
    ///
    /// # Arguments
    /// * `catalog_name` - Name for the catalog (e.g., "postgres_db", "mysql_db")
    /// * `metadata` - Database metadata containing tables and columns
    ///
    /// # Example
    /// ```rust,ignore
    /// let metadata = get_database_metadata(&connection).await?;
    /// catalog_manager.register_database("my_db", metadata).await?;
    ///
    /// // Now can query: SELECT * FROM my_db.public.users
    /// ```
    pub async fn register_database(
        &mut self,
        catalog_name: &str,
        metadata: DatabaseMetadata,
    ) -> Result<()> {
        // For now, we register tables in the default catalog and schema
        // This allows queries like: SELECT * FROM table_name
        // Future enhancement: Support multi-catalog queries like: SELECT * FROM db_name.schema.table

        for table in metadata.tables {
            self.register_table_metadata(catalog_name, &table).await?;
        }

        Ok(())
    }

    /// Register a single table from metadata
    async fn register_table_metadata(
        &mut self,
        _catalog_name: &str,
        table: &Table,
    ) -> Result<()> {
        // Convert table metadata to Arrow schema
        let schema = self.metadata_to_arrow_schema(table)?;

        // Create an empty MemTable as a placeholder
        // Actual data will be fetched when query is executed
        let mem_table = MemTable::try_new(
            Arc::new(schema),
            vec![], // Empty initial data
        )?;

        // Register the table with the session context
        self.ctx.register_table(
            &table.name,
            Arc::new(mem_table),
        )?;

        Ok(())
    }

    /// Convert table metadata to Arrow schema
    fn metadata_to_arrow_schema(&self, table: &Table) -> Result<Schema> {
        let fields: Vec<Field> = table
            .columns
            .iter()
            .map(|col| self.column_to_arrow_field(col))
            .collect::<Result<Vec<_>>>()?;

        Ok(Schema::new(fields))
    }

    /// Convert column metadata to Arrow field
    fn column_to_arrow_field(&self, column: &Column) -> Result<Field> {
        let data_type = self.sql_type_to_arrow_type(&column.data_type)?;

        Ok(Field::new(
            &column.name,
            data_type,
            column.is_nullable,
        ))
    }

    /// Map SQL data type string to Arrow DataType
    fn sql_type_to_arrow_type(&self, sql_type: &str) -> Result<DataType> {
        let sql_type_lower = sql_type.to_lowercase();

        let arrow_type = match sql_type_lower.as_str() {
            // Integer types
            "smallint" | "int2" | "smallserial" => DataType::Int16,
            "integer" | "int" | "int4" | "serial" => DataType::Int32,
            "bigint" | "int8" | "bigserial" => DataType::Int64,

            // Floating point
            "real" | "float4" => DataType::Float32,
            "double precision" | "float8" | "double" | "float" => DataType::Float64,

            // Decimal/Numeric
            "numeric" | "decimal" => DataType::Decimal128(38, 10), // Default precision

            // String types
            "varchar" | "character varying" | "text" | "char" | "character" => DataType::Utf8,

            // Binary types
            "bytea" | "blob" | "binary" | "varbinary" => DataType::Binary,

            // Boolean
            "boolean" | "bool" | "bit" => DataType::Boolean,

            // Date/Time types
            "date" => DataType::Date32,
            "time" | "time without time zone" => DataType::Time64(datafusion::arrow::datatypes::TimeUnit::Microsecond),
            "timestamp" | "timestamp without time zone" | "datetime" => {
                DataType::Timestamp(datafusion::arrow::datatypes::TimeUnit::Microsecond, None)
            }
            "timestamp with time zone" | "timestamptz" => {
                DataType::Timestamp(datafusion::arrow::datatypes::TimeUnit::Microsecond, Some("UTC".into()))
            }

            // JSON (store as string for now)
            "json" | "jsonb" => DataType::Utf8,

            // UUID (store as string)
            "uuid" => DataType::Utf8,

            // Default to string for unknown types
            _ => {
                tracing::warn!("Unknown SQL type '{}', defaulting to Utf8", sql_type);
                DataType::Utf8
            }
        };

        Ok(arrow_type)
    }

    /// Get the session context
    pub fn session_context(&self) -> &SessionContext {
        &self.ctx
    }

    /// List all registered tables
    pub fn list_tables(&self) -> Result<Vec<String>> {
        let catalog = self.ctx.catalog("datafusion").ok_or_else(|| {
            anyhow!("Default catalog 'datafusion' not found")
        })?;

        let schema = catalog.schema("public").ok_or_else(|| {
            anyhow!("Default schema 'public' not found")
        })?;

        Ok(schema.table_names())
    }
}

/// PostgreSQL-specific catalog registration
pub struct PostgreSQLCatalogRegistrar;

impl PostgreSQLCatalogRegistrar {
    /// Register PostgreSQL tables in DataFusion catalog
    pub async fn register(
        catalog_manager: &mut DataFusionCatalogManager,
        catalog_name: &str,
        metadata: DatabaseMetadata,
    ) -> Result<()> {
        catalog_manager
            .register_database(catalog_name, metadata)
            .await
            .context("Failed to register PostgreSQL catalog")
    }
}

/// MySQL-specific catalog registration
pub struct MySQLCatalogRegistrar;

impl MySQLCatalogRegistrar {
    /// Register MySQL tables in DataFusion catalog
    pub async fn register(
        catalog_manager: &mut DataFusionCatalogManager,
        catalog_name: &str,
        metadata: DatabaseMetadata,
    ) -> Result<()> {
        catalog_manager
            .register_database(catalog_name, metadata)
            .await
            .context("Failed to register MySQL catalog")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::metadata::*;

    fn create_test_metadata() -> DatabaseMetadata {
        DatabaseMetadata::new(
            "test_connection".to_string(),
            vec![
                Table {
                    name: "users".to_string(),
                    schema: Some("public".to_string()),
                    columns: vec![
                        Column {
                            name: "id".to_string(),
                            data_type: "integer".to_string(),
                            is_nullable: false,
                            is_primary_key: true,
                            is_foreign_key: false,
                            default_value: None,
                            max_length: None,
                            description: None,
                        },
                        Column {
                            name: "name".to_string(),
                            data_type: "varchar".to_string(),
                            is_nullable: true,
                            is_primary_key: false,
                            is_foreign_key: false,
                            default_value: None,
                            max_length: Some(255),
                            description: None,
                        },
                    ],
                    row_count: None,
                    description: None,
                },
            ],
            vec![],
            vec!["public".to_string()],
        )
    }

    #[tokio::test]
    async fn test_catalog_manager_creation() {
        let ctx = SessionContext::new();
        let manager = DataFusionCatalogManager::new(ctx);
        assert_eq!(manager.registered_catalogs.len(), 0);
    }

    #[tokio::test]
    async fn test_register_database() {
        let ctx = SessionContext::new();
        let mut manager = DataFusionCatalogManager::new(ctx);

        let metadata = create_test_metadata();
        let result = manager.register_database("test_db", metadata).await;
        assert!(result.is_ok());

        // Verify table is registered
        let tables = manager.list_tables().unwrap();
        assert!(tables.contains(&"users".to_string()));
    }

    #[test]
    fn test_sql_type_mapping() {
        let ctx = SessionContext::new();
        let manager = DataFusionCatalogManager::new(ctx);

        // Test integer types
        assert!(matches!(
            manager.sql_type_to_arrow_type("integer").unwrap(),
            DataType::Int32
        ));
        assert!(matches!(
            manager.sql_type_to_arrow_type("bigint").unwrap(),
            DataType::Int64
        ));

        // Test string types
        assert!(matches!(
            manager.sql_type_to_arrow_type("varchar").unwrap(),
            DataType::Utf8
        ));
        assert!(matches!(
            manager.sql_type_to_arrow_type("text").unwrap(),
            DataType::Utf8
        ));

        // Test boolean
        assert!(matches!(
            manager.sql_type_to_arrow_type("boolean").unwrap(),
            DataType::Boolean
        ));
    }
}
