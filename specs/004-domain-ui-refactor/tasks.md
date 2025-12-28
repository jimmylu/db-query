# Implementation Tasks - Domain-Based UI Refactoring

**Feature**: 004-domain-ui-refactor
**Branch**: `004-domain-ui-refactor`
**Created**: 2025-12-27
**Last Updated**: 2025-12-28
**Status**: 69% Complete (29/42 tasks) - Phase 6 Complete, Phase 7 Next

---

## Progress Summary (As of 2025-12-28)

### Completed: 29/42 tasks (69%)
- ✅ **Phase 1**: Backend Domain Management (6/6 tasks - 100%)
- ✅ **Phase 2**: Frontend Domain Management UI (5/6 tasks - 90%)
- ✅ **Phase 3**: Backend Domain-Scoped Data Sources (4/4 tasks - 100%)
- ✅ **Phase 4**: Frontend Domain-Scoped Data Sources UI (4/4 tasks - 100%)
- ✅ **Phase 5**: Backend Domain-Scoped Queries (4/4 tasks - 100%) ← **COMPLETE**
- ✅ **Phase 6**: Frontend Domain-Scoped Query UI (4/4 tasks - 100%) ← **COMPLETE**
- ⏳ **Phase 7**: AWS Cloudscape Styling (0/4 tasks - 0%) ← **NEXT**
- ⏳ **Phase 8**: Testing & QA (0/5 tasks - 0%)
- ⏳ **Phase 9**: Documentation (0/5 tasks - 0%)

### Key Implementation Notes
1. **Modified Approach**: Domain UI integrated in header (CustomHeader) instead of separate sidebar - simpler, better UX
2. **Enhanced**: Logo-based database selection (4 databases: PostgreSQL, MySQL, Doris, Druid)
3. **Enhanced**: Current domain display with Alert component for stronger awareness
4. **NEW**: Saved queries with domain scoping - full CRUD operations
5. **NEW**: Query history with automatic logging (AI vs manual tracking)
6. **NEW**: Backend API for saved queries and history (7 endpoints)

### Latest Completions (2025-12-28)
- ✅ Phase 5: SavedQuery & QueryHistory models + SQLite storage + API handlers
- ✅ Phase 6: SavedQueries & QueryHistory React components + API services

### Next Steps (Recommended)
1. **Skip Phase 7**: AWS Cloudscape styling (optional UI enhancement)
2. **Immediate**: Phase 8 - Testing & QA (ensure production readiness)
3. **Then**: Phase 9 - Documentation
4. **Goal**: Production-ready MVP with full domain isolation

See [PROGRESS_UPDATE.md](./PROGRESS_UPDATE.md) for detailed progress tracking.

---

## Task Organization

Tasks are organized by user story (P1 → P2 → P3) with strict dependency ordering. Each task includes:
- **Task ID**: Unique identifier (T001-T042)
- **Dependencies**: Must-complete-first tasks
- **Estimated Effort**: Time estimate in hours
- **Acceptance Criteria**: Definition of done
- **Status**: ✅ Complete | ⏳ Pending | 🚧 In Progress

---

## Phase 1: Backend - Domain Management (P1) ✅ COMPLETED

### T001: Create Domain Data Model ✅
**Dependencies**: None
**Effort**: 1h
**Files**: `backend/src/models/domain.rs` (new)

**Implementation**:
- Create `Domain` struct with fields:
  - `id: String` (UUID)
  - `name: String` (unique)
  - `description: Option<String>`
  - `created_at: DateTime<Utc>`
  - `updated_at: DateTime<Utc>`
- Implement `Serialize`/`Deserialize` traits
- Add validation (name 1-50 chars, unique)

**Acceptance Criteria**:
- [x] Domain model compiles without errors
- [x] Validation logic prevents duplicate names
- [x] JSON serialization works correctly

---

### T002: Update Database Schema for Domains ✅
**Dependencies**: T001
**Effort**: 1.5h
**Files**: `backend/src/storage/sqlite.rs`

