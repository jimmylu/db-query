# Tasks: Unified SQL Semantic Layer Refactoring

**Input**: Design documents from `/specs/003-union-semantic-support/`
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

## Phase 1: Setup & Research

**Purpose**: Research DataFusion capabilities and setup unified semantic layer infrastructure

- [X] T001 Research DataFusion SessionContext and catalog registration capabilities
- [X] T002 Research DataFusion dialect translation and SQL standard compliance
- [X] T003 Research DataFusion cross-database query execution patterns
- [X] T004 [P] Create plan.md document in specs/003-union-semantic-support/plan.md
- [X] T005 [P] Create spec.md document in specs/003-union-semantic-support/spec.md
- [X] T006 [P] Create research.md document in specs/003-union-semantic-support/research.md
- [X] T007 [P] Update backend/Cargo.toml to ensure DataFusion dependencies are properly configured
- [X] T008 [P] Create datafusion module structure in backend/src/services/datafusion/

**Checkpoint**: Research complete, architecture decisions documented

---

## Phase 2: Foundational - DataFusion Core Infrastructure

**Purpose**: Core DataFusion infrastructure that MUST be complete before semantic layer implementation

**⚠️ CRITICAL**: No semantic layer work can begin until this phase is complete

- [X] T009 Create DataFusionSessionManager in backend/src/services/datafusion/session.rs
- [X] T010 [P] Implement SessionContext factory with connection pool integration in backend/src/services/datafusion/session.rs
- [X] T011 [P] Create DataFusionCatalogManager in backend/src/services/datafusion/catalog.rs
- [X] T012 [P] Implement catalog registration for PostgreSQL tables in backend/src/services/datafusion/catalog.rs
- [X] T013 [P] Implement catalog registration for MySQL tables in backend/src/services/datafusion/catalog.rs
- [X] T014 [P] Create DataFusionDialectTranslator trait in backend/src/services/datafusion/dialect.rs
- [X] T015 [P] Implement PostgreSQL dialect translator in backend/src/services/datafusion/dialect.rs
- [X] T016 [P] Implement MySQL dialect translator in backend/src/services/datafusion/dialect.rs
- [X] T017 [P] Create DataFusionQueryExecutor in backend/src/services/datafusion/executor.rs
- [X] T018 [P] Implement query parsing and logical plan generation in backend/src/services/datafusion/executor.rs
- [X] T019 [P] Implement query execution with timeout handling in backend/src/services/datafusion/executor.rs
- [X] T020 [P] Create DataFusionResultConverter in backend/src/services/datafusion/converter.rs
- [X] T021 [P] Implement RecordBatch to JSON conversion in backend/src/services/datafusion/converter.rs

**Checkpoint**: DataFusion core infrastructure ready - semantic layer implementation can now begin

---

## Phase 3: User Story 1 - Single Database Query with Unified SQL (Priority: P1) 🎯 MVP

**Goal**: Users can execute SQL queries using DataFusion's unified SQL syntax against any supported database. The system automatically translates DataFusion SQL to the target database's dialect.

**Independent Test**: Execute a DataFusion SQL query against PostgreSQL and MySQL databases, verify that the query is automatically translated to the correct dialect and executed successfully. This delivers value independently as users can use one SQL syntax for all databases.

### Implementation for User Story 1

- [X] T022 [P] [US1] Create UnifiedQueryRequest model in backend/src/models/unified_query.rs
- [X] T023 [P] [US1] Add database_type field to Query model in backend/src/models/query.rs
- [ ] T024 [US1] Refactor PostgreSQLAdapter to use DataFusion executor in backend/src/services/database/postgresql.rs
- [ ] T025 [US1] Refactor MySQLAdapter to use DataFusion executor in backend/src/services/database/mysql.rs
- [X] T026 [US1] Update DatabaseAdapter trait to support DataFusion execution in backend/src/services/database/adapter.rs
- [ ] T027 [US1] Implement DataFusion-based query execution in PostgreSQLAdapter.execute_query in backend/src/services/database/postgresql.rs
- [ ] T028 [US1] Implement DataFusion-based query execution in MySQLAdapter.execute_query in backend/src/services/database/mysql.rs
- [X] T029 [US1] Create dialect translation service in backend/src/services/datafusion/translator.rs
- [X] T030 [US1] Implement PostgreSQL dialect translation logic in backend/src/services/datafusion/translator.rs
- [X] T031 [US1] Implement MySQL dialect translation logic in backend/src/services/datafusion/translator.rs
- [X] T032 [US1] Update QueryService to use DataFusion executor in backend/src/services/query_service.rs
- [X] T033 [US1] Integrate dialect translation into query execution flow in backend/src/services/query_service.rs
- [X] T034 [US1] Update POST /api/connections/{id}/query endpoint to use unified SQL in backend/src/api/handlers/query.rs
- [X] T035 [US1] Add database type detection in query handler in backend/src/api/handlers/query.rs
- [ ] T036 [US1] Update SQL validator to support DataFusion SQL syntax in backend/src/validation/sql_validator.rs
- [ ] T037 [US1] Create unified query API client service in frontend/src/services/unified_query.ts
- [ ] T038 [US1] Update QueryEditor component to support unified SQL syntax in frontend/src/components/QueryEditor/index.tsx
- [ ] T039 [US1] Add database type indicator in QueryPage in frontend/src/pages/QueryPage.tsx
- [ ] T040 [US1] Update error messages to indicate dialect translation in frontend components
- [ ] T041 [US1] Add loading states for dialect translation in frontend components

