# Implementation Execution Plan
## Feature 004: Domain-Based UI Refactoring

**Generated**: 2025-12-28
**Status**: Ready for Execution
**Total Effort**: 70-75 hours (2-3 weeks for 1 developer)
**Total Tasks**: 42

---

## Executive Summary

This document provides a detailed execution plan for implementing the domain-based UI refactoring feature. The implementation follows a strict phase-by-phase approach with dependency management and parallel execution opportunities.

### Key Outcomes
- ✅ Domain-based organization of database connections
- ✅ 100% data isolation between domains
- ✅ AWS Cloudscape design system integration
- ✅ Saved queries and query history scoped to domains
- ✅ Complete migration path for existing users

---

## Implementation Strategy

### 1. Phase-Based Execution

The implementation follows 9 distinct phases:

1. **Phase 1**: Backend - Domain Management (8.5h)
2. **Phase 2**: Frontend - Domain Management UI (8.5h)
3. **Phase 3**: Backend - Domain-Scoped Data Sources (5h)
4. **Phase 4**: Frontend - Domain-Scoped Data Sources UI (4.5h)
5. **Phase 5**: Backend - Domain-Scoped Queries (7h)
6. **Phase 6**: Frontend - Domain-Scoped Query UI (8.5h)
7. **Phase 7**: UI Styling - AWS Cloudscape Design (8.5h)
8. **Phase 8**: Testing & Quality Assurance (9h)
9. **Phase 9**: Documentation & Deployment (9.5h)

### 2. Critical Path

**MVP Critical Path** (45-50 hours):
```
T001 → T002 → T003 → T004 → T005 → T009 → T010 → T012
  ↓
T013 → T014 → T015 → T018 → T019
  ↓
T021 → T022 → T023 → T027 → T028
  ↓
T029 → T030 → T031 → T032
  ↓
T033 → T034 → T037
```

### 3. Parallel Execution Opportunities

**Backend-Frontend Parallelization**:
- T007-T008 (frontend types/services) || T003-T005 (backend domain API)
- T017 (frontend types) || T014-T015 (backend connection scoping)
- T025-T026 (query UI) || T023-T024 (query backend)

**Early Preparation**:
- T029-T030 (Cloudscape setup) can start in Phase 1-2

**Documentation**:
- T038-T041 (docs) || T037 (security audit)

---

## Phase-by-Phase Execution Details

## PHASE 1: Backend - Domain Management (8.5 hours)

### Pre-Phase Checklist
- [x] Git repository is clean (no uncommitted changes)
- [ ] Branch `004-domain-ui-refactor` created and checked out
- [ ] Backend dependencies up to date (`cargo update`)
- [ ] Backend compiles without errors (`cargo check`)

### Task Execution Flow

#### T001: Create Domain Data Model (1h)
**Status**: ⏳ Pending
**Dependencies**: None
**Execution Steps**:
1. Create file `backend/src/models/domain.rs`
2. Define `Domain` struct with fields (id, name, description, created_at, updated_at)
3. Implement Serialize/Deserialize traits (serde)
4. Add validation functions (name 1-50 chars, alphanumeric + spaces/hyphens/underscores)
5. Add `pub mod domain;` to `backend/src/models/mod.rs`
6. Run `cargo check` to verify compilation

**Validation**:
```bash
cargo check --lib
cargo test --lib models::domain --  --nocapture
```

**Expected Output**:
- ✅ Domain model compiles
- ✅ Validation prevents empty/long names
- ✅ JSON serialization works

---

#### T002: Update Database Schema for Domains (1.5h)
**Status**: ⏳ Pending
**Dependencies**: T001 ✅
**Execution Steps**:
1. Open `backend/src/storage/sqlite.rs`
2. Add `domains` table creation in `initialize_storage()`:
   ```sql
   CREATE TABLE IF NOT EXISTS domains (
       id TEXT PRIMARY KEY,
       name TEXT UNIQUE NOT NULL,
       description TEXT,
       created_at TEXT NOT NULL,
       updated_at TEXT NOT NULL
   )
   ```
3. Add migration logic to create default domain for existing connections:
   ```sql
   INSERT OR IGNORE INTO domains (id, name, description, created_at, updated_at)
   VALUES ('default-domain-id', 'Default Domain', 'Auto-created for existing connections', datetime('now'), datetime('now'));
   ```
4. Add `domain_id` column to `connections` table:
   ```sql
   ALTER TABLE connections ADD COLUMN domain_id TEXT REFERENCES domains(id) ON DELETE CASCADE;
   UPDATE connections SET domain_id = 'default-domain-id' WHERE domain_id IS NULL;
   ```
