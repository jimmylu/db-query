# Specification: Unified SQL Semantic Layer

**Feature ID**: 003-union-semantic-support
**Priority**: P1 (High)
**Status**: In Development
**Created**: 2025-12-26

## Executive Summary

Enable users to query multiple database types using a single, unified SQL syntax. The system will automatically translate queries to the appropriate database dialect and support cross-database operations like JOINs and UNIONs.

## Problem Statement

### Current Limitations

1. **Multiple SQL Dialects**: Users must learn different SQL syntax for PostgreSQL, MySQL, Doris, and Druid
2. **Query Portability**: Queries written for one database don't work on another without modification
3. **Cross-Database Queries**: No way to JOIN data from PostgreSQL with data from MySQL
4. **Maintenance Burden**: Each database type requires custom query logic and testing

### User Pain Points

> "I have to remember whether to use `CONCAT()` or `||` depending on which database I'm querying."

> "I need to join user data from PostgreSQL with order data from MySQL, but I have to export and manually combine them."

> "Every time I switch databases, I have to rewrite my queries. It's frustrating and error-prone."

## Goals

### Primary Goals

1. **Unified SQL Syntax**: One SQL dialect works across all supported databases
2. **Automatic Translation**: System handles dialect differences transparently
3. **Cross-Database Queries**: Support JOINs and UNIONs across different database types
4. **Extensibility**: Easy to add new database types

### Non-Goals

1. **Write Operations**: Still limited to SELECT queries only (security requirement)
2. **Transaction Coordination**: No distributed transactions across databases
3. **Schema Migration**: Not a database migration tool
4. **Full SQL Standard**: Not all SQL features will be supported in all databases

## User Stories

### User Story 1: Single Database Query with Unified SQL (Priority: P1) 🎯 MVP

**As a** database user
**I want to** write SQL queries using a single syntax
**So that** I don't have to learn different dialects for each database type

#### Acceptance Criteria

1. **Standard SQL Syntax**:
   - User writes a SELECT query using standard SQL
   - Query includes common operations: WHERE, JOIN, GROUP BY, ORDER BY, LIMIT
   - System accepts the query without dialect-specific syntax

2. **Automatic Dialect Translation**:
   - System detects target database type (PostgreSQL, MySQL, etc.)
   - Query is automatically translated to target dialect
   - Translation preserves query semantics
   - User receives results without knowing translation occurred

3. **Error Handling**:
   - If SQL feature not supported in target dialect, clear error message
   - Error message suggests alternative approach or compatible syntax
   - Validation errors caught before sending to database

4. **Performance**:
   - Query execution time within 10% of direct database query
   - No noticeable latency from translation layer
   - Results returned in existing JSON format

#### Example Scenarios

**Scenario 1: String Concatenation**

```sql
-- User writes (standard SQL):
SELECT first_name || ' ' || last_name AS full_name FROM users;

-- PostgreSQL: Query runs as-is (|| supported)
-- MySQL: Automatically translated to:
SELECT CONCAT(first_name, ' ', last_name) AS full_name FROM users;
```

**Scenario 2: Date Functions**

```sql
-- User writes:
SELECT * FROM orders WHERE order_date >= CURRENT_DATE - INTERVAL '7 days';

-- PostgreSQL: Query runs as-is
-- MySQL: Automatically translated to:
SELECT * FROM orders WHERE order_date >= DATE_SUB(CURDATE(), INTERVAL 7 DAY);
```

**Scenario 3: Identifier Quoting**

```sql
-- User writes:
SELECT "order_id", "total" FROM "orders";

-- PostgreSQL: Query runs as-is (double quotes)
-- MySQL: Automatically translated to:
SELECT `order_id`, `total` FROM `orders`;
```

#### Testing

- [ ] Execute unified SQL query against PostgreSQL database
- [ ] Execute same query against MySQL database
- [ ] Verify both return identical results (accounting for data differences)
- [ ] Test string functions (concatenation, substring, etc.)
- [ ] Test date functions (current date, date arithmetic, etc.)
- [ ] Test identifier quoting for reserved words
- [ ] Test aggregation functions (COUNT, SUM, AVG, etc.)
- [ ] Test JOINs within single database
- [ ] Test subqueries
- [ ] Test LIMIT/OFFSET pagination
- [ ] Verify error messages for unsupported features
- [ ] Performance benchmark vs direct queries

