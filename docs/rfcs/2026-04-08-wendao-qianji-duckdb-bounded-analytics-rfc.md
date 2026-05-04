---
type: knowledge
title: "RFC: DuckDB as a Bounded In-Process Analytic Lane for Wendao and Qianji"
category: "rfc"
status: "draft"
authors:
  - codex
created: 2026-04-08
tags:
  - rfc
  - wendao
  - qianji
  - duckdb
  - duckdb-rs
  - arrow
  - datafusion
  - valkey
metadata:
  title: "RFC: DuckDB as a Bounded In-Process Analytic Lane for Wendao and Qianji"
---

# RFC: DuckDB as a Bounded In-Process Analytic Lane for Wendao and Qianji

## 1. Summary

This RFC proposes adding `duckdb-rs` as a bounded in-process analytic lane for
Wendao and Qianji.

The primary decision is:

1. DuckDB may be used for request-scoped or bounded-lived SQL analytics over
   Arrow-first relations
2. Wendao keeps Arrow Flight as the external business boundary
3. the current shared query system inside `xiuxian-wendao` still contains
   DataFusion-led residue, but the intended database execution direction for
   search-side SQL is DuckDB-first
4. Valkey remains the hot-cache, workflow-state, checkpoint, and
   transient-coordination layer
5. the vector store remains the embedding and ANN layer

This RFC does not make DuckDB the new primary database, the new vector store,
or the new checkpoint coordinator.

## 2. Alignment

This RFC aligns with the following stable references:

1. [RFC: Wendao Query Engine on DataFusion, LanceDB, and link_graph](2026-03-26-wendao-query-engine-rfc.md)
2. [RFC: Wendao Arrow-First Plugin Protocol with Flight-First Transport](2026-03-27-wendao-arrow-plugin-flight-rfc.md)
3. [Search Queries Architecture](../../packages/rust/crates/xiuxian-wendao/docs/03_features/210_search_queries_architecture.md)
4. [RFC: Data-Centric Workflow Orchestration on Wendao Relations](../../packages/rust/crates/xiuxian-qianji/docs/rfcs/2026-03-26-qianji-data-centric-workflow-rfc.md)
5. [Spec: Qianji Runtime Config Layering](../../packages/rust/crates/xiuxian-qianji/docs/20_specs/2026-04-07-qianji-runtime-config-layering.md)
6. [xiuxian-wendao-runtime README](../../packages/rust/crates/xiuxian-wendao-runtime/README.md)

The paired execution tracking for this RFC follows an active blueprint and
ExecPlan, but canonical RFCs do not link hidden workspace tracking paths
directly.

## 3. Audit Snapshot

### 3.1 Flight Is the Current Wendao Business Boundary

The current repository already encodes stable Flight business routes through
`xiuxian-wendao-runtime` query-contract constants and route tests. The active
business family includes:

1. `/search/intent`
2. `/search/attachments`
3. `/search/references`
4. `/search/symbols`
5. `/search/ast`
6. `/analysis/markdown`
7. `/analysis/code-ast`

DuckDB must not change that external contract.

### 3.2 The Shared Query System Is DataFusion-Based Today

`xiuxian-wendao` currently centralizes shared query translation under
`src/search/queries/`, where SQL, FlightSQL, GraphQL, REST-style query
adapters, and CLI query entrypoints are described as one DataFusion-backed
query family.

This matters because the DuckDB lane proposed here is intentionally narrower:
it is not a silent replacement of the current shared query core.

This also should not be read as DataFusion owning the cross-language Arrow
substrate. The current code shows that `WendaoArrow`, pyarrow, julia-arrow,
and Flight own the Rust-Julia `RecordBatch` data plane. DataFusion's only
defensible future value is Rust-side live Arrow compute, request and response
shaping, and migration-baseline support where the data is still an in-memory
Arrow workset rather than a published Parquet corpus or a DuckDB relation.

More concretely, the remaining DataFusion paths now split into two groups:

1. same-layer search execution residue that should migrate away, such as the
   non-routed request-scoped SQL discovery and logical-view planning, and any
   adapter that still depends on that shared DataFusion-led execution seam for
   search-side database work after routed published-Parquet reads move to
   DuckDB
2. distinct residual value that can remain, such as in-memory live Arrow
   compute over generated batches and migration-baseline comparisons while
   DuckDB cutover is still active

### 3.3 A Bounded Local Markdown SQL Lane Already Exists

The repository already contains a concrete bounded local relation workflow:

1. `xiuxian-wendao::search::queries::sql::bounded_work_markdown`
2. `xiuxian-qianji::workdir::query`

The default bounded path still uses DataFusion over an in-memory `markdown`
table for bounded work surfaces. The same lane now also has a feature-gated
`DuckDbLocalRelationEngine` pilot helper for request-scoped local execution.
That keeps the correctness baseline and the DuckDB pilot in the same bounded
workload, which makes it the most credible first pilot shape because the
workflow is already local, bounded, relation-oriented, and Arrow-friendly.

### 3.4 Qianji Runtime Config Already Keeps Checkpoint Ownership Explicit

`xiuxian-qianji` currently documents and resolves checkpoint persistence as a
Valkey-backed runtime-config lane with TOML-first precedence.

This RFC therefore must not repurpose DuckDB into checkpoint storage or a
replacement for Qianji runtime-state coordination. Any future Qianji DuckDB
pilot must stay downstream of that boundary and operate only as stage-local
relational compute over already materialized Arrow relations.

### 3.5 A Bounded DuckDB Landing Now Exists

There is now a bounded DuckDB integration inside `xiuxian-wendao` and
`xiuxian-wendao-runtime`.

The currently landed Wendao slices are:

1. a narrow local relation-engine seam plus a feature-gated `src/duckdb/`
   bridge
2. typed `search.duckdb` runtime config resolution with TOML-first precedence
3. request-scoped registration policy that can keep small Arrow worksets
   virtual or materialize them through `appender-arrow`
4. a bounded markdown pilot that can execute through DataFusion or DuckDB
   while keeping the default path DataFusion-backed
5. additive bounded execution metadata for engine choice, row and byte counts,
   registration time, local execution time, materialization state, and peak
   temp-storage bytes
6. workspace Arrow `58.1.0` alignment and `arrow-flight` `flight-sql`
   enablement across the Wendao crates that participate in this lane