5. Enable foreign key enforcement:
   ```sql
   PRAGMA foreign_keys = ON;
   ```
6. Create composite index for performance:
   ```sql
   CREATE INDEX IF NOT EXISTS idx_connections_domain_created
       ON connections(domain_id, created_at DESC);
   ```

**Validation**:
```bash
rm backend/metadata.db  # Fresh start for testing
cargo run  # Should auto-migrate
sqlite3 backend/metadata.db "SELECT name FROM sqlite_master WHERE type='table';"
# Should show: domains, connections, ...
```

**Expected Output**:
- ✅ `domains` table exists
- ✅ `connections.domain_id` column exists
- ✅ Foreign key constraints enforced
- ✅ Default domain created

---

#### T003: Implement Domain Storage Service (2h)
**Status**: ⏳ Pending
**Dependencies**: T002 ✅
**Execution Steps**:
1. Add `DomainUpdate` struct to `backend/src/models/domain.rs`
2. Implement CRUD methods in `SqliteStorage`:
   ```rust
   pub async fn create_domain(&self, domain: &Domain) -> Result<(), AppError>
   pub async fn get_domain(&self, id: &str) -> Result<Option<Domain>, AppError>
   pub async fn list_domains(&self) -> Result<Vec<Domain>, AppError>
   pub async fn update_domain(&self, id: &str, updates: DomainUpdate) -> Result<(), AppError>
   pub async fn delete_domain(&self, id: &str) -> Result<(), AppError>
   ```
3. Implement CASCADE delete logic (SQLite handles via foreign key)
4. Add transaction support for atomic operations
5. Add unique name constraint enforcement
6. Compute resource counts (connections, queries) in `list_domains()`

**Validation**:
```bash
cargo test --lib storage::sqlite::test_domain_crud
cargo test --lib storage::sqlite::test_domain_cascade_delete
```

**Expected Output**:
- ✅ CRUD operations work
- ✅ CASCADE delete removes associated resources
- ✅ Transactions rollback on error

---

#### T004: Create Domain API Handlers (2h)
**Status**: ⏳ Pending
**Dependencies**: T003 ✅
**Execution Steps**:
1. Create file `backend/src/api/handlers/domain.rs`
2. Define request/response types:
   ```rust
   #[derive(Deserialize)]
   pub struct CreateDomainRequest {
       pub name: String,
       pub description: Option<String>,
   }

   #[derive(Serialize)]
   pub struct DomainResponse {
       pub id: String,
       pub name: String,
       pub description: Option<String>,
       pub created_at: String,
       pub updated_at: String,
       pub connection_count: usize,
       pub saved_query_count: usize,
   }
   ```
3. Implement handler functions:
   - `create_domain` (POST /api/domains)
   - `list_domains` (GET /api/domains)
   - `get_domain` (GET /api/domains/:id)
   - `update_domain` (PUT /api/domains/:id)
   - `delete_domain` (DELETE /api/domains/:id)
4. Add error handling (AppError::DomainNotFound, AppError::DuplicateDomainName)
5. Return resource counts with domain list

**Validation**:
```bash
cargo test --lib api::handlers::domain
```

**Expected Output**:
- ✅ All endpoints return correct HTTP status codes (200, 201, 204, 404, 409)
- ✅ Domain list includes resource counts
- ✅ Error responses actionable

---

#### T005: Register Domain Routes (0.5h)
**Status**: ⏳ Pending
**Dependencies**: T004 ✅
**Execution Steps**:
1. Add `pub mod domain;` to `backend/src/api/handlers/mod.rs`
2. Register routes in `backend/src/api/routes.rs`:
   ```rust
   .route("/api/domains",
       get(handlers::domain::list_domains)
       .post(handlers::domain::create_domain))
   .route("/api/domains/:id",
       get(handlers::domain::get_domain)
       .put(handlers::domain::update_domain)
       .delete(handlers::domain::delete_domain))
   ```
3. Verify CORS allows domain routes

**Validation**:
```bash
cargo run &
curl http://localhost:3000/api/domains
# Should return [] or default domain
```

**Expected Output**:
- ✅ Routes accessible via HTTP
- ✅ Correct HTTP methods
- ✅ CORS configured

---