**Checkpoint**: At this point, User Story 1 should be fully functional. Users can execute unified SQL queries against PostgreSQL and MySQL databases with automatic dialect translation.

---

## Phase 4: User Story 2 - Cross-Database Join Queries (Priority: P2)

**Goal**: Users can execute SQL queries that join tables across different databases (e.g., PostgreSQL table JOIN MySQL table). DataFusion handles the cross-database query execution.

**Independent Test**: Execute a cross-database JOIN query (e.g., SELECT * FROM postgres_db.users JOIN mysql_db.orders ON users.id = orders.user_id) and verify that DataFusion coordinates the query execution across both databases and returns unified results. This delivers value independently as users can query data across multiple databases without manual data movement.

### Implementation for User Story 2

- [ ] T042 [P] [US2] Create CrossDatabaseQueryRequest model in backend/src/models/cross_database_query.rs
- [ ] T043 [US2] Research DataFusion federated query execution patterns
- [ ] T044 [US2] Create CrossDatabaseQueryPlanner in backend/src/services/datafusion/cross_db_planner.rs
- [ ] T045 [US2] Implement table identification across databases in backend/src/services/datafusion/cross_db_planner.rs
- [ ] T046 [US2] Implement query decomposition for cross-database queries in backend/src/services/datafusion/cross_db_planner.rs
- [ ] T047 [US2] Create DataFusionFederatedExecutor in backend/src/services/datafusion/federated_executor.rs
- [ ] T048 [US2] Implement sub-query execution per database in backend/src/services/datafusion/federated_executor.rs
- [ ] T049 [US2] Implement result merging logic for JOIN operations in backend/src/services/datafusion/federated_executor.rs
- [ ] T050 [US2] Implement result merging logic for UNION operations in backend/src/services/datafusion/federated_executor.rs
- [ ] T051 [US2] Add cross-database query validation in backend/src/validation/cross_db_validator.rs
- [ ] T052 [US2] Implement connection pool management for multiple databases in backend/src/services/datafusion/session.rs
- [ ] T053 [US2] Update QueryService to support cross-database queries in backend/src/services/query_service.rs
- [ ] T054 [US2] Create POST /api/cross-database/query endpoint in backend/src/api/handlers/cross_database_query.rs
- [ ] T055 [US2] Register cross-database query route in backend/src/api/routes.rs
- [ ] T056 [US2] Create cross-database query API client service in frontend/src/services/cross_database_query.ts
- [ ] T057 [US2] Create CrossDatabaseQueryBuilder component in frontend/src/components/CrossDatabaseQueryBuilder/
- [ ] T058 [US2] Add database selection UI for cross-database queries in frontend/src/components/CrossDatabaseQueryBuilder/
- [ ] T059 [US2] Add table selection UI for cross-database queries in frontend/src/components/CrossDatabaseQueryBuilder/
- [ ] T060 [US2] Integrate cross-database query builder in QueryPage in frontend/src/pages/QueryPage.tsx
- [ ] T061 [US2] Add visual indicators for cross-database queries in query results display
- [ ] T062 [US2] Add error handling for cross-database query failures in frontend

**Checkpoint**: At this point, User Stories 1 AND 2 should both work. Users can execute unified SQL queries and cross-database JOIN queries.

---

## Phase 5: User Story 3 - Extensible Database Support (Priority: P3)

**Goal**: The system architecture makes it easy to add support for new database types. Adding a new database requires minimal code changes and automatically gains unified SQL and cross-database query capabilities.