7. a bounded repo-backed Parquet query-engine seam that can execute
   `repo_entity` and `repo_content_chunk` gateway publication reads through
   DuckDB when `search.duckdb` is enabled, while preserving DataFusion
   fallback for non-`duckdb` builds and disabled runtime policy
8. a bounded local-corpus reuse of the same Parquet query-engine seam for
   published `local_symbol` search, autocomplete, and payload hydration reads,
   again selecting DuckDB only when `search.duckdb` is enabled
9. a bounded local-corpus reuse of the same Parquet query-engine seam for the
   published `reference_occurrence` lane behind `/search/references`, with
   engine-safe identifier quoting so the same published parquet read path
   stays valid in both DataFusion and DuckDB
10. a bounded local-corpus reuse of the same Parquet query-engine seam for the
    published `attachment` lane behind `/search/attachments`, with the same
    engine-safe SQL generation and DuckDB selection policy
11. a bounded local-corpus reuse of the same Parquet query-engine seam for the
    published `knowledge_section` lane behind gateway knowledge search, again
    with engine-safe SQL generation and DuckDB selection policy
12. a bounded gateway aggregation proof for `/search/intent`, where the route
    now exposes internal source-lane query-engine metadata and focused tests
    prove it composes DuckDB-fed `knowledge_section`, `local_symbol`, and
    repo-intent lanes without changing the public contract
13. a bounded gateway read-engine cutover for `/search/symbols`, where the
    route now reuses the published `local_symbol` lane instead of the
    in-memory `UnifiedSymbolIndex`, while preserving the existing response
    contract and pending/indexing semantics
14. a bounded build-owner cutover for `local_symbol`, where the corpus now
    rewrites published partition tables directly to Parquet, uses Parquet-only
    local epoch discovery, and no longer
    participates in local Lance compaction scheduling
15. a bounded build-owner cutover for `reference_occurrence`, where the corpus
    now rewrites its published table directly to Parquet and no longer
    participates in local Lance compaction scheduling
16. a bounded build-owner cutover for `attachment`, where the corpus now
    rewrites its published table directly to Parquet and no longer
    participates in local Lance compaction scheduling
17. a bounded build-owner cutover for `knowledge_section`, where the corpus
    now rewrites its published table directly to Parquet and no longer
    participates in local Lance compaction scheduling
18. a bounded internal diagnostics rollup for the Studio search-index status
    route, where top-level totals, phase counts, `compactionPending`, and the
    aggregate maintenance summary now compute through the local
    relation-engine seam while preserving the public payload and Rust fallback
19. a bounded local-publication boundary cutover where local epoch discovery
    now ignores legacy `.lance` artifacts and local prewarm rejects missing
    Parquet publications instead of falling back to store scans
20. a bounded local-maintenance retirement where local compaction queue and
    worker state are removed, `publish_ready_and_maintain(...)` becomes a pure
    local publish step, and runtime status no longer projects local
    compaction backlog while preserving repo-backed compaction status
21. a bounded diagnostics expansion where the Studio search-index status route
    now also rolls up `query_telemetry_summary`, including per-scope buckets,
    through the same local relation-engine seam while preserving the payload
    contract and Rust fallback path
22. a bounded diagnostics expansion where the Studio search-index status route
    now also selects aggregate `status_reason` through the same local
    relation-engine seam while preserving severity and code priority,
    affected and readable counts, the payload contract, and the Rust fallback
    path
23. a bounded diagnostics expansion where the Studio search-index status route
    now also maps top-level `repo_read_pressure` through the same local
    relation-engine seam while preserving the payload contract, optional field
    semantics, and the Rust fallback path
24. a bounded FlightSQL statement-routing slice where single-table statements
    against published local `reference_occurrence`, `attachment`, and
    `knowledge_section` corpora now reuse the same Parquet query-engine seam,
    while all other statements still fall back to the shared SQL surface and
    routed batches normalize top-level string columns back to the existing
    FlightSQL Arrow shape
25. a bounded FlightSQL repo-source-table slice where concrete repo
    publication tables already exposed by catalog discovery now route through
    the same Parquet query-engine seam, while logical repo views still stay on
    shared SQL fallback and FlightSQL does not take on multi-source repo-view
    planning
26. a bounded FlightSQL local-symbol source-table slice where concrete
    published `local_symbol` source tables, including partitioned active-epoch
    names, now route through the same Parquet query-engine seam, while the
    logical `local_symbol` view still stays on shared SQL fallback and
    FlightSQL does not take on local-view planning
27. a bounded FlightSQL latency-breakdown slice where the routed
    single-table statement benchmark now persists per-phase timing metadata
    for a direct-engine lower bound plus `get_flight_info`, `do_get`
    collection, decode, and validation, so bounded evidence can distinguish
    query-engine time from FlightSQL statement-planning overhead
28. a bounded `appender-arrow` utilization slice where the Studio
    search-index diagnostics helper now marks `query_telemetry_rows` as a
    repeated-use request-scoped relation so DuckDB can prefer
    `MaterializedAppender` without changing payload or fallback semantics
29. a bounded repo/runtime diagnostics slice where the Studio repo-index
    analysis Flight route now rolls up phase summary counts from the
    per-repository `repos` relation through the same local relation-engine
    seam while preserving the JSON repo-index contract and the Flight payload
    shape
30. a bounded repo/runtime diagnostics follow-up slice where the same
    repo-index analysis Flight relation now also preserves runtime active
    ordering through an explicit `active_order` column so
    `active_repo_ids` and `current_repo_id` are recomputed from request-scoped
    rows instead of being copied from the incoming response
31. a bounded repo/runtime diagnostics HTTP follow-up slice where the Studio
    `repo_index_status` JSON route now reuses the same diagnostics helper as
    the repo-index Flight route before serialization, so aggregate counts and
    active identity fields are recomputed consistently across both surfaces
    without widening the JSON envelope or telemetry contract
32. a bounded documentation-only audit slice now records the current Wendao
    ownership matrix directly: mutable runtime state remains in-process, shared
    cache remains Valkey-backed where enabled, published corpora remain
    Parquet, and DuckDB remains only a bounded execution lane over Arrow and
    Parquet relations
33. a bounded documentation-only repo-lane audit slice now narrows that
    ownership matrix further: `repo_index` state remains in-process, repo
    analysis and query-result caches remain in-memory plus Valkey-backed,
    repo publications remain Parquet, and DuckDB/DataFusion remain only local
    execution lanes over those repo publications
