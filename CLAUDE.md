# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a database query tool that allows users to connect to databases (PostgreSQL and MySQL), view metadata, execute SQL queries, and use natural language to generate SQL queries. The application consists of a Rust backend (Axum/Tokio) and a React frontend (Refine 5/Ant Design).

## Key Commands

### Development

```bash
# Install all dependencies
make install

# Start backend server (port 3000)
make dev-backend

# Start frontend dev server (port 5173)
make dev-frontend

# Start both (requires two terminals)
make dev
```

### Testing

```bash
# Run all tests
make test

# Run backend tests only
make test-backend
cd backend && cargo test

# Run a single test
cd backend && cargo test test_name

# Run tests with output
cd backend && cargo test -- --nocapture
```

### Code Quality

```bash
# Lint and format everything
make lint
make format

# Backend specific
make lint-backend      # cargo clippy
make format-backend    # cargo fmt
make check-backend     # cargo check without building

# Frontend specific
make lint-frontend     # ESLint
make format-frontend   # Prettier
make check-frontend    # TypeScript check without building
```

### Build

```bash
# Build both for production
make build

# Backend release build
make build-backend  # creates backend/target/release/db-query-backend

# Frontend production build
make build-frontend  # creates frontend/dist
```

## Architecture

### Security-First Design

**Critical**: All SQL queries must pass validation before execution:
1. Parsed with `sqlparser` crate (PostgreSqlDialect)
2. Only SELECT statements are permitted - all other statement types are rejected
3. Auto-append `LIMIT 1000` if no LIMIT clause exists
4. Validation occurs in `backend/src/validation/sql_validator.rs`

This security layer is non-negotiable per the project constitution (`.specify/memory/constitution.md`).

### Backend Structure (Rust)

```
backend/src/
├── main.rs                   # Entry point, initializes server
├── api/
│   ├── routes.rs            # Route definitions and health check
│   ├── handlers/            # Request handlers
│   │   ├── connection.rs    # Connection CRUD endpoints
│   │   ├── metadata.rs      # Metadata retrieval endpoint
│   │   └── query.rs         # Query execution (SQL + NL)
│   └── middleware.rs        # Error handling (AppError types)
├── models/                  # Data models (Connection, Metadata, Query)
├── services/
│   ├── db_service.rs        # Database connection management
│   ├── query_service.rs     # Query execution logic
│   ├── llm_service.rs       # LLM integration for NL-to-SQL
│   ├── metadata_cache.rs    # Metadata caching service
│   └── database/            # Multi-database adapter layer
│       ├── adapter.rs       # DatabaseAdapter trait (uses DataFusion)
│       ├── postgresql.rs    # PostgreSQL implementation
│       ├── mysql.rs         # MySQL implementation
│       ├── doris.rs         # Apache Doris implementation
│       └── druid.rs         # Apache Druid implementation
├── storage/
│   └── sqlite.rs           # SQLite storage for metadata
└── validation/
    └── sql_validator.rs    # SQL validation and sanitization
```

**Key Flow**:
1. User connects via URL → `connection::create_connection` handler
2. Backend retrieves metadata → `database/adapter.rs` implementations fetch schema info
3. LLM converts metadata to JSON → stored in SQLite via `storage/sqlite.rs`
4. Query requests → `query::execute_query` or `query::execute_natural_language_query`
5. Validation → `SqlValidator::validate_and_prepare` ensures SELECT-only + LIMIT
6. Execution → `DatabaseAdapter::execute_query` runs through DataFusion
7. Results returned as JSON

### Frontend Structure (React)

