# Cross-Database Query Architecture Design
## Phase 4: User Story 2 Implementation

**Date**: 2025-12-26
**Status**: Design Phase
**Goal**: Enable JOIN and UNION queries across multiple databases

---

## Overview

This document outlines the architecture for cross-database query execution using Apache DataFusion as the semantic layer coordinator.

### Key Concept
DataFusion acts as a **federated query engine** that:
1. Accepts a single SQL query referencing multiple databases
2. Decomposes the query into database-specific sub-queries
3. Executes sub-queries in parallel against respective databases
4. Merges results using DataFusion's join/union operators

---

## Architecture Components

### 1. CrossDatabaseQueryRequest Model
**File**: `backend/src/models/cross_database_query.rs`

```rust
pub struct CrossDatabaseQueryRequest {
    /// SQL query with qualified table names (db1.table1, db2.table2)
    pub query: String,

    /// List of connection IDs involved in the query
    pub connection_ids: Vec<String>,

    /// Timeout for entire cross-database query (default: 60s)
    pub timeout_secs: Option<u64>,

    /// Whether to apply LIMIT
    pub apply_limit: Option<bool>,

    /// LIMIT value
    pub limit_value: Option<u32>,
}

pub struct CrossDatabaseQueryResponse {
    /// Original cross-database query
    pub original_query: String,

    /// Sub-queries executed per database
    pub sub_queries: Vec<SubQueryExecution>,

    /// Final merged results
    pub results: Vec<serde_json::Value>,

    /// Total row count
    pub row_count: usize,

    /// Total execution time
    pub execution_time_ms: u128,

    /// Execution timestamp
    pub executed_at: String,
}

pub struct SubQueryExecution {
    pub connection_id: String,
    pub database_type: String,
    pub query: String,
    pub row_count: usize,
    pub execution_time_ms: u128,
}
```

---

### 2. CrossDatabaseQueryPlanner
**File**: `backend/src/services/datafusion/cross_db_planner.rs`

**Responsibilities**:
- Parse cross-database SQL query
- Identify tables and their source databases
- Decompose query into sub-queries per database
- Generate execution plan

**Algorithm**:
```
1. Parse SQL using sqlparser
2. Extract table references (e.g., mysql_db.users, postgres_db.orders)
3. Map tables to connection IDs
4. Generate sub-queries:
   - For JOINs: Push down filters, extract join columns
   - For UNIONs: Split query per database
5. Create DataFusion logical plan for result merging
```

**Key Methods**:
```rust
impl CrossDatabaseQueryPlanner {
    /// Parse and plan cross-database query
    pub fn plan_query(&self, request: &CrossDatabaseQueryRequest)
        -> Result<CrossDatabaseExecutionPlan>;

    /// Identify which tables belong to which databases
    pub fn map_tables_to_connections(&self, query: &str)
        -> Result<HashMap<String, String>>;

    /// Generate sub-queries for each database
    pub fn decompose_query(&self, query: &str, table_map: &HashMap<String, String>)
        -> Result<Vec<SubQuery>>;
}
```

---

### 3. DataFusionFederatedExecutor
**File**: `backend/src/services/datafusion/federated_executor.rs`

**Responsibilities**:
- Execute sub-queries in parallel
- Collect results from each database
- Use DataFusion to merge results

**Execution Flow**:
```
1. Execute sub-queries concurrently using tokio::join!
2. Convert each result set to Arrow RecordBatch
3. Register results as in-memory tables in DataFusion
4. Execute merge operation (JOIN/UNION) using DataFusion
5. Convert final results to JSON
```

**Key Methods**:
```rust
impl DataFusionFederatedExecutor {
    /// Execute cross-database query
    pub async fn execute_cross_database_query(
        &self,
        plan: CrossDatabaseExecutionPlan,
    ) -> Result<CrossDatabaseQueryResponse>;

    /// Execute sub-queries in parallel
    async fn execute_sub_queries(&self, sub_queries: Vec<SubQuery>)
        -> Result<Vec<SubQueryResult>>;

    /// Merge results using DataFusion
    async fn merge_results(&self, sub_results: Vec<SubQueryResult>, merge_plan: &MergePlan)
        -> Result<Vec<RecordBatch>>;
}
```

---

## Example Use Cases

### Use Case 1: Cross-Database JOIN

**Scenario**: Join users from MySQL with orders from PostgreSQL

**SQL Query**:
```sql
SELECT u.username, o.order_id, o.total
FROM mysql_db.users u
JOIN postgres_db.orders o ON u.id = o.user_id
WHERE o.status = 'completed'
```

**Execution Plan**:
1. **Sub-query 1** (MySQL):
   ```sql
   SELECT id, username FROM users
   ```
2. **Sub-query 2** (PostgreSQL):
   ```sql
   SELECT order_id, user_id, total, status FROM orders
   WHERE status = 'completed'
   ```
3. **DataFusion Merge**:
   - Load both results as in-memory tables
   - Execute JOIN in DataFusion
   - Return unified results