#### T006: Add Domain Tests (1.5h)
**Status**: ⏳ Pending
**Dependencies**: T005 ✅
**Execution Steps**:
1. Add unit tests in `backend/src/storage/sqlite.rs`:
   ```rust
   #[cfg(test)]
   mod tests {
       #[tokio::test]
       async fn test_create_domain() { ... }

       #[tokio::test]
       async fn test_unique_domain_name() { ... }

       #[tokio::test]
       async fn test_cascade_delete_domain() { ... }
   }
   ```
2. Add integration tests in `backend/src/api/handlers/domain.rs`:
   ```rust
   #[cfg(test)]
   mod tests {
       #[tokio::test]
       async fn test_domain_creation_flow() { ... }

       #[tokio::test]
       async fn test_duplicate_name_rejection() { ... }
   }
   ```
3. Run full test suite

**Validation**:
```bash
cargo test --lib
cargo test --lib domain -- --nocapture
```

**Expected Output**:
- ✅ All tests pass
- ✅ Test coverage > 80% for domain module
- ✅ Edge cases covered

---

### Phase 1 Completion Criteria
- [ ] All 6 tasks (T001-T006) completed
- [ ] Backend compiles without errors
- [ ] All tests pass
- [ ] Domain CRUD API endpoints working
- [ ] Default domain created for existing connections

**Estimated Time**: 8.5 hours
**Actual Time**: ___ hours
**Blockers**: ___

---

## PHASE 2: Frontend - Domain Management UI (8.5 hours)

### Pre-Phase Checklist
- [ ] Phase 1 completed (backend domain API functional)
- [ ] Frontend dependencies up to date (`npm install`)
- [ ] Frontend dev server running (`npm run dev`)
- [ ] Backend server running (port 3000)

### Task Execution Flow

#### T007: Create Domain Types (0.5h)
**Status**: ⏳ Pending
**Dependencies**: None
**Execution Steps**:
1. Create file `frontend/src/types/domain.ts`
2. Define TypeScript interfaces matching backend models:
   ```typescript
   export interface Domain {
     id: string;
     name: string;
     description?: string;
     created_at: string;
     updated_at: string;
     connection_count?: number;
     saved_query_count?: number;
     query_history_count?: number;
   }

   export interface CreateDomainRequest {
     name: string;
     description?: string;
   }

   export interface UpdateDomainRequest {
     name?: string;
     description?: string;
   }
   ```
3. Export from `frontend/src/types/index.ts`

**Validation**:
```bash
npm run build
# Should compile without TypeScript errors
```

---

#### T008: Create Domain API Service (1h)
**Status**: ⏳ Pending
**Dependencies**: T007 ✅
**Execution Steps**:
1. Create file `frontend/src/services/domain.ts`
2. Implement API client functions:
   ```typescript
   export const domainService = {
     listDomains: (): Promise<Domain[]> =>
       api.get('/domains').then(res => res.data),

     getDomain: (id: string): Promise<Domain> =>
       api.get(`/domains/${id}`).then(res => res.data),

     createDomain: (request: CreateDomainRequest): Promise<Domain> =>
       api.post('/domains', request).then(res => res.data),

     updateDomain: (id: string, updates: UpdateDomainRequest): Promise<Domain> =>
       api.put(`/domains/${id}`, updates).then(res => res.data),

     deleteDomain: (id: string): Promise<void> =>
       api.delete(`/domains/${id}`).then(() => undefined),
   };
   ```
3. Add error handling

**Validation**:
```bash
# In browser console
import { domainService } from './services/domain';
await domainService.listDomains();
# Should return domains from backend
```

---

#### T009: Create Domain Context (1.5h)
**Status**: ⏳ Pending
**Dependencies**: T008 ✅
**Execution Steps**:
1. Create file `frontend/src/contexts/DomainContext.tsx`
2. Implement React Context with useSyncExternalStore:
   ```typescript
   const STORAGE_KEY = 'active_domain_id';

   const subscribe = (callback: () => void) => {
     const handleStorageChange = (e: StorageEvent) => {
       if (e.key === STORAGE_KEY) callback();
     };
     window.addEventListener('storage', handleStorageChange);
     return () => window.removeEventListener('storage', handleStorageChange);
   };

   export function DomainProvider({ children }: { children: React.ReactNode }) {
     const activeDomainId = useSyncExternalStore(subscribe,
       () => localStorage.getItem(STORAGE_KEY),
       () => null
     );
     // ... context implementation
   }
   ```
3. Implement domain state management (create, delete, set active)
4. Add localStorage persistence

**Validation**:
```bash
# Test in browser DevTools
localStorage.setItem('active_domain_id', 'test-uuid');
# Refresh page - context should restore active domain
```

---