34. a bounded documentation-only local-corpus audit slice now narrows the
    ownership matrix further for local corpora: local publication ownership is
    Parquet-first, local epoch discovery is Parquet-only, and DuckDB/DataFusion
    remain only local execution lanes over those local corpus publications
35. a bounded documentation-only state/cache audit slice now narrows the
    runtime split further: mutable `repo_index` and search-plane runtime state
    remain in-process, shared analysis and search-plane caches remain
    Valkey-backed where enabled, and DuckDB remains only an execution and
    bounded analytics lane over Arrow and Parquet relations
36. a bounded documentation-only protocol-surface audit slice now narrows the
    transport split further: native Flight, bounded FlightSQL, and JSON routes
    remain protocol adapters over Parquet publications, in-process state, and
    Valkey-backed caches, while DataFusion or DuckDB continue to sit only in
    the underlying execution lane
37. a bounded performance-gate slice now compares the same deterministic
    synthetic Parquet fixture through the DataFusion and DuckDB
    `ParquetQueryEngine` lanes, emits durable perf reports through
    Wendao-owned performance support, and enforces a configurable
    DuckDB/DataFusion p95 ratio budget without widening storage or protocol
    ownership
38. a bounded GraphQL execution cutover now keeps document parsing and
    GraphQL argument decoding adapter-local, but translates the resulting
    table query into SQL text and executes it through the shared SQL seam
    instead of planning DataFusion expressions directly inside the GraphQL
    adapter
39. a bounded shared-SQL parquet-routing cutover now reuses the same
    published-Parquet target resolution helper directly under the shared SQL
    execution seam, so simple single-table queries over active local corpora
    and concrete repo publication source tables can execute through
    `ParquetQueryEngine`, while discovery catalogs, logical views, and
    multi-source queries still stay on shared SQL fallback
40. a bounded publication-readability naming cutover now treats published
    repo and local corpora as Parquet/query-engine readable rather than as
    "DataFusion-readable", so the storage semantics match the actual
    Parquet publication owner and the bounded execution-kernel split
41. a bounded shared-SQL routed-metadata follow-up now also builds metadata
    for eligible routed published-Parquet queries from the request-scoped
    `SqlQuerySurface`, so routed SQL no longer opens the residual DataFusion
    query core after Parquet execution just to recover discovery metadata
42. a bounded FlightSQL discovery-surface cutover now builds
    `CommandGetDbSchemas` and `CommandGetTables` from publication metadata
    plus logical-view contracts through one request-scoped `SqlQuerySurface`,
    so FlightSQL discovery no longer opens the residual DataFusion query core

Qianji does not yet have a stage-local DuckDB pilot, and the shared query
system still contains DataFusion-led residue on some shared query paths.

## 4. Problem Statement

Wendao and Qianji now have a gap between two realities:

1. both systems already center Arrow-first relation handoff
2. many bounded analytics tasks still fall back to either request-scoped
   DataFusion everywhere or custom Rust row traversal

That gap appears in three places:

1. bounded markdown and diagnostics analytics inside Wendao
2. workflow-stage audit/reduce and consistency checks inside Qianji
3. repo/runtime status or explain-facing local joins that benefit from fast
   in-process SQL without introducing a new external service

For Qianji, this gap is specifically about stage-local relation compute inside
steps such as `audit_step` or `reduce_step`. It is not about workflow-state
persistence, checkpoint ownership, or coordination state, which remain outside
the DuckDB lane.

Without a clear boundary, DuckDB adoption risks failing in one of two ways:

1. it becomes too small and ad hoc to justify the dependency
2. it grows into a new central storage policy and blurs Valkey, vector, and
   query-core ownership

## 5. Goals

This RFC has the following goals:

1. introduce DuckDB as a bounded in-process analytic helper over Arrow-first
   relations
2. keep Arrow Flight unchanged as the external Wendao business boundary
3. let Qianji consume relation-level analytic results without taking ownership
   of retrieval planning or storage policy
4. keep Valkey as the owner of workflow state, checkpoint, resume, and
   coordination state in Qianji
5. narrow DataFusion toward residual live Arrow compute and migration-baseline
   roles instead of treating it as a second long-term search database
6. require explain and telemetry coverage for the DuckDB lane from the start

## 6. Non-Goals

This RFC does not attempt to:

1. replace DataFusion globally in one step
2. make DuckDB the main external query protocol
3. make DuckDB the new cache, state, or checkpoint layer
4. make DuckDB the new vector store
5. commit the workspace to a dedicated shared DuckDB crate before bounded
   pilots exist

## 7. Why `duckdb-rs`

At the time of writing, the upstream `duckdb` crate documents the following
properties that match this lane:

1. ergonomic Rust bindings with in-memory and file-backed connection support
2. `bundled` builds for low-friction local and CI setup
3. `vtab-arrow` for Arrow virtual-table integration
4. `appender-arrow` for efficient Arrow bulk ingest
5. `parquet` and `json` feature flags for file-oriented analytic inputs
6. `vscalar` and `vscalar-arrow` support for custom scalar functions

That combination makes `duckdb-rs` a plausible bounded analytic helper for
Arrow-native relations without adding a second network service.

The current bounded Wendao landing only depends on `bundled` and
`appender-arrow`. Other upstream features such as `vtab-arrow`, `parquet`,
`json`, and custom scalar support remain optional future expansion points
rather than current repository requirements.

## 8. Architectural Decision

### 8.1 Core Decision

Adopt DuckDB only as a bounded internal analytic lane.

The intended shape is:

1. external clients still see Wendao Flight business routes and existing query
   surfaces
2. Wendao and Qianji may register Arrow batches into a local relation engine
   for bounded SQL work
3. DuckDB is one implementation of that local relation-engine seam
4. DataFusion remains only where the code still needs live Arrow compute,
   request and response shaping, or migration-baseline support

### 8.2 Package Ownership

#### `xiuxian-wendao`

Wendao owns:

1. search-plane and analysis business semantics
2. relation registration over Wendao-owned corpora and worksets
3. internal selection of a bounded local relation engine for Wendao-owned
   analytics
4. explain and telemetry emitted for those analytics

#### `xiuxian-qianji`

Qianji owns:

1. workflow-stage orchestration
2. stage-level audit/reduce/consistency narratives
3. consumption of relation-engine results inside workflow stages
4. workflow-facing explain binding

Qianji does not gain ownership of Wendao retrieval planning, storage policy,
or external DuckDB exposure.

#### `xiuxian-wendao-runtime`

The runtime crate owns only the host-side Wendao runtime concerns that would
be needed if Wendao embeds DuckDB:

1. typed host config
2. temp/spill directory policy
3. host bootstrap or long-lived connection helpers

It does not become the owner of search semantics or workflow-stage logic.

#### Valkey

Valkey continues to own:

1. hot cache
2. transient coordination
3. checkpoint-like runtime state
4. other explicit fast-state roles already assigned to it

#### Current Wendao Ownership Matrix

The current Wendao codebase now shows a narrower ownership split than the old
"one search store" framing:

1. mutable repo-index runtime state still lives inside the in-process
   coordinator
2. repo analysis cache and repo-search query cache still use in-memory plus
   Valkey-backed cache paths
3. published local and repo search corpora are persisted as Parquet
4. DuckDB and DataFusion are bounded local execution lanes over Arrow or
   Parquet relations
5. native Flight and bounded FlightSQL remain protocol surfaces rather than
   storage owners

This is the intended reading of the current Wendao landing: DuckDB executes
over Arrow and Parquet; it does not replace mutable runtime state, cache, or
publication ownership.

For the repo lane specifically, the current code-proven split is:

1. `repo_index` mutable runtime state stays inside the in-process coordinator
2. repo analysis cache and repo query-result cache stay in-memory plus
   Valkey-backed
3. repo publications stay Parquet
4. repo search execution uses DataFusion or DuckDB over those Parquet
   publications
5. repo diagnostics routes may use bounded request-scoped relation SQL, but
   they do not become storage owners

For the local corpus lane specifically, the current code-proven split is:

1. `local_symbol`, `reference_occurrence`, `attachment`, and
   `knowledge_section` publication ownership is now Parquet-first
2. local epoch discovery and prewarm are Parquet-only for those migrated local
   corpora
3. local search and hydration execution use DataFusion or DuckDB over those
   Parquet publications
4. gateway routes and bounded FlightSQL may read those publications, but they
   do not become storage owners
5. this does not introduce a separate DuckDB-owned local corpus cache layer

For mutable state and shared cache specifically, the current code-proven split
is:

1. `RepoIndexCoordinator` mutable state remains in-process
2. `SearchPlaneCoordinator` and `SearchPlaneService` mutable runtime state
   remain in-process
3. repository analysis and query-result caches remain in-memory with
   Valkey-backed sharing where configured
4. `SearchPlaneCache` remains the Valkey-backed cache path for manifests,
   leases, and short-lived search-plane cache values where enabled
5. DuckDB remains an execution and bounded analytics lane rather than a
   mutable-state owner or shared cache backplane

For protocol surfaces specifically, the current code-proven split is:

1. native Flight routes remain transport adapters that package batches and
   metadata over underlying search results
2. bounded FlightSQL remains a query protocol over the shared SQL system and
   published Parquet query-engine seam
3. JSON gateway handlers remain response adapters over the underlying search
   and diagnostics lanes
4. those protocol surfaces may expose results produced through DataFusion or
   DuckDB execution, but they do not become persistence owners
5. Parquet publications, in-process state, and Valkey-backed caches keep their
   existing ownership underneath those surfaces

#### Vector Store

The vector layer continues to own:

1. embeddings
2. ANN retrieval
3. vector-index lifecycle

### 8.3 Module Landing Strategy

The first implementation landing has stayed bounded inside `xiuxian-wendao`:

```text
packages/rust/crates/xiuxian-wendao/src/duckdb/
  mod.rs
  runtime.rs
  connection.rs
  arrow.rs
  engine.rs
```

Responsibilities for the current landed shape are:

1. `runtime.rs`: feature gate and typed policy inputs
2. `connection.rs`: connection bootstrap and lifecycle
3. `arrow.rs`: Arrow batch registration and result decode
4. `engine.rs`: local relation-engine policy, request-scoped registration,
   query execution, and bounded engine metadata exposure
5. `parquet.rs`: repo-backed Parquet query-engine selection and bounded
   DataFusion or DuckDB execution for gateway publication reads

The current bounded landing does not yet need separate `registration.rs`,
`query.rs`, or `telemetry.rs` files. If later slices prove real cross-package
reuse or broader query-surface integration, those separations may be justified
then.

## 9. Execution Model

### 9.1 Narrow Local Relation-Engine Seam

The correct abstraction boundary is a narrow local relation engine, not direct
DuckDB calls everywhere.

One acceptable shape is:

```rust
trait LocalRelationEngine {
    fn register_batches(&self, name: &str, batches: &[RecordBatch]) -> Result<()>;
    fn query_arrow(&self, sql: &str) -> Result<Vec<RecordBatch>>;
}
```

This keeps the current architecture honest:

1. DataFusion remains valid
2. DuckDB can replace same-layer search database execution in bounded slices
   without a flag day
3. bounded analytics can choose the right internal engine without changing the
   external contract

### 9.2 Registration Strategy

The first two registration modes should be:

1. ephemeral request-scoped Arrow registration for one-shot analytics
2. bounded materialized registration when the same rows are reused across
   multiple joins, windows, or diagnostics queries

The default preference should be Arrow virtual registration first, with
materialization only when repeat use or spill pressure justifies it.

### 9.3 Runtime Policy

The bounded DuckDB host lane should preserve current repo conventions:

1. TOML-first config precedence
2. explicit feature gating
3. request-scoped or bounded-lived usage only
4. project-aware path resolution through the existing runtime/path helpers

Example configuration shape:

```toml
[search.duckdb]
enabled = true
database_path = ":memory:"
temp_directory = "$PRJ_CACHE_HOME/duckdb/tmp"
threads = 4
materialize_threshold_rows = 200000
prefer_virtual_arrow = true

[qianji.duckdb]
enabled = true
database_path = "$PRJ_DATA_HOME/qianji/duckdb/workflow.db"
temp_directory = "$PRJ_CACHE_HOME/qianji/duckdb/tmp"
```

These keys are architectural placeholders in this RFC. Exact naming and
resolution rules must be revalidated when implementation expands further. The
`search.duckdb` keys above now have a bounded Wendao landing, while the
`qianji.duckdb` example remains future-facing until a Qianji stage-local pilot
exists.

