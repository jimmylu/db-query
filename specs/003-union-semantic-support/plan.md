# Implementation Plan: Unified SQL Semantic Layer

**Feature ID**: 003-union-semantic-support
**Status**: In Progress
**Created**: 2025-12-26

## Overview

This plan outlines the implementation of a unified SQL semantic layer using Apache Arrow DataFusion 51.0.0. The semantic layer will enable users to query multiple databases using a single SQL syntax, with automatic dialect translation and support for cross-database operations.

## Goals

1. **Unified SQL Syntax**: Users can write SQL queries once and execute them against any supported database (PostgreSQL, MySQL, Doris, Druid)
2. **Automatic Dialect Translation**: DataFusion SQL is automatically translated to the target database's native dialect
3. **Cross-Database Queries**: Support JOINs and UNIONs across different database types
4. **Extensibility**: Easy addition of new database types through plugin architecture

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         Frontend                             │
│  (React + Refine + Monaco Editor)                           │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                      Backend API Layer                       │
│              (Axum + REST Endpoints)                         │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                  Query Service Layer                         │
│         (Validation + Query Orchestration)                   │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│            DataFusion Semantic Layer (NEW)                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  SessionManager   CatalogManager   DialectTranslator │   │
│  │  QueryExecutor    ResultConverter  FederatedExecutor │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│              Database Adapter Layer                          │
│  PostgreSQL  │  MySQL  │  Doris  │  Druid                   │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                   Physical Databases                         │
└─────────────────────────────────────────────────────────────┘
```

### DataFusion Semantic Layer Components

#### 1. SessionManager (`session.rs`)
- Manages DataFusion SessionContext lifecycle
- Integrates with existing connection pool
- Provides session factory for query execution
- Handles session configuration and cleanup

#### 2. CatalogManager (`catalog.rs`)
- Registers database tables as DataFusion catalogs
- Maps database schemas to DataFusion SchemaProviders
- Dynamically loads table metadata from connected databases
- Maintains catalog registry for multi-database queries

#### 3. DialectTranslator (`dialect.rs` + `translator.rs`)
- Defines DialectTranslator trait for database-specific translation
- Implements PostgreSQL dialect translator
- Implements MySQL dialect translator
- Handles dialect-specific SQL syntax differences

#### 4. QueryExecutor (`executor.rs`)
- Parses SQL queries into DataFusion logical plans
- Optimizes query execution plans
- Executes queries with timeout handling
- Coordinates with database adapters for actual execution

#### 5. ResultConverter (`converter.rs`)
- Converts DataFusion RecordBatch to JSON format
- Handles type conversions for all supported data types
- Maintains compatibility with existing QueryResult model

#### 6. FederatedExecutor (`federated_executor.rs`) - Phase 4
- Decomposes cross-database queries into sub-queries
- Coordinates execution across multiple databases
- Merges results from different sources
- Handles JOIN and UNION operations across databases

## Tech Stack

### Backend (Rust)

**Core Framework**:
- Axum 0.8.8 - Web framework
- Tokio 1.x - Async runtime
- Tower 0.5.2 - Middleware

**DataFusion Integration** (NEW):
- `datafusion = "51.0.0"` - Query engine and semantic layer
- `datafusion-table-providers` (optional) - Multi-database connectors
- `datafusion-federation` (optional) - Cross-database query support

**Existing Dependencies**:
- `sqlparser = "0.60.0"` - SQL parsing and validation
- `tokio-postgres = "0.7"` - PostgreSQL driver
- `mysql_async = "0.35"` - MySQL driver
- `deadpool-postgres = "0.14"` - PostgreSQL connection pooling
- `rusqlite = "0.38.0"` - SQLite for metadata storage
- `serde = "1.0"` + `serde_json = "1.0"` - Serialization
- `reqwest = "0.12.26"` - LLM API client
- `anyhow = "1.0"` + `thiserror = "2.0.17"` - Error handling
- `tracing = "0.1"` + `tracing-subscriber = "0.3"` - Logging

### Frontend (React + TypeScript)

**No changes required for Phase 1-2**. Frontend will continue using existing:
- React 18 + TypeScript
- Refine 5 + Ant Design
- Monaco Editor for SQL editing
- Axios for API calls

## File Structure

### New Files

```
backend/src/services/datafusion/
├── mod.rs                      # Module exports and common types
├── session.rs                  # DataFusionSessionManager
├── catalog.rs                  # DataFusionCatalogManager
├── dialect.rs                  # DialectTranslator trait + implementations
├── translator.rs               # Dialect translation service
├── executor.rs                 # DataFusionQueryExecutor
├── converter.rs                # DataFusionResultConverter
├── cross_db_planner.rs         # CrossDatabaseQueryPlanner (Phase 4)
├── federated_executor.rs       # FederatedExecutor (Phase 4)
└── dialect_registry.rs         # DatabaseDialectRegistry (Phase 5)
```

### Modified Files

```
backend/src/
├── models/
│   ├── query.rs                # Add database_type field
│   ├── unified_query.rs        # New: UnifiedQueryRequest model
│   └── cross_database_query.rs # New: CrossDatabaseQueryRequest (Phase 4)
├── services/
│   ├── query_service.rs        # Integrate DataFusion executor
│   ├── database/
│   │   ├── adapter.rs          # Update trait for DataFusion support
│   │   ├── postgresql.rs       # Refactor to use DataFusion
│   │   ├── mysql.rs            # Refactor to use DataFusion
│   │   ├── doris.rs            # Update for DataFusion (Phase 5)
│   │   ├── druid.rs            # Update for DataFusion (Phase 5)
│   │   ├── factory.rs          # New: Adapter factory (Phase 5)
│   │   └── plugin.rs           # New: Plugin system (Phase 5)
├── validation/
│   ├── sql_validator.rs        # Support DataFusion SQL syntax
│   └── cross_db_validator.rs   # New: Cross-DB validation (Phase 4)
└── api/handlers/
    ├── query.rs                # Use unified SQL
    └── cross_database_query.rs # New: Cross-DB endpoint (Phase 4)
