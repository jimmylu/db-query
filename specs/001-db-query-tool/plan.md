# Implementation Plan: Database Query Tool

**Branch**: `001-db-query-tool` | **Date**: 2024-12-22 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-db-query-tool/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

构建一个数据库查询工具，允许用户连接数据库、查看元数据、执行SQL查询，并通过自然语言生成SQL查询。系统采用前后端分离架构，后端使用Rust构建RESTful API，前端使用React构建用户界面。所有SQL查询必须经过验证，仅允许SELECT语句，并自动添加LIMIT子句以防止资源耗尽。元数据存储在本地SQLite数据库中，使用LLM转换为JSON格式以便重用。

## Technical Context

**Language/Version**: 
- Backend: Rust (latest stable)
- Frontend: TypeScript/JavaScript (ES2020+)

**Primary Dependencies**: 
- Backend: Axum (web framework), Tokio (async runtime), DataFusion (SQL engine), SQLParser (SQL validation), rig.rs (LLM gateway), SQLite (metadata storage)
- Frontend: React 18+, Refine 5, Tailwind CSS, Ant Design, Monaco Editor

**Storage**: 
- Metadata: SQLite (local storage for connection strings and metadata)
- Query results: In-memory (returned as JSON, not persisted)

**Testing**: 
- Backend: cargo test (Rust unit/integration tests)
- Frontend: Jest/Vitest + React Testing Library

**Target Platform**: 
- Backend: Linux/macOS/Windows (server)
- Frontend: Modern web browsers (Chrome, Firefox, Safari, Edge)

**Project Type**: Web application (frontend + backend)

**Performance Goals**: 
- Database connection and metadata retrieval: <10 seconds
- Query execution: <5 seconds for 95% of queries
- Metadata caching reduces subsequent connection time by 50%
- Support up to 500 tables/views without performance degradation

**Constraints**: 
- All queries must include LIMIT clause (auto-appended if missing, default 1000)
- Only SELECT statements permitted
- Query results limited to 1000 rows maximum
- JSON output format for all responses

**Scale/Scope**: 
- Support multiple database connections (PostgreSQL initially, extensible to other databases)
- Handle databases with up to 500 tables/views efficiently
- Support concurrent queries to same database
- Target: Single-user to small team usage initially

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Security First (NON-NEGOTIABLE) ✅

**Requirement**: All SQL queries MUST be validated through SQLParser before execution. Only SELECT statements are permitted.

**Compliance**: 
- ✅ SQLParser will be used for all query validation
- ✅ Non-SELECT statements will be rejected before execution
- ✅ Clear error messages will be provided for rejected queries
- ✅ LLM-generated queries will also pass through same validation

**Status**: PASS - Design ensures all queries validated before execution

### II. Performance Optimization ✅

**Requirement**: All queries MUST include a LIMIT clause. If missing, automatically append `LIMIT 1000`.

**Compliance**:
- ✅ Query parser will detect missing LIMIT clauses
- ✅ System will automatically append LIMIT 1000 before execution
- ✅ Prevents resource exhaustion from unbounded queries

**Status**: PASS - Auto-append LIMIT 1000 implemented in query processing

### III. Metadata Reusability ✅

**Requirement**: Database connection strings and metadata MUST be stored in local SQLite database. Metadata MUST be converted to JSON format using LLM processing.

**Compliance**:
- ✅ SQLite will be used for metadata storage
- ✅ LLM (via rig.rs) will convert metadata to JSON format
- ✅ Cached metadata will be reused for subsequent connections
- ✅ Reduces redundant database introspection queries

**Status**: PASS - SQLite storage and LLM conversion planned

### IV. Error Handling & User Feedback ✅

**Requirement**: All SQL syntax errors MUST be clearly communicated with actionable error messages.

**Compliance**:
- ✅ SQLParser will provide specific syntax error details
- ✅ Error messages will indicate specific issues
- ✅ Suggestions for corrections will be provided when possible
- ✅ Connection errors will have informative messages

**Status**: PASS - Error handling designed with clear user feedback

### V. Output Format Standardization ✅

**Requirement**: All query results MUST be returned in JSON format. Frontend renders JSON into tables.

**Compliance**:
- ✅ Backend will return all results as JSON arrays of objects
- ✅ Frontend will handle JSON to table rendering
- ✅ Separation of concerns maintained (backend JSON, frontend presentation)

**Status**: PASS - JSON format enforced, frontend handles rendering

### Overall Status: ✅ ALL GATES PASSED