## 10. First Pilot Targets

### 10.1 Wendao Bounded Markdown and Diagnostics Analytics

The existing bounded-work markdown SQL lane is the best first Wendao pilot:

1. the workload is already local and bounded
2. the rows are already normalized into a relation-friendly shape
3. the current DataFusion lane provides a correctness baseline
4. the external Flight business contract does not need to change

### 10.2 Repo-Backed Gateway Reads [landed in first bounded form]

The first gateway-facing DuckDB cut is now landed for repo-backed published
Parquet reads:

1. `repo_entity` publication reads now go through a bounded
   `ParquetQueryEngine` seam
2. `repo_content_chunk` publication reads now go through the same seam
3. DuckDB-enabled Wendao hosts can execute those gateway-facing Parquet reads
   locally in DuckDB without changing payload contracts
4. non-repo gateway handlers and local-corpus Lance-backed build paths remain
   future migration work

### 10.3 Local Symbol Gateway Reads [landed in bounded form]

The next local-corpus gateway cut is now landed for published `local_symbol`
reads:

1. active-epoch local-symbol parquet tables now register through the same
   bounded `ParquetQueryEngine` seam
2. local-symbol search, autocomplete, and payload hydration now execute
   through that seam instead of reading directly from `SearchEngineContext`
3. DuckDB-enabled Wendao hosts can execute those published local-symbol reads
   locally in DuckDB without changing response payloads
4. the unified in-memory symbol index behind `/search/symbols` remained a
   separate subsystem and was not part of that first cut

### 10.4 Reference Occurrence Gateway Reads [landed in bounded form]

The next local-corpus gateway cut is now landed for published
`reference_occurrence` reads behind `/search/references`:

1. the active-epoch published parquet file now registers through the same
   bounded `ParquetQueryEngine` seam
2. the stage-one scan and payload hydration path now execute through that seam
   instead of reading directly from `SearchEngineContext`
3. the SQL builder for this lane now quotes engine-facing identifiers such as
   `column`, so the same published parquet read path stays valid in both
   DataFusion and DuckDB
4. Lance-backed reference-occurrence build ownership remains outside this cut

### 10.5 Attachment Gateway Reads [landed in bounded form]

The next local-corpus gateway cut is now landed for published `attachment`
reads behind `/search/attachments`:

1. the active-epoch published parquet file now registers through the same
   bounded `ParquetQueryEngine` seam
2. the stage-one scan and payload hydration path now execute through that seam
   instead of reading directly from `SearchEngineContext`
3. the SQL builder for this lane now quotes engine-facing identifiers and
   table names so the same published parquet read path stays valid in both
   DataFusion and DuckDB
4. this earlier read-engine cut deliberately left build ownership out of scope;
   a later bounded storage-owner slice lands below

### 10.6 Knowledge Gateway Reads [landed in bounded form]

The next local-corpus gateway cut is now landed for published
`knowledge_section` reads behind gateway knowledge search:

1. the active-epoch published parquet file now registers through the same
   bounded `ParquetQueryEngine` seam
2. the stage-one scan and payload hydration path now execute through that seam
   instead of reading directly from `SearchEngineContext`
3. the SQL builder for this lane now quotes engine-facing identifiers and
   table names so the same published parquet read path stays valid in both
   DataFusion and DuckDB
4. this earlier read-engine cut deliberately left build ownership out of scope;
   a later bounded storage-owner slice lands below

### 10.7 Intent Gateway Composition [landed in bounded form]

The next gateway-facing bounded slice is now landed for `/search/intent`
composition:

1. `/search/intent` remains a gateway aggregation surface rather than a new
   parquet-read owner
2. additive internal transport metadata now records query-engine labels for
   the `knowledge_section`, `local_symbol`, and repo-intent source lanes
3. the existing repo-content transport metadata remains in place for the
   Flight-backed repo source path
4. focused handler and Flight tests now prove that the route composes
   DuckDB-fed source lanes under `search.duckdb.enabled`
5. public response and Flight contracts, cache semantics, and merge behavior
   remain unchanged

### 10.8 Symbols Gateway Reads [landed in bounded form]

The next gateway-facing bounded slice is now landed for `/search/symbols`:

1. the route now starts and reads from the published `local_symbol` search
   plane instead of querying `UnifiedSymbolIndex::search_unified(...)`
2. the bounded adapter maps published `AstSearchHit` payloads back into the
   existing `SymbolSearchHit` contract without widening the public schema
3. the handler keeps the existing partial response semantics when no published
   local-symbol epoch is available yet
4. the route filters the broader `local_symbol` corpus back down to code
   symbol hits so the previous gateway payload shape remains stable
5. focused handler and Flight-provider tests now prove that `/search/symbols`
   can return DuckDB-fed symbol hits without warming the old in-memory symbol
   index

### 10.9 Local Symbol Build Ownership [landed in bounded form]

The next bounded storage-owner slice is now landed for `local_symbol`:

1. the `local_symbol` build owner now rewrites published partition tables
   directly to Parquet through a bounded local-publication helper instead of
   cloning and mutating Lance tables
2. local epoch discovery is now Parquet-only, so already-migrated
   local-symbol readers and gateway routes keep the same published contract
3. `local_symbol` no longer participates in local Lance compaction scheduling
   because it no longer owns a local Lance publication store
4. later bounded slices land `reference_occurrence` and `attachment` build
   ownership; `knowledge_section` remains future work

### 10.10 Reference Occurrence Build Ownership [landed in bounded form]

The next bounded storage-owner slice is now landed for `reference_occurrence`:

1. the `reference_occurrence` build owner now rewrites its published table
   directly to Parquet through the bounded local-publication helper instead of
   cloning and mutating a Lance table
2. the already-landed published read lane behind `/search/references` keeps
   the same contract because it was already reading the Parquet publication
3. `reference_occurrence` no longer participates in local Lance compaction
   scheduling because it no longer owns a local Lance publication store
4. a later bounded slice lands `attachment` build ownership; broader
   retirement for `knowledge_section` remains future work

### 10.11 Attachment Build Ownership [landed in bounded form]

The next bounded storage-owner slice is now landed for `attachment`:

1. the `attachment` build owner now rewrites its published table directly to
   Parquet through the bounded local-publication helper instead of cloning and
   mutating a Lance table