#### Success Metrics

- 95% of common queries work without modification across databases
- Performance overhead < 10% compared to direct queries
- User survey: 80%+ find unified SQL easier than multiple dialects
- Support tickets related to dialect issues reduced by 70%

---

### User Story 2: Cross-Database Join Queries (Priority: P2)

**As a** data analyst
**I want to** JOIN tables from different databases in a single query
**So that** I can analyze related data without manual export/import

#### Acceptance Criteria

1. **Cross-Database JOIN Syntax**:
   - User can specify database in table reference: `database_name.table_name`
   - Standard JOIN syntax works across databases
   - System coordinates query execution across multiple databases

2. **Query Execution**:
   - System decomposes query into sub-queries per database
   - Sub-queries executed in parallel where possible
   - Results merged according to JOIN conditions
   - Final result returned in unified format

3. **Supported Operations**:
   - INNER JOIN across databases
   - LEFT/RIGHT JOIN across databases
   - UNION across databases
   - WHERE clause filters applied optimally (pushdown)

4. **Performance**:
   - Query completes in < 1 second for typical datasets (< 10,000 rows)
   - Progress indicator for long-running queries
   - Query can be cancelled mid-execution

#### Example Scenarios

**Scenario 1: Simple Cross-Database JOIN**

```sql
-- Join PostgreSQL users with MySQL orders
SELECT
    u.user_id,
    u.name,
    o.order_id,
    o.total
FROM postgres_db.users u
JOIN mysql_db.orders o ON u.user_id = o.user_id
WHERE o.order_date >= CURRENT_DATE - INTERVAL '30 days';
```

**Execution Plan**:
1. Fetch users from PostgreSQL (with filters pushed down)
2. Fetch orders from MySQL (with filters pushed down)
3. Perform JOIN in DataFusion execution engine
4. Return unified results

**Scenario 2: Cross-Database UNION**

```sql
-- Combine events from multiple databases
SELECT event_id, event_type, event_date FROM postgres_db.events
UNION ALL
SELECT event_id, event_type, event_date FROM mysql_db.events
ORDER BY event_date DESC
LIMIT 100;
```

**Scenario 3: Complex Multi-Database Query**

```sql
-- Join three tables from two databases
SELECT
    u.name,
    COUNT(o.order_id) as order_count,
    SUM(oi.quantity * oi.price) as total_spent
FROM postgres_db.users u
LEFT JOIN mysql_db.orders o ON u.user_id = o.user_id
LEFT JOIN mysql_db.order_items oi ON o.order_id = oi.order_id
GROUP BY u.user_id, u.name
HAVING total_spent > 1000
ORDER BY total_spent DESC;
```

#### Testing

- [ ] Execute INNER JOIN across PostgreSQL and MySQL
- [ ] Execute LEFT JOIN with NULL handling
- [ ] Execute UNION of tables from different databases
- [ ] Test WHERE clause pushdown optimization
- [ ] Test aggregation after cross-database JOIN
- [ ] Test subquery in cross-database context
- [ ] Verify result correctness vs manual joins
- [ ] Performance test with 10K, 100K, 1M rows
- [ ] Test query cancellation
- [ ] Test error handling (connection failure, table not found)
- [ ] Test with 3+ databases in single query

#### Success Metrics

- Cross-database JOINs complete in < 1 second for 10K rows
- Query accuracy: 100% match with manual join results
- User adoption: 50%+ of users try cross-database feature
- Use cases enabled: 10+ real-world scenarios documented

---

### User Story 3: Extensible Database Support (Priority: P3)

**As a** platform administrator
**I want to** easily add support for new database types
**So that** the system can grow with our needs

#### Acceptance Criteria

1. **Plugin Architecture**:
   - New database type can be added by implementing adapter interface
   - Dialect translator defined separately from adapter
   - Minimal code changes to core system

2. **Registration System**:
   - Databases register their dialect translator at startup
   - System discovers available database types dynamically
   - Frontend displays all registered database types

3. **Documentation**:
   - Plugin template with clear examples
   - Step-by-step guide for adding new database
   - API documentation for adapter interface