No violations detected. Design complies with all constitutional principles.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
backend/
├── src/
│   ├── models/              # Data models (DatabaseConnection, DatabaseMetadata, Query, etc.)
│   │   ├── connection.rs
│   │   ├── metadata.rs
│   │   └── query.rs
│   ├── services/            # Business logic
│   │   ├── db_service.rs    # Database connection and metadata retrieval
│   │   ├── query_service.rs # SQL query execution and validation
│   │   ├── llm_service.rs   # Natural language to SQL conversion
│   │   └── metadata_cache.rs # Metadata caching and retrieval
│   ├── api/                 # API handlers
│   │   ├── routes.rs        # Route definitions
│   │   ├── handlers/        # Request handlers
│   │   │   ├── connection.rs
│   │   │   ├── query.rs
│   │   │   └── metadata.rs
│   │   └── middleware.rs    # Error handling, validation middleware
│   ├── storage/             # Storage layer
│   │   └── sqlite.rs        # SQLite operations for metadata storage
│   ├── validation/          # SQL validation
│   │   └── sql_validator.rs # SQLParser integration
│   └── main.rs              # Application entry point
├── tests/
│   ├── unit/                # Unit tests
│   ├── integration/         # Integration tests
│   └── contract/            # Contract tests
└── Cargo.toml

frontend/
├── src/
│   ├── components/          # React components
│   │   ├── DatabaseConnection/ # Connection form and status
│   │   ├── MetadataViewer/    # Tables and views display
│   │   ├── QueryEditor/       # Monaco Editor for SQL input
│   │   ├── QueryResults/       # Results table display
│   │   └── NaturalLanguageQuery/ # Natural language input
│   ├── pages/               # Page components
│   │   ├── Dashboard.tsx    # Main application page
│   │   └── QueryPage.tsx    # Query execution page
│   ├── services/            # API client services
│   │   ├── api.ts           # API client configuration
│   │   ├── connection.ts    # Connection API calls
│   │   ├── query.ts         # Query API calls
│   │   └── metadata.ts      # Metadata API calls
│   ├── hooks/               # Custom React hooks
│   ├── utils/               # Utility functions
│   └── App.tsx              # Root component
├── tests/
│   ├── unit/                # Component tests
│   └── integration/         # Integration tests
├── package.json
└── tsconfig.json
```

**Structure Decision**: Web application structure selected (Option 2) because the feature requires both a backend API (Rust/Axum) and a frontend UI (React). The backend handles database connections, query execution, and LLM integration, while the frontend provides the user interface for database exploration and query execution. This separation allows independent development and deployment of each component.

## Phase 0: Research Complete ✅

**Status**: Complete  
**Output**: `research.md`

All technical decisions documented:
- Backend framework: Axum
- SQL engine: DataFusion
- SQL validation: SQLParser
- LLM gateway: rig.rs
- Metadata storage: SQLite
- Frontend framework: React + Refine 5
- SQL editor: Monaco Editor

All NEEDS CLARIFICATION items resolved through research and architectural decisions.

## Phase 1: Design Complete ✅

**Status**: Complete  
**Outputs**: 
- `data-model.md` - Core entities and relationships defined
- `contracts/openapi.yaml` - RESTful API specification
- `quickstart.md` - User guide and setup instructions

### Data Model Summary

Core entities defined:
- **DatabaseConnection**: Connection management and status
- **DatabaseMetadata**: Cached metadata with JSON representation
- **Table/View/Column**: Database structure representation
- **Query**: Query execution tracking and results

### API Contracts Summary

RESTful API endpoints defined:
- `/api/connections` - Connection management (GET, POST)
- `/api/connections/{id}` - Connection details (GET, DELETE)
- `/api/connections/{id}/metadata` - Metadata retrieval (GET)
- `/api/connections/{id}/query` - SQL query execution (POST)
- `/api/connections/{id}/nl-query` - Natural language queries (POST)

All endpoints return JSON with proper error handling.

### Quick Start Guide

Complete setup and usage instructions provided:
- Installation steps for backend and frontend
- Basic usage scenarios
- API usage examples
- Troubleshooting guide

## Constitution Check Re-evaluation ✅

After Phase 1 design, all constitutional principles remain compliant:

- ✅ **Security First**: SQLParser validation implemented in API contracts
- ✅ **Performance Optimization**: LIMIT enforcement documented in query flow
- ✅ **Metadata Reusability**: SQLite storage and LLM conversion in data model
- ✅ **Error Handling**: Structured error responses in API contracts
- ✅ **Output Format**: JSON format enforced in all API responses

**Status**: ALL GATES STILL PASSED - No violations introduced

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations detected. Design complies with all constitutional principles.
