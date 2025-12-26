# Research: Database Query Tool

**Date**: 2024-12-22  
**Feature**: Database Query Tool  
**Purpose**: Document technical decisions and research findings for implementation

## Technology Stack Decisions

### Backend Framework: Axum

**Decision**: Use Axum as the web framework for Rust backend.

**Rationale**:
- Axum is a modern, ergonomic web framework built on top of Tokio
- Excellent async/await support, aligning with Tokio runtime
- Type-safe request/response handling
- Built-in support for JSON serialization/deserialization
- Active development and strong community support
- Performance-focused design suitable for database query workloads

**Alternatives Considered**:
- **Actix-web**: More mature but heavier API surface
- **Warp**: Functional style, less ergonomic for complex routes
- **Rocket**: Simpler API but requires nightly Rust

### SQL Engine: DataFusion

**Decision**: Use DataFusion for SQL query execution.

**Rationale**:
- Native Rust implementation, no FFI overhead
- Supports PostgreSQL and other database backends
- Efficient query execution with columnar processing
- Good integration with Rust async ecosystem
- Extensible architecture for custom functions

**Alternatives Considered**:
- **Direct PostgreSQL driver**: Would require manual query execution logic
- **SQLx**: Good for simple queries but less flexible for complex SQL parsing
- **DuckDB**: Embedded database, but we need to connect to external databases

### SQL Validation: SQLParser

**Decision**: Use SQLParser crate for SQL syntax validation.

**Rationale**:
- Pure Rust implementation, no external dependencies
- Comprehensive SQL parsing support
- Can detect statement types (SELECT, INSERT, etc.)
- Provides detailed error messages for syntax issues
- Aligns with constitutional requirement for SQL validation

**Alternatives Considered**:
- **Custom parser**: Too complex and error-prone
- **External SQL parser service**: Adds latency and dependency
- **Regex-based validation**: Insufficient for proper SQL parsing

### LLM Gateway: rig.rs

**Decision**: Use rig.rs for LLM integration.

**Rationale**:
- Rust-native LLM gateway
- Supports multiple LLM providers
- Efficient request handling
- Good integration with Rust async ecosystem
- Handles LLM API complexities (retries, rate limiting, etc.)

**Alternatives Considered**:
- **Direct OpenAI API**: Would require manual API handling
- **LangChain Rust**: Less mature than rig.rs
- **Custom LLM client**: Too much implementation overhead

### Metadata Storage: SQLite

**Decision**: Use SQLite for local metadata storage.

**Rationale**:
- Lightweight, embedded database
- No separate server process required
- ACID compliance for data integrity
- Good performance for metadata storage use case
- Constitutional requirement

**Alternatives Considered**:
- **JSON files**: Less structured, harder to query
- **PostgreSQL**: Overkill for local metadata storage
- **In-memory storage**: Data lost on restart

### Frontend Framework: React + Refine 5

**Decision**: Use React with Refine 5 for admin interface.

**Rationale**:
- React is industry standard with large ecosystem
- Refine 5 provides rapid development for data-heavy applications
- Built-in support for CRUD operations
- Good TypeScript support
- Extensive component library integration

**Alternatives Considered**:
- **Vue.js**: Smaller ecosystem for admin interfaces
- **Svelte**: Less mature admin framework options
- **Vanilla JS**: Too much manual work for admin interface

### SQL Editor: Monaco Editor

**Decision**: Use Monaco Editor for SQL input.

**Rationale**:
- Same editor engine as VS Code
- Excellent SQL syntax highlighting
- Code completion and IntelliSense support
- Familiar user experience
- Good TypeScript integration

**Alternatives Considered**:
- **CodeMirror**: Less feature-rich
- **Ace Editor**: Less maintained
- **Simple textarea**: Poor user experience

## Architecture Decisions

### RESTful API Design

**Decision**: Use RESTful API patterns for backend endpoints.

**Rationale**:
- Standard HTTP methods (GET, POST, PUT, DELETE)
- Clear resource-based URLs
- JSON request/response format
- Easy to document and test
- Frontend-agnostic design

