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
7. DuckDB inspection for Episteme candidate Parquet read models, including row
   counts, kind counts, blocked-review checks, ontology-truth checks, and
   relation endpoint integrity checks without reading candidate TSV projections
8. Arrow schema contracts for bounded SQL data-plane tables, including
   bounded-work markdown, semantic read-model, and dataset ontology tables, so
   `RecordBatch` payloads use Arrow field names, Arrow data types,
   nullability policy, and schema metadata as the primary contract instead of
   JSON Schema. The table declarations live here, while reusable Arrow schema
   construction and compatibility validation mechanics come from
   `xiuxian-db-store`.

The semantic read-model `RecordBatch` builders are also the accepted Rust owner
surface for WendaoGraph ontology quality checks. Downstream bridges may package
those batches for Arrow Flight, but they must not read registry JSON files or
promote advisory Julia diagnostics into SQL authority.

Data-plane contracts in this crate are Arrow-first. JSON Schema remains valid
for source manifests, registry snapshots, external JSON API payloads, and
source-contract reports, but table-shaped SQL, DuckDB, Flight, and
WendaoGraph handoff payloads must validate against Arrow schemas. Dataset
ontology materialization now validates the `semantic_objects`,
`semantic_relations`, and `semantic_projection_state` SQL outputs against the
same Arrow contracts used by the semantic read-model `RecordBatch` builders
before registering those tables for downstream quality checks. The same
materialization pass validates `ontology_object_observation`,
`ontology_link_observation`, `ontology_evidence`, `ontology_entity`, and
`ontology_relation` with dataset ontology Arrow contracts before validation SQL
can consume them. The bounded-work markdown registration path also generates
and validates its `markdown` table schema through the same Arrow contract
adapter before registering rows into the local relation engine.

Episteme candidate inspection consumes
`ontology_candidate_objects.parquet`,
`ontology_candidate_relations.parquet`, and
`ontology_candidate_evidence.parquet` through DuckDB `read_parquet()` views.
The TSV files emitted beside those read models remain compatibility projections
for reporting only; they are not a SQL, ontology, or search contract.

This crate does not own the full shared-query architecture. Broader shared
query semantics, gateway adapters, and persistent DuckDB-backed business
behavior remain in `xiuxian-wendao`. For dataset-to-ontology ingestion, this
crate owns the bounded local-relation helper; `xiuxian-wendao` owns the
runtime execution path.
