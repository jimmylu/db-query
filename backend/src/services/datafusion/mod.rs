// DataFusion Semantic Layer Module
//
// This module provides a unified SQL semantic layer using Apache Arrow DataFusion 51.0.0.
// It enables:
// 1. Unified SQL syntax across multiple database types (PostgreSQL, MySQL, Doris, Druid)
// 2. Automatic dialect translation from DataFusion SQL to target database dialects
// 3. Cross-database query execution (federated queries)
// 4. Extensible plugin architecture for new database types

// Phase 2: Core Infrastructure
pub mod session; // DataFusionSessionManager
pub mod catalog; // DataFusionCatalogManager
pub mod dialect; // DialectTranslator trait
pub mod executor; // DataFusionQueryExecutor
pub mod converter; // DataFusionResultConverter

// Phase 3: User Story 1 - Unified SQL
pub mod translator; // Dialect translation service

// Phase 4: User Story 2 - Cross-Database Queries
// pub mod cross_db_planner;  // CrossDatabaseQueryPlanner (coming soon)
// pub mod federated_executor; // FederatedExecutor (coming soon)

// Phase 5: User Story 3 - Extensible Architecture
// pub mod dialect_registry; // DatabaseDialectRegistry (coming soon)

// Re-exports for convenient access
pub use session::{DataFusionSessionManager, SessionConfig};
pub use catalog::DataFusionCatalogManager;
pub use dialect::DialectTranslator;
pub use executor::DataFusionQueryExecutor;
pub use converter::DataFusionResultConverter;
pub use translator::{DialectTranslationService, DatabaseType};