**API Structure**:
- `/api/connections` - Database connection management
- `/api/connections/{id}/metadata` - Metadata retrieval
- `/api/connections/{id}/query` - SQL query execution
- `/api/connections/{id}/nl-query` - Natural language query

### Metadata Caching Strategy

**Decision**: Cache metadata in SQLite after first retrieval, refresh on demand.

**Rationale**:
- Reduces database load for repeated connections
- Faster subsequent connections (constitutional requirement: 50% reduction)
- Enables offline metadata viewing
- LLM conversion to JSON happens once per metadata set

**Refresh Strategy**:
- Cache metadata on first connection
- Provide manual refresh option
- Consider TTL-based refresh (future enhancement)

### Query Validation Flow

**Decision**: Validate SQL queries before execution using SQLParser.

**Rationale**:
- Prevents SQL injection attacks
- Ensures only SELECT statements (constitutional requirement)
- Provides clear error messages before execution
- Applies to both manual and LLM-generated queries

**Validation Steps**:
1. Parse SQL using SQLParser
2. Check statement type (must be SELECT)
3. Validate syntax
4. Auto-append LIMIT 1000 if missing
5. Execute if validation passes

### Error Handling Strategy

**Decision**: Return structured JSON error responses with clear messages.

**Rationale**:
- Constitutional requirement for clear error messages
- Enables frontend to display user-friendly errors
- Consistent error format across all endpoints
- Includes error codes for programmatic handling

**Error Format**:
```json
{
  "error": {
    "code": "INVALID_SQL",
    "message": "Only SELECT statements are permitted",
    "details": "..."
  }
}
```

## Integration Patterns

### Database Connection Management

**Decision**: Support multiple concurrent database connections.

**Rationale**:
- Users may need to query multiple databases
- Each connection maintains its own metadata cache
- Connection pooling for efficiency

**Implementation**:
- Store connections in memory with unique IDs
- Metadata cached per connection
- Connection timeouts and cleanup

### LLM Context Management

**Decision**: Include full database metadata as context for LLM queries.

**Rationale**:
- LLM needs schema information to generate accurate SQL
- Metadata provides table/column names and relationships
- Improves SQL generation accuracy

**Context Format**:
- JSON representation of tables, views, columns
- Include column types and constraints
- Include relationships if available

## Performance Considerations

### Query Result Limiting

**Decision**: Enforce LIMIT 1000 maximum for all queries.

**Rationale**:
- Prevents memory exhaustion
- Constitutional requirement
- Ensures responsive UI
- Reasonable default for most use cases

### Metadata Retrieval Optimization

**Decision**: Use PostgreSQL system catalogs for efficient metadata retrieval.

**Rationale**:
- Faster than querying individual tables
- Standard PostgreSQL approach
- Comprehensive metadata available
- Can be extended to other databases

**PostgreSQL Queries**:
- `information_schema.tables` for tables
- `information_schema.views` for views
- `information_schema.columns` for column details

## Security Considerations

### SQL Injection Prevention

**Decision**: Use SQLParser validation + parameterized queries.

**Rationale**:
- SQLParser ensures only SELECT statements
- Parameterized queries prevent injection in generated queries
- Multiple layers of protection
- Constitutional requirement

### Connection String Security

**Decision**: Store connection strings securely in SQLite.

**Rationale**:
- Connection strings contain credentials
- SQLite database should be encrypted (future enhancement)
- No connection strings in logs or error messages

## Open Questions Resolved

### Q: Should we support multiple database types initially?

**A**: Start with PostgreSQL support, design architecture to be extensible. Other databases can be added later.

### Q: How to handle very large metadata sets?

**A**: Paginate metadata display in frontend. Cache full metadata but display in chunks.

### Q: What happens if LLM service is unavailable?

**A**: Natural language queries will fail gracefully with error message. Manual SQL queries remain available.

### Q: How to handle database schema changes?

**A**: Provide manual refresh option. Future: detect schema changes and prompt for refresh.

## References

- Axum documentation: https://docs.rs/axum/
- DataFusion documentation: https://arrow.apache.org/datafusion/
- SQLParser crate: https://docs.rs/sqlparser/
- Refine documentation: https://refine.dev/docs/
- Monaco Editor: https://microsoft.github.io/monaco-editor/

