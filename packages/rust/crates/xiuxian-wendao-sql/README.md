# xiuxian-wendao-sql

`xiuxian-wendao-sql` owns narrow bounded SQL helper surfaces that downstream
crates can consume without pulling the full default `xiuxian-wendao` feature
graph.

Current ownership in this crate:

1. bounded-work markdown SQL discovery and row building
2. a request-scoped DuckDB local relation engine for bounded Arrow SQL
   execution
3. stable SQL payload rendering over `xiuxian-wendao-core` DTOs
4. advisory repo-native semantic read-model rows, catalogs, deterministic
   snapshots, snapshot checks, materialization plans, executable
   materialization preflights, Arrow record batches, and query payloads for
   `semantic_objects`, `semantic_relations`, and `semantic_projection_state`
5. single-statement read-only SQL admission for semantic read-model queries
6. engine-neutral dataset-to-ontology materialization helpers that register raw
   Arrow source tables, execute SELECT-only mapping SQL, and return observation,
   read-model, and validation counts without owning persistent DuckDB runtime
   storage policy

The semantic read-model `RecordBatch` builders are also the accepted Rust owner
surface for WendaoGraph ontology quality checks. Downstream bridges may package
those batches for Arrow Flight, but they must not read registry JSON files or
promote advisory Julia diagnostics into SQL authority.

This crate does not own the full shared-query architecture. Broader shared
query semantics, gateway adapters, and persistent DuckDB-backed business
behavior remain in `xiuxian-wendao`. For dataset-to-ontology ingestion, this
crate owns the bounded local-relation helper; `xiuxian-wendao` owns the
runtime execution path.