#### T010: Create Domain Sidebar Component (2.5h)
**Status**: ⏳ Pending
**Dependencies**: T009 ✅
**Execution Steps**:
1. Create file `frontend/src/components/DomainSidebar/index.tsx`
2. Implement sidebar with Ant Design components:
   ```tsx
   export function DomainSidebar() {
     const { domains, activeDomain, setActiveDomain, createDomain, deleteDomain } = useDomain();

     return (
       <Sider width={250} style={{ background: '#f0f2f5' }}>
         <Button onClick={() => setShowCreateModal(true)}>Create Domain</Button>
         <Menu
           selectedKeys={[activeDomain?.id || '']}
           items={domains.map(domain => ({
             key: domain.id,
             label: (
               <div>
                 <div>{domain.name}</div>
                 <small>{domain.connection_count} connections</small>
               </div>
             ),
             onClick: () => setActiveDomain(domain)
           }))}
         />
       </Sider>
     );
   }
   ```
3. Add context menu for edit/delete
4. Style to match AWS Cloudscape aesthetic (prepare for T031)

**Validation**:
```bash
npm run dev
# Visit http://localhost:5173
# Sidebar should show domains, allow create/delete
```

---

#### T011: Create Domain Modal Forms (2h)
**Status**: ⏳ Pending
**Dependencies**: T009 ✅
**Execution Steps**:
1. Create file `frontend/src/components/DomainModal/index.tsx`
2. Implement create/edit modal:
   ```tsx
   export function DomainModal({ visible, mode, domain, onClose, onSave }) {
     const [form] = Form.useForm();

     const handleSubmit = async (values) => {
       if (mode === 'create') {
         await createDomain(values);
       } else {
         await updateDomain(domain.id, values);
       }
       onClose();
     };

     return (
       <Modal visible={visible} onCancel={onClose} onOk={() => form.submit()}>
         <Form form={form} onFinish={handleSubmit}>
           <Form.Item name="name" rules={[{ required: true, max: 50 }]}>
             <Input placeholder="Domain Name" />
           </Form.Item>
           <Form.Item name="description">
             <TextArea placeholder="Description (optional)" />
           </Form.Item>
         </Form>
       </Modal>
     );
   }
   ```
3. Add validation (name 1-50 chars)
4. Show loading state during save

**Validation**:
- ✅ Form validates correctly
- ✅ Create mode creates domain
- ✅ Edit mode updates domain

---

#### T012: Integrate Domain Sidebar into Layout (1h)
**Status**: ⏳ Pending
**Dependencies**: T010 ✅
**Execution Steps**:
1. Update `frontend/src/App.tsx`:
   ```tsx
   function App() {
     return (
       <DomainProvider>
         <Refine dataProvider={dataProvider}>
           <Layout>
             <DomainSidebar />
             <Layout.Content>
               <Outlet />
             </Layout.Content>
           </Layout>
         </Refine>
       </DomainProvider>
     );
   }
   ```
2. Ensure three-panel layout (left: domains, center: content, right: optional)
3. Set minimum viewport width 1024px

**Validation**:
- ✅ Domain sidebar visible on all pages
- ✅ Layout responsive at 1024px+
- ✅ Domain context accessible globally

---

### Phase 2 Completion Criteria
- [ ] All 6 tasks (T007-T012) completed
- [ ] Frontend compiles without TypeScript errors
- [ ] Domain sidebar functional
- [ ] Can create/edit/delete domains via UI
- [ ] Active domain persists across page refresh

**Estimated Time**: 8.5 hours
**Actual Time**: ___ hours
**Blockers**: ___

---

## PHASE 3-9 Summary

Due to space constraints, here's a condensed overview of remaining phases:

### PHASE 3: Backend - Domain-Scoped Data Sources (5h)
- T013: Update Connection Model (add domain_id)
- T014: Update Connection Storage (domain filtering)
- T015: Update Connection API Handlers
- T016: Update Connection Tests

### PHASE 4: Frontend - Domain-Scoped Data Sources UI (4.5h)
- T017: Update Connection Types
- T018: Update Connection Service (domain filtering)
- T019: Update DatabaseConnection Component
- T020: Update MetadataViewer (domain scope)

### PHASE 5: Backend - Domain-Scoped Queries (7h)
- T021: Create SavedQuery & QueryHistory models
- T022: Create Query Storage (saved_queries, query_history tables)
- T023: Create Query API Handlers (save, history)
- T024: Update Query Service (domain validation, logging)