**Implementation**:
- Create `domains` table in SQLite:
  ```sql
  CREATE TABLE domains (
    id TEXT PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
  )
  ```
- Add foreign key `domain_id` to `connections` table
- Add foreign key `domain_id` to future `saved_queries` table
- Write migration logic to handle existing connections (default domain)

**Acceptance Criteria**:
- [x] Schema migration runs successfully
- [x] Existing connections assigned to default domain
- [x] Foreign key constraints enforced

---

### T003: Implement Domain Storage Service
**Dependencies**: T002
**Effort**: 2h
**Files**: `backend/src/storage/sqlite.rs`

**Implementation**:
- Add methods to `SqliteStorage`:
  - `create_domain(domain: &Domain) -> Result<()>`
  - `get_domain(id: &str) -> Result<Option<Domain>>`
  - `list_domains() -> Result<Vec<Domain>>`
  - `update_domain(id: &str, updates: DomainUpdate) -> Result<()>`
  - `delete_domain(id: &str) -> Result<()>`
- Implement CASCADE delete logic for domain resources
- Add transaction support for atomic operations

**Acceptance Criteria**:
- [ ] All CRUD operations work correctly
- [ ] Deleting domain removes associated resources
- [ ] Transactions rollback on error

---

### T004: Create Domain API Handlers
**Dependencies**: T003
**Effort**: 2h
**Files**: `backend/src/api/handlers/domain.rs` (new)

**Implementation**:
- Create handler functions:
  - `create_domain(State, Json<CreateDomainRequest>)` → POST /api/domains
  - `list_domains(State)` → GET /api/domains
  - `get_domain(State, Path)` → GET /api/domains/:id
  - `update_domain(State, Path, Json)` → PUT /api/domains/:id
  - `delete_domain(State, Path)` → DELETE /api/domains/:id
- Return resource counts with each domain (connections, queries)
- Add error handling (duplicate name, not found, etc.)

**Acceptance Criteria**:
- [ ] All endpoints return correct HTTP status codes
- [ ] Domain list includes resource counts
- [ ] Error responses include actionable messages

---

### T005: Register Domain Routes
**Dependencies**: T004
**Effort**: 0.5h
**Files**: `backend/src/api/routes.rs`, `backend/src/api/handlers/mod.rs`

**Implementation**:
- Add `pub mod domain;` to handlers/mod.rs
- Register routes in `create_router_with_state()`:
  ```rust
  .route("/api/domains", get(domain::list_domains).post(domain::create_domain))
  .route("/api/domains/:id", get(domain::get_domain)
      .put(domain::update_domain)
      .delete(domain::delete_domain))
  ```

**Acceptance Criteria**:
- [ ] All routes accessible via HTTP
- [ ] Routes use correct HTTP methods
- [ ] CORS configured properly

---

### T006: Add Domain Tests
**Dependencies**: T005
**Effort**: 1.5h
**Files**: `backend/src/api/handlers/domain.rs`, `backend/src/storage/sqlite.rs`

**Implementation**:
- Unit tests for domain storage:
  - Test CRUD operations
  - Test unique name constraint
  - Test CASCADE delete
- Integration tests for API:
  - Test domain creation flow
  - Test duplicate name rejection
  - Test domain deletion with resources

**Acceptance Criteria**:
- [ ] All tests pass
- [ ] Test coverage > 80% for domain module
- [ ] Edge cases covered (empty name, long description, etc.)

---

## Phase 2: Frontend - Domain Management UI (P1)

### T007: Create Domain Types
**Dependencies**: None
**Effort**: 0.5h
**Files**: `frontend/src/types/domain.ts` (new)

**Implementation**:
- Define TypeScript interfaces:
  ```typescript
  export interface Domain {
    id: string;
    name: string;
    description?: string;
    created_at: string;
    updated_at: string;
    resource_counts?: {
      connections: number;
      queries: number;
    };
  }

  export interface CreateDomainRequest {
    name: string;
    description?: string;
  }
  ```

