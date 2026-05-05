# xiuxian-wendao-sql

`xiuxian-wendao-sql` owns narrow bounded SQL helper surfaces that downstream
crates can consume without pulling the full default `xiuxian-wendao` feature
graph.

Current ownership in this crate:

1. bounded-work markdown SQL discovery and row building
2. a request-scoped DataFusion local relation engine for that bounded lane
3. stable SQL payload rendering over `xiuxian-wendao-core` DTOs
4. advisory repo-native semantic read-model rows and query payloads for
   `semantic_objects`, `semantic_relations`, and
   `semantic_projection_state`

This crate does not own the full shared-query architecture. Broader shared
query semantics, gateway adapters, and DuckDB-backed business behavior remain
in `xiuxian-wendao`.
