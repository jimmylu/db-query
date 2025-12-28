# Data Model: Domain-Based UI Refactoring

**Date**: 2025-12-28
**Feature**: 004-domain-ui-refactor
**Status**: Complete

This document defines the data entities and relationships for the domain-based UI refactoring feature. The feature introduces organizational domains as the primary grouping mechanism, with data sources, queries, and history scoped to each domain for complete isolation.

---

## Table of Contents

1. [Entities](#entities)
   - [Domain](#1-domain)
   - [Connection](#2-connection)
   - [SavedQuery](#3-savedquery)
   - [QueryHistory](#4-queryhistory)
2. [Relationships](#relationships)
3. [Validation Rules](#validation-rules)
4. [State Machines](#state-machines)
5. [Database Schema](#database-schema)
6. [API Request/Response Examples](#api-requestresponse-examples)

---

## Entities

### 1. Domain

Organizational unit for grouping related database connections and queries.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | TEXT | PRIMARY KEY | UUID v4 in hyphenated format (36 chars) |
| name | TEXT | UNIQUE, NOT NULL, 1-50 chars | User-friendly domain name |
| description | TEXT | OPTIONAL, max 500 chars | Optional domain description |
| created_at | TEXT | NOT NULL | ISO 8601 timestamp (UTC) |
| updated_at | TEXT | NOT NULL | ISO 8601 timestamp (UTC) |

**Computed Fields** (not stored in DB):
- `connection_count`: Number of connections in this domain
- `saved_query_count`: Number of saved queries in this domain
- `query_history_count`: Number of query history entries in this domain

**Business Rules**:
- Domain names must be unique across all domains
- Domain names cannot be empty or whitespace-only
- Deleting a domain CASCADE deletes all associated connections, saved queries, and query history
- Domain deletion requires user confirmation with resource counts displayed

**Validation**:
```typescript
interface DomainValidation {
  name: {
    minLength: 1,
    maxLength: 50,
    pattern: /^[a-zA-Z0-9\s\-_]+$/, // Alphanumeric, spaces, hyphens, underscores
    unique: true
  },
  description: {
    maxLength: 500,
    optional: true
  }
}
```

**TypeScript Type**:
```typescript
interface Domain {
  id: string; // UUID v4
  name: string;
  description?: string;
  created_at: string; // ISO 8601
  updated_at: string; // ISO 8601
  // Computed fields (from API)
  connection_count?: number;
  saved_query_count?: number;
  query_history_count?: number;
}
```

---

### 2. Connection

Database connection configuration scoped to a specific domain.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | TEXT | PRIMARY KEY | UUID v4 in hyphenated format (36 chars) |
| domain_id | TEXT | NOT NULL, FOREIGN KEY | Reference to domains.id (CASCADE DELETE) |
| name | TEXT | NOT NULL | Connection display name |
| database_type | TEXT | NOT NULL | Database type: "postgresql", "mysql", "doris", "druid" |
| connection_url | TEXT | NOT NULL | Database connection URL |
| status | TEXT | NOT NULL | Connection status: "connected", "disconnected", "error" |
| created_at | TEXT | NOT NULL | ISO 8601 timestamp (UTC) |
| updated_at | TEXT | NOT NULL | ISO 8601 timestamp (UTC) |

**Composite Unique Constraint**:
- `UNIQUE(domain_id, name)` - Connection names must be unique within a domain

**Business Rules**:
- Connections are strictly scoped to one domain (no sharing between domains)
- Connection URLs must be valid for the specified database type
- Connection status is transient (not persisted, derived from last test/query)
- Deleting a connection does NOT cascade to saved queries or history (those retain connection reference)

**Validation**:
```typescript
interface ConnectionValidation {
  name: {
    minLength: 1,
    maxLength: 100,
    uniqueWithinDomain: true
  },
  database_type: {
    enum: ['postgresql', 'mysql', 'doris', 'druid']
  },
  connection_url: {
    pattern: /^(postgresql|mysql):\/\/.+$/,
    validateFormat: true // Validate host, port, database format
  }
}
```

**TypeScript Type**:
```typescript
interface Connection {
  id: string; // UUID v4
  domain_id: string;
  name: string;
  database_type: 'postgresql' | 'mysql' | 'doris' | 'druid';
  connection_url: string;
  status: 'connected' | 'disconnected' | 'error';
  created_at: string; // ISO 8601
  updated_at: string; // ISO 8601
}
```

---

### 3. SavedQuery

User-saved SQL queries scoped to a domain.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | TEXT | PRIMARY KEY | UUID v4 in hyphenated format (36 chars) |
| domain_id | TEXT | NOT NULL, FOREIGN KEY | Reference to domains.id (CASCADE DELETE) |
| connection_id | TEXT | NOT NULL | Reference to connections.id (NOT CASCADE) |
| name | TEXT | NOT NULL | Query display name |
| sql_text | TEXT | NOT NULL | SQL query text (SELECT only) |
| created_at | TEXT | NOT NULL | ISO 8601 timestamp (UTC) |
| updated_at | TEXT | NOT NULL | ISO 8601 timestamp (UTC) |

**Composite Unique Constraint**:
- `UNIQUE(domain_id, name)` - Query names must be unique within a domain

**Business Rules**:
- Saved queries are scoped to one domain
- Queries reference a connection but survive connection deletion (allow orphaned queries)
- SQL text must pass validation (SELECT-only, auto-LIMIT)
- Saved queries can be edited (SQL text and name)

**Validation**:
```typescript
interface SavedQueryValidation {
  name: {
    minLength: 1,
    maxLength: 100,
    uniqueWithinDomain: true
  },
  sql_text: {
    minLength: 1,
    maxLength: 10000,
    validateSql: true // Must be SELECT-only, auto-LIMIT applied
  }
}
```

**TypeScript Type**:
```typescript
interface SavedQuery {
  id: string; // UUID v4
  domain_id: string;
  connection_id: string;
  name: string;
  sql_text: string;
  created_at: string; // ISO 8601
  updated_at: string; // ISO 8601
}
```

---

### 4. QueryHistory

Record of executed queries within a domain.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | TEXT | PRIMARY KEY | UUID v4 in hyphenated format (36 chars) |
| domain_id | TEXT | NOT NULL, FOREIGN KEY | Reference to domains.id (CASCADE DELETE) |
| connection_id | TEXT | NOT NULL | Reference to connections.id (NOT CASCADE) |
| sql_text | TEXT | NOT NULL | SQL query text executed |
| execution_time_ms | INTEGER | NOT NULL | Query execution time in milliseconds |
| row_count | INTEGER | NOT NULL | Number of rows returned |
| status | TEXT | NOT NULL | "success", "error", "cancelled" |
| error_message | TEXT | OPTIONAL | Error message if status = "error" |
| executed_at | TEXT | NOT NULL | ISO 8601 timestamp (UTC) |

**Business Rules**:
- Query history is scoped to one domain
- History entries reference connection but survive connection deletion
- History is append-only (no updates, only inserts)
- Cancelled queries (from domain switching) are recorded with status="cancelled"

**Validation**:
```typescript
interface QueryHistoryValidation {
  sql_text: {
    minLength: 1,
    maxLength: 10000
  },
  execution_time_ms: {
    min: 0
  },
  row_count: {
    min: 0
  },
  status: {
    enum: ['success', 'error', 'cancelled']
  },
  error_message: {
    maxLength: 1000,
    requiredIf: (status) => status === 'error'
  }
}
```

**TypeScript Type**:
```typescript
interface QueryHistory {
  id: string; // UUID v4
  domain_id: string;
  connection_id: string;
  sql_text: string;
  execution_time_ms: number;
  row_count: number;
  status: 'success' | 'error' | 'cancelled';
  error_message?: string;
  executed_at: string; // ISO 8601
}
```

---

## Relationships

### Entity Relationship Diagram

```
┌─────────────────┐
│     Domain      │
│  (id, name)     │
└────────┬────────┘
         │ 1
         │
         │ N
    ┌────┴────┬─────────────┬──────────────┐
    │         │             │              │
┌───▼───────┐ │ ┌───────────▼─┐  ┌─────────▼────────┐
│Connection │ │ │ SavedQuery  │  │  QueryHistory    │
│           │ │ │             │  │                  │
└───────────┘ │ └─────────────┘  └──────────────────┘
              │
              │ (connection_id reference, not CASCADE)
              └─────────────────┐
                                │
                    ┌───────────▼─┐  ┌──────────────────┐
                    │ SavedQuery  │  │  QueryHistory    │
                    │             │  │                  │
                    └─────────────┘  └──────────────────┘
```

### Relationship Details

1. **Domain → Connection (1:N)**
   - One domain contains many connections
   - Foreign key: `connections.domain_id → domains.id`
   - Cascade: `ON DELETE CASCADE` (deleting domain deletes all connections)

2. **Domain → SavedQuery (1:N)**
   - One domain contains many saved queries
   - Foreign key: `saved_queries.domain_id → domains.id`
   - Cascade: `ON DELETE CASCADE` (deleting domain deletes all saved queries)

3. **Domain → QueryHistory (1:N)**
   - One domain contains many query history entries
   - Foreign key: `query_history.domain_id → domains.id`
   - Cascade: `ON DELETE CASCADE` (deleting domain deletes all history)

4. **Connection → SavedQuery (1:N)**
   - One connection can be used by many saved queries
   - Reference: `saved_queries.connection_id → connections.id`
   - Cascade: **NO CASCADE** (deleting connection orphans saved queries)

5. **Connection → QueryHistory (1:N)**
   - One connection can have many query history entries
   - Reference: `query_history.connection_id → connections.id`
   - Cascade: **NO CASCADE** (deleting connection orphans history)

### Isolation Rules

- **100% Domain Isolation**: Queries filtering by `domain_id` ensure zero cross-domain data leakage
- **Active Domain Context**: Frontend maintains active domain in localStorage
- **API Filtering**: All list endpoints (`GET /connections`, `GET /saved-queries`, etc.) require `domain_id` parameter
- **Domain Switching**: Switching domains cancels in-flight queries (AbortController)

---

## Validation Rules

### Domain Validation

```rust
// backend/src/models/domain.rs
impl Domain {
    pub fn validate_name(name: &str) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Domain name cannot be empty".to_string());
        }
        if name.len() > 50 {
            return Err("Domain name cannot exceed 50 characters".to_string());
        }
        if !name.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_') {
            return Err("Domain name contains invalid characters".to_string());
        }
        Ok(())
    }

    pub fn validate_description(description: &Option<String>) -> Result<(), String> {
        if let Some(desc) = description {
            if desc.len() > 500 {
                return Err("Description cannot exceed 500 characters".to_string());
            }
        }
        Ok(())
    }
}
```

### Connection Validation

```rust
// backend/src/models/connection.rs
impl Connection {
    pub fn validate_url(database_type: &str, url: &str) -> Result<(), String> {
        match database_type {
            "postgresql" => {
                if !url.starts_with("postgresql://") {
                    return Err("PostgreSQL URL must start with 'postgresql://'".to_string());
                }
            }
            "mysql" => {
                if !url.starts_with("mysql://") {
                    return Err("MySQL URL must start with 'mysql://'".to_string());
                }
            }
            _ => return Err(format!("Unsupported database type: {}", database_type)),
        }
        Ok(())
    }
}
```

### SQL Validation

```rust
// backend/src/validation/sql_validator.rs (existing)
impl SqlValidator {
    pub fn validate_and_prepare(sql: &str, default_limit: usize) -> Result<(String, bool), AppError> {
        // 1. Parse SQL with sqlparser
        // 2. Ensure SELECT-only (reject INSERT, UPDATE, DELETE, DROP, etc.)
        // 3. Auto-append LIMIT if missing
        // 4. Return (validated_sql, limit_applied)
    }
}
```

---

## State Machines

### Domain State

Domains have no explicit state machine (always "active" once created).

**State Transitions**:
```
[Create] → Active → [Delete with Confirmation]
```

### Connection Status State

```
          ┌─────────────┐
          │             │
   ┌──────▼────────┐    │
   │ disconnected  │    │ (test connection)
   └───────┬───────┘    │
           │            │
           │ (test)     │
           │            │
   ┌───────▼────────┐   │
   │   connected    │───┘
   └───────┬────────┘
           │
           │ (error during query/test)
           │
   ┌───────▼────────┐
   │     error      │
   └────────────────┘
```

**Transitions**:
- `disconnected → connected`: Successful connection test
- `connected → error`: Failed query or connection test
- `error → connected`: Successful reconnection test
- `* → disconnected`: Manual disconnection or initial state

### Query Execution Status

```
[Start] → executing → success/error/cancelled → [End]
```

**Transitions**:
- `executing → success`: Query completes successfully
- `executing → error`: Query fails (timeout, syntax error, connection error)
- `executing → cancelled`: User switches domains during execution (AbortController)

---

## Database Schema

### SQLite Schema (backend/src/storage/sqlite.rs)

```sql
-- Enable foreign keys (MUST be set per-connection)
PRAGMA foreign_keys = ON;

-- Domains table
CREATE TABLE IF NOT EXISTS domains (
    id TEXT PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Connections table (existing table, add domain_id)
CREATE TABLE IF NOT EXISTS connections (
    id TEXT PRIMARY KEY,
    domain_id TEXT NOT NULL,
    name TEXT NOT NULL,
    database_type TEXT NOT NULL,
    connection_url TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'disconnected',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE CASCADE,
    UNIQUE(domain_id, name)
);

-- Saved queries table (new)
CREATE TABLE IF NOT EXISTS saved_queries (
    id TEXT PRIMARY KEY,
    domain_id TEXT NOT NULL,
    connection_id TEXT NOT NULL,
    name TEXT NOT NULL,
    sql_text TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE CASCADE,
    UNIQUE(domain_id, name)
);

-- Query history table (new)
CREATE TABLE IF NOT EXISTS query_history (
    id TEXT PRIMARY KEY,
    domain_id TEXT NOT NULL,
    connection_id TEXT NOT NULL,
    sql_text TEXT NOT NULL,
    execution_time_ms INTEGER NOT NULL,
    row_count INTEGER NOT NULL,
    status TEXT NOT NULL,
    error_message TEXT,
    executed_at TEXT NOT NULL,
    FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_connections_domain_created
    ON connections(domain_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_saved_queries_domain_created
    ON saved_queries(domain_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_query_history_domain_executed
    ON query_history(domain_id, executed_at DESC);

-- Index for connection lookups in saved queries/history
CREATE INDEX IF NOT EXISTS idx_saved_queries_connection
    ON saved_queries(connection_id);

CREATE INDEX IF NOT EXISTS idx_query_history_connection
    ON query_history(connection_id);
```

### Migration Strategy

**Existing `connections` table**:
```sql
-- Add domain_id column (nullable initially for migration)
ALTER TABLE connections ADD COLUMN domain_id TEXT;

-- Create default domain for existing connections
INSERT INTO domains (id, name, description, created_at, updated_at)
VALUES (
    'default-domain-id',
    'Default Domain',
    'Auto-created for existing connections',
    datetime('now'),
    datetime('now')
);

-- Assign all existing connections to default domain
UPDATE connections SET domain_id = 'default-domain-id' WHERE domain_id IS NULL;

-- Make domain_id NOT NULL after migration
-- (SQLite doesn't support ALTER COLUMN, so recreate table with foreign key)
```

---

## API Request/Response Examples

### Create Domain

**Request**:
```http
POST /api/domains
Content-Type: application/json

{
  "name": "Production",
  "description": "Production database connections"
}
```

**Response** (201 Created):
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Production",
  "description": "Production database connections",
  "created_at": "2025-12-28T10:30:00Z",
  "updated_at": "2025-12-28T10:30:00Z",
  "connection_count": 0,
  "saved_query_count": 0,
  "query_history_count": 0
}
```

### List Domains

**Request**:
```http
GET /api/domains
```

**Response** (200 OK):
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Production",
    "description": "Production database connections",
    "created_at": "2025-12-28T10:30:00Z",
    "updated_at": "2025-12-28T10:30:00Z",
    "connection_count": 3,
    "saved_query_count": 5,
    "query_history_count": 42
  },
  {
    "id": "660e8400-e29b-41d4-a716-446655440001",
    "name": "Development",
    "description": null,
    "created_at": "2025-12-28T11:00:00Z",
    "updated_at": "2025-12-28T11:00:00Z",
    "connection_count": 1,
    "saved_query_count": 2,
    "query_history_count": 10
  }
]
```

### Create Connection (Domain-Scoped)

**Request**:
```http
POST /api/connections
Content-Type: application/json

{
  "domain_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Production DB",
  "database_type": "postgresql",
  "connection_url": "postgresql://user:pass@localhost:5432/proddb"
}
```

**Response** (201 Created):
```json
{
  "id": "770e8400-e29b-41d4-a716-446655440002",
  "domain_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Production DB",
  "database_type": "postgresql",
  "connection_url": "postgresql://user:pass@localhost:5432/proddb",
  "status": "connected",
  "created_at": "2025-12-28T10:35:00Z",
  "updated_at": "2025-12-28T10:35:00Z"
}
```

### List Connections (Filtered by Domain)

**Request**:
```http
GET /api/connections?domain_id=550e8400-e29b-41d4-a716-446655440000
```

**Response** (200 OK):
```json
[
  {
    "id": "770e8400-e29b-41d4-a716-446655440002",
    "domain_id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Production DB",
    "database_type": "postgresql",
    "connection_url": "postgresql://user:pass@localhost:5432/proddb",
    "status": "connected",
    "created_at": "2025-12-28T10:35:00Z",
    "updated_at": "2025-12-28T10:35:00Z"
  }
]
```

### Save Query

**Request**:
```http
POST /api/domains/550e8400-e29b-41d4-a716-446655440000/queries/save
Content-Type: application/json

{
  "connection_id": "770e8400-e29b-41d4-a716-446655440002",
  "name": "Active Users",
  "sql_text": "SELECT * FROM users WHERE status = 'active'"
}
```

**Response** (201 Created):
```json
{
  "id": "880e8400-e29b-41d4-a716-446655440003",
  "domain_id": "550e8400-e29b-41d4-a716-446655440000",
  "connection_id": "770e8400-e29b-41d4-a716-446655440002",
  "name": "Active Users",
  "sql_text": "SELECT * FROM users WHERE status = 'active' LIMIT 1000",
  "created_at": "2025-12-28T10:40:00Z",
  "updated_at": "2025-12-28T10:40:00Z"
}
```

### Query History Entry

**Request**:
```http
GET /api/domains/550e8400-e29b-41d4-a716-446655440000/queries/history
```

**Response** (200 OK):
```json
[
  {
    "id": "990e8400-e29b-41d4-a716-446655440004",
    "domain_id": "550e8400-e29b-41d4-a716-446655440000",
    "connection_id": "770e8400-e29b-41d4-a716-446655440002",
    "sql_text": "SELECT * FROM users WHERE status = 'active' LIMIT 1000",
    "execution_time_ms": 234,
    "row_count": 42,
    "status": "success",
    "error_message": null,
    "executed_at": "2025-12-28T10:45:00Z"
  },
  {
    "id": "aa0e8400-e29b-41d4-a716-446655440005",
    "domain_id": "550e8400-e29b-41d4-a716-446655440000",
    "connection_id": "770e8400-e29b-41d4-a716-446655440002",
    "sql_text": "SELECT COUNT(*) FROM orders",
    "execution_time_ms": 0,
    "row_count": 0,
    "status": "cancelled",
    "error_message": "User switched domains during execution",
    "executed_at": "2025-12-28T10:50:00Z"
  }
]
```

---

## Frontend State Management

### DomainContext

```typescript
// frontend/src/contexts/DomainContext.tsx
interface DomainContextValue {
  activeDomainId: string | null;
  setActiveDomain: (domainId: string) => void;
  domains: Domain[];
  refreshDomains: () => Promise<void>;
}

const DOMAIN_STORAGE_KEY = 'active_domain_id';

// localStorage sync with useSyncExternalStore
const subscribe = (callback: () => void) => {
  const handleStorageChange = (e: StorageEvent) => {
    if (e.key === DOMAIN_STORAGE_KEY) callback();
  };
  window.addEventListener('storage', handleStorageChange);
  return () => window.removeEventListener('storage', handleStorageChange);
};

const getSnapshot = () => localStorage.getItem(DOMAIN_STORAGE_KEY);

export function DomainProvider({ children }: { children: React.ReactNode }) {
  const activeDomainId = useSyncExternalStore(subscribe, getSnapshot);
  // ...
}
```

### Query Cancellation

```typescript
// frontend/src/services/query.ts
const abortControllers = new Map<string, AbortController>();

export async function executeQuery(
  connectionId: string,
  sql: string,
  domainId: string
): Promise<QueryResult> {
  const controller = new AbortController();
  abortControllers.set(domainId, controller);

  try {
    const response = await api.post('/query/execute',
      { connection_id: connectionId, sql },
      { signal: controller.signal }
    );
    return response.data;
  } catch (error) {
    if (error.name === 'AbortError') {
      throw new Error('Query cancelled due to domain switch');
    }
    throw error;
  } finally {
    abortControllers.delete(domainId);
  }
}

export function cancelQueriesForDomain(domainId: string) {
  const controller = abortControllers.get(domainId);
  if (controller) {
    controller.abort();
    abortControllers.delete(domainId);
  }
}
```

---

**End of Data Model Specification**
