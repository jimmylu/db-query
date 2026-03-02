# AGENTS.md

## Cursor Cloud specific instructions

### System Dependencies

- **Rust toolchain**: DataFusion 51.0 requires Rust >= 1.88. The VM default may be older; the update script ensures `stable` is the default via `rustup default stable`.
- **OpenSSL dev**: `libssl-dev` is required for `openssl-sys` crate (used by `reqwest`/`tokio-postgres`). Must be installed before `cargo build`/`cargo test`.
- **Docker** (optional): Needed only to spin up test PostgreSQL/MySQL instances for end-to-end testing. Not required for unit tests or building.

### Services Overview

| Service | Command | Port | Notes |
|---------|---------|------|-------|
| Backend (Rust/Axum) | `cd backend && cargo run` | 3000 | Health: `GET /health` |
| Frontend (Vite/React) | `cd frontend && npm run dev` | **3001** | Configured in `vite.config.ts` (NOT 5173 as README states) |

### Important Gotchas

- **Frontend port is 3001**, not 5173. The `vite.config.ts` sets `server.port: 3001` and proxies `/api` to `http://localhost:3000`.
- **`make setup`** copies `.env.example` to `.env` for both backend and frontend (only if `.env` doesn't exist) and fetches dependencies.
- **SQLite is embedded**: The `rusqlite` crate uses the `bundled` feature, so no SQLite system install is needed.
- **Pre-existing lint errors**: Both `cargo clippy -- -D warnings` and `npm run lint` (with `--max-warnings 0`) fail due to pre-existing issues in the codebase (unused imports, `any` types, etc.).
- **Pre-existing doctest failure**: One doctest in `src/models/cross_database_query.rs` fails (missing imports). All 156 unit tests pass.

### Standard Commands

See `CLAUDE.md` and `Makefile` for the full list of dev commands (`make lint`, `make test`, `make dev-backend`, `make dev-frontend`, etc.).