**Acceptance Criteria**:
- [ ] Types match backend models
- [ ] No TypeScript compilation errors

---

### T008: Create Domain API Service
**Dependencies**: T007
**Effort**: 1h
**Files**: `frontend/src/services/domain.ts` (new)

**Implementation**:
- Create API client functions:
  - `listDomains(): Promise<Domain[]>`
  - `getDomain(id: string): Promise<Domain>`
  - `createDomain(request: CreateDomainRequest): Promise<Domain>`
  - `updateDomain(id: string, updates: Partial<Domain>): Promise<Domain>`
  - `deleteDomain(id: string): Promise<void>`
- Use axios instance from `api.ts`
- Add error handling and type safety

**Acceptance Criteria**:
- [ ] All API calls return correctly typed responses
- [ ] Error handling propagates API errors
- [ ] Service works with backend endpoints

---

### T009: Create Domain Context
**Dependencies**: T008
**Effort**: 1.5h
**Files**: `frontend/src/contexts/DomainContext.tsx` (new)

**Implementation**:
- Create React Context for active domain:
  ```typescript
  interface DomainContextType {
    activeDomain: Domain | null;
    domains: Domain[];
    setActiveDomain: (domain: Domain) => void;
    loadDomains: () => Promise<void>;
    createDomain: (request: CreateDomainRequest) => Promise<Domain>;
    deleteDomain: (id: string) => Promise<void>;
  }
  ```
- Persist active domain to localStorage
- Restore domain on app load

**Acceptance Criteria**:
- [ ] Context provides domain state to all components
- [ ] Active domain persists across page refresh
- [ ] Domain list updates on create/delete

---

### T010: Create Domain Sidebar Component
**Dependencies**: T009
**Effort**: 2.5h
**Files**: `frontend/src/components/DomainSidebar/index.tsx` (new)

**Implementation**:
- Create sidebar component with AWS Cloudscape styling:
  - Domain list (scrollable)
  - Active domain indicator (highlighted)
  - "Create Domain" button
  - Domain context menu (edit, delete)
- Use Ant Design components customized to match AWS style:
  - Menu component for domain list
  - Modal for create/edit forms
  - Popconfirm for delete confirmation
- Show resource counts per domain

**Acceptance Criteria**:
- [ ] Sidebar matches AWS Cloudscape visual design
- [ ] Domain selection updates active context
- [ ] Create/delete operations work correctly
- [ ] Resource counts displayed accurately

---

### T011: Create Domain Modal Forms
**Dependencies**: T009
**Effort**: 2h
**Files**: `frontend/src/components/DomainModal/index.tsx` (new)

**Implementation**:
- Create domain form modal:
  - Name field (required, 1-50 chars)
  - Description field (optional, textarea)
  - Validation messages
  - Save/Cancel buttons
- Support both create and edit modes
- Show loading state during API calls
- Display error messages

**Acceptance Criteria**:
- [ ] Form validates input correctly
- [ ] Create mode creates new domain
- [ ] Edit mode updates existing domain
- [ ] Errors display user-friendly messages

---

### T012: Integrate Domain Sidebar into Layout
**Dependencies**: T010
**Effort**: 1h
**Files**: `frontend/src/App.tsx`

**Implementation**:
- Wrap app with `DomainProvider`
- Replace left sidebar with `DomainSidebar` component
- Update layout to three-panel structure:
  - Left: Domain navigation (fixed width 250px)
  - Center: Main content (flexible)
  - Right: Context panel (conditional)
- Ensure responsive behavior (min 1024px)

**Acceptance Criteria**:
- [ ] Domain sidebar visible on all pages
- [ ] Layout renders correctly at 1024px+ width
- [ ] Domain context accessible in all components

---

## Phase 3: Backend - Domain-Scoped Data Sources (P2)

### T013: Update Connection Model for Domains
**Dependencies**: T002
**Effort**: 1h
**Files**: `backend/src/models/connection.rs`

