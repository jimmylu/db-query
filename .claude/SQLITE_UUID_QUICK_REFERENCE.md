# SQLite & UUID Quick Reference Card

## CASCADE DELETE Checklist

```rust
// 1. Enable per connection (in init_schema)
conn.execute("PRAGMA foreign_keys = ON", [])?;

// 2. Define constraints in schema
FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE CASCADE

// 3. Use explicit transactions
let tx = conn.transaction()?;
tx.execute("DELETE FROM domains WHERE id = ?", [id])?;
tx.commit()?;  // Explicit commit required

// 4. Index foreign key columns for performance
CREATE INDEX idx_connections_domain
ON connections(domain_id);
```

**Performance**: CASCADE with indexes = 100x faster than full table scans

---

## Indexing Strategy Quick Table

| Query Pattern | Index | Performance |
|---|---|---|
| `WHERE domain_id = ?` | `(domain_id)` | Good |
| `WHERE domain_id = ? ORDER BY created_at` | `(domain_id, created_at)` | Excellent |
| `WHERE domain_id = ? AND status = 'active'` | `(domain_id, status)` | Very Good |
| Range: `WHERE domain_id = ? AND created_at > ?` | `(domain_id, created_at)` | Excellent |

**Rule**: Put equality columns first, range columns second in composite indexes

---

## UUID Generation Pattern

```rust
// Recommended: Handler layer
pub async fn create_domain(
    State(storage): State<Arc<Storage>>,
    Json(payload): Json<Request>,
) -> Response {
    // Generate here - explicit and testable
    let id = Uuid::new_v4().to_string();

    // Pass to storage layer
    storage.save_domain(&id, payload).await
}

// Storage layer signature
pub async fn save_domain(&self, id: &str, ...) -> Result<()>
```

**Storage Format**: `TEXT` for user-facing IDs (standard format: `550e8400-e29b-41d4-a716-446655440000`)

---

## Cargo.toml Essentials

```toml
[dependencies]
uuid = { version = "1.0", features = ["v4", "serde"] }
rusqlite = { version = "0.38.0", features = ["bundled"] }
tokio = { version = "1", features = ["full"] }
```

- **v4**: Random UUID generation
- **serde**: JSON serialization
- **bundled**: Compile SQLite from source
- **full**: All tokio features (Mutex, runtime)

---

## Transaction Pattern (Safe Default)

```rust
// Transactions rollback by default on drop
async fn atomic_operation(conn: &Connection) -> Result<()> {
    let tx = conn.transaction()?;

    // Multiple operations
    tx.execute("INSERT INTO ...", params![])?;
    tx.execute("UPDATE ...", params![])?;

    // Explicit commit required
    tx.commit()?;

    // If any execute() fails, commit is skipped
    // Rollback happens automatically
    Ok(())
}
```

**Key Property**: Automatic rollback prevents partial updates

---

## Denormalization Pattern for Multi-Tenant Queries

```sql
-- Instead of:
SELECT c.* FROM connections c
JOIN domains d ON c.domain_id = d.id
WHERE d.id = ?;

-- Denormalize domain_id in child table:
CREATE TABLE connections (
    id TEXT PRIMARY KEY,
    domain_id TEXT NOT NULL,  -- Duplicate for fast filtering
    -- ... other columns
    FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE CASCADE
);

-- Query becomes:
SELECT * FROM connections
WHERE domain_id = ?
ORDER BY created_at DESC;
```

**Benefit**: Single table scan + index lookup vs JOIN

---

## Performance Baselines (SQLite + rusqlite)

| Operation | Time | Scale |
|---|---|---|
| UUID v4 generation | <1μs | 1 domain |
| Single row insert | 0.5ms | 1000 domains |
| Full table scan | 10ms | 10,000 rows |
| Indexed lookup | 0.1ms | 10,000 rows |
| CASCADE delete (50 children) | 2ms | With indexes |
| CASCADE delete (50 children) | 200ms | Without indexes |

---

## Troubleshooting

**Q: CASCADE DELETE not working?**
```rust
// Check: PRAGMA foreign_keys must be ON
conn.execute("PRAGMA foreign_keys = ON", [])?;
// Must be set per-connection, not persistent
```

**Q: Slow domain filtering?**
```sql
-- Add composite index
CREATE INDEX idx_domain_created
ON table_name(domain_id, created_at DESC);

-- Verify usage with EXPLAIN
EXPLAIN QUERY PLAN
SELECT * FROM table_name
WHERE domain_id = ?
ORDER BY created_at DESC;
```

**Q: UUID thread safety in async?**
```rust
// Uuid::new_v4() is NOT async - fully safe
async fn handler() {
    let id = Uuid::new_v4().to_string();  // Blocks <1μs
    // No await, no mutex, no Arc needed
}
```

---

## Multi-Tenant Schema Template

```sql
PRAGMA foreign_keys = ON;

-- Tenant container
CREATE TABLE domains (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tenant resource
CREATE TABLE connections (
    id TEXT PRIMARY KEY,
    domain_id TEXT NOT NULL REFERENCES domains(id) ON DELETE CASCADE,
    name TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Nested resource
CREATE TABLE metadata_cache (
    id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL REFERENCES connections(id) ON DELETE CASCADE,
    domain_id TEXT NOT NULL REFERENCES domains(id) ON DELETE CASCADE,
    json TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Performance indexes
CREATE INDEX idx_conn_domain ON connections(domain_id, created_at);
CREATE INDEX idx_meta_domain ON metadata_cache(domain_id, created_at);
CREATE INDEX idx_meta_conn ON metadata_cache(connection_id);
```

---

## Test Verification Queries

```sql
-- Verify CASCADE worked
SELECT COUNT(*) FROM connections WHERE domain_id = ?;
-- Should be 0 after domain deletion

-- Verify index usage
EXPLAIN QUERY PLAN
SELECT * FROM connections WHERE domain_id = ? ORDER BY created_at DESC;
-- Should show: "SEARCH connections USING idx_conn_domain"

-- Verify foreign key constraints
PRAGMA foreign_keys;
-- Should return: 1 (enabled)

-- Check index statistics
ANALYZE;
SELECT name, tbl_name, stat
FROM sqlite_stat1
WHERE tbl_name IN ('connections', 'metadata_cache');
```

---

## References

- **CASCADE DELETE**: https://sqlite.org/foreignkeys.html
- **Indexing**: https://highperformancesqlite.com/watch/composite-indexes
- **Transactions**: https://docs.rs/rusqlite/latest/rusqlite/struct.Transaction.html
- **UUID**: https://docs.rs/uuid/latest/uuid/