```
frontend/src/
├── App.tsx                      # Main app with Refine setup and routing
├── main.tsx                     # Entry point
├── pages/
│   ├── Dashboard.tsx            # Main page (connection + metadata)
│   └── QueryPage.tsx            # Query execution page
├── components/
│   ├── DatabaseConnection/      # Connection form component
│   ├── MetadataViewer/          # Display tables/views/columns
│   ├── QueryEditor/             # Monaco Editor for SQL
│   ├── QueryResults/            # Table display for results
│   └── NaturalLanguageQuery/    # NL query interface
├── services/
│   ├── api.ts                   # Axios base configuration
│   ├── connection.ts            # Connection API calls
│   ├── metadata.ts              # Metadata API calls
│   └── query.ts                 # Query execution API calls
├── providers/
│   └── dataProvider.ts          # Refine data provider
└── types/
    └── index.ts                 # TypeScript type definitions
```

**UI Flow**: Dashboard (connections + metadata) → QueryPage (Monaco Editor + results table)

### Multi-Database Support via DataFusion

The system uses **DataFusion** as a semantic layer for database abstraction:
- All database types implement the `DatabaseAdapter` trait (backend/src/services/database/adapter.rs)
- SQL validation happens via sqlparser before reaching DataFusion
- DataFusion converts validated SQL to logical plans and executes against target databases
- Currently supports: PostgreSQL, MySQL, Apache Doris, Apache Druid

When adding new database support:
1. Create new adapter file in `backend/src/services/database/`
2. Implement `DatabaseAdapter` trait
3. Use DataFusion's catalog and execution context for query routing

### LLM Integration (Natural Language to SQL)

- Backend uses `reqwest` to call LLM gateway (configured via `LLM_GATEWAY_URL`)
- Plan: Replace with `rig.rs` for LLM orchestration (per constitution)
- Context: Cached database metadata (tables, columns, types) is passed to LLM
- Generated SQL **must** pass same validation as manual queries
- Service: `backend/src/services/llm_service.rs`

### Metadata Caching Strategy

- Connection strings + metadata stored in SQLite (`backend/metadata.db`)
- Initial connection: Query database for schema info → process with LLM → cache JSON
- Subsequent connections: Use cached metadata (faster, reduces DB load)
- Refresh available via `?refresh=true` query parameter
- Cache implementation: `backend/src/services/metadata_cache.rs`

## Configuration

### Backend (.env in backend/)
```env
DATABASE_URL=./metadata.db        # SQLite metadata storage
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
LLM_GATEWAY_URL=http://localhost:8080
LLM_API_KEY=optional-api-key
```

### Frontend (.env in frontend/)
```env
VITE_API_URL=http://localhost:3000/api
```

## Database Support

### Supported Databases

The system supports multiple database types through a unified adapter interface:

- **PostgreSQL** - Full support with connection pooling
- **MySQL** - Full support with connection pooling (including MariaDB)
- **Apache Doris** - Placeholder (implementation pending)
- **Apache Druid** - Placeholder (implementation pending)

### MySQL Configuration

**Connection URL Format**:
```
mysql://username:password@host:port/database
```

**Example**:
```
mysql://root:password@localhost:3306/mydb
```

**MySQL-Specific Features**:
- Uses `mysql_async` crate for async MySQL connectivity
- Built-in connection pooling for optimal performance
- Supports MySQL 5.7+, MySQL 8.0+, and MariaDB
- Retrieves metadata from `information_schema`
- Filters out system schemas: `information_schema`, `mysql`, `performance_schema`, `sys`
- Detects primary keys and foreign keys automatically
- Natural language SQL generation supports MySQL-specific syntax:
  - MySQL date functions: `NOW()`, `CURDATE()`, `DATE_SUB()`
  - String functions: `CONCAT()`
  - Backtick identifier quoting: `` `table_name` ``

**MySQL Data Type Handling**:
The adapter supports automatic conversion of MySQL types to JSON:
- Numeric: `TINYINT`, `SMALLINT`, `INT`, `BIGINT`, `DECIMAL`, `FLOAT`, `DOUBLE`
- String: `VARCHAR`, `CHAR`, `TEXT`, `MEDIUMTEXT`, `LONGTEXT`
- Date/Time: `DATE`, `DATETIME`, `TIMESTAMP`, `TIME`, `YEAR`
- Binary: `BLOB`, `BINARY`, `VARBINARY` (converted to string representation)

