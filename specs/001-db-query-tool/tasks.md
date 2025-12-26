# Tasks: Database Query Tool

**Input**: Design documents from `/specs/001-db-query-tool/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are OPTIONAL - not explicitly requested in feature specification, so test tasks are not included.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Web app**: `backend/src/`, `frontend/src/`
- Paths follow the structure defined in plan.md

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Create backend project structure in backend/ directory
- [x] T002 Create frontend project structure in frontend/ directory
- [x] T003 Initialize Rust project with Cargo.toml in backend/
- [x] T004 [P] Initialize React project with package.json in frontend/
- [x] T005 [P] Configure Rust dependencies (Axum, Tokio, DataFusion, SQLParser, rig.rs, SQLite) in backend/Cargo.toml
- [x] T006 [P] Configure frontend dependencies (React, Refine 5, Tailwind CSS, Ant Design, Monaco Editor) in frontend/package.json
- [x] T007 [P] Setup Rust linting and formatting (clippy, rustfmt) in backend/
- [x] T008 [P] Setup TypeScript/ESLint configuration in frontend/
- [x] T009 [P] Create .env.example files for backend and frontend
- [x] T010 [P] Setup Git ignore files for backend and frontend

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T011 Setup SQLite database schema and migrations in backend/src/storage/sqlite.rs
- [x] T012 Create database connection pool structure in backend/src/storage/
- [x] T013 [P] Setup Axum web server and routing structure in backend/src/api/routes.rs
- [x] T014 [P] Implement error handling middleware in backend/src/api/middleware.rs
- [x] T015 [P] Create base error types and error response format in backend/src/api/middleware.rs
- [x] T016 [P] Setup environment configuration management in backend/src/config.rs
- [x] T017 [P] Create API client configuration in frontend/src/services/api.ts
- [x] T018 [P] Setup React app structure with routing in frontend/src/App.tsx
- [x] T019 [P] Configure Tailwind CSS in frontend/
- [x] T020 [P] Setup Refine 5 provider and data provider in frontend/src/App.tsx

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Connect Database and View Metadata (Priority: P1) 🎯 MVP

**Goal**: Users can connect to a database, retrieve metadata (tables, views), and view the database structure. Metadata is cached in SQLite for reuse.

**Independent Test**: Connect to a test PostgreSQL database and verify that tables and views are retrieved, displayed, and cached correctly. This delivers value independently as users can explore database structures without needing query capabilities.

### Implementation for User Story 1

- [x] T021 [P] [US1] Create DatabaseConnection model in backend/src/models/connection.rs
- [x] T022 [P] [US1] Create DatabaseMetadata model in backend/src/models/metadata.rs
- [x] T023 [P] [US1] Create Table model in backend/src/models/metadata.rs
- [x] T024 [P] [US1] Create View model in backend/src/models/metadata.rs
- [x] T025 [P] [US1] Create Column model in backend/src/models/metadata.rs
- [x] T026 [US1] Implement SQLite storage operations for connections in backend/src/storage/sqlite.rs
- [x] T027 [US1] Implement SQLite storage operations for metadata cache in backend/src/storage/sqlite.rs
- [x] T028 [US1] Implement database connection service using DataFusion in backend/src/services/db_service.rs
- [x] T029 [US1] Implement PostgreSQL metadata retrieval in backend/src/services/db_service.rs
- [x] T030 [US1] Implement metadata caching service in backend/src/services/metadata_cache.rs
- [x] T031 [US1] Implement LLM service for metadata JSON conversion in backend/src/services/llm_service.rs
- [x] T032 [US1] Implement POST /api/connections endpoint handler in backend/src/api/handlers/connection.rs
- [x] T033 [US1] Implement GET /api/connections endpoint handler in backend/src/api/handlers/connection.rs
- [x] T034 [US1] Implement GET /api/connections/{id} endpoint handler in backend/src/api/handlers/connection.rs
- [x] T035 [US1] Implement GET /api/connections/{id}/metadata endpoint handler in backend/src/api/handlers/metadata.rs
- [x] T036 [US1] Register connection routes in backend/src/api/routes.rs
- [x] T037 [US1] Register metadata routes in backend/src/api/routes.rs
- [x] T038 [US1] Create connection API client service in frontend/src/services/connection.ts
- [x] T039 [US1] Create metadata API client service in frontend/src/services/metadata.ts
- [x] T040 [US1] Create DatabaseConnection component in frontend/src/components/DatabaseConnection/
- [x] T041 [US1] Create MetadataViewer component in frontend/src/components/MetadataViewer/
- [x] T042 [US1] Create Dashboard page component in frontend/src/pages/Dashboard.tsx
- [x] T043 [US1] Integrate connection form and metadata display in Dashboard page
- [x] T044 [US1] Add error handling for connection failures in frontend components
- [x] T045 [US1] Add loading states for connection and metadata retrieval in frontend components

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently. Users can connect to databases and view metadata.

---

## Phase 4: User Story 2 - Execute SQL Queries (Priority: P2)

**Goal**: Users can execute SQL SELECT queries against connected databases. Queries are validated, executed, and results displayed in table format. Non-SELECT statements are rejected, and LIMIT 1000 is auto-appended if missing.

**Independent Test**: Execute various SELECT queries against a test database and verify that results are returned correctly, validation works, and LIMIT is auto-appended. This delivers value independently as users can query data even without natural language capabilities.

### Implementation for User Story 2

- [x] T046 [P] [US2] Create Query model in backend/src/models/query.rs
- [x] T047 [US2] Implement SQL validation service using SQLParser in backend/src/validation/sql_validator.rs
- [x] T048 [US2] Implement SELECT-only validation logic in backend/src/validation/sql_validator.rs
- [x] T049 [US2] Implement LIMIT clause detection and auto-append logic in backend/src/validation/sql_validator.rs
- [x] T050 [US2] Implement query execution service using DataFusion in backend/src/services/query_service.rs
- [x] T051 [US2] Implement query result formatting to JSON in backend/src/services/query_service.rs
- [x] T052 [US2] Implement POST /api/connections/{id}/query endpoint handler in backend/src/api/handlers/query.rs
- [x] T053 [US2] Register query routes in backend/src/api/routes.rs
- [x] T054 [US2] Create query API client service in frontend/src/services/query.ts
- [x] T055 [US2] Create QueryEditor component with Monaco Editor in frontend/src/components/QueryEditor/
- [x] T056 [US2] Create QueryResults component for displaying results table in frontend/src/components/QueryResults/
- [x] T057 [US2] Create QueryPage component in frontend/src/pages/QueryPage.tsx
- [x] T058 [US2] Integrate query editor and results display in QueryPage
- [x] T059 [US2] Add SQL syntax highlighting in Monaco Editor
- [x] T060 [US2] Add error display for validation and execution errors in frontend
- [x] T061 [US2] Add loading states for query execution in frontend
- [x] T062 [US2] Add query result table with proper column alignment and scrolling

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently. Users can connect to databases, view metadata, and execute SQL queries.

---

## Phase 5: User Story 3 - Natural Language to SQL Query Generation (Priority: P3)

**Goal**: Users can query databases using natural language. System uses LLM to generate SQL queries based on database metadata, validates them, and executes them. Generated queries are clearly marked.

**Independent Test**: Provide natural language questions about a connected database and verify that appropriate SQL queries are generated, validated, and executed. This delivers value independently as users can query data without SQL knowledge.

### Implementation for User Story 3

- [x] T063 [US3] Enhance LLM service for natural language to SQL conversion in backend/src/services/llm_service.rs
- [x] T064 [US3] Implement metadata context preparation for LLM in backend/src/services/llm_service.rs
- [x] T065 [US3] Implement SQL query generation from natural language in backend/src/services/llm_service.rs
- [x] T066 [US3] Integrate LLM-generated queries with validation service in backend/src/services/query_service.rs
- [x] T067 [US3] Implement POST /api/connections/{id}/nl-query endpoint handler in backend/src/api/handlers/query.rs
- [x] T068 [US3] Register natural language query route in backend/src/api/routes.rs
- [x] T069 [US3] Create natural language query API client service in frontend/src/services/query.ts
- [x] T070 [US3] Create NaturalLanguageQuery component in frontend/src/components/NaturalLanguageQuery/
- [x] T071 [US3] Integrate natural language query component in QueryPage
- [x] T072 [US3] Add display of generated SQL query before execution
- [x] T073 [US3] Add LLM-generated query indicator in results display
- [x] T074 [US3] Add error handling for LLM generation failures in frontend
- [x] T075 [US3] Add loading states for LLM query generation in frontend

**Checkpoint**: All user stories should now be independently functional. Users can connect to databases, view metadata, execute SQL queries, and use natural language queries.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T076 [P] Add connection status indicators in frontend components
- [x] T077 [P] Implement DELETE /api/connections/{id} endpoint handler in backend/src/api/handlers/connection.rs
- [x] T078 [P] Add connection deletion functionality in frontend
- [x] T079 [P] Add metadata refresh functionality (force refresh from database)
- [x] T080 [P] Improve error messages with actionable suggestions
- [x] T081 [P] Add connection timeout handling
- [x] T082 [P] Add query execution timeout handling
- [x] T083 [P] Add input validation for connection URLs
- [x] T084 [P] Add input sanitization for SQL queries
- [x] T085 [P] Add logging for all operations in backend
- [x] T086 [P] Add performance monitoring and metrics
- [x] T087 [P] Update documentation in README.md
- [x] T088 [P] Run quickstart.md validation and update if needed
- [x] T089 [P] Code cleanup and refactoring
- [x] T090 [P] Security review and hardening

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Depends on US1 for database connection, but query execution can be tested independently
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - Depends on US1 (metadata) and US2 (query execution), but natural language generation can be tested independently

### Within Each User Story

- Models before services
- Services before endpoints/handlers
- Backend API before frontend integration
- Core implementation before error handling and polish
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, User Stories 1 and 2 can start in parallel (User Story 3 depends on both)
- Models within a story marked [P] can run in parallel
- Frontend and backend tasks can run in parallel within each user story
- Different user stories can be worked on in parallel by different team members (with dependency awareness)

---

## Parallel Example: User Story 1

```bash
# Launch all models for User Story 1 together:
Task: "Create DatabaseConnection model in backend/src/models/connection.rs"
Task: "Create DatabaseMetadata model in backend/src/models/metadata.rs"
Task: "Create Table model in backend/src/models/metadata.rs"
Task: "Create View model in backend/src/models/metadata.rs"
Task: "Create Column model in backend/src/models/metadata.rs"

# Launch frontend and backend API tasks in parallel:
Task: "Create connection API client service in frontend/src/services/connection.ts"
Task: "Implement POST /api/connections endpoint handler in backend/src/api/handlers/connection.rs"
Task: "Implement GET /api/connections endpoint handler in backend/src/api/handlers/connection.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (backend focus)
   - Developer B: User Story 1 (frontend focus)
   - Developer C: User Story 2 (can start after US1 backend is ready)
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
- Backend and frontend can be developed in parallel within each user story
- All SQL queries must pass SQLParser validation (constitutional requirement)
- All queries must have LIMIT clause (auto-appended if missing, constitutional requirement)
- All results must be in JSON format (constitutional requirement)