---

### Use Case 2: Cross-Database UNION

**Scenario**: Combine todos from MySQL and tickets from PostgreSQL

**SQL Query**:
```sql
SELECT id, title, status FROM mysql_db.todos
UNION ALL
SELECT id, title, status FROM postgres_db.tickets
```

**Execution Plan**:
1. **Sub-query 1** (MySQL):
   ```sql
   SELECT id, title, status FROM todos
   ```
2. **Sub-query 2** (PostgreSQL):
   ```sql
   SELECT id, title, status FROM tickets
   ```
3. **DataFusion Merge**:
   - UNION ALL results in DataFusion
   - Return combined results

---

## API Design

### Endpoint: POST /api/cross-database/query

**Request**:
```json
{
  "query": "SELECT u.username, t.title FROM mysql_conn.todos t JOIN pg_conn.users u ON t.user_id = u.id",
  "connection_ids": [
    "1bb2bc4c-b575-49c2-a382-6032a3abe23e",  // MySQL
    "a0a03e3a-c604-4990-99cb-b2c939426a8c"   // PostgreSQL
  ],
  "timeout_secs": 60,
  "apply_limit": true,
  "limit_value": 100
}
```

**Response**:
```json
{
  "original_query": "SELECT ...",
  "sub_queries": [
    {
      "connection_id": "1bb2bc4c...",
      "database_type": "mysql",
      "query": "SELECT id, user_id, title FROM todos",
      "row_count": 15,
      "execution_time_ms": 5
    },
    {
      "connection_id": "a0a03e3a...",
      "database_type": "postgresql",
      "query": "SELECT id, username FROM users",
      "row_count": 10,
      "execution_time_ms": 8
    }
  ],
  "results": [...],
  "row_count": 15,
  "execution_time_ms": 25,
  "executed_at": "2025-12-26T11:00:00Z"
}
```

---

## Implementation Strategy

### Phase 4.1: Core Infrastructure (T042-T047)
- ✅ Create models
- ✅ Design planner architecture
- ✅ Implement federated executor
- ✅ Basic JOIN support

### Phase 4.2: Query Optimization (T048-T052)
- ✅ Parallel sub-query execution
- ✅ Result merging optimization
- ✅ Connection pool management
- ✅ UNION support

### Phase 4.3: API Integration (T053-T055)
- ✅ Update QueryService
- ✅ Create API endpoint
- ✅ Add validation

### Phase 4.4: Frontend (T056-T062)
- ✅ Query builder UI
- ✅ Database/table selection
- ✅ Visual indicators
- ✅ Error handling

---

## Technical Challenges & Solutions

### Challenge 1: Table Name Qualification
**Problem**: How to specify which database a table belongs to?

**Solution**: Use prefixed notation
- `mysql_conn.users` → users table from MySQL connection
- `pg_conn.tickets` → tickets table from PostgreSQL connection
- Planner resolves prefix to connection ID

### Challenge 2: Schema Compatibility
**Problem**: JOIN columns may have different types across databases

**Solution**: DataFusion type coercion
- Use DataFusion's built-in type casting
- Explicit CAST in user query if needed
- Validate compatible types before execution

### Challenge 3: Performance
**Problem**: Network latency for cross-database queries

**Solution**: Optimization strategies
1. **Pushdown**: Apply filters before data transfer
2. **Parallel**: Execute sub-queries concurrently
3. **Limit Early**: Apply LIMIT to sub-queries when possible
4. **Caching**: Cache frequently joined tables

### Challenge 4: Transaction Consistency
**Problem**: No distributed transactions across databases

**Solution**: Document limitations
- Read-only queries only
- No ACID guarantees across databases
- Use consistent timestamp if needed

---

## Testing Strategy

### Unit Tests
- Planner: Query decomposition logic
- Executor: Result merging logic
- Validator: Cross-database query validation

### Integration Tests
- MySQL + PostgreSQL JOIN
- MySQL + PostgreSQL UNION
- Error handling for invalid queries
- Performance benchmarks

### End-to-End Tests
- Full API workflow
- Frontend query builder
- Multiple database types

---

## Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Sub-query execution | < 50ms each | Depends on database |
| Result merging | < 100ms | DataFusion in-memory |
| Total latency | < 500ms | For typical queries |
| Max rows per sub-query | 10,000 | Safety limit |

---

## Security Considerations

1. **Query Validation**: Only allow SELECT statements
2. **Connection Authorization**: Verify user has access to all connections
3. **Resource Limits**: Enforce timeouts and row limits
4. **SQL Injection**: Use parameterized queries where possible

---

## Future Enhancements

- **Query Caching**: Cache cross-database query results
- **Incremental Updates**: Stream results as sub-queries complete
- **Cost-Based Optimization**: Choose optimal execution order
- **Distributed Transactions**: Explore 2PC for writes

---

**Document Status**: Ready for Implementation
**Next Steps**: Begin T042 - Create CrossDatabaseQueryRequest model