**MySQL vs PostgreSQL Differences**:
- Connection URL scheme: `mysql://` vs `postgresql://`
- Identifier quoting: backticks vs double quotes
- Date functions differ: MySQL uses `NOW()`, PostgreSQL uses `NOW()` or `CURRENT_TIMESTAMP`
- String concatenation: MySQL uses `CONCAT()`, PostgreSQL uses `||` or `CONCAT()`

### Testing with MySQL

Use Docker to quickly spin up a MySQL test instance:

```bash
# Start MySQL container
docker run -d --name test-mysql \
  -e MYSQL_ROOT_PASSWORD=password \
  -e MYSQL_DATABASE=testdb \
  -p 3306:3306 \
  mysql:8.0

# Wait for MySQL to be ready
sleep 10

# Connect using the application
# Connection URL: mysql://root:password@localhost:3306/testdb
```

## Common Development Patterns

### Adding a New API Endpoint

1. Define handler in `backend/src/api/handlers/` module
2. Add route to `create_router_with_state()` in `backend/src/api/routes.rs`
3. Use `AppState` for accessing storage and config
4. Return appropriate `AppError` types for error handling

### Adding Frontend API Integration

1. Add API function to appropriate service file (`frontend/src/services/`)
2. Use the configured axios instance from `api.ts`
3. Update TypeScript types in `frontend/src/types/index.ts`
4. Consume in components using React hooks

### SQL Validation Pattern

Always use `SqlValidator::validate_and_prepare(sql, default_limit)`:
- Returns `(final_sql: String, limit_applied: bool)`
- Ensures SELECT-only + LIMIT clause
- Errors propagate as `AppError::InvalidSql`

Example:
```rust
use crate::validation::SqlValidator;

let (validated_sql, limit_applied) = SqlValidator::validate_and_prepare(&query, 1000)?;
// Execute validated_sql safely
```

### Query Execution Flow

1. Receive query request (SQL or NL)
2. If NL: Call LLM service to generate SQL
3. Validate with `SqlValidator::validate_and_prepare`
4. Get database adapter for connection
5. Execute via `adapter.execute_query(validated_sql, timeout)`
6. Return `QueryResult` as JSON

## Important Project Conventions

### Constitution Compliance

The project has a formal constitution (`.specify/memory/constitution.md`) defining five core principles:
1. **Security First**: Only SELECT queries, SQLParser validation (non-negotiable)
2. **Performance Optimization**: Auto-LIMIT 1000 for resource protection
3. **Metadata Reusability**: Cache in SQLite, use LLM for JSON conversion
4. **Error Handling**: Clear, actionable error messages for users
5. **Output Standardization**: JSON results, frontend renders tables

All changes must align with these principles. Security violations block merge.

### Specification-Driven Development

This project uses the Specify methodology:
- Specs in `specs/001-db-query-tool/`
- User stories prioritized (P1: metadata, P2: queries, P3: NL)
- Each story must be independently testable
- See `specs/001-db-query-tool/spec.md` for acceptance criteria

### Testing API Endpoints

Use VS Code REST Client with `fixtures/test.rest` for manual API testing.

## Troubleshooting

### Backend won't start
- Check SQLite database path in `.env`
- Verify port 3000 is available
- Check logs for database initialization errors

### Frontend can't connect to backend
- Verify `VITE_API_URL` in frontend `.env`
- Check CORS configuration in `backend/src/api/routes.rs`
- Ensure backend is running on expected port

### SQL validation failing
- Only SELECT statements are permitted
- Check for SQL injection patterns
- Verify query syntax with PostgreSQL dialect
- Check `backend/src/validation/sql_validator.rs` for validation rules

### Metadata not refreshing
- Use `GET /api/connections/{id}/metadata?refresh=true` to force refresh
- Check SQLite storage for cached metadata
- Verify database connection is still valid