**Implementation**:
- Add `domain_id: String` field to `Connection` struct
- Update `CreateConnectionRequest` to require `domain_id`
- Add validation to ensure domain exists before creating connection

**Acceptance Criteria**:
- [ ] Connection model includes domain_id
- [ ] Creating connection validates domain existence
- [ ] Existing tests updated for domain_id

---

### T014: Update Connection Storage for Domain Filtering
**Dependencies**: T013
**Effort**: 1.5h
**Files**: `backend/src/storage/sqlite.rs`

**Implementation**:
- Update `list_connections()` to accept optional `domain_id` filter:
  - `list_connections(domain_id: Option<&str>) -> Result<Vec<Connection>>`
- Update `create_connection()` to validate domain exists
- Ensure domain foreign key constraint enforced

**Acceptance Criteria**:
- [ ] Filtering by domain returns only scoped connections
- [ ] Creating connection in non-existent domain fails
- [ ] Foreign key constraint prevents orphaned connections

---

### T015: Update Connection API Handlers
**Dependencies**: T014
**Effort**: 1.5h
**Files**: `backend/src/api/handlers/connection.rs`

**Implementation**:
- Update `list_connections` handler:
  - Accept optional `domain_id` query parameter
  - Filter connections by domain if provided
- Update `create_connection` handler:
  - Require `domain_id` in request body
  - Validate domain exists
- Add endpoint: GET /api/domains/:id/connections

**Acceptance Criteria**:
- [ ] List endpoint filters by domain when provided
- [ ] Create endpoint validates domain
- [ ] Domain-specific endpoint returns scoped connections

---

### T016: Update Connection Tests for Domains
**Dependencies**: T015
**Effort**: 1h
**Files**: `backend/src/api/handlers/connection.rs`

**Implementation**:
- Update existing connection tests to include domain_id
- Add new tests:
  - Test domain filtering
  - Test domain validation on create
  - Test domain isolation (connections not visible across domains)

**Acceptance Criteria**:
- [ ] All existing tests pass with domain updates
- [ ] Domain isolation verified by tests
- [ ] Edge cases covered

---

## Phase 4: Frontend - Domain-Scoped Data Sources UI (P2)

### T017: Update Connection Types for Domains
**Dependencies**: T007
**Effort**: 0.5h
**Files**: `frontend/src/types/index.ts`

**Implementation**:
- Add `domain_id: string` to `Connection` interface
- Update `CreateConnectionRequest` to include `domain_id`
- Ensure type compatibility with backend

**Acceptance Criteria**:
- [ ] Types match backend models
- [ ] No TypeScript errors

---

### T018: Update Connection Service for Domain Filtering
**Dependencies**: T017, T009
**Effort**: 1h
**Files**: `frontend/src/services/connection.ts`

**Implementation**:
- Update `listConnections()` to use active domain from context
- Update `createConnection()` to include active domain_id
- Add domain filtering to API calls

**Acceptance Criteria**:
- [ ] Connection list filtered by active domain
- [ ] New connections created in active domain
- [ ] Switching domains updates connection list

---

### T019: Update DatabaseConnection Component
**Dependencies**: T018
**Effort**: 2h
**Files**: `frontend/src/components/DatabaseConnection/index.tsx`

**Implementation**:
- Use `DomainContext` to get active domain
- Display active domain in connection form
- Auto-populate domain_id in create requests
- Show domain name with each connection in list
- Disable form if no active domain selected

**Acceptance Criteria**:
- [ ] Form displays current domain
- [ ] Connections tagged with domain name
- [ ] Form disabled when no domain active

---

### T020: Update MetadataViewer for Domain Scope
**Dependencies**: T018
**Effort**: 1h
**Files**: `frontend/src/components/MetadataViewer/index.tsx`

**Implementation**:
- Filter metadata by active domain
- Update when active domain changes
- Show "No domain selected" state
- Clear metadata when switching domains

**Acceptance Criteria**:
- [ ] Metadata updates on domain switch
- [ ] No cross-domain metadata leakage
- [ ] Empty state shown when appropriate