```

### Frontend Files (Phase 3)

```
frontend/src/
├── services/
│   ├── unified_query.ts        # New: Unified query API client
│   └── cross_database_query.ts # New: Cross-DB API client (Phase 4)
├── components/
│   ├── QueryEditor/            # Update for unified SQL
│   └── CrossDatabaseQueryBuilder/ # New: Cross-DB UI (Phase 4)
└── pages/
    └── QueryPage.tsx           # Add database type indicator
```

## Implementation Phases

### Phase 1: Setup & Research ✓
- [x] Research DataFusion capabilities
- [x] Create planning documents
- [ ] Verify DataFusion dependencies
- [ ] Create datafusion module structure

### Phase 2: DataFusion Core Infrastructure (CRITICAL)
**Purpose**: Build the foundation that all other features depend on

**Deliverables**:
1. SessionManager with connection pool integration
2. CatalogManager for PostgreSQL and MySQL
3. DialectTranslator trait and implementations
4. QueryExecutor with timeout handling
5. ResultConverter for JSON output

**Acceptance Criteria**:
- Can create SessionContext from existing connection pools
- Can register PostgreSQL and MySQL tables as catalogs
- Can translate DataFusion SQL to PostgreSQL/MySQL dialects
- Can execute queries and convert results to JSON
- All existing query tests still pass

### Phase 3: User Story 1 - Unified SQL (MVP)
**Goal**: Single database queries with unified SQL syntax

**Deliverables**:
1. UnifiedQueryRequest model
2. Refactored database adapters using DataFusion
3. Updated QueryService
4. Frontend unified query support

**Acceptance Criteria**:
- Users can execute DataFusion SQL against PostgreSQL
- Users can execute DataFusion SQL against MySQL
- Dialect translation happens automatically
- Error messages indicate dialect issues
- All security validations still enforced

### Phase 4: User Story 2 - Cross-Database Queries
**Goal**: JOIN and UNION across databases

**Deliverables**:
1. CrossDatabaseQueryPlanner
2. FederatedExecutor
3. Cross-database API endpoint
4. Frontend query builder UI

**Acceptance Criteria**:
- Can JOIN PostgreSQL and MySQL tables
- Can UNION results from different databases
- Query optimization works across databases
- Performance is acceptable for typical queries

### Phase 5: User Story 3 - Extensible Architecture
**Goal**: Plugin-based database support

**Deliverables**:
1. DatabaseDialectRegistry
2. Adapter factory pattern
3. Plugin system
4. Doris and Druid adapters as plugins

**Acceptance Criteria**:
- New database can be added with minimal code
- Plugins can be loaded dynamically
- Plugin configuration is documented
- Doris and Druid fully supported

### Phase 6: Polish & Optimization
**Deliverables**:
1. Query plan visualization
2. Performance metrics
3. Result caching
4. Documentation and migration guide

## Security Considerations

### Maintained Security Requirements

All existing security requirements from the project constitution must be maintained:

1. **SELECT-Only Queries**: Only SELECT statements allowed (enforced by SqlValidator)
2. **SQL Injection Prevention**: Continue using sqlparser for validation
3. **Query Limits**: Auto-append LIMIT 1000 if not specified
4. **Connection Security**: Maintain existing connection pooling and timeouts
5. **Error Message Safety**: Don't expose connection details in errors

### New Security Considerations

1. **DataFusion Query Validation**: Ensure DataFusion logical plans only contain SELECT operations
2. **Cross-Database Access Control**: Validate user has access to all databases in cross-DB queries
3. **Resource Limits**: Set memory limits for DataFusion execution
4. **Catalog Isolation**: Ensure users can only access catalogs they're authorized for

## Performance Considerations

### Expected Performance Characteristics

1. **Single Database Queries**:
   - Slight overhead from DataFusion layer (~5-10ms)
   - Benefit from DataFusion optimizations (pushdown, vectorization)
   - Overall: Should be comparable to current implementation

2. **Cross-Database Queries**:
   - Higher latency due to multiple network calls
   - DataFusion will optimize pushdown where possible
   - Expected: 100-500ms for typical JOINs

3. **Optimization Strategies**:
   - Predicate pushdown to databases
   - Projection pushdown (only fetch needed columns)
   - Join reordering
   - Result caching (Phase 6)

### Resource Management

1. **Memory**: DataFusion uses streaming execution, limited by batch size
2. **CPU**: Vectorized execution for efficient processing
3. **Network**: Connection pooling reused from existing implementation
4. **Timeout**: Existing timeout mechanisms apply

## Error Handling

### Error Categories

1. **SQL Parsing Errors**: Invalid SQL syntax
2. **Dialect Translation Errors**: Unsupported SQL features in target dialect
3. **Execution Errors**: Database connection, timeout, data errors
4. **Cross-Database Errors**: Table not found, incompatible schemas

### Error Messages

All errors should be user-friendly and actionable:
- "SQL syntax not supported in MySQL dialect: RETURNING clause"
- "Cross-database query failed: table 'users' not found in PostgreSQL connection"
- "Query timeout after 30s: consider adding filters or reducing data range"

## Testing Strategy

### Unit Tests
- SessionManager lifecycle
- CatalogManager registration
- Dialect translation accuracy
- Result conversion correctness

### Integration Tests
- Query execution against test databases
- Cross-database JOINs
- Error handling scenarios
- Performance benchmarks

### Acceptance Tests
- User Story 1: Single database unified SQL
- User Story 2: Cross-database queries
- User Story 3: Plugin system

## Migration Strategy

### Backward Compatibility

**Phase 1-3**: Existing API endpoints continue to work
- `/api/connections/{id}/query` still accepts direct SQL
- Old queries automatically use new DataFusion layer
- No breaking changes to frontend

**Phase 4**: New endpoints for cross-database queries
- `/api/cross-database/query` for federated queries
- Old single-database queries unchanged

**Phase 5**: Plugin system enables new databases
- Existing PostgreSQL/MySQL adapters work as before
- New plugin-based adapters for Doris/Druid

### Rollout Plan

1. **Phase 1-2**: Internal testing only, no user-facing changes
2. **Phase 3**: Gradual rollout with feature flag
3. **Phase 4-5**: Opt-in beta for cross-database features
4. **Phase 6**: Full production release

## Risks and Mitigation

### Risk 1: DataFusion Learning Curve
**Impact**: Medium
**Likelihood**: High
**Mitigation**: Comprehensive research completed, examples available

### Risk 2: Performance Regression
**Impact**: High
**Likelihood**: Low
**Mitigation**: Benchmark tests, gradual rollout, feature flag

### Risk 3: Dialect Translation Accuracy
**Impact**: High
**Likelihood**: Medium
**Mitigation**: Comprehensive test suite, user feedback loop

### Risk 4: Breaking Changes in DataFusion
**Impact**: Medium
**Likelihood**: Low
**Mitigation**: Pin to specific version (51.0.0), monitor releases

## Success Metrics

### Phase 3 (MVP) Success Criteria
- 100% of existing single-database queries work with new layer
- Performance within 10% of current implementation
- Zero security regressions
- Positive user feedback on unified SQL syntax

### Phase 4 Success Criteria
- Cross-database JOINs complete in < 1 second for typical queries
- Support for at least 3 common cross-database use cases
- Clear error messages for 90% of failure scenarios

### Phase 5 Success Criteria
- New database type can be added in < 2 days
- Plugin documentation enables third-party contributions
- Doris and Druid fully functional

## Timeline Estimate

- **Phase 1**: 1 day (Setup & Research) ✓
- **Phase 2**: 3-5 days (Core Infrastructure) - CRITICAL PATH
- **Phase 3**: 5-7 days (MVP - Unified SQL)
- **Phase 4**: 7-10 days (Cross-Database Queries)
- **Phase 5**: 5-7 days (Plugin Architecture)
- **Phase 6**: 3-5 days (Polish & Documentation)

**Total**: 24-35 days for full implementation

**MVP Delivery**: Phase 1-3 = 9-13 days

## References

- [DataFusion Documentation](https://arrow.apache.org/datafusion/)
- [DataFusion GitHub](https://github.com/apache/arrow-datafusion)
- Research Document: `specs/003-union-semantic-support/research.md`
- Tasks Breakdown: `specs/003-union-semantic-support/tasks.md`
- Project Constitution: `.specify/memory/constitution.md`

## Approval

**Technical Lead**: [Pending]
**Security Review**: [Pending]
**Architecture Review**: [Pending]

---

*Last Updated*: 2025-12-26
*Status*: Phase 1 In Progress
