# Tasks: MySQL Database Support

**Input**: User request to add MySQL support following PostgreSQL implementation pattern
**Prerequisites**: Existing PostgreSQL implementation in backend/src/services/database/postgresql.rs
**Feature Branch**: `002-mysql-support`

**Organization**: Tasks are organized to mirror the PostgreSQL implementation pattern, enabling independent testing and validation of MySQL support.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

---

## Phase 1: Setup (MySQL Infrastructure)

**Purpose**: Set up MySQL dependencies and basic adapter structure

- [X] T001 Add MySQL client dependencies to backend/Cargo.toml (mysql_async, deadpool crate for connection pooling)
- [X] T002 [P] Create MySQL adapter structure following PostgreSQL pattern in backend/src/services/database/mysql.rs
- [X] T003 [P] Add MySQL URL validation and connection initialization in backend/src/services/database/mysql.rs
- [X] T004 Update backend/src/services/db_service.rs to support MySQL database type detection and adapter creation

---

## Phase 2: Foundational (Core MySQL Adapter)

**Purpose**: Core MySQL adapter implementation that MUST be complete before query support can be added

**⚠️ CRITICAL**: Query execution cannot work until metadata retrieval is functional

- [X] T005 Implement connection pooling for MySQL in backend/src/services/database/mysql.rs (use deadpool-mysql pattern)
- [X] T006 Implement test_connection method for MySQL in backend/src/services/database/mysql.rs
- [X] T007 Add MySQL error mapping to AppError types in backend/src/services/database/mysql.rs

**Checkpoint**: MySQL connection infrastructure ready - metadata and query implementation can now proceed

---

## Phase 3: User Story 1 - MySQL Metadata Extraction (Priority: P1) 🎯 MVP

**Goal**: Enable users to connect to MySQL databases and view their schema structure (tables, views, columns) just like PostgreSQL

**Independent Test**: Connect to a MySQL test database and verify that all tables, views, and columns are correctly retrieved and displayed

### Implementation for User Story 1

- [X] T008 [P] [US1] Implement get_schemas method for MySQL using information_schema.schemata in backend/src/services/database/mysql.rs
- [X] T009 [P] [US1] Implement get_tables method for MySQL using information_schema.tables in backend/src/services/database/mysql.rs
- [X] T010 [P] [US1] Implement get_views method for MySQL using information_schema.views in backend/src/services/database/mysql.rs
- [X] T011 [US1] Implement get_table_columns method for MySQL using information_schema.columns in backend/src/services/database/mysql.rs
- [X] T012 [US1] Add primary key and foreign key detection for MySQL columns in backend/src/services/database/mysql.rs
- [X] T013 [US1] Implement connect_and_get_metadata method for MySQL adapter in backend/src/services/database/mysql.rs
- [X] T014 [US1] Implement retrieve_metadata helper method for MySQL in backend/src/services/database/mysql.rs
- [X] T015 [US1] Update OpenAPI spec to include mysql in database_type enum in specs/001-db-query-tool/contracts/openapi.yaml
- [ ] T016 [US1] Test metadata extraction with real MySQL database connection

**Checkpoint**: At this point, MySQL metadata extraction should be fully functional and testable independently

---

## Phase 4: User Story 2 - MySQL Query Execution (Priority: P2)

**Goal**: Enable users to execute validated SQL queries against MySQL databases with the same security guarantees as PostgreSQL

**Independent Test**: Execute various SELECT queries against a MySQL test database and verify results are returned correctly in JSON format

### Implementation for User Story 2

- [X] T017 [US2] Implement MySQL row-to-JSON conversion for common data types (INT, VARCHAR, DECIMAL, DATE, DATETIME) in backend/src/services/database/mysql.rs
- [X] T018 [US2] Add support for MySQL-specific data types (TINYINT, MEDIUMINT, BIGINT, TEXT, BLOB) in backend/src/services/database/mysql.rs
- [X] T019 [US2] Implement execute_query method for MySQL with timeout handling in backend/src/services/database/mysql.rs
- [X] T020 [US2] Add MySQL query error handling and detailed error messages in backend/src/services/database/mysql.rs
- [ ] T021 [US2] Test query execution with various SELECT statements on MySQL database
- [ ] T022 [US2] Verify LIMIT clause enforcement works correctly with MySQL syntax

**Checkpoint**: At this point, MySQL query execution should work independently with full validation

---

## Phase 5: User Story 3 - Natural Language SQL Generation for MySQL (Priority: P3)

**Goal**: Enable LLM to generate MySQL-compatible SQL queries from natural language questions

**Independent Test**: Provide natural language questions about MySQL database and verify generated SQL uses MySQL syntax and executes successfully

### Implementation for User Story 3

- [X] T023 [US3] Update LLM prompt template to detect database type (PostgreSQL vs MySQL) in backend/src/services/llm_service.rs
- [X] T024 [US3] Modify generate_sql_from_natural_language to generate MySQL-specific SQL syntax in backend/src/services/llm_service.rs
- [X] T025 [US3] Update fallback_sql_generation to support MySQL syntax in backend/src/services/llm_service.rs
- [X] T026 [US3] Add MySQL-specific SQL hints to LLM prompt (e.g., LIMIT syntax, date functions) in backend/src/services/llm_service.rs
- [ ] T027 [US3] Test natural language query generation with MySQL-specific questions
- [ ] T028 [US3] Verify generated MySQL queries pass validation and execute correctly

**Checkpoint**: All user stories should now be independently functional for MySQL

---

## Phase 6: Integration & Testing

**Purpose**: Integration testing and validation across all MySQL features