---

## Phase 5: Backend - Domain-Scoped Queries (P3)

### T021: Create Saved Query Model
**Dependencies**: T001
**Effort**: 1.5h
**Files**: `backend/src/models/query.rs`

**Implementation**:
- Create `SavedQuery` struct:
  - `id: String`
  - `domain_id: String`
  - `connection_id: String`
  - `name: String`
  - `query_text: String`
  - `created_at: DateTime<Utc>`
- Create `QueryHistory` struct:
  - `id: String`
  - `domain_id: String`
  - `connection_id: String`
  - `query_text: String`
  - `execution_time_ms: i64`
  - `row_count: usize`
  - `executed_at: DateTime<Utc>`

**Acceptance Criteria**:
- [ ] Models compile without errors
- [ ] Serialization works correctly
- [ ] Domain and connection references validated

---

### T022: Create Saved Query Storage
**Dependencies**: T021
**Effort**: 2h
**Files**: `backend/src/storage/sqlite.rs`

**Implementation**:
- Create `saved_queries` table
- Create `query_history` table
- Implement CRUD operations:
  - `save_query(query: &SavedQuery) -> Result<()>`
  - `list_saved_queries(domain_id: &str) -> Result<Vec<SavedQuery>>`
  - `delete_saved_query(id: &str) -> Result<()>`
  - `add_query_history(entry: &QueryHistory) -> Result<()>`
  - `list_query_history(domain_id: &str, limit: usize) -> Result<Vec<QueryHistory>>`
- Add foreign key constraints (domain_id, connection_id)

**Acceptance Criteria**:
- [ ] Tables created successfully
- [ ] CRUD operations work correctly
- [ ] Domain isolation enforced at storage level

---

### T023: Create Query API Handlers
**Dependencies**: T022
**Effort**: 2h
**Files**: `backend/src/api/handlers/query.rs`

**Implementation**:
- Update `execute_query` to log to history:
  - Extract domain_id from connection
  - Create QueryHistory entry after execution
- Add new endpoints:
  - POST /api/domains/:id/queries/save
  - GET /api/domains/:id/queries/saved
  - DELETE /api/queries/:id
  - GET /api/domains/:id/queries/history

**Acceptance Criteria**:
- [ ] Query execution logs to history
- [ ] Saved queries scoped to domain
- [ ] History scoped to domain
- [ ] Endpoints return correct data

---

### T024: Update Query Service for Domain Context
**Dependencies**: T023
**Effort**: 1.5h
**Files**: `backend/src/services/query_service.rs`

**Implementation**:
- Add domain validation to query execution:
  - Verify connection belongs to domain
  - Prevent cross-domain query execution
- Add query history logging
- Update error messages to mention domain context

**Acceptance Criteria**:
- [ ] Cross-domain queries blocked
- [ ] All queries logged to history
- [ ] Error messages include domain context

---

## Phase 6: Frontend - Domain-Scoped Query UI (P3)

### T025: Create Saved Query Components
**Dependencies**: T017
**Effort**: 2.5h
**Files**: `frontend/src/components/SavedQueries/index.tsx` (new)

**Implementation**:
- Create saved query sidebar:
  - List saved queries for active domain
  - "Save Query" button
  - Click to load query into editor
  - Delete option per query
- Create save query modal:
  - Query name input
  - Preview of query text
  - Save/Cancel buttons
- AWS Cloudscape styling

**Acceptance Criteria**:
- [ ] Saved queries display correctly
- [ ] Clicking query loads into editor
- [ ] Save modal validates query name
- [ ] Delete confirms before removing

---

### T026: Create Query History Component
**Dependencies**: T017
**Effort**: 2h
**Files**: `frontend/src/components/QueryHistory/index.tsx` (new)

**Implementation**:
- Create history panel:
  - List last 50 queries in active domain
  - Show execution time and row count
  - Show timestamp
  - Click to load query into editor
- Add domain filtering (auto-update on domain change)
- AWS Cloudscape table styling

