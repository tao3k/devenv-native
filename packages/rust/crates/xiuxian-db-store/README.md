# xiuxian-db-store

`xiuxian-db-store` is the shared storage boundary for database-backed local
helpers that should not be owned by Wendao search internals or Qianji workflow
runtime code.

## Feature Surfaces

- `vector-store`: re-exports the Lance/vector storage surface from
  `xiuxian-vector` for existing vector-store consumers.
- `sqlite`: exposes bounded SQLite helpers for lightweight local persistence.
- `duckdb-types`: exposes generic DuckDB runtime config and SQL helper types
  without compiling the DuckDB runtime.
- `duckdb`: enables real DuckDB connection opening and initialization.

## Ownership Boundary

Generic DuckDB storage primitives belong here. Wendao may still own
search-specific runtime resolution, Flight-facing behavior, and query routing,
but it should consume the generic connection and SQL helper surface from this
crate. Qianji can later depend on this crate for workflow-local DuckDB storage
without importing Wendao search modules.