### PHASE 6: Frontend - Domain-Scoped Query UI (8.5h)
- T025: Create Saved Query Components (sidebar, modal)
- T026: Create Query History Component
- T027: Update QueryEditor (integrate saved queries)
- T028: Update QueryPage (domain integration)

### PHASE 7: UI Styling - AWS Cloudscape Design (8.5h)
- T029: Install Cloudscape Design System
- T030: Create Cloudscape Theme Wrapper
- T031: Refactor Components to Cloudscape
- T032: Update Layout to AWS Three-Panel Pattern

### PHASE 8: Testing & Quality Assurance (9h)
- T033: End-to-End Domain Workflow Test
- T034: Performance Testing (<5s switch, <2s load)
- T035: Accessibility Audit (WCAG 2.1 AA)
- T036: Cross-Browser Testing
- T037: Security Audit (100% isolation)

### PHASE 9: Documentation & Deployment (9.5h)
- T038: Update API Documentation (OpenAPI)
- T039: Create User Guide
- T040: Create Migration Guide
- T041: Update Project README
- T042: Create Deployment Checklist

---

## Risk Management

### High-Risk Tasks

1. **T031: Cloudscape Component Migration (4h)**
   - **Risk**: Unexpected styling issues, component incompatibilities
   - **Mitigation**: Prototype key components early, allocate buffer time

2. **T033: E2E Workflow Test (2h)**
   - **Risk**: Integration bugs surface late
   - **Mitigation**: Run smoke tests after each phase

3. **T002: Database Schema Migration (1.5h)**
   - **Risk**: Existing data corruption, foreign key issues
   - **Mitigation**: Backup database, test on copy first

### Buffer Time Allocation

- **Planned**: 70 hours
- **Buffer**: 15 hours (20%)
- **Total**: 85 hours
- **Justification**: Accounts for debugging, re-work, unexpected issues

---

## Success Metrics (Validation)

### Performance (SC-001, SC-004, SC-006)
```bash
# Measure domain switching time
console.time('domainSwitch');
setActiveDomain(newDomain);
console.timeEnd('domainSwitch');
# Target: <5 seconds

# Measure app load time
# Chrome DevTools → Performance → Record page load
# Target: <2 seconds

# Verify query performance unchanged
# Compare query execution times before/after implementation
```

### Isolation (SC-002, SC-007)
```bash
# Backend test
cargo test test_domain_isolation
# Should verify connections not accessible across domains

# Frontend test
# Create domain A with connection X
# Switch to domain B
# Verify connection X not visible in UI
```

### Visual Consistency (SC-003)
```bash
# Manual review
# Compare app screenshots to AWS Cloudscape screenshots
# Target: 90% visual similarity
```

---

## Rollback Plan

### Pre-Deployment Checklist
- [ ] Branch merged to main
- [ ] All tests passing
- [ ] Database backup created
- [ ] Rollback commit SHA documented

### Rollback Procedure
1. **Stop application**: `systemctl stop db-query-backend`
2. **Restore database**: `cp metadata.db.backup metadata.db`
3. **Revert code**: `git revert <merge-commit-sha>`
4. **Rebuild**: `cargo build --release && npm run build`
5. **Restart**: `systemctl start db-query-backend`
6. **Verify**: Run smoke tests

**Rollback Time Estimate**: 15 minutes

---

## Post-Implementation Checklist

### Validation
- [ ] All 42 tasks completed
- [ ] All acceptance criteria met
- [ ] All tests passing (unit, integration, E2E)
- [ ] Performance targets achieved
- [ ] Security audit passed (100% isolation)
- [ ] Accessibility compliant (WCAG 2.1 AA)
- [ ] Cross-browser testing complete

### Documentation
- [ ] API documentation updated
- [ ] User guide published
- [ ] Migration guide published
- [ ] README updated
- [ ] Deployment checklist verified

### Deployment
- [ ] Database migration tested on staging
- [ ] Frontend build optimized (bundle size)
- [ ] Environment variables configured
- [ ] Monitoring alerts configured
- [ ] Rollback plan tested

---

## Next Steps

1. **Review this plan** with the development team
2. **Assign task ownership** (if working in a team)
3. **Create tracking board** (GitHub Projects, Jira, etc.)
4. **Schedule daily standups** to track progress
5. **Begin Phase 1**: Execute T001-T006

**Ready to start?** Run the following:

```bash
# Ensure you're on the feature branch
git checkout 004-domain-ui-refactor

# Verify prerequisites
cargo check
npm run build
cargo test

# Start implementation
# Begin with T001: Create Domain Data Model
```

---

**End of Implementation Execution Plan**