2. the already-landed published read lane behind `/search/attachments` keeps
   the same contract because it was already reading the Parquet publication
3. `attachment` no longer participates in local Lance compaction scheduling
   because it no longer owns a local Lance publication store
4. a later bounded slice lands `knowledge_section` build ownership

### 10.12 Knowledge Build Ownership [landed in bounded form]

The next bounded storage-owner slice is now landed for `knowledge_section`:

1. the `knowledge_section` build owner now rewrites its published table
   directly to Parquet through the bounded local-publication helper instead of
   cloning and mutating a Lance table
2. the already-landed published read lane behind gateway knowledge search
   keeps the same contract because it was already reading the Parquet
   publication
3. `knowledge_section` no longer participates in local Lance compaction
   scheduling because it no longer owns a local Lance publication store
4. knowledge intent/source merge orchestration remains future work

### 10.13 Local Epoch Discovery and Prewarm [landed in bounded form]

The next bounded local-publication compatibility slice is now landed:

1. local epoch discovery for search-plane corpora now ignores legacy `.lance`
   artifacts and only observes Parquet publications
2. local prewarm now rejects missing Parquet publications instead of falling
   back to opening a local store scan
3. focused construction and maintenance tests now prove that stale local
   `.lance` directories no longer keep search-plane read ownership alive

### 10.14 Local Compaction Runtime Retirement [landed in bounded form]

The next bounded local-maintenance retirement slice is now landed:

1. Wendao no longer ships a local compaction queue or worker runtime for
   search-plane corpora
2. `publish_ready_and_maintain(...)` now performs a pure publish step for
   local corpora instead of implying local compaction scheduling side effects
3. local maintenance runtime state is now shutdown-only, and runtime status
   annotation no longer projects local compaction backlog or running views
4. focused coordinator, maintenance, and status tests now keep local
   compaction metadata idle while preserving the repo-backed compaction
   status path

### 10.15 Qianji Audit and Reduce Stages

The next likely pilot is stage-local relation analytics over workflow-held
Arrow batches, especially:

1. `audit_step`
2. `reduce_step`
3. contradiction or consistency joins
4. explain-support rollups

These are relation-oriented workloads, but they still sit above retrieval and
storage ownership.

### 10.16 Repo and Runtime Diagnostics [partially landed in bounded form]

The first bounded diagnostics slice is now landed for the Studio search-index
status route:

1. top-level totals, phase counts, `compactionPending`, and aggregate
   maintenance summary now compute through a bounded local relation-engine
   helper over a request-scoped in-memory relation
2. the public `SearchIndexStatusResponse` payload remains unchanged, and the
   route falls back to the existing Rust summary path if local diagnostics
   execution fails
3. the same diagnostics helper now also rolls up `query_telemetry_summary`,
   including per-scope buckets, through the local relation-engine seam
4. the same diagnostics helper now also selects aggregate `status_reason`
   through the local relation-engine seam, preserving severity and code
   priority plus affected, readable, and blocking corpus counts
5. the same diagnostics helper now also maps top-level `repo_read_pressure`
   through the local relation-engine seam while preserving all optional repo
   gate fields
6. focused unit and route-level tests now prove the same payload under both
   fallback and DuckDB-enabled runtime policy
7. broader repo/runtime status, degraded-state diagnostics, and explain-facing
   status analytics remain future work

### 10.17 Wendao Parquet Query Engine Performance Gate [landed in bounded form]

The next bounded performance slice is now landed for the shared Parquet
execution seam:

1. Wendao now carries one deterministic synthetic Parquet benchmark under the
   shared performance harness
2. the same SQL workload now executes through both the DataFusion and DuckDB
   `ParquetQueryEngine` lanes over that identical fixture
3. the gate now emits durable perf reports and enforces a configurable
   DuckDB/DataFusion p95 ratio budget at the query-engine seam
4. first local evidence is favorable for DuckDB on that bounded workload, but
   broader performance-gate expansion remains future work

### 10.18 Wendao FlightSQL Statement Performance Gate [landed in bounded form]

The next bounded performance slice is now landed for the routed FlightSQL
statement surface:

1. Wendao now carries one routed FlightSQL statement benchmark under the
   shared performance harness
2. the benchmark uses a Julia parser-summary-aware gateway perf fixture so the
   same published repo-content source-table statement executes through both
   DataFusion and DuckDB over the already-landed statement seam
3. the gate now emits durable perf reports and enforces a configurable
   DuckDB/DataFusion p95 ratio budget at the routed FlightSQL statement seam
4. first local evidence is still favorable for DuckDB on that bounded
   workload, with `duckdb_p95_ms=180.863`, `datafusion_p95_ms=214.143`, and
   `ratio=0.845`
5. FlightSQL planning remains intentionally narrow: only the already-routed
   single-table statement surface is measured, and multi-source planning or
   new discovery ownership remains future work

### 10.19 Wendao FlightSQL Statement Latency Breakdown [landed in bounded form]

The next bounded follow-up slice is now landed for the same routed FlightSQL
statement surface:

1. the same benchmark now persists per-phase timing metadata into its durable
   reports, including a direct-engine lower bound plus bounded timings for
   `get_flight_info`, `do_get` collection, decode, and validation
2. current local evidence shows that the routed statement seam is dominated by
   `get_flight_info` statement-planning overhead rather than by DuckDB query
   execution itself
3. on the bounded rerun, the direct-engine lower bound was materially lower
   than the routed statement seam for both engines, with
   `datafusion_phase_direct_engine_p95_ms=27.127`,
   `duckdb_phase_direct_engine_p95_ms=2.620`,
   `datafusion_phase_get_flight_info_p95_ms=77.093`, and
   `duckdb_phase_get_flight_info_p95_ms=54.212`
4. `do_get` collection and decode stayed negligible on the same workload, so
   the absolute routed statement number should not be read as "DuckDB query
   execution took that long"
5. the slice stays bounded to performance evidence only: it does not widen
   FlightSQL planning, discovery ownership, or the published Parquet surface

### 10.20 Wendao Semantic SSOT Synchronization Lane

The next bounded pilot is the **Semantic SSOT Synchronization Lane**:

1. DuckDB acts as the high-performance "Active Semantic Index" for the
   repo-native SSOT layer (defined in `2026-05-03-repo-native-semantic-ssot-layer-rfc.md`).
