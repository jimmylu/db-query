# Feature Specification: Domain-Based UI Refactoring

**Feature Branch**: `004-domain-ui-refactor`
**Created**: 2025-12-27
**Status**: Draft
**Input**: User description: "重构前端UI，按照功能包含领域管理、数据源管理、sql查询编辑器。数据源和sql按照领域管理和隔离。UI风格按照Amazon Cloud的风格。"

## Clarifications

### Session 2025-12-28

- Q: AWS Cloudscape Integration Strategy - Should the implementation use @cloudscape-design/components library, custom CSS, or hybrid approach? → A: Use @cloudscape-design/components with selective customization (keep some Ant Design for Monaco integration)
- Q: Domain Deletion Behavior - What happens when user tries to delete a domain that contains data sources and saved queries? → A: CASCADE delete with confirmation dialog showing count of resources to be deleted
- Q: Domain ID Generation Strategy - What format should domain IDs use? → A: UUID v4 (random, universally unique)
- Q: Domain Switching During Query Execution - How does system handle switching domains while a query is executing? → A: Allow domain switch but cancel in-flight query with notification to user
- Q: Loading State Feedback Timing - What type of loading indicators should be used for different operation durations? → A: Spinner for <3s, progress for 3s+

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Domain Management (Priority: P1)

Users need to organize their database connections and queries into logical domains (e.g., "Production", "Development", "Analytics") to maintain separation of concerns and improve workflow organization.

**Why this priority**: Foundation for data isolation. Without domains, users cannot achieve proper separation between different environments or projects. This is the core architectural change that enables all other features.

**Independent Test**: Can be fully tested by creating a domain, verifying it persists, and confirming it appears in the domain list. Delivers immediate value by allowing users to categorize their work.

**Acceptance Scenarios**:

1. **Given** user is on the main dashboard, **When** user clicks "Create Domain", **Then** system displays domain creation form
2. **Given** domain creation form is open, **When** user enters domain name "Production" and optional description, **Then** system creates domain and displays it in the domain list
3. **Given** multiple domains exist, **When** user selects a domain from the list, **Then** system switches active context to that domain
4. **Given** user is viewing a domain, **When** user attempts to access resources from another domain, **Then** system prevents access and displays appropriate message
5. **Given** user has created a domain, **When** user refreshes the page, **Then** the selected domain context is preserved

---

### User Story 2 - Domain-Scoped Data Source Management (Priority: P2)

Users need to create and manage database connections within the scope of a specific domain, ensuring complete isolation between domains.

**Why this priority**: Builds on domain foundation (P1). Provides practical value by allowing users to connect to databases within their organized domains. Critical for multi-environment workflows.

**Independent Test**: After P1 is complete, can be tested by creating a domain, adding a data source to it, verifying the data source is only accessible within that domain, and confirming connections work. Delivers value by enabling users to connect to their databases in an organized manner.

**Acceptance Scenarios**:

1. **Given** user has selected an active domain, **When** user navigates to "Data Sources" section, **Then** system displays only data sources belonging to the current domain
2. **Given** user is viewing data sources in a domain, **When** user clicks "Add Data Source", **Then** system displays connection form with domain pre-selected
3. **Given** user enters valid connection details (URL, type, name), **When** user clicks "Test Connection", **Then** system validates connection and displays success/error message
4. **Given** connection test succeeds, **When** user clicks "Save", **Then** system saves data source to current domain only
5. **Given** user switches to different domain, **When** user views data sources, **Then** previously created data source is NOT visible
6. **Given** user has a saved data source, **When** user clicks on it, **Then** system displays metadata (tables, views, columns) for that data source

---

### User Story 3 - Domain-Scoped SQL Query Editor (Priority: P3)

Users need to write and execute SQL queries against data sources within the current domain, with query history and saved queries also scoped to the domain.

**Why this priority**: Completes the domain-isolated workflow. Depends on P1 (domains) and P2 (data sources). This is where users perform their actual work.

**Independent Test**: After P1 and P2 are complete, can be tested by selecting a domain, choosing a data source, writing a query, executing it, and verifying results are displayed and history is saved within the domain. Delivers value by providing the core query functionality.

**Acceptance Scenarios**:

1. **Given** user has selected a domain and data source, **When** user opens SQL editor, **Then** system displays query editor with syntax highlighting and domain/data source context visible
2. **Given** user has written a SQL query, **When** user clicks "Execute" or presses Cmd/Ctrl+Enter, **Then** system executes query against selected data source and displays results
3. **Given** query execution succeeds, **When** results are returned, **Then** system displays data in tabular format with export options (CSV, JSON)
4. **Given** user has executed queries, **When** user views query history, **Then** system shows only queries executed within current domain
5. **Given** user wants to reuse a query, **When** user clicks "Save Query" with a name, **Then** system saves query to current domain only
6. **Given** user switches domains, **When** user views saved queries or history, **Then** only queries from the new domain are visible

---

### Edge Cases

- ✅ **Resolved**: Domain deletion with resources - CASCADE delete removes all associated data sources, queries, and history with confirmation dialog showing counts
- ✅ **Resolved**: Domain switching during query execution - Allow switch but cancel in-flight query with user notification
- What happens if a data source connection fails during query execution?
- How does system handle very large query result sets (>10,000 rows)?
- What happens when user tries to create a domain with duplicate name?
- How does system handle concurrent modifications to the same data source from different browser tabs?

## Requirements *(mandatory)*

### Functional Requirements