**Acceptance Criteria**:
- [ ] History shows domain-scoped queries only
- [ ] Updates when domain changes
- [ ] Clicking loads query into editor
- [ ] Shows execution metrics correctly

---

### T027: Update QueryEditor Component
**Dependencies**: T025, T026
**Effort**: 2h
**Files**: `frontend/src/components/QueryEditor/index.tsx`

**Implementation**:
- Integrate saved query sidebar (optional right panel)
- Add "Save Query" button to toolbar
- Add "History" button to show history panel
- Ensure domain context visible in editor
- Update Monaco editor theme to AWS Cloudscape colors

**Acceptance Criteria**:
- [ ] Save button opens save modal
- [ ] History button toggles history panel
- [ ] Editor shows current domain
- [ ] Theme matches AWS Cloudscape

---

### T028: Update QueryPage for Domain Integration
**Dependencies**: T027
**Effort**: 2h
**Files**: `frontend/src/pages/QueryPage.tsx`

**Implementation**:
- Use `DomainContext` for active domain
- Filter data sources by domain
- Pass domain context to editor and results
- Show "Select domain" message if no active domain
- Update query execution to include domain validation

**Acceptance Criteria**:
- [ ] Page requires active domain
- [ ] Data sources filtered correctly
- [ ] Query execution validates domain
- [ ] Empty state shown when needed

---

## Phase 7: UI Styling - AWS Cloudscape Design

### T029: Install Cloudscape Design System
**Dependencies**: None
**Effort**: 1h
**Files**: `frontend/package.json`

**Implementation**:
- Install `@cloudscape-design/components`
- Install `@cloudscape-design/global-styles`
- Update vite.config.ts for Cloudscape CSS
- Configure theme (dark mode optional)

**Acceptance Criteria**:
- [ ] Package installed successfully
- [ ] Global styles imported
- [ ] Theme configured
- [ ] No build errors

---

### T030: Create Cloudscape Theme Wrapper
**Dependencies**: T029
**Effort**: 1.5h
**Files**: `frontend/src/theme/cloudscape.tsx` (new)

**Implementation**:
- Create theme configuration:
  - AWS color palette
  - Typography (Amazon Ember font)
  - Spacing system
  - Border radius and shadows
- Create ThemeProvider wrapper
- Map Ant Design components to Cloudscape styles

**Acceptance Criteria**:
- [ ] Theme matches AWS Cloudscape visuals
- [ ] Components use correct colors/fonts
- [ ] Dark mode supported (optional)

---

### T031: Refactor Components to Use Cloudscape
**Dependencies**: T030
**Effort**: 4h
**Files**: Multiple component files

**Implementation**:
- Replace Ant Design components with Cloudscape equivalents:
  - Table → CloudscapeTable
  - Button → CloudscapeButton
  - Modal → CloudscapeModal
  - Form → CloudscapeForm
- Update component styling to match AWS patterns
- Ensure 90% visual consistency with AWS console

**Acceptance Criteria**:
- [ ] All major components use Cloudscape
- [ ] Visual consistency >= 90%
- [ ] No style regressions
- [ ] User testing confirms AWS-like feel

---

### T032: Update Layout to AWS Three-Panel Pattern
**Dependencies**: T031
**Effort**: 2h
**Files**: `frontend/src/App.tsx`, `frontend/src/index.css`

**Implementation**:
- Implement three-panel layout:
  - Left sidebar: 250px fixed width (domain navigation)
  - Center panel: Flexible width (main content)
  - Right panel: Conditional 300px (context/help)
- Add AWS-style header with breadcrumbs
- Ensure responsive design (min 1024px)

**Acceptance Criteria**:
- [ ] Layout matches AWS console structure
- [ ] Panels resize correctly
- [ ] Responsive at 1024px minimum
- [ ] Navigation feels AWS-like

---

## Phase 8: Testing & Quality Assurance

### T033: End-to-End Domain Workflow Test
**Dependencies**: T032
**Effort**: 2h
**Files**: `frontend/cypress/e2e/domain-workflow.cy.ts` (new, if using Cypress)