2. YAML objects from `semantic/` are harvested and materialized into a
   `semantic_ssot` table.
3. Relations are materialized as a `semantic_relations` edge table.
4. This lane enables **SQL-based Invariants**, allowing Qianji guards to
   perform complex relational checks (e.g., recursive dependency trust
   validation) using standard SQL instead of script-based row traversal.
5. The synchronization is managed by a dedicated in-process watcher to ensure
   low-latency updates between Git writes and DuckDB availability.

## 11. Telemetry and Explain

The DuckDB lane must participate in the same explain discipline as the rest of
the stack.

Minimum execution metadata should include:

1. input batch count
2. input rows and bytes
3. registration time
4. SQL execution time
5. output rows and bytes
6. virtual versus materialized registration choice
7. spill or temp usage indicators

The current bounded Wendao pilot already reports input batch count, input
rows and bytes, registration time, local query execution time, output rows and
bytes, materialization state, and peak temp-storage bytes.

The causal narrative should remain explicit:

1. Wendao explains why a relation exists
2. DuckDB explains what bounded SQL happened over that relation
3. Qianji explains why a workflow stage used that relation

## 12. Gates

### 12.1 Functional Gates

Any later pilot must preserve:

1. unchanged Flight business contracts
2. correct Arrow schema roundtrips
3. explicit Valkey and vector ownership boundaries
4. reproducible Qianji stage outputs

### 12.2 Performance Gates

For a bounded pilot to expand, it should prove at least one of:

1. materially lower latency than the current implementation
2. materially lower peak memory
3. materially lower maintenance complexity while keeping comparable
   performance

### 12.3 Correctness Gates

Pilot outputs must remain auditable:

1. status and diagnostics queries must agree with current canonical surfaces
2. audit and contradiction joins must match current rule outputs
3. row counts, schema, and stage outputs must remain explainable

## 13. Risks and Revisit Triggers

### 13.1 Main Risks

1. two-engine complexity can create maintenance overhead
2. Arrow-friendly does not guarantee zero-copy in every path
3. bundled builds can increase build size or build time
4. scope creep can silently turn DuckDB into a storage-policy catch-all

### 13.2 Revisit Triggers

Revisit this direction if:

1. the first pilots fail to show a meaningful bounded-use benefit
2. the runtime/config burden outweighs the local analytics gain
3. later evidence suggests DataFusion alone is sufficient
4. a future shared crate becomes justified by real cross-package reuse

## 14. Rollout Phases and Current Status

### Phase 0: RFC and Boundaries [landed]

1. the canonical RFC, blueprint, ExecPlans, and nearest package-doc sync
   points are now present
2. the external Flight boundary and DataFusion-led shared query-core rule are
   explicit

### Phase 1: Narrow Relation-Engine Seam [landed in bounded Wendao form]

1. the bounded local relation-engine abstraction is present
2. `search.duckdb` runtime/config policy is landed with TOML-first precedence
3. current DataFusion paths remain intact

### Phase 2: Wendao Pilot [landed in bounded form]

1. the bounded-work markdown lane can execute through DataFusion or DuckDB
2. the request-scoped registration policy is real and engine-visible
3. additive bounded metadata now reports engine choice, rows, bytes, timing,
   materialization state, and peak temp-storage bytes
4. repo-backed `repo_entity` and `repo_content_chunk` publication reads now
   route through a bounded Parquet query-engine seam that selects DuckDB when
   `search.duckdb` is enabled and otherwise preserves DataFusion fallback
5. published `local_symbol` reads now reuse the same bounded Parquet
   query-engine seam for search, autocomplete, and payload hydration
6. published `reference_occurrence` reads behind `/search/references` now
   reuse the same bounded Parquet query-engine seam, with identifier-safe SQL
   generation for both DataFusion and DuckDB
7. published `attachment` reads behind `/search/attachments` now reuse the
   same bounded Parquet query-engine seam, again with engine-safe SQL
   generation for both DataFusion and DuckDB
8. published `knowledge_section` reads behind gateway knowledge search now
   reuse the same bounded Parquet query-engine seam, again with engine-safe
   SQL generation for both DataFusion and DuckDB
9. `/search/intent` now has a bounded composition proof that records internal
   source-lane query-engine metadata and proves the route composes DuckDB-fed
   `knowledge_section`, `local_symbol`, and repo-intent lanes without widening
   the public contract
10. `/search/symbols` now reuses the published `local_symbol` lane instead of
    the in-memory `UnifiedSymbolIndex`, while preserving the existing route
    contract and pending/indexing behavior
11. `local_symbol` build ownership now rewrites published partition tables
    directly to Parquet, uses Parquet-only local epoch discovery, and no
    longer participates in local Lance compaction scheduling
12. `reference_occurrence` build ownership now rewrites its published table
    directly to Parquet and no longer participates in local Lance compaction
    scheduling
13. `attachment` build ownership now rewrites its published table directly to
    Parquet and no longer participates in local Lance compaction scheduling
14. `knowledge_section` build ownership now rewrites its published table
    directly to Parquet and no longer participates in local Lance compaction
    scheduling
15. the Studio search-index status route now computes bounded diagnostics
    rollups through the local relation-engine seam while preserving the
    current payload contract and a Rust fallback path
16. local epoch discovery now ignores legacy `.lance` artifacts and local
    prewarm rejects missing Parquet publications instead of falling back to
    store scans
17. Wendao no longer ships a local compaction queue or worker runtime for
    search-plane corpora, while repo-backed compaction status remains intact
18. the Studio search-index status route now also rolls up
    `query_telemetry_summary` through the bounded diagnostics helper instead
    of the old pure-Rust accumulator
19. the Studio search-index status route now also selects aggregate
    `status_reason` through the bounded diagnostics helper instead of leaving
    that top-level priority rollup on pure Rust traversal
20. the Studio search-index status route now also maps top-level
    `repo_read_pressure` through the bounded diagnostics helper instead of
    leaving that field on direct Rust snapshot mapping
21. the Studio search-index status route now also marks
    `query_telemetry_rows` as a repeated-use request-scoped relation so
    DuckDB can prefer `MaterializedAppender` through the same bounded
    diagnostics helper
22. the Studio repo-index analysis Flight route now also rolls up its phase
    summary counts from per-repository rows through the bounded local
    relation-engine seam, with explicit `BIGINT` aggregate casts to keep the
    output Arrow type stable across DataFusion and DuckDB