**Independent Test**: Add support for a new database type (e.g., Doris or Druid) by implementing only the database-specific adapter and dialect translator. Verify that the new database automatically supports unified SQL queries and can participate in cross-database queries. This delivers value independently as the system becomes more useful with each new database added.

### Implementation for User Story 3

- [ ] T063 [US3] Create DatabaseDialectRegistry in backend/src/services/datafusion/dialect_registry.rs
- [ ] T064 [US3] Implement dialect registration mechanism in backend/src/services/datafusion/dialect_registry.rs
- [ ] T065 [US3] Create DatabaseAdapterFactory trait in backend/src/services/database/factory.rs
- [ ] T066 [US3] Implement adapter factory pattern in backend/src/services/database/factory.rs
- [ ] T067 [US3] Create DatabaseSupportPlugin trait in backend/src/services/database/plugin.rs
- [ ] T068 [US3] Implement plugin registration system in backend/src/services/database/plugin.rs
- [ ] T069 [US3] Refactor PostgreSQL support as a plugin in backend/src/services/database/postgresql_plugin.rs
- [ ] T070 [US3] Refactor MySQL support as a plugin in backend/src/services/database/mysql_plugin.rs
- [ ] T071 [US3] Create Doris adapter plugin in backend/src/services/database/doris_plugin.rs
- [ ] T072 [US3] Implement Doris dialect translator in backend/src/services/datafusion/dialect.rs
- [ ] T073 [US3] Create Druid adapter plugin in backend/src/services/database/druid_plugin.rs
- [ ] T074 [US3] Implement Druid dialect translator in backend/src/services/datafusion/dialect.rs
- [ ] T075 [US3] Create plugin configuration system in backend/src/config/plugins.rs
- [ ] T076 [US3] Update database type enum to support dynamic registration in backend/src/services/database/mod.rs
- [ ] T077 [US3] Create plugin documentation template in docs/plugins/PLUGIN_TEMPLATE.md
- [ ] T078 [US3] Update frontend to dynamically load database types from API in frontend/src/services/connection.ts
- [ ] T079 [US3] Add plugin management UI in frontend/src/components/PluginManager/
- [ ] T080 [US3] Add database type discovery endpoint GET /api/database-types in backend/src/api/handlers/database_types.rs

**Checkpoint**: All user stories should now be complete. The system supports unified SQL, cross-database queries, and is easily extensible to new database types.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T081 [P] Add query plan visualization for DataFusion logical plans
- [ ] T082 [P] Add query performance metrics and profiling
- [ ] T083 [P] Implement query result caching for cross-database queries
- [ ] T084 [P] Add query optimization hints for DataFusion
- [ ] T085 [P] Improve error messages with dialect-specific suggestions
- [ ] T086 [P] Add query execution logging with dialect translation details
- [ ] T087 [P] Create unified SQL syntax documentation
- [ ] T088 [P] Add dialect translation test suite
- [ ] T089 [P] Add cross-database query test suite
- [ ] T090 [P] Update README.md with unified SQL usage examples
- [ ] T091 [P] Create migration guide from old query API to unified SQL API
- [ ] T092 [P] Add query plan explanation endpoint GET /api/queries/{id}/plan
- [ ] T093 [P] Add dialect translation preview endpoint POST /api/dialect/translate
- [ ] T094 [P] Code cleanup and refactoring
- [ ] T095 [P] Performance optimization for cross-database queries

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
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Depends on US1 for unified SQL execution
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - Depends on US1 and US2 for plugin architecture

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
Task: "Create UnifiedQueryRequest model in backend/src/models/unified_query.rs"
Task: "Add database_type field to Query model in backend/src/models/query.rs"

# Launch dialect translators in parallel:
Task: "Implement PostgreSQL dialect translator in backend/src/services/datafusion/translator.rs"
Task: "Implement MySQL dialect translator in backend/src/services/datafusion/translator.rs"

# Launch adapter refactoring in parallel:
Task: "Refactor PostgreSQLAdapter to use DataFusion executor in backend/src/services/database/postgresql.rs"
Task: "Refactor MySQLAdapter to use DataFusion executor in backend/src/services/database/mysql.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup & Research
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
   - Developer A: User Story 1 (PostgreSQL dialect)
   - Developer B: User Story 1 (MySQL dialect)
   - Developer C: User Story 2 (cross-database planner)
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
- DataFusion SQL syntax should follow SQL standard as much as possible
- Dialect translation should be transparent to users
- Cross-database queries should handle network failures gracefully
- Plugin system should support hot-reloading (future enhancement)