#### Domain Management
- **FR-001**: System MUST allow users to create new domains with a unique name and optional description
- **FR-002**: System MUST display all domains in a navigable list in the left sidebar
- **FR-003**: System MUST maintain active domain context across page navigation within the application
- **FR-004**: System MUST persist selected domain in browser storage to restore context after page refresh
- **FR-005**: System MUST prevent access to resources (data sources, queries) from inactive domains
- **FR-006**: System MUST allow users to edit domain name and description
- **FR-007**: System MUST allow users to delete domains via CASCADE delete (all associated data sources, queries, and history are deleted) with confirmation dialog showing exact count of resources to be deleted
- **FR-031**: System MUST cancel in-flight queries when user switches domains and display notification to user about cancelled operation

#### Data Source Management
- **FR-008**: System MUST display data sources filtered by active domain only
- **FR-009**: System MUST associate each data source with exactly one domain upon creation
- **FR-010**: System MUST support creating connections to PostgreSQL and MySQL databases
- **FR-011**: System MUST validate connection details before saving (test connection functionality)
- **FR-012**: System MUST retrieve and display metadata (tables, views, columns) for each data source
- **FR-013**: System MUST allow users to edit data source connection details
- **FR-014**: System MUST allow users to delete data sources with confirmation
- **FR-015**: System MUST indicate connection status (connected/disconnected/error) for each data source

#### SQL Query Editor
- **FR-016**: System MUST provide SQL editor with syntax highlighting for PostgreSQL and MySQL
- **FR-017**: System MUST execute queries only against data sources in the active domain
- **FR-018**: System MUST display query results in tabular format with column headers
- **FR-019**: System MUST provide export functionality for results (CSV, JSON formats)
- **FR-020**: System MUST save query execution history scoped to active domain
- **FR-021**: System MUST allow users to save queries with names, scoped to active domain
- **FR-022**: System MUST provide keyboard shortcut (Cmd/Ctrl+Enter) for query execution
- **FR-023**: System MUST display query execution time and row count
- **FR-024**: System MUST enforce security validation (SELECT-only queries, auto-LIMIT)

#### UI/UX Requirements
- **FR-025**: System MUST use @cloudscape-design/components library for UI components with selective retention of Ant Design for Monaco Editor integration
- **FR-026**: System MUST use a three-panel layout: left sidebar (domain navigation), center (main content), right (context panel when needed)
- **FR-027**: System MUST provide clear visual indicators for active domain and data source
- **FR-028**: System MUST display loading states for async operations triggered within 200ms: indeterminate spinner for operations <3 seconds, progress bar for operations ≥3 seconds (connection tests, query execution, metadata loading)
- **FR-029**: System MUST display user-friendly error messages with actionable guidance
- **FR-030**: System MUST be responsive and support minimum viewport width of 1024px

### Key Entities

- **Domain**: Organizational unit for grouping related data sources and queries. Attributes: id (UUID v4), unique name (string, 1-50 chars), description (optional string), created_at (timestamp), updated_at (timestamp), resource counts (data sources, queries)
- **Data Source**: Database connection configuration scoped to a domain. Attributes: name, type (PostgreSQL/MySQL), connection URL, domain reference, connection status, metadata cache
- **Query**: SQL query text with execution context. Attributes: SQL text, name (for saved queries), domain reference, data source reference, execution timestamp, results metadata
- **Query History**: Record of executed queries within a domain. Attributes: query text, execution time, row count, timestamp, data source reference

### Relationships

- One Domain contains many Data Sources (1:N)
- One Domain contains many Queries (1:N)
- One Domain contains many Query History entries (1:N)
- Data Sources, Queries, and History are strictly isolated by Domain
- One Data Source can be used by many Queries (1:N)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can create and switch between domains in under 5 seconds per operation
- **SC-002**: Data source isolation is enforced - switching domains shows zero resources from other domains
- **SC-003**: UI matches Amazon Cloud design patterns with 90% visual consistency (assessed by design review)
- **SC-004**: Query execution maintains existing performance (same execution time as current implementation)
- **SC-005**: Users can successfully complete the full workflow (create domain → add data source → execute query) within 3 minutes on first attempt
- **SC-006**: Application loads and renders domain list within 2 seconds
- **SC-007**: No data leakage between domains - 100% isolation verified through testing
- **SC-008**: UI is fully responsive and functional at 1024px viewport width and above
- **SC-009**: All async operations (connection tests, queries) provide immediate visual feedback within 200ms

### User Experience Goals

- Reduce cognitive load by organizing connections into clear domains
- Eliminate accidental cross-environment queries through strict isolation
- Provide familiar UI patterns for users coming from AWS console
- Maintain or improve query execution workflow efficiency

## Assumptions

1. Users are familiar with domain/environment concepts (Production, Development, etc.)
2. Existing backend API supports or will be extended to support domain-scoped operations
3. Amazon Cloud design system implemented via @cloudscape-design/components npm package with selective Ant Design retention for existing Monaco Editor integration
4. Browser local storage is acceptable for persisting domain selection
5. Users primarily work with one domain at a time (no multi-domain query support needed in MVP)
6. Existing database metadata and query execution APIs can be reused with domain context added
7. Application will continue to use React, Ant Design can be replaced or customized to match AWS patterns

## Out of Scope

- Multi-domain query joins (queries spanning multiple domains)
- Domain-level permissions or user access control
- Domain import/export functionality
- Domain templates or cloning
- Cross-domain resource migration tools
- Real-time collaboration within domains
- Domain-level analytics or usage metrics
- Custom domain color themes or branding