**Implementation**:
- Create test scenario:
  1. Create new domain "Production"
  2. Add PostgreSQL connection to domain
  3. View metadata for connection
  4. Execute query in domain
  5. Save query
  6. Switch to different domain
  7. Verify isolation (no Production resources visible)
- Verify success criteria from spec

**Acceptance Criteria**:
- [ ] Full workflow completes < 3 minutes
- [ ] Domain isolation verified (SC-007)
- [ ] No data leakage between domains
- [ ] All operations succeed

---

### T034: Performance Testing
**Dependencies**: T033
**Effort**: 2h
**Tools**: Chrome DevTools, Lighthouse

**Implementation**:
- Measure domain switching time (target < 5s)
- Measure app load time (target < 2s)
- Measure query execution time (maintain current performance)
- Test with 10 domains, 50 connections, 100 queries
- Profile React renders for unnecessary re-renders

**Acceptance Criteria**:
- [ ] Domain switching < 5 seconds (SC-001)
- [ ] App loads < 2 seconds (SC-006)
- [ ] Query performance unchanged (SC-004)
- [ ] No memory leaks detected

---

### T035: Accessibility Audit
**Dependencies**: T032
**Effort**: 1.5h
**Tools**: axe DevTools, WAVE

**Implementation**:
- Run accessibility scans on all pages
- Ensure keyboard navigation works
- Verify screen reader compatibility
- Test color contrast ratios
- Add ARIA labels where needed

**Acceptance Criteria**:
- [ ] WCAG 2.1 AA compliance
- [ ] Keyboard navigation functional
- [ ] Screen reader announces correctly
- [ ] No critical accessibility issues

---

### T036: Cross-Browser Testing
**Dependencies**: T032
**Effort**: 1.5h
**Browsers**: Chrome, Firefox, Safari, Edge

**Implementation**:
- Test full workflow in each browser
- Verify AWS Cloudscape styling renders correctly
- Test localStorage persistence
- Check for browser-specific bugs

**Acceptance Criteria**:
- [ ] Works in Chrome (latest)
- [ ] Works in Firefox (latest)
- [ ] Works in Safari (latest)
- [ ] Works in Edge (latest)

---

### T037: Security Audit for Domain Isolation
**Dependencies**: T033
**Effort**: 2h
**Files**: Backend API handlers

**Implementation**:
- Verify domain isolation at API level:
  - Test connection access across domains (should fail)
  - Test query execution across domains (should fail)
  - Test history access across domains (should fail)
- Check for SQL injection in domain names
- Verify CASCADE delete doesn't orphan resources
- Test concurrent domain operations

**Acceptance Criteria**:
- [ ] 100% domain isolation verified (SC-007)
- [ ] No SQL injection vulnerabilities
- [ ] Concurrent operations safe
- [ ] Audit log confirms isolation

---

## Phase 9: Documentation & Deployment

### T038: Update API Documentation
**Dependencies**: T037
**Effort**: 2h
**Files**: `specs/004-domain-ui-refactor/contracts/openapi.yaml` (new)

**Implementation**:
- Document all new domain endpoints
- Update connection endpoints with domain parameters
- Add query endpoints with domain scoping
- Include example requests/responses
- Generate Swagger UI

**Acceptance Criteria**:
- [ ] All endpoints documented
- [ ] Examples accurate and tested
- [ ] Swagger UI renders correctly

---

### T039: Create User Guide
**Dependencies**: T037
**Effort**: 2.5h
**Files**: `docs/USER_GUIDE_DOMAINS.md` (new)

**Implementation**:
- Write user guide covering:
  - What are domains and why use them
  - Creating and managing domains
  - Adding data sources to domains
  - Writing queries within domains
  - Switching between domains
  - Best practices
- Include screenshots

**Acceptance Criteria**:
- [ ] Guide covers all user stories
- [ ] Screenshots show AWS Cloudscape UI
- [ ] Step-by-step instructions clear
- [ ] User feedback positive