4. **Validation**:
   - New database automatically gains unified SQL support
   - New database can participate in cross-database queries
   - Existing databases unaffected by new additions

#### Example: Adding Apache Doris Support

**Step 1**: Implement Adapter

```rust
// backend/src/services/database/doris_plugin.rs
pub struct DorisAdapter {
    connection_pool: DorisConnectionPool,
}

#[async_trait]
impl DatabaseAdapter for DorisAdapter {
    async fn execute_query(&self, sql: &str, timeout: Duration) -> Result<QueryResult> {
        // Implementation
    }

    async fn get_metadata(&self) -> Result<DatabaseMetadata> {
        // Implementation
    }
}
```

**Step 2**: Implement Dialect Translator

```rust
// backend/src/services/datafusion/dialect.rs
pub struct DorisDialectTranslator;

impl DialectTranslator for DorisDialectTranslator {
    fn translate(&self, datafusion_sql: &str) -> Result<String> {
        // Translation logic
    }
}
```

**Step 3**: Register Plugin

```rust
// backend/src/main.rs
database_registry.register("doris", DorisPlugin::new());
```

**Result**: Doris now works with:
- Unified SQL queries
- Cross-database JOINs with PostgreSQL/MySQL
- All existing features

#### Testing

- [ ] Add Doris adapter following plugin template
- [ ] Add Druid adapter following plugin template
- [ ] Verify both work with unified SQL
- [ ] Verify both work in cross-database queries
- [ ] Test plugin registration system
- [ ] Test dynamic database type discovery
- [ ] Verify plugin documentation accuracy
- [ ] Ensure existing databases still work
- [ ] Test error handling for invalid plugins

#### Success Metrics

- New database type added in < 2 days by senior engineer
- Plugin documentation rated 4/5+ by developers
- 2+ community-contributed database adapters
- Zero regressions when adding new databases

---

## Functional Requirements

### FR-1: Unified SQL Parser
**Priority**: P1
- System must accept standard SQL syntax
- Must support SELECT, WHERE, JOIN, GROUP BY, ORDER BY, LIMIT, OFFSET
- Must validate queries before execution
- Must maintain existing security restrictions (SELECT-only, LIMIT enforcement)

### FR-2: Dialect Translator
**Priority**: P1
- Must translate standard SQL to PostgreSQL dialect
- Must translate standard SQL to MySQL dialect
- Must handle function differences (string, date, etc.)
- Must handle identifier quoting differences
- Must provide clear errors for unsupported features

### FR-3: Query Executor
**Priority**: P1
- Must execute translated queries against target databases
- Must integrate with existing connection pools
- Must enforce existing timeout mechanisms
- Must return results in existing JSON format

### FR-4: Cross-Database Coordinator
**Priority**: P2
- Must decompose cross-database queries into sub-queries
- Must execute sub-queries in parallel where possible
- Must merge results according to JOIN/UNION logic
- Must handle errors from any participating database

### FR-5: Plugin System
**Priority**: P3
- Must define clear adapter interface
- Must provide dialect registration mechanism
- Must discover plugins dynamically
- Must isolate plugins from core system

### FR-6: Frontend Integration
**Priority**: P1
- Query editor must accept unified SQL
- Must indicate which database query will target
- Must display dialect translation in error messages
- Must show loading states during translation

## Non-Functional Requirements

### NFR-1: Performance
- Single-database queries: < 10% overhead vs direct queries
- Cross-database queries: < 1 second for typical workloads (< 10K rows)
- Memory usage: < 100MB additional per active query

### NFR-2: Reliability
- 99.9% success rate for supported query types
- Graceful degradation if one database unavailable
- Automatic retry for transient failures

### NFR-3: Security
- Maintain all existing security restrictions
- Validate queries cannot bypass SELECT-only rule
- Ensure cross-database queries respect access controls
- No exposure of connection details in errors

### NFR-4: Usability
- 80%+ user comprehension of unified SQL syntax
- Error messages actionable within 30 seconds
- Documentation sufficient for self-service

### NFR-5: Maintainability
- Code coverage > 80% for new components
- Plugin system enables third-party contributions
- Clear separation of concerns in architecture

## Out of Scope