- [ ] T029 [P] Add integration test for MySQL connection creation in backend/tests/integration/
- [ ] T030 [P] Add integration test for MySQL metadata retrieval in backend/tests/integration/
- [ ] T031 [P] Add integration test for MySQL query execution in backend/tests/integration/
- [ ] T032 [P] Add integration test for MySQL natural language query in backend/tests/integration/
- [ ] T033 Test MySQL with various database configurations (different versions, character sets, collations)
- [ ] T034 Verify error handling for MySQL-specific error codes and conditions
- [ ] T035 Test MySQL connection pooling under load

**Checkpoint**: MySQL support is fully tested and production-ready

---

## Phase 7: Documentation & Polish

**Purpose**: Documentation and improvements that affect MySQL support

- [X] T036 [P] Update CLAUDE.md to document MySQL support and configuration
- [X] T037 [P] Add MySQL connection examples to fixtures/test.rest
- [X] T038 [P] Update README.md with MySQL setup instructions
- [ ] T039 Add MySQL-specific troubleshooting guide to documentation
- [X] T040 [P] Update frontend UI to show MySQL connection option
- [ ] T041 Add MySQL to constitution compliance verification
- [X] T042 Run full test suite to verify no regressions in PostgreSQL support

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-5)**: All depend on Foundational phase completion
  - US1 (Metadata) is independent - can start after Phase 2
  - US2 (Query Execution) depends on US1 completion (needs metadata for context)
  - US3 (Natural Language) depends on US1 and US2 completion (needs both metadata and query execution)
- **Integration & Testing (Phase 6)**: Depends on all user stories being complete
- **Documentation (Phase 7)**: Can start after any phase completes; some tasks depend on Phase 6

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after US1 (needs metadata for testing context) - Partially independent but benefits from US1
- **User Story 3 (P3)**: Depends on US1 (needs metadata) and US2 (needs query execution) - Full dependency on previous stories

### Within Each User Story

**User Story 1 (Metadata)**:
- T008, T009, T010 can run in parallel (different query methods)
- T011 depends on T009, T010 (needs table/view info)
- T012 depends on T011 (extends column detection)
- T013 depends on T008-T012 (orchestrates all metadata retrieval)
- T014 depends on T013 (helper method)
- T015, T016 can run after T014

**User Story 2 (Query Execution)**:
- T017, T018 can run in parallel (different type conversions)
- T019 depends on T017, T018 (uses type conversion)
- T020-T022 depend on T019

**User Story 3 (Natural Language)**:
- T023-T025 should run sequentially (modify same file)
- T026-T028 depend on T025

### Parallel Opportunities

- **Setup Phase**: T002, T003 can run in parallel
- **User Story 1**: T008, T009, T010 can run in parallel
- **User Story 2**: T017, T018 can run in parallel
- **Integration Testing**: T029, T030, T031, T032 can run in parallel
- **Documentation**: T036, T037, T038, T040 can run in parallel

---

## Parallel Example: User Story 1 (Metadata)

```bash
# Launch metadata query methods in parallel:
Task: "Implement get_schemas method in backend/src/services/database/mysql.rs"
Task: "Implement get_tables method in backend/src/services/database/mysql.rs"
Task: "Implement get_views method in backend/src/services/database/mysql.rs"

# Wait for completion, then proceed with:
Task: "Implement get_table_columns method in backend/src/services/database/mysql.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (MySQL dependencies and adapter structure)
2. Complete Phase 2: Foundational (connection pooling and error handling)
3. Complete Phase 3: User Story 1 (metadata extraction)
4. **STOP and VALIDATE**: Connect to MySQL test database and verify all metadata is retrieved
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → MySQL connection infrastructure ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP: MySQL metadata viewing!)
3. Add User Story 2 → Test independently → Deploy/Demo (MySQL query execution!)
4. Add User Story 3 → Test independently → Deploy/Demo (Full MySQL parity with PostgreSQL!)
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (metadata extraction)
   - Once US1 complete:
     - Developer A continues: User Story 2 (query execution)
     - Developer B starts: Integration tests (Phase 6)
   - Once US2 complete:
     - Developer A: User Story 3 (natural language)
     - Developer C: Documentation (Phase 7)
3. Stories complete and integrate sequentially

---

## Notes

- Follow PostgreSQL adapter pattern exactly for consistency
- MySQL uses `information_schema` just like PostgreSQL, but with some syntax differences
- MySQL connection strings use `mysql://` scheme instead of `postgresql://`
- SQL validation rules remain the same (SELECT only, LIMIT enforcement)
- Type mappings may differ between MySQL and PostgreSQL (handle carefully)
- Test with MySQL 5.7+ and MySQL 8.0+ for compatibility
- Consider MariaDB compatibility (uses same `mysql://` scheme)
- All security principles from constitution apply equally to MySQL
- Metadata caching works identically for MySQL
- Connection pooling pattern should mirror PostgreSQL implementation

---

## Summary

**Total Tasks**: 42
- Phase 1 (Setup): 4 tasks
- Phase 2 (Foundational): 3 tasks
- Phase 3 (US1 - Metadata): 9 tasks
- Phase 4 (US2 - Query Execution): 6 tasks
- Phase 5 (US3 - Natural Language): 6 tasks
- Phase 6 (Integration): 7 tasks
- Phase 7 (Documentation): 7 tasks

**Parallel Opportunities**: 15 tasks can run in parallel with others
**MVP Scope**: Phases 1-3 (16 tasks) for basic MySQL metadata viewing
**Full Parity**: All 42 tasks for complete PostgreSQL feature parity

**Key Implementation Pattern**: Mirror PostgreSQL adapter implementation while adapting for MySQL-specific syntax and type system differences