---

### T040: Create Migration Guide
**Dependencies**: T039
**Effort**: 2h
**Files**: `docs/MIGRATION_TO_DOMAINS.md` (new)

**Implementation**:
- Write migration guide for existing users:
  - Explain domain architecture changes
  - Provide migration script to assign connections to default domain
  - Document breaking changes in API
  - Provide rollback instructions
- Create migration script (Rust or Node.js)

**Acceptance Criteria**:
- [ ] Guide explains all breaking changes
- [ ] Migration script tested on sample data
- [ ] Rollback process documented
- [ ] User can migrate without data loss

---

### T041: Update Project README
**Dependencies**: T040
**Effort**: 1h
**Files**: `README.md`

**Implementation**:
- Add domain features to feature list
- Update screenshots to show AWS Cloudscape UI
- Add link to user guide
- Update architecture diagram
- Add migration notes

**Acceptance Criteria**:
- [ ] README reflects domain features
- [ ] Screenshots updated
- [ ] Links working
- [ ] Accurate feature description

---

### T042: Create Deployment Checklist
**Dependencies**: T041
**Effort**: 1h
**Files**: `specs/004-domain-ui-refactor/DEPLOYMENT_CHECKLIST.md` (new)

**Implementation**:
- Create checklist covering:
  - Database migration steps
  - Environment variable updates
  - Frontend build configuration
  - Rollback procedures
  - Post-deployment verification
- Include smoke test scenarios

**Acceptance Criteria**:
- [ ] Checklist complete and tested
- [ ] Smoke tests defined
- [ ] Rollback plan documented

---

## Summary

**Total Tasks**: 42
**Estimated Total Effort**: 70-75 hours (2-3 weeks for 1 developer)

### Phase Breakdown

| Phase | Tasks | Effort | Focus |
|-------|-------|--------|-------|
| Phase 1: Backend Domain Management | T001-T006 | 8.5h | Domain CRUD, storage, API |
| Phase 2: Frontend Domain UI | T007-T012 | 8.5h | Domain context, sidebar, forms |
| Phase 3: Backend Data Source Scoping | T013-T016 | 5h | Connection domain isolation |
| Phase 4: Frontend Data Source UI | T017-T020 | 4.5h | Connection filtering UI |
| Phase 5: Backend Query Scoping | T021-T024 | 7h | Saved queries, history |
| Phase 6: Frontend Query UI | T025-T028 | 8.5h | Saved queries, history UI |
| Phase 7: AWS Cloudscape Styling | T029-T032 | 8.5h | Theme, component migration |
| Phase 8: Testing & QA | T033-T037 | 9h | E2E, performance, security |
| Phase 9: Documentation | T038-T042 | 9.5h | Docs, migration, deployment |

### Critical Path

The critical path for MVP (minimum viable product) is:

**P1 Domain Management**: T001 → T002 → T003 → T004 → T005 → T009 → T010 → T012
**P2 Data Source Scoping**: T013 → T014 → T015 → T018 → T019
**P3 Query Scoping**: T021 → T022 → T023 → T027 → T028
**AWS Styling**: T029 → T030 → T031 → T032
**Testing**: T033 → T034 → T037

**Critical Path Duration**: ~45-50 hours

### Parallel Work Opportunities

- T007-T008 (frontend types/services) can run parallel to T003-T005 (backend)
- T017 (frontend types) can run parallel to T014-T015 (backend)
- T025-T026 (saved queries/history UI) can run parallel to T023-T024 (backend)
- T029-T030 (Cloudscape setup) can start early in parallel
- T038-T041 (documentation) can run parallel to T037 (security audit)

### Risk Mitigation

**High Risk**:
- T031: Cloudscape component migration (4h) - May uncover styling issues
- T033: E2E workflow test (2h) - May find integration bugs

**Mitigation**: Allocate 20% buffer time for re-work (15h total)

---

**Next Step**: Review task breakdown with team, assign ownership, and begin Phase 1 (T001-T006).
