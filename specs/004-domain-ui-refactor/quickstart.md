# Quickstart Guide: Domain-Based UI Refactoring

**Feature**: 004-domain-ui-refactor
**Date**: 2025-12-28
**Audience**: End users, QA testers, product managers

This guide walks you through the domain-based workflow in the database query tool, from creating your first domain to executing queries in an organized, isolated environment.

---

## Table of Contents

1. [What are Domains?](#what-are-domains)
2. [Creating Your First Domain](#creating-your-first-domain)
3. [Adding a Data Source to a Domain](#adding-a-data-source-to-a-domain)
4. [Executing Queries in a Domain](#executing-queries-in-a-domain)
5. [Saving and Reusing Queries](#saving-and-reusing-queries)
6. [Switching Between Domains](#switching-between-domains)
7. [Managing Query History](#managing-query-history)
8. [Deleting Domains and Data Sources](#deleting-domains-and-data-sources)
9. [Common Workflows](#common-workflows)
10. [Troubleshooting](#troubleshooting)

---

## What are Domains?

Domains are organizational units that group your database connections, saved queries, and query history. Think of them as isolated workspaces for different environments or projects:

- **Production** - Connections to your live production databases
- **Development** - Development and staging database connections
- **Analytics** - Read-only analytics databases
- **Testing** - Test environment databases

**Key Benefits**:
- **Complete Isolation**: Data sources and queries in one domain are invisible to other domains
- **Organization**: No more mixing production and development connections
- **Safety**: Switching domains automatically cancels any running queries from the previous domain
- **Persistence**: Your active domain selection is saved across browser sessions

---

## Creating Your First Domain

### Step 1: Open the Domain Sidebar

When you first open the application (or after upgrading), you'll see the new domain sidebar on the left side of the screen with AWS Cloudscape styling.

**Initial State**: If no domains exist, you'll see a "Create Domain" button.

### Step 2: Click "Create Domain"

A modal dialog will appear with the following fields:

| Field | Required | Description |
|-------|----------|-------------|
| **Name** | Yes | Domain name (1-50 characters, alphanumeric with spaces/hyphens/underscores) |
| **Description** | No | Optional description (up to 500 characters) |

### Step 3: Fill in Domain Details

**Example**:
```
Name: Production
Description: Production database connections for customer-facing systems
```

### Step 4: Submit

Click the **"Create"** button. You'll see:
- A success notification
- The new domain appears in the sidebar
- The domain is automatically selected as your active domain
- Resource counts (0 connections, 0 queries, 0 history) are displayed

**Screenshot Reference** (Visual Guide):
```
┌─────────────────────────────────────────┐
│ Domain Sidebar                          │
├─────────────────────────────────────────┤
│ [+ Create Domain]                       │
│                                         │
│ ┌─────────────────────────────────────┐ │
│ │ ● Production              [Active] │ │
│ │   Production database connections   │ │
│ │   📊 0 connections | 0 queries      │ │
│ └─────────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

---

## Adding a Data Source to a Domain

### Step 1: Navigate to Data Sources

With your domain selected (Production), navigate to the **"Data Sources"** section in the main content area.

**Initial State**: You'll see an empty list with an **"Add Data Source"** button.

### Step 2: Click "Add Data Source"

A connection form will appear with the domain pre-selected.

### Step 3: Fill in Connection Details

**PostgreSQL Example**:
```
Domain: Production (read-only, pre-selected)
Name: Customer Database
Type: PostgreSQL
Connection URL: postgresql://user:password@prod-db.example.com:5432/customers
```

**MySQL Example**:
```
Domain: Production (read-only, pre-selected)
Name: Orders Database
Type: MySQL
Connection URL: mysql://user:password@prod-mysql.example.com:3306/orders
```

### Step 4: Test Connection

Click **"Test Connection"** to verify the connection details.

**Success**: You'll see a green checkmark with "Connection successful"
**Failure**: An error message will explain the issue (e.g., "Connection refused", "Invalid credentials")

### Step 5: Save Connection

Click **"Save"** to add the data source to your domain.

**Result**:
- Connection appears in the data sources list
- Connection status shows "connected"
- Metadata (tables, columns) is loaded automatically
- Resource count updates in the domain sidebar (1 connection)

**Screenshot Reference**:
```
┌─────────────────────────────────────────────────────────────┐
│ Data Sources (Production Domain)                            │
├─────────────────────────────────────────────────────────────┤
│ [+ Add Data Source]                                         │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ ✓ Customer Database                        [Connected]  │ │
│ │   PostgreSQL | prod-db.example.com                      │ │
│ │   📊 12 tables | 87 columns                             │ │
│ │   [View Metadata] [Edit] [Delete]                       │ │
│ └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Step 6: View Metadata

Click **"View Metadata"** to see the database schema:
- Tables and views
- Column names and data types
- Primary keys and foreign keys (if available)

---

## Executing Queries in a Domain

### Step 1: Navigate to Query Editor

Click **"Query Editor"** in the navigation menu.

**UI Elements**:
- **Left Sidebar**: Domain list (always visible)
- **Center Panel**: SQL editor (Monaco Editor with Cloudscape theme)
- **Right Panel**: Data source selector + query results

### Step 2: Select a Data Source

In the right panel, select a data source from the dropdown:
```
Data Source: Customer Database (PostgreSQL)
```

**Visibility**: Only data sources from the **active domain** (Production) are shown.

### Step 3: Write Your SQL Query

Use the Monaco Editor in the center panel with syntax highlighting:

```sql
SELECT
    customer_id,
    email,
    created_at
FROM customers
WHERE status = 'active'
ORDER BY created_at DESC
```

**Features**:
- SQL syntax highlighting (PostgreSQL/MySQL)
- Auto-completion (basic keywords)
- Line numbers
- Keyboard shortcuts (Cmd/Ctrl+Enter to execute)

### Step 4: Execute Query

Click **"Execute"** or press **Cmd/Ctrl+Enter**.

**Loading State**:
- Spinner appears within 200ms
- If query takes >3 seconds, spinner changes to progress bar

**Result**:
- Query results appear in a table below the editor
- Execution time and row count displayed (e.g., "234ms, 42 rows")
- Export buttons available (CSV, JSON)

**Screenshot Reference**:
```
┌─────────────────────────────────────────────────────────────┐
│ Query Editor                                                │
├─────────────────────────────────────────────────────────────┤
│ 1  SELECT                                                   │
│ 2      customer_id,                                         │
│ 3      email,                                               │
│ 4      created_at                                           │
│ 5  FROM customers                                           │
│ 6  WHERE status = 'active'                                  │
│                                                             │
│ [Execute (Cmd+Enter)] [Save Query]                         │
├─────────────────────────────────────────────────────────────┤
│ Results (234ms, 42 rows) [Export CSV] [Export JSON]        │
├─────────────────────────────────────────────────────────────┤
│ customer_id │ email              │ created_at              │
│─────────────┼────────────────────┼─────────────────────────│
│ 1001        │ alice@example.com  │ 2025-12-15 10:30:00    │
│ 1002        │ bob@example.com    │ 2025-12-20 14:22:00    │
│ ...         │ ...                │ ...                    │
└─────────────────────────────────────────────────────────────┘
```

### Security Note

All queries are validated before execution:
- ✅ **Allowed**: SELECT statements only
- ❌ **Blocked**: INSERT, UPDATE, DELETE, DROP, CREATE, ALTER, etc.
- 🔒 **Auto-LIMIT**: If your query doesn't have a LIMIT clause, the system automatically adds `LIMIT 1000`

**Example**:
```sql
-- Your query
SELECT * FROM customers WHERE status = 'active'

-- Executed as
SELECT * FROM customers WHERE status = 'active' LIMIT 1000
```

---

## Saving and Reusing Queries

### Step 1: Execute a Query

After executing a query successfully, you'll see a **"Save Query"** button.

### Step 2: Click "Save Query"

A dialog appears:
```
Query Name: Active Customers
Domain: Production (read-only)
Data Source: Customer Database (read-only)
SQL: [Your query text, read-only]
```

### Step 3: Enter Query Name

Choose a descriptive name (unique within the domain):
```
Query Name: Active Customers by Creation Date
```

### Step 4: Save

Click **"Save"** to store the query.

**Result**:
- Query appears in the "Saved Queries" panel
- Resource count updates in domain sidebar (1 saved query)
- Query can be loaded with a single click

### Step 5: Load a Saved Query

**Option A**: Click on the saved query in the "Saved Queries" panel
**Option B**: Use the saved queries dropdown in the query editor

**Result**:
- SQL text is loaded into the editor
- Data source is automatically selected
- You can edit and re-execute immediately

**Screenshot Reference**:
```
┌─────────────────────────────────────────┐
│ Saved Queries (Production)              │
├─────────────────────────────────────────┤
│ ┌─────────────────────────────────────┐ │
│ │ 📝 Active Customers by Creation    │ │
│ │    Customer Database (PostgreSQL)   │ │
│ │    Created: Dec 28, 2025 10:40 AM  │ │
│ │    [Load] [Edit] [Delete]          │ │
│ └─────────────────────────────────────┘ │
│                                         │
│ ┌─────────────────────────────────────┐ │
│ │ 📝 Top 10 Paying Customers         │ │
│ │    Customer Database (PostgreSQL)   │ │
│ │    Created: Dec 28, 2025 11:15 AM  │ │
│ │    [Load] [Edit] [Delete]          │ │
│ └─────────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

---

## Switching Between Domains

### Scenario: You've created multiple domains

```
Domains:
- Production (active)
- Development
- Analytics
```

### Step 1: Click on a Different Domain

In the domain sidebar, click **"Development"**.

**What Happens**:
1. **Query Cancellation**: If a query is running in Production, it's automatically cancelled
2. **Notification**: "Query cancelled due to domain switch" appears briefly
3. **Context Switch**: The UI updates to show Development domain resources
4. **Persistence**: Active domain is saved to localStorage

### Step 2: Verify Context Switch

**Changes You'll See**:
- Domain sidebar highlights "Development" as active
- Data sources list shows only Development connections
- Saved queries show only Development queries
- Query history shows only Development executions

**Screenshot Reference**:
```
Before Switch (Production):
┌────────────────────────┐
│ ● Production  [Active] │
│ ○ Development          │
│ ○ Analytics            │
└────────────────────────┘

After Switch (Development):
┌────────────────────────┐
│ ○ Production           │
│ ● Development [Active] │
│ ○ Analytics            │
└────────────────────────┘
```

### Step 3: Confirm Isolation

**Test**: Try to access a Production data source from the Development domain.

**Result**: ❌ **Not possible** - Data sources, queries, and history are 100% isolated by domain.

---

## Managing Query History

### Viewing History

Navigate to **"Query History"** in the navigation menu.

**What You See**:
- All queries executed in the **active domain**
- Execution timestamp
- SQL text
- Execution time and row count
- Status: Success, Error, or Cancelled

**Screenshot Reference**:
```
┌─────────────────────────────────────────────────────────────┐
│ Query History (Production Domain)                           │
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ ✅ Success | Dec 28, 2025 10:45 AM | 234ms | 42 rows   │ │
│ │    SELECT * FROM users WHERE status = 'active' LIMIT... │ │
│ │    Customer Database (PostgreSQL)                       │ │
│ │    [Re-run] [Save as Query]                            │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ ❌ Cancelled | Dec 28, 2025 10:50 AM | 0ms | 0 rows    │ │
│ │    SELECT COUNT(*) FROM orders                          │ │
│ │    Customer Database (PostgreSQL)                       │ │
│ │    Error: User switched domains during execution        │ │
│ └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Re-running a Query from History

Click **"Re-run"** on any history entry to:
1. Load the SQL into the query editor
2. Select the original data source
3. Execute immediately (or edit first)

### Saving a History Entry as a Query

Click **"Save as Query"** to convert a history entry into a saved query.

---

## Deleting Domains and Data Sources

### Deleting a Data Source

**Step 1**: Navigate to Data Sources
**Step 2**: Click **"Delete"** on a connection
**Step 3**: Confirm deletion

**What Happens**:
- ✅ Data source is deleted
- ⚠️ **Saved queries referencing this connection are NOT deleted** (they become "orphaned" but remain accessible)
- ⚠️ **Query history entries are NOT deleted** (historical record is preserved)

### Deleting a Domain (CASCADE)

**Step 1**: Click on a domain in the sidebar
**Step 2**: Click the **"Delete Domain"** button (trash icon)
**Step 3**: Review the confirmation dialog

**Confirmation Dialog Shows**:
```
⚠️ Delete Domain "Production"?

This action will permanently delete:
- 3 database connections
- 5 saved queries
- 42 query history entries

This action cannot be undone.

[Cancel] [Delete Domain]
```

**Step 4**: Confirm deletion

**What Happens** (CASCADE DELETE):
- ✅ Domain is deleted
- ✅ All connections in the domain are deleted
- ✅ All saved queries in the domain are deleted
- ✅ All query history in the domain is deleted
- ✅ If this was the active domain, the first remaining domain is selected automatically

**Safety**: The confirmation dialog prevents accidental deletions by showing exact resource counts.

---

## Common Workflows

### Workflow 1: Setting Up Multiple Environments

**Goal**: Separate production, development, and analytics databases

1. Create three domains:
   - Production
   - Development
   - Analytics

2. Add connections to each domain:
   - **Production**: prod-db (PostgreSQL), prod-mysql (MySQL)
   - **Development**: dev-db (PostgreSQL), dev-mysql (MySQL)
   - **Analytics**: analytics-db (PostgreSQL, read-only)

3. Create saved queries in each domain:
   - **Production**: Customer health checks, order summaries
   - **Development**: Test data queries, schema exploration
   - **Analytics**: Business metrics, reporting queries

4. Switch between domains as needed:
   - Use Production domain for monitoring and debugging production issues
   - Use Development domain for testing new queries
   - Use Analytics domain for business reporting

**Benefit**: Zero risk of running a development query against production databases.

---

### Workflow 2: Collaborative Team Workflow

**Goal**: Multiple team members working on different projects

1. Each team member creates domains for their projects:
   - "Project Alpha - Production"
   - "Project Alpha - Development"
   - "Project Beta - Production"
   - "Project Beta - Development"

2. Team members share saved queries by exporting and importing (future feature)

3. Domain context is saved per-browser, so switching browsers or clearing storage resets to the first domain

**Benefit**: Clear separation of work contexts.

---

### Workflow 3: Auditing and Compliance

**Goal**: Track all queries executed against production databases

1. Create a **"Production Audit"** domain

2. Add all production data sources to this domain (read-only connections)

3. Execute all production queries through this domain

4. Review query history for compliance audits:
   - Who executed what query?
   - When was it executed?
   - What was the result?

**Benefit**: Complete audit trail scoped to production environment.

---

## Troubleshooting

### Issue: "Domain not found" Error

**Symptom**: Clicking on a domain shows "Domain not found"

**Cause**: Domain was deleted while you had it selected

**Solution**:
1. Refresh the page
2. Select a different domain from the sidebar

---

### Issue: Data Sources Not Appearing

**Symptom**: Data sources list is empty

**Cause**: You're viewing a different domain than expected

**Solution**:
1. Check which domain is active in the sidebar (highlighted with ●)
2. Switch to the correct domain
3. Verify data sources appear

---

### Issue: Saved Query Won't Load

**Symptom**: Clicking a saved query doesn't load it into the editor

**Cause**: Associated data source was deleted (orphaned query)

**Solution**:
1. The query is still accessible (SQL text is preserved)
2. Manually select a different data source
3. Copy the SQL text and create a new saved query

---

### Issue: Query Cancelled Unexpectedly

**Symptom**: "Query cancelled due to domain switch" notification

**Cause**: You or another browser tab switched domains while the query was running

**Solution**:
1. This is expected behavior (safety feature)
2. Switch back to the original domain
3. Re-execute the query

---

### Issue: Can't Delete Domain

**Symptom**: Delete button is disabled or grayed out

**Cause**: This is the last remaining domain (you must have at least one domain)

**Solution**:
1. Create a new domain first
2. Then delete the unwanted domain

---

## Next Steps

**After completing this quickstart**, you should be able to:

- ✅ Create and manage domains
- ✅ Add data sources scoped to domains
- ✅ Execute queries within a domain context
- ✅ Save and reuse queries
- ✅ Switch between domains safely
- ✅ Review query history
- ✅ Delete domains and data sources with confidence

**Advanced Topics** (See Full Documentation):
- Domain naming conventions and best practices
- Performance optimization for large result sets
- Natural language query generation (existing feature)
- Metadata caching and refresh strategies
- Cross-browser domain synchronization (future feature)
- Domain import/export (future feature)

---

**End of Quickstart Guide**