23. the same Studio repo-index analysis Flight diagnostics relation now also
    recomputes `active_repo_ids` and `current_repo_id` from request-scoped
    rows, using an explicit `active_order` column plus repeated-use
    registration instead of copying those fields directly from the incoming
    response
24. the Studio `repo_index_status` JSON route now also reuses the same
    bounded diagnostics helper as the repo-index Flight route before
    serialization, so aggregate counts and active identity fields stay
    consistent across both surfaces without changing the JSON envelope or
    bootstrap telemetry
25. the current Wendao landing now explicitly records its ownership matrix:
    mutable runtime state stays in-process, shared cache stays Valkey-backed
    where enabled, published corpora stay Parquet, and DuckDB stays bounded to
    local execution over Arrow and Parquet relations
26. the current Wendao repo lane now also records its narrower ownership
    split explicitly: `repo_index` state is in-process, repo caches are
    in-memory plus Valkey-backed, repo publications are Parquet, and DuckDB is
    only a local execution lane over those publications
27. the current Wendao local corpus lane now also records its narrower
    ownership split explicitly: local publication ownership is Parquet-first,
    local epoch discovery is Parquet-only, and DuckDB remains only a local
    execution lane over those publications
28. the current Wendao mutable-state and shared-cache split is now also
    recorded explicitly: runtime state remains in-process, shared caches
    remain Valkey-backed where enabled, and DuckDB remains only an execution
    and bounded analytics lane
29. the current Wendao protocol-surface split is now also recorded
    explicitly: native Flight, bounded FlightSQL, and JSON routes remain only
    protocol adapters over Parquet publications, in-process state, and
    Valkey-backed caches
30. the Wendao performance suite now also carries one bounded
    `ParquetQueryEngine` gate that compares the same deterministic synthetic
    Parquet fixture through the DataFusion and DuckDB lanes, emits durable
    perf reports, and enforces a configurable DuckDB/DataFusion p95 ratio
    budget
31. the routed FlightSQL performance gate now also leaves the required
    `gRPCServer` runtime dependency under `WendaoSearch.jl`'s own live
    `run_search_service.jl` bootstrap, so the Rust harness no longer owns
    that live listener dependency and the package itself chooses between an
    explicit local override, a vendored checkout, or the official
    `gRPCServer.jl` `develop` branch source when it prepares the live env
32. the shared request-scoped SQL seam now also reuses the bounded
    published-Parquet routing helper directly, so simple single-table SQL
    queries over active local corpora and concrete repo publication source
    tables can execute through `ParquetQueryEngine`, while discovery
    catalogs, logical views, and multi-source queries still stay on the
    shared SQL fallback
33. publication readability helpers and readiness checks now also use
    Parquet/query-engine terminology instead of `DataFusion` terminology,
    while keeping the storage and execution behavior unchanged
34. a bounded naming cutover now exposes the retained `DataFusion`
    search-plane fallback on `SearchPlaneService` as
    `datafusion_query_engine()`, so parquet-routing call sites no longer
    present that fallback as a generic search owner
35. routed published-Parquet execution in `duckdb` builds no longer accepts a
    production `DataFusion` fallback context: eligible routed SQL, FlightSQL,
    and gateway publication reads now select DuckDB directly through
    `ParquetQueryEngine`, while non-`duckdb` builds retain the explicit
    baseline and shared discovery or logical-view fallback remains separate
36. the surviving shared SQL fallback is now also named explicitly as a
    request-scoped DataFusion query core, so discovery catalogs, logical-view
    assembly, and non-routed fallback execution no longer present themselves
    as a generic query owner
37. the bounded FlightSQL `CommandGetTables` discovery path now rebuilds
    `include_schema=true` payloads from `SqlQuerySurface.columns`, so the
    residual DataFusion owner line narrows to discovery-surface registration
    instead of discovery-schema lookup
38. a bounded FlightSQL discovery-surface follow-up now builds
    `CommandGetDbSchemas` and `CommandGetTables` from publication metadata
    plus logical-view contracts through one request-scoped `SqlQuerySurface`,
    so FlightSQL discovery no longer opens the residual DataFusion query core
39. a bounded shared-SQL routed-metadata follow-up now also builds metadata
    for eligible routed published-Parquet queries from the request-scoped
    `SqlQuerySurface`, so routed SQL no longer opens the residual DataFusion
    query core after Parquet execution just to recover discovery metadata
40. a bounded shared-SQL execution cutover now uses one request-scoped
    `SqlSurfaceAssembly` to register Parquet tables, logical views, and
    catalog batches into a DuckDB local relation core in `duckdb` builds, so
    non-routed shared SQL execution also stops using same-layer DataFusion on
    the DuckDB production path; the explicit DataFusion query core remains
    only as the non-`duckdb` baseline
41. broader performance gating and broader diagnostics pilots are still open

### Phase 3: Qianji Pilot [future]

1. pilot one audit/reduce-stage relation workload
2. wire stage-level explain and telemetry

### Phase 4: Expand or Hold [future]

Use the gates to decide whether to:

1. expand the DuckDB lane
2. keep it bounded to a few high-value pilots
3. stop at documentation and narrow local experiments

## 15. Final Decision

The final decision of this RFC is:

1. use `duckdb-rs` if DuckDB is adopted in this workspace
2. keep Arrow Flight as the Wendao external business boundary
3. keep Valkey as the cache and transient-state layer
4. keep the vector store as the embedding and ANN layer
5. add DuckDB only as a bounded in-process analytic lane
6. keep DataFusion as the current shared query core until later evidence says
   otherwise

## Appendix A: Current Bounded Dependency Set

The current bounded Wendao landing uses the following DuckDB dependency:

```toml
[dependencies]
duckdb = { version = "=1.10501.0", default-features = false, features = [
  "bundled",
  "appender-arrow",
] }
```

The current workspace Arrow baseline for this lane is `58.1.0`, and the
participating Wendao crates enable `arrow-flight` with `flight-sql`.

## Appendix B: One-Line Ownership Map

1. Arrow Flight: business protocol boundary
2. DuckDB: bounded in-process analytic lane
3. DataFusion: current shared query core
4. Valkey: hot cache and transient state
5. Vector store: embedding and ANN layer
6. Qianji: workflow orchestration
7. Wendao: retrieval, graph, and business semantics