The following are explicitly **not** included in this feature:

1. **Write Operations**: INSERT, UPDATE, DELETE remain prohibited
2. **Transactions**: No distributed transaction coordination
3. **Schema Changes**: No DDL operations (CREATE, ALTER, DROP)
4. **Real-time Replication**: Not a data sync or CDC tool
5. **Query Caching**: Addressed in separate feature
6. **Migration Tool**: Not for migrating data between databases
7. **NoSQL Databases**: Focus on SQL databases only
8. **Full SQL-92 Compliance**: Best-effort standard SQL support

## Dependencies

### External Dependencies
- Apache Arrow DataFusion 51.0.0 (already in Cargo.toml)
- Existing database drivers (tokio-postgres, mysql_async)
- Existing connection pooling infrastructure

### Internal Dependencies
- Existing SqlValidator must be extended
- Existing QueryService must be refactored
- Existing API endpoints must be updated
- Frontend QueryEditor must be enhanced

## Risks and Assumptions

### Assumptions
1. DataFusion 51.0.0 is stable and production-ready
2. Most common SQL queries can be translated accurately
3. Users accept slight performance overhead for unified syntax
4. Cross-database queries will typically involve small-to-medium datasets

### Risks
| Risk | Impact | Likelihood | Mitigation |
|------|--------|-----------|------------|
| DataFusion API changes in future | High | Low | Pin to 51.0.0, monitor releases |
| Dialect translation accuracy | High | Medium | Comprehensive test suite, user feedback |
| Performance regression | High | Low | Benchmark tests, feature flag |
| User confusion with new syntax | Medium | Medium | Documentation, training, migration guide |
| Cross-database query slowness | Medium | Medium | Optimize pushdown, add caching layer |

## Acceptance Criteria Summary

### Phase 3 (MVP) - User Story 1
- [ ] All Phase 1-2 tasks completed
- [ ] Unified SQL works for PostgreSQL
- [ ] Unified SQL works for MySQL
- [ ] Dialect translation tested for 20+ query patterns
- [ ] Performance within 10% of baseline
- [ ] All existing tests pass
- [ ] Security validations maintained
- [ ] Documentation updated

### Phase 4 - User Story 2
- [ ] Cross-database JOIN works (PostgreSQL + MySQL)
- [ ] Cross-database UNION works
- [ ] Query optimization (pushdown) demonstrated
- [ ] Error handling for all failure modes
- [ ] Performance acceptable (< 1s for 10K rows)
- [ ] Query cancellation works

### Phase 5 - User Story 3
- [ ] Plugin system implemented
- [ ] Doris adapter completed as plugin
- [ ] Druid adapter completed as plugin
- [ ] Plugin documentation written
- [ ] Third-party plugin successfully added
- [ ] No regressions in existing databases

## Open Questions

1. **Q**: Should we support stored procedures in cross-database queries?
   **A**: No, out of scope for v1. Revisit if users request it.

2. **Q**: How do we handle schema differences (same table name, different columns)?
   **A**: Require explicit column lists in cross-database queries. Error if column doesn't exist.

3. **Q**: What's the maximum number of databases in a single cross-database query?
   **A**: Start with 2-3, can increase if performance is acceptable.

4. **Q**: Should we cache translated queries?
   **A**: Not in Phase 1-3. Consider for Phase 6 optimization.

5. **Q**: How do we handle NULL semantics differences between databases?
   **A**: DataFusion will normalize NULL handling. Document any edge cases.

## Success Criteria

This feature is considered successful if:

1. **Adoption**: 70%+ of active users try unified SQL within 30 days
2. **Satisfaction**: User survey score 4/5+ for ease of use
3. **Performance**: 90%+ of queries complete with < 10% overhead
4. **Reliability**: 99%+ query success rate for supported patterns
5. **Extensibility**: New database type added within sprint

## References

- [Implementation Plan](./plan.md)
- [Task Breakdown](./tasks.md)
- [Research Document](./research.md)
- [DataFusion Documentation](https://arrow.apache.org/datafusion/)
- [Project Constitution](../../.specify/memory/constitution.md)

---

*Last Updated*: 2025-12-26
*Status*: Phase 1 In Progress
*Next Review*: After Phase 2 completion
