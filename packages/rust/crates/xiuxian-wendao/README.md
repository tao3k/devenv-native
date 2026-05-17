# 🌀 Wendao (问道)

**The Sovereign High-Performance Knowledge & Link-Graph Runtime.**

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Valkey](https://img.shields.io/badge/storage-Valkey-red.svg)](https://valkey.io/)
[![DuckDB](https://img.shields.io/badge/local%20cache-DuckDB-yellow.svg)](https://duckdb.org/)
[![LanceDB](https://img.shields.io/badge/vector%20opt--in-LanceDB-blue.svg)](https://lancedb.com/)
[![Arrow](https://img.shields.io/badge/protocol-Apache--Arrow-brightgreen.svg)](https://arrow.apache.org/)

**Wendao** is a next-generation knowledge management engine. While tools like Obsidian revolutionized human note-taking, **Wendao** is designed for the era of Autonomous Agents, providing a high-performance, programmable substrate for structured reasoning and massive-scale retrieval.

Default Wendao builds do not depend on `xiuxian-vector`, `xiuxian-lance`, or
LanceDB. SQL, document parsing, and Arrow Flight surfaces use the lightweight
`xiuxian-db-store/engine` boundary; Lance-backed storage remains available only
through the explicit `vector-store` feature for vector or retrieval-storage
paths.

## Package Position In The Split

The Wendao package family now has three ownership layers:

- `xiuxian-wendao-core`: stable shared contracts
- `xiuxian-wendao-runtime`: generic host behavior
- `xiuxian-wendao`: Wendao business domain and the remaining migration shell

`xiuxian-wendao` is therefore the product crate, not the place for every new
shared type or runtime helper.

When the `julia` feature is enabled, `xiuxian-wendao` now exposes one thin
memory-family bridge at `xiuxian_wendao::memory::julia`, while
`xiuxian-wendao-julia` continues to own the actual Julia-specific memory ABI
surface behind it. The product crate does not keep a second host-local profile
implementation layer, but it can now resolve the current
`memory.julia_compute` runtime and generic capability bindings through that
thin bridge.

The same feature boundary now covers Modelica repo-intelligence too:
`plugins = ["modelica"]` still selects the Modelica analyzer at runtime, but
the Rust implementation now lives on the Julia line in
`xiuxian-wendao-julia` instead of a standalone `xiuxian-wendao-modelica`
crate. The same Julia-owned line now also owns parser-summary transport
discovery for both languages, so plain `plugins = ["julia-code-parser"]` and
`plugins = ["modelica"]` repository config can resolve the standard
`WendaoSearch.jl` parser-summary endpoint without repo-local Rust
AST fallback or per-repository inline transport blocks.
The builtin plugin bundle now links that Julia-owned Julia plus Modelica line
by default as well, so builtin registry bootstrap no longer treats those two
repo-intelligence plugins as an optional second bundle.
The remaining `xiuxian-ast` dependency in `xiuxian-wendao` is now only for
the general local workspace languages that still use ast-grep or Python
tree-sitter. Julia and Modelica no longer have Rust-local AST ownership in
this crate.

Studio `code_search` now follows that same ownership rule end to end. The
production handler lives under
`src/gateway/studio/search/handlers/code_search/search/`, reuses the shared
`src/search/repo_search/` seam, and consumes Julia-owned native parser
publications for both `plugins = ["julia-code-parser"]` and `plugins = ["modelica"]`
repositories. The gateway no longer path-mounts a test-only code-search
implementation or keeps a second Rust-local Julia or Modelica AST route. The
focused gateway `code_search` proofs for plain Julia and plain Modelica plugin
repositories now also live on the default unit-test surface instead of hiding
behind a Julia-only test gate. The linked `studio_repo_sync_api` Modelica
proofs now follow the same rule: repo overview, module search, repo index,
repo sync, projected pages, planner, gap reports, and symbol search all
execute on the default unit-test surface through the Julia-owned native
parser routes, with environment drift normalized in the test seam instead of
reviving a Rust-local AST path.
The repo-backed Flight analysis route-wiring proofs now follow the same rule
too. `analysis/code-ast`, `analysis/repo-overview`,
`analysis/repo-doc-coverage`, `analysis/repo-projected-page-index-tree`, and
`analysis/refine-doc` now boot the linked Julia parser-summary service
through the shared repo fixture and run on the default unit-test surface
without a Julia-only gate.
That same shared fixture can now also target the canonical process-managed
`wendaocodeparser-parser-summary` parser-summary service when
`RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST=1` is set, so focused gateway
`code_search` and `code_ast` proofs can validate the formal `process.nix`
startup path instead of only the self-spawn test helper.
The same default-surface rule now covers the focused repo-intelligence
integration proofs that most directly back gateway search outcomes:
`repo_symbol_search` and `repo_overview` Modelica integration tests no longer
hide behind a Julia-only test gate, and their snapshots are pinned to the
native parser's current Modelica span contract.

Studio `code_ast` now follows the same rule. The repo-aware analysis loader
under `src/gateway/studio/router/handlers/analysis/service/code_ast.rs`
materializes repository analysis through the Julia-owned native parser line
for both plain Julia and plain Modelica plugin repositories, so the gateway no
longer needs a second Rust-local Julia or Modelica code-AST execution path.

### Memory-Family Julia Layering

The memory-family Julia lane follows the same ownership rule as the RFC:

1. `WendaoMemory.jl` owns high-performance memory compute only
2. `xiuxian-wendao-julia` owns the Julia-specific ABI surface: typed rows,
   request/response batches, manifest projection, schema validation, decoding,
   route defaults, and plugin-owned host-adapter helpers over Rust memory
   read models or evidence
3. `xiuxian-wendao` exposes only a thin host-facing bridge and delegates the
   actual Julia memory ABI work into `xiuxian-wendao-julia`
4. `xiuxian-memory-engine` and the Rust host remain authoritative for memory
   state, lifecycle, fallback, audit, and final mutation decisions

That means new memory-family profile semantics, schema fragments, manifest
logic, decoder logic, or Julia-specific validation rules should land in
`xiuxian-wendao-julia`, not in `xiuxian-wendao`.

The product crate's side of that boundary is now explicit:
`xiuxian_wendao::memory::julia` is the host-facing seam for

1. resolving merged `memory.julia_compute` runtime state,
2. materializing generic capability bindings from that runtime, and
3. constructing one configured `ComputeClient`, and
4. exposing only the minimum typed DTOs needed to call that client.

Host callers that only want configured memory-family Julia compute should stay
on `xiuxian_wendao::memory::julia::*`, with
`xiuxian_wendao::memory::julia::ComputeClient` as the primary host-facing
entry. Raw downcall helpers, runtime-to-binding builders, and gate-evidence
builder helpers remain plugin-owned in `xiuxian-wendao-julia` and are no
longer re-exported from the product crate.

Use `xiuxian-wendao` for:

- knowledge graph and link-graph behavior
- search, retrieval, and search-plane business semantics
- Valkey/Lance-backed storage behavior and DuckDB-backed local cache behavior
- Wendao event-lake schema and query semantics over the generic DuckLake
  substrate exposed by `xiuxian-db-store`; `WendaoEventLake` is the
  connection-light handle for attach, table setup, reusable appenders,
  chunked event-record ingestion, grouped count queries, and bounded filtered
  event-row reads with benchmark coverage for append and query throughput;
  event payloads are serialized once into compact JSON text at record
  construction so append and query hot paths avoid repeated JSON conversion;
  `WendaoEventLakeLocalConfig`
  resolves the local embedded path convention under
  `$PRJ_DATA_HOME/wendao/event_lake/`
- analyzers, enhancers, and other Wendao domain services
- DuckDB-backed dataset-to-ontology materialization for raw Arrow/Flight source
  tables, using source contracts from
  [`wendao-episteme`](../../../../wendao-episteme/README.md) and bounded SQL
  helper seams from
  [`xiuxian-wendao-sql`](../xiuxian-wendao-sql/README.md). The callable
  runtime seam is `duckdb::DatasetOntologyDuckDbMaterializer`; production
  Gateway or Flight upload routes must call this materializer rather than
  reimplementing DuckDB setup or source-contract SQL orchestration. The same
  seam can build requests from named Arrow IPC stream files emitted by
  source-contract tooling, keeping Python on validation and handoff only.
  The stable transport manifest and route constants live in
  [`xiuxian-wendao-runtime`](../xiuxian-wendao-runtime/README.md) under the
  ontology query-contract family.
- Episteme source-contract validation and run planning for
  customer-owned domain extensions. Source-contract parse and DTO shape are
  parser-owned in
  [`xiuxian-wendao-parsers`](../xiuxian-wendao-parsers/docs/episteme-source-contracts.md);
  this crate selects the active source manifest and mapping ledger from
  `ontology/manifest.toml`, then owns source hash validation, queue selection,
  persisted run-plan receipts, scheduling, cache identity, and compiled
  read-model seed materialization into `semantic_objects`,
  `semantic_relations`, and `semantic_projection_state` Arrow batches. Customer
  repository names, domain names, corpus-root environment variable
  names, source manifest paths, and mapping ledger paths are episteme
  configuration facts, not Rust defaults. A single-contract episteme repository
  can be selected automatically; multi-domain repositories must declare
  `[active_source_contract]`. The read-model seed contains
  source-contract facts and provenance only; it does not embed source corpus
  text or promote raw rows to RDF truth. When the `julia` feature is enabled,
  the compiled seed can be packaged into the existing WendaoGraph ontology
  read-model quality request shape, still as accepted read-model batches rather
  than TSV, RDF, or corpus parsing by Julia. Python tools in an episteme
  repository are diagnostic or analyzer adapters only; they do not own
  planning, promotion, or backend orchestration. The current operator
  entrypoints are the Studio-owned
  `wendao episteme structure write-toc` CLI command, which writes an
  evidence-only Org TOC ledger from source-contract file rows with
  route/category summaries ahead of the full file table, and
  `wendao episteme evidence write-selection-plan`, which writes an
  evidence-only selection ledger from chosen `file_id` values before any
  extractor execution, and `wendao episteme source-contract plan-extraction-run`
  CLI command, which can consume that selection ledger as a hard `file_id`
  constraint before writing extractor tasks. A bounded Studio Gateway admission
  endpoint exposes the same Rust writer. Episteme repositories may also provide
  `episteme.toml` with runtime defaults for corpus and run roots; Rust resolves
  those defaults generically, while explicit CLI or Gateway values remain
  overrides. Structure
  TOC generation defaults to metadata-only validation for fast human/LLM
  orientation and keeps full sha256 validation as an explicit `full-hash` gate.
  Extraction run planning uses `contract_shape_only` validation: it validates
  manifests, mapping ledger shape, queue/file consistency, filters, and selected
  `file_id` coverage without traversing or hashing the source corpus. Full
  sha256 proof remains the validation/read-model/promotion boundary, not the
  no-execution planning boundary. Image evidence enters this same admission
  path through the explicit `image_ocr_evidence` route: the planner writes
  cache-only planned OCR tasks and preserves `raw_to_rdf_promotion_allowed =
false`, while actual OCR execution remains a later Gateway/analyzer
  extraction step.
  The same source-contract service also exposes targeted evidence reads by
  `file_id`, returning bounded metadata and plain-text previews without
  arbitrary path reads, extractor execution, or ontology promotion.
  Full-audio transcript ledgers can enter this same graph-quality lane through
  `materialize_episteme_audio_evidence_review_seed`: the API accepts
  model-neutral audio evidence source and segment rows, emits review-required
  `semantic_objects`, `semantic_relations`, and `semantic_projection_state`
  batches, and validates source hashes, shard ids, transcript hashes, timing,
  and duplicate segment identity. Raw transcript text is validated as evidence
  input but is not embedded into emitted semantic objects, and the output is a
  review queue for WendaoGraph quality checks rather than RDF materialization
  or ontology truth promotion. Human-accepted audio claims can then enter the
  same read-model contract through
  `materialize_episteme_audio_reviewed_claim_seed`: reviewed claim rows must
  reference known audio evidence segment ids, carry reviewer and evidence-hash
  provenance, and emit promotion-candidate semantic objects plus
  claim-to-evidence relations for graph-quality review. This second gate still
  does not write ontology source files or promote transcript text directly to
  RDF truth. Accepted reviewed claims can also be written to local proposal
  artifacts with `write_episteme_audio_claim_promotion_proposal`, which emits
  deterministic reviewed-claim TSV and receipt JSON files for human or
  source-contract promotion review. These proposal artifacts carry ontology
  triple intent and evidence hashes only; they do not materialize RDF, mutate
  ontology source files, or include raw transcript text.
  Studio can also load episteme repositories from
  thin deployment registry entries:
  local entries use `path = "..."`, Git entries use `url = "..."`, and Rust
  owns source-kind inference, managed checkout materialization, resolved
  revision receipts, and cache paths. The primary user config does not require
  backend-shaped fields such as `source.kind`, `remote`, or `rev`. After
  loading the enabled registry entries, Rust builds a deterministic reference
  graph from each repository's `ontology/manifest.toml`: declared domain ids
  must be unique, and any manifest extension target must be owned by a loaded
  registry entry before a registry-selected run plan is admitted. A validated
  registry graph can also be compiled into the same `semantic_objects`,
  `semantic_relations`, and `semantic_projection_state` Arrow read-model
  tables used by WendaoGraph quality checks, representing registry entries,
  domains, ownership links, and extension links without requiring common
  ontology repositories to declare source-contract TSVs. The
  episteme source-contract test surface also includes an opt-in live diagnostic
  guarded by `RUN_EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_LIVE_TEST=1`;
  it requires `WENDAO_EPISTEME_SOURCE_CONTRACT_ROOT=<path>`, reads the selected
  source manifest's `corpus_root_env`, sends the compiled seed through the
  runtime Arrow Flight
  client, and writes cache-local evidence. That diagnostic is not a default
  test, not a production Gateway route, and not a semantic RDF promotion step.
  The diagnostic accepts `WENDAO_EPISTEME_SOURCE_CONTRACT_QUALITY_REPEATS=<n>` for
  bounded warm-service timing runs and
  `WENDAO_EPISTEME_SOURCE_CONTRACT_QUALITY_PREWARM_ROUNDS=<n>` for unmeasured prewarm
  quality exchanges before measured repeats. It can also target a prestarted
  quality service with
  `WENDAO_EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_BASE_URL=<url>`; if unset, the
  diagnostic still spawns a local Julia service. The live diagnostic can also
  opt into `WENDAO_EPISTEME_SOURCE_CONTRACT_VALIDATION_HASH_CACHE_PATH=<path>`; this
  cache is a Rust-owned validation accelerator only, and entries are used only
  when the current path metadata and recorded manifest SHA-256 still match.
  Default validation still performs full-file hashing.
- business handlers that materialize Wendao-specific responses
- temporary compatibility seams that have not been extracted yet

SearchStrategyFlow uses that same ownership split. The product crate owns the
structured search backend, including DuckDB-backed candidate retrieval where
configured. `xiuxian-wendao-julia` receives narrowed candidate batches for
Julia strategy/frontier selection; Julia does not own DuckDB access or full
candidate discovery. The domain-owned Flight host also prewarms bootstrap
projected pages and page-index trees, then serves projected page-index and
retrieval-context materialization from a host-local cache before falling back to
analysis-backed route construction.

Do not place new code here when it is better classified as:

- `xiuxian-wendao-core`: stable plugin/runtime contract records
- `xiuxian-wendao-runtime`: config resolution, transport negotiation, host
  assembly, and other deployment-dependent helpers
- `xiuxian-git-repo`: repository materialization, ghq-style managed layout,
  checkout locking, revision sync, and other generic repo substrate behavior

`xiuxian-wendao` no longer carries a `src/git/` compatibility facade. The
only Wendao-local repo bridge that remains is
`src/analyzers/repo_source.rs`, which maps `RegisteredRepository` config and
domain errors onto `xiuxian-git-repo` contracts. The crate itself no longer
depends on `git2`, and new generic repo substrate behavior should land in
`xiuxian-git-repo`, not back in Wendao.

Repo-index ownership is explicit too: implementation lives under
`src/repo_index/`, and `src/gateway/studio/` only consumes crate-level
repo-index adapters and bootstrap seams. The same owner-path rule applies to
search: `src/search/` is the only search implementation root.

Detailed repo-intelligence rollout notes, repo-index performance proofs, and
cache/runtime evolution now belong in the package docs and GTD tracking
surfaces instead of this README. Local CLI/audit link-graph bootstrap now uses
a DuckDB-backed snapshot cache under the project cache root; Valkey remains the
explicit shared runtime cache path for gateway/service-backed link-graph cache
use. The local DuckDB cache stores the high-cardinality link-graph core as
native Arrow IPC streams and keeps a bounded page-index residual so warm hits
avoid rebuilding every page-index tree. The page-index residual remains a
single bounded payload until a measured columnar form beats it; cache schema
validity is tracked by fingerprint rather than rolling local table versions.
Warm-hit lookup opens existing local cache files with DuckDB read-only access
and lets the prepared payload SELECT validate the stable table shape before
reading streams; cache writes still own table creation and
incompatible-table rebuilds through the normal DuckDB runtime path.
The local cache performance gate records cold build, miss/write, hit/read,
native Arrow core-stream timings, and a hit-vs-cold p95 ratio gate. Use
[`docs/06_roadmap/402_repo_intelligence_mvp.md`](docs/06_roadmap/402_repo_intelligence_mvp.md)
for the active repo-intelligence lane and
[`docs/06_roadmap/417_wendao_package_boundary_matrix.md`](docs/06_roadmap/417_wendao_package_boundary_matrix.md)
for the canonical boundary matrix.

---

## 💎 Why Wendao? (The Obsidian Leap)

Wendao moves beyond the limitations of traditional bi-link tools by introducing **Topological Sovereignty**:

| Feature         | Obsidian (Human-Centric)    | **Wendao (Agent-Centric)**                        |
| :-------------- | :-------------------------- | :------------------------------------------------ |
| **Structure**   | Flat Bi-links & Folders     | **Hierarchical Semantic Trees (PageIndex)**       |
| **Retrieval**   | Simple Search / Dataview    | **Quantum Fusion (Vector + Graph + PPR)**         |
| **Scale**       | Electron / Local Filesystem | **Rust Core / LanceDB / Valkey Cluster**          |
| **Context**     | Manual "Maps of Content"    | **Automated Ancestry Uplink (Zero-loss context)** |
| **Performance** | Sequential scanning         | **Arrow-Native Zero-Copy (15x throughput)**       |

---

## 🚀 Key Evolutionary Features

### 1. PageIndex Rust Core (Hierarchical Indexing)

Unlike Obsidian's flat structure, Wendao builds a recursive **Semantic Tree** of your documents. It understands the logical hierarchy (Root > Chapter > Section), allowing agents to navigate complex long-form content with "God's eye" perspective.

### 2. Quantum Fusion (Hybrid Retrieval)

Fuses fuzzy **Vector Search** (semantic intuition) with precise **Graph Diffusion** (logical reasoning). Using a neurobiologically inspired **PPR algorithm** (Personalized PageRank), Wendao finds not just "similar" text, but "logically relevant" knowledge clusters.

### 3. Apache Arrow Flight

Built on top of the **Arrow Data Ecosystem**. Knowledge flows through the engine as columnar memory batches. This ensures **Zero-copy** overhead during retrieval, re-ranking, and injection, making it capable of handling millions of nodes at sub-millisecond latency.

The business boundary is now pure Arrow Flight. The outward HTTP business
surfaces `/api/search/{intent,attachments,references,symbols,ast}` and
`/api/analysis/{markdown,code-ast}` are retired.

The canonical business contracts are:

- `/search/intent`
- `/search/attachments`
- `/search/references`
- `/search/symbols`
- `/search/ast`
- `/analysis/markdown`
- `/analysis/code-ast`

The stable runtime metadata keys are:

- `x-wendao-search-query`
- `x-wendao-search-limit`
- `x-wendao-analysis-path`
- `x-wendao-analysis-repo`
- `x-wendao-analysis-line`

### Bounded Work Markdown SQL Surface

`xiuxian-wendao-sql` now owns the bounded-work markdown SQL helper, while
`xiuxian-wendao` keeps a compatibility re-export at
`xiuxian_wendao::search::queries::sql::bounded_work_markdown`.

That helper scans only the `blueprint/` and `plan/` surfaces of a bounded
workdir, builds the RFC-minimum `markdown` rows (`path`, `surface`,
`heading_path`, `title`, `level`, `skeleton`, `body`), and registers them as
an in-memory DataFusion table. The same surface now also provides one opt-in
bootstrap helper that creates a fresh query engine and registers the bounded
`markdown` table in one step, plus one stable payload helper that executes SQL
directly over that bounded-work surface. This slice remains intentionally
local: it does not yet wire the `markdown` table into the global Wendao SQL
surface.
collector.

The bundled OpenAPI artifact now keeps only the JSON control plane for these
families. The semantic Flight search contract is pinned by
`tests/snapshots/gateway/studio/search_flight_service_route_contracts.snap`.
`/api/ui/capabilities` no longer advertises a browser search Arrow transport,
because there is no remaining browser-facing search Arrow business surface.

The same `analysis/` folder is now also split by lane internally:

- `analysis/service/markdown.rs`
- `analysis/service/code_ast.rs`
- `analysis/types/markdown.rs`
- `analysis/types/code_ast.rs`

This keeps the canonical Flight routes unchanged while removing the remaining
mixed `service.rs` and `types.rs` internals.

The old `analysis_exports.rs` forwarding hop is also gone now. Handler-level
barrels re-export the canonical analysis routes directly from the `analysis`
feature folder instead of bouncing through a second legacy file.

The same cleanup pattern has now started on the remaining handler families:
the old `capabilities_exports.rs` forwarding hop is gone, and
`handlers/mod.rs` re-exports `get_ui_capabilities` and `get_plugin_artifact`
directly from the `capabilities` feature folder.

The old UI-config handler hop is now retired entirely. The runtime still
bootstraps `UiConfig` from backend-owned config imports, but `/api/ui/config`
is no longer part of the outward Studio route surface.

The same cleanup now also covers VFS: `vfs_exports.rs` is gone, and
`handlers/mod.rs` re-exports `vfs_root_entries`, `vfs_scan`, `vfs_cat`,
`vfs_resolve`, and `vfs_entry` directly from the `vfs` handler module. During
that cleanup, the checked-in bundled OpenAPI artifact was also brought back
into sync with the declared route inventory by restoring the generic plugin
artifact inspection path.

The same direct re-export pattern now also covers graph handlers:
`graph_exports.rs` is gone, and `handlers/mod.rs` re-exports
`graph_neighbors`, `node_neighbors`, and `topology_3d` directly from the
`graph` feature folder.

Current fit audit for the remaining search-family Flight contracts is explicit:

- `references`: fits the generic Flight search seam (`query + limit`)
- `symbols`: fits the generic Flight search seam (`query + limit`)
- `attachments`: does not fit the generic seam because it requires `ext`,
  `kind`, and `case_sensitive`, so it now uses its own dedicated attachment
  Flight provider seam instead
- `ast`: not a clean generic-seam candidate; it is tied to the local
  symbol-index path, so the canonical surface is the dedicated Flight route
  `/search/ast` backed by its own AST provider seam

That audit result is now reflected in the runtime substrate. The generic
Flight search matcher no longer claims `/search/attachments` or `/search/ast`.
`xiuxian-wendao-runtime` now exposes a dedicated attachment-search Flight
contract with explicit metadata headers for `ext`, `kind`, and
`case_sensitive`, plus a separate `AttachmentSearchFlightRouteProvider`
boundary, and a separate `AstSearchFlightRouteProvider` boundary for the
local symbol-index path. The semantic search contract is now locked by the
Studio Flight snapshot suite rather than by browser Arrow IPC routes.

The Studio Flight handler itself now follows the same feature-folder rule.
`src/gateway/studio/search/handlers/flight/mod.rs` is the interface seam,
while the implementation is split into:

- `flight/provider.rs`: aggregate search provider dispatch
- `flight/service.rs`: route-provider wiring into `WendaoFlightService`
- `flight/tests/*`: fixtures, request-header helpers, contract snapshots, and
  route-family tests

The same query modularization lane also moved the Studio SQL/DataFusion
provider internals into `queries/sql/mod.rs` and the search query DTOs into
`handlers/queries/mod.rs`, so the remaining query families no longer share one
flat handler file or a root `.rs` barrel.

The next search-boundary cleanup is now explicit too: native Flight remains the
Wendao business protocol surface, while SQL, FlightSQL, GraphQL, REST-style
query entrypoints, and the future CLI `query` command should all consume one
shared `queries/` system over the same request-scoped SQL execution seam. The
canonical architecture note is
[`docs/03_features/210_search_queries_architecture.md`](docs/03_features/210_search_queries_architecture.md).

The first non-Flight consumer under that architecture is now the CLI adapter:
`src/bin/wendao/.../query.rs` renders results from the shared
`queries/sql/` execution seam rather than introducing a second planner path or
calling a Flight-only provider abstraction directly.

The next non-SQL adapter slice is now landed too: GraphQL stays folder-first
under `search/queries/graphql/` and behaves like a table-query frontend that
translates GraphQL documents into SQL text over the request-scoped SQL
surface instead of growing a GraphQL-only business DSL or a GraphQL-local
dataframe planner. The CLI now exposes the same adapter directly with
`wendao query graphql --document ...`.

The next protocol adapter slice is landed too: FlightSQL now lives under
`search/queries/flightsql/` as a dedicated server over the same
request-scoped SQL execution seam. The first cut is intentionally
narrow: `CommandStatementQuery`, minimal `CommandGetSqlInfo`, one shared-query
service builder, and `wendao_search_flightsql_server`, without widening the
native Wendao business Flight router.

The next shared-SQL cutover is landed too: the canonical SQL execution seam
now reuses the same published-Parquet routing helper for simple single-table
queries over active local corpora and concrete repo publication source tables.
That means SQL, FlightSQL, and GraphQL-to-SQL can all hit the same bounded
`ParquetQueryEngine` lane for eligible table queries. In `duckdb` builds that
routed published-Parquet lane is now DuckDB-owned, while discovery catalogs,
logical views, and multi-source statements still stay on the shared fallback
path.

The next residual naming cutover is landed too: the remaining shared fallback
now names itself explicitly as a request-scoped DataFusion query core instead
of a generic query core, so the surviving owner line for discovery catalogs,
logical views, and non-routed execution is visible in code and docs.

The next bounded FlightSQL discovery cutover is landed too:
`CommandGetDbSchemas` and `CommandGetTables` now build the request-scoped
discovery surface directly from publication metadata and logical-view
contracts instead of opening the residual DataFusion query core. Meanwhile
`include_schema=true` still rebuilds table schemas from
`SqlQuerySurface.columns`, so the public FlightSQL discovery contract stays
stable while the residual DataFusion owner line no longer includes FlightSQL
discovery itself.

The next bounded shared-SQL follow-up is landed too: when an eligible SQL
statement routes directly to published Parquet, the shared SQL seam now builds
its response metadata from the same request-scoped `SqlQuerySurface` instead
of opening the residual DataFusion query core just to recover discovery
metadata. Non-routed discovery, logical-view, and multi-source fallback still
stay on the shared DataFusion core.

The next shared-SQL execution cutover is landed too: in `duckdb` builds the
non-routed shared SQL branch now builds one request-scoped
`SqlSurfaceAssembly`, registers Parquet tables, logical views, and catalog
batches into a DuckDB local relation core, and executes discovery-catalog and
logical-view queries there as well. That means GraphQL-to-SQL, shared SQL, and
FlightSQL statement fallback no longer keep a same-layer DataFusion execution
role on the DuckDB production path. The explicit DataFusion query core remains
only as the non-`duckdb` baseline.

The canonical shared-query implementation owner is now `src/search/queries/`.
The former gateway-side `src/gateway/studio/search/queries/` shadow tree is
retired; native Flight now imports the canonical adapters directly instead of
keeping a second query namespace alive.

The GraphQL and FlightSQL adapter tests now live with their canonical adapter
implementations under `src/search/queries/{graphql,flightsql}/tests/`.
Gateway-side `search/queries/graphql/` and `search/queries/flightsql/` are no
longer long-lived test homes; they should exist only when a real stable bridge
namespace is still needed.

The same ownership rule now applies to SQL too: the canonical SQL regression
suite now lives under `src/search/queries/sql/tests/`. The former
gateway-side `search/queries/` test home is retired, and native Flight wiring
imports the canonical SQL provider directly.

The shared request-scoped query core is landed too:
`src/search/queries/core/` now owns the shared request-scoped query core.
SQL, GraphQL, and FlightSQL open that one canonical
`SearchEngineContext + SqlQuerySurface` seam instead of each calling the
low-level SQL surface-registration helper directly.

The next ownership tightening is landed too: canonical adapters and CLI
entrypoints now consume one `SearchQueryService` above that shared query core
instead of carrying raw `SearchPlaneService` ownership independently.

The next thin frontend slice is landed too: `search/queries/rest/` now acts
as a REST-style adapter over that same `SearchQueryService`. The first bounded
surface stays request/response-only and is currently proven by
`wendao query rest --payload ...`, so REST still consumes the shared query
system instead of introducing a new planner path or widening native business
HTTP routes.

Snapshot-level regression coverage is now mandatory for every canonical
adapter under `src/search/queries/*`, not just for SQL. GraphQL, FlightSQL,
and REST now keep adapter-local snapshot suites beside their canonical tests.
Those shared-query baselines now live under the canonical
`tests/snapshots/search/queries/` tree, while
`tests/snapshots/gateway/studio/` stays reserved for gateway-owned route
contracts and business payloads.

Repeated shared-query test support now lives under `src/search/queries/tests/`,
while protocol-specific decode helpers remain adapter-local. GraphQL, REST,
and FlightSQL now reuse one canonical corpus-fixture seam instead of each
keeping a private copy of the same search-plane setup.

The next gateway-downpressure slice is landed too: repo-content business
search execution now lives under `src/search/repo_search/`, so both native
Flight repo-search and code-search consume the same shared seam without
treating a transport provider as the canonical execution owner.

The next bounded downpressure slice is landed too: repo-entity execution and
relation-to-`SearchHit` shaping now live under the same
`src/search/repo_search/` feature folder, so gateway code-search no longer
owns the canonical repo-entity execution flow and the knowledge-intent path
reuses the same shared seam through the thin gateway wrapper.

The next bounded downpressure slice is landed too: repo dispatch planning
ownership now lives under `src/search/repo_search/dispatch.rs`. Shared
`RepoSearchTarget`, target partitioning, and parallelism selection are no
longer owned by gateway `code_search/query.rs`.

The next bounded downpressure slice is landed too: buffered repo-search queue
draining, spawn policy, and repo-level query execution now live under
`src/search/repo_search/`. Gateway `knowledge/merge.rs` and code-search both
consume the same shared buffered owner, and the old
`code_search/search/{buffered,task}.rs` modules are retired.

The next bounded downpressure slice is landed too: shared repo response
orchestration now lives under `src/search/repo_search/orchestration.rs`.
Publication-state lookup, dispatch telemetry, and repo-search
partial/pending/skipped state assembly are no longer duplicated across
`knowledge/merge.rs` and the code-search response path.

The remaining gateway-local boundary is now explicit: Studio config-driven
repo-id resolution, cache policy, and final response DTO shaping still remain
under the Studio gateway layer by design.

The next bounded FlightSQL metadata slice is discovery:
`CommandGetCatalogs`, `CommandGetDbSchemas`, and `CommandGetTables` over one
stable logical catalog name (`wendao`) and schema names derived from the
shared SQL registration scope, still without introducing a second catalog or
planner model.

The SQL handler now follows the same folder-first rule at a finer granularity:

- `sql/provider/mod.rs`: interface seam for the SQL Flight provider
- `sql/provider/route.rs`: query execution plus Flight batch/materialization
- `sql/provider/metadata.rs`: SQL app-metadata payload shape
- `sql/registration/mod.rs`: interface seam for table registration
- `sql/registration/collector.rs`: request-scoped SQL surface assembly
- `sql/registration/naming.rs`: stable SQL-facing table naming policy
- `sql/registration/session.rs`: request-scoped DataFusion session config
- `sql/registration/table.rs`: registered table/column descriptors
- `sql/registration/local.rs`: local corpus parquet registration
- `sql/registration/repo.rs`: repo publication parquet registration
- `sql/registration/views/{mod,local,repo}.rs`: request-scoped local/repo
  logical views
- `sql/registration/catalog/{tables,columns,view_sources}.rs`: stable
  `wendao_sql_tables`, `wendao_sql_columns`, and
  `wendao_sql_view_sources` system catalogs
- `sql/tests/provider.rs`: SQL execution plus app-metadata coverage
- `sql/tests/catalog.rs`: SQL table/column catalog coverage
- `sql/tests/information_schema.rs`: standard SQL introspection coverage
- `sql/tests/local_symbol/mod.rs`: local-symbol logical-view coverage
- `sql/tests/repo/{mod,content_chunk,entity}.rs`: repo-backed SQL alias,
  logical-view, and view-source coverage
- `sql/tests/snapshots.rs`: snapshot baselines for the SQL query surface and
  discovery catalogs

That SQL lane now also exposes a request-scoped discovery surface instead of
only opaque engine table names. The SQL provider builds a fresh
`SearchEngineContext` per request, registers the currently readable search-plane
tables into that context, then registers two stable system catalogs:

- `wendao_sql_tables`: queryable table inventory for the current request
  including `sql_object_kind` (`table` / `view` / `system`) and `source_count`
- `wendao_sql_columns`: column inventory for every queryable table in the
  current request, including `source_column_name`, `sql_object_kind`, and
  `column_origin_kind`
- `wendao_sql_view_sources`: logical-view membership for every request-scoped
  SQL view that unions repo-backed aliases

The returned SQL app metadata now reports all three catalog table names plus
the registered table/view/column/view-source counts, so clients can bootstrap
discovery without guessing from internal engine table naming.

The same request-scoped SQL engine now also enables DataFusion
`information_schema`, so clients can use standard SQL discovery routes such as
`information_schema.tables` and `information_schema.columns` in addition to the
Wendao-specific catalogs. The SQL app metadata exposes
`supportsInformationSchema = true` to advertise that standard introspection
surface.

The next internal ownership rule is now explicit too: SQL, GraphQL, and
FlightSQL should not each assemble the request-scoped query surface on their
own. That surface belongs to one shared query-core seam under `src/search/`
so adapter code only handles protocol translation and result shaping.

That naming cleanup is now landed too: now that the canonical
implementation, tests, and snapshots live under `src/search/queries/`,
query-owned core, execution, request, response, and payload names now use
neutral shared-query naming. Explicitly Studio-owned gateway
transport/provider names remain a separate boundary.

The local corpus side of that SQL surface now also prefers stable SQL-facing
table aliases over epoch-publication engine names. For example, the current
reference-occurrence table is queryable as `reference_occurrence`, while the
catalog still preserves the underlying `engine_table_name` for debugging and
internal mapping.

The same request-scoped SQL layer now also exposes a stable `local_symbol`
logical view on top of the currently readable local-symbol tables. This gives
SQL clients one canonical surface for local symbol search data even when the
underlying corpus is partitioned into multiple local tables.

The repo-backed side now follows the same rule with stable repo-scoped aliases.
Instead of exposing publication ids as the primary SQL surface, the provider
registers repo corpora under stable repo table names such as
`SearchPlaneService::repo_content_chunk_table_name(repo_id)` and
`SearchPlaneService::repo_entity_table_name(repo_id)`, while the catalog keeps
the publication-derived `engine_table_name` for traceability.

The request-scoped repo logical views now also expose their source composition
through `wendao_sql_view_sources`, ordered by `repo_id`, so clients can
inspect which stable repo aliases feed a logical SQL view before querying it.
The same `wendao_sql_tables` catalog now also reports that those objects are
SQL `view`s and records each logical view's source fan-in directly through
`source_count`, so clients can inspect view shape without joining the
view-source catalog first.

The column catalog now mirrors that same discovery goal one level deeper.
`wendao_sql_columns` distinguishes stored base-table columns from logical-view
projections and synthetic columns. For example, the injected repo logical-view
`repo_id` column is marked synthetic with no `source_column_name`, while the
other logical-view columns stay projected back to their source column names.

That logical-view lane now spans both local and repo-backed corpora. The
stable view name `local_symbol` aggregates the currently readable local-symbol
tables, while `repo_content_chunk` and `repo_entity` aggregate the currently
readable repo aliases and inject a synthetic `repo_id` column, so SQL clients
can query across partitions or repos without first discovering every physical
source table.

The real-repository search precision harness is documented in
[`docs/03_features/216_real_repo_search_precision.md`](docs/03_features/216_real_repo_search_precision.md).
The README keeps only the crate-level map: the harness stays opt-in, reuses
the existing LinkGraph and repo-AST search contracts, writes cache-local
receipts, and validates both this repository and optional external
orchestration sources such as [`pi-wendao`](https://github.com/tao3k/pi-wendao)
without committing runtime checkouts. The feature note records the latest
real-validation reflection: graph-first reasoning-tree search is strongest for
evidence-rich knowledge tasks, while simple known-item or small-repository
queries should remain eligible for cheaper profile routing.

The SQL lane now also keeps snapshot-level regression coverage through
`src/search/queries/sql/tests/snapshots.rs`, with canonical baselines under
`tests/snapshots/search/queries/`. That namespace now belongs to the shared
query system rather than the legacy gateway snapshot tree.

The SQL docs now also record concrete query expressions instead of only naming
the exposed tables. The canonical note includes copyable discovery queries for
`wendao_sql_tables`, `wendao_sql_columns`, and `wendao_sql_view_sources`,
standard `information_schema` examples, plus stable logical-view queries for
`local_symbol`, `repo_content_chunk`, and `repo_entity`.

The canonical feature note for this lane now lives at
[`docs/03_features/209_datafusion_sql_query_surface.md`](docs/03_features/209_datafusion_sql_query_surface.md).

The canonical REST adapter note now lives at
[`docs/03_features/213_rest_query_surface.md`](docs/03_features/213_rest_query_surface.md).

That lane now extends to the search-family handlers themselves. The old flat
`ast.rs`, `attachments.rs`, `autocomplete.rs`, `definition.rs`,
`references.rs`, and `symbols.rs` modules are gone. Each family now owns a
dedicated folder with an interface-only `mod.rs` seam plus responsibility-split
leaf modules such as `batch.rs`, `provider.rs`, `response.rs`, `path.rs`,
`matcher.rs`, or `tests.rs` as required by the family contract.

The same folder-first rule now also covers the large Studio repo gateway
contract suite. `tests/unit/studio_repo_sync_api/` is the new feature folder,
with `mod.rs` acting as the seam and query-specific coverage moving into
`gap_reports.rs` and `planner.rs` instead of keeping those contracts inside one
flat test file.

The same query-folder pressure inside the search plane is now also reduced for
local symbol retrieval. The canonical search-plane owner path is now
`src/search/`, and the old `src/search_plane/` module has been retired.
`search/local_symbol/query/` no longer keeps
`autocomplete.rs`, `search.rs`, and one flat `tests.rs` beside each other.
Instead it is split into:

- `local_symbol/query/autocomplete/mod.rs` and `autocomplete/route.rs`
- `local_symbol/query/lookup/mod.rs` and `lookup/route.rs`
- `local_symbol/query/shared/readiness.rs` for shared read-table preparation
- `local_symbol/query/tests/{fixtures,search,autocomplete}.rs`

The same search-plane rule now also covers reference occurrences.
`search/reference_occurrence/query/` no longer mixes `candidates.rs`,
`decode.rs`, `search.rs`, and one flat `tests.rs` in the root. It is now split
into:

- `reference_occurrence/query/lookup/mod.rs`
- `reference_occurrence/query/lookup/{route,candidates,decode,helpers}.rs`
- `reference_occurrence/query/tests/{fixtures,search}.rs`

The same rule now also covers knowledge sections. The old flat
`search/knowledge_section/query/{candidates,errors,ranking,search,tests}.rs`
set is gone. It is now split into:

- `knowledge_section/query/lookup/mod.rs`
- `knowledge_section/query/lookup/{route,candidates,error,helpers}.rs`
- `knowledge_section/query/tests/ranking.rs`

The same rule now also covers attachments. The old flat
`search/attachment/query/{scan,scoring,search,types,tests}.rs` set is
gone. It is now split into:

- `attachment/query/lookup/mod.rs`
- `attachment/query/lookup/{route,scan,scoring,decode,helpers,types}.rs`
- `attachment/query/tests/{fixtures,ranking,search}.rs`

The same rule now also covers repo entities. The old flat
`search/repo_entity/query/{execution,prepare,search,types,tests}.rs` set
is gone. It is now split into:

- `repo_entity/query/lookup/{mod,execution,prepare,route,types}.rs`
- `repo_entity/query/results/{mod,module,symbol,example,import,shared}.rs`
- `repo_entity/query/tests/{fixtures,ranking,results}.rs`

The same rule now also covers repo content chunks. The old flat
`search/repo_content_chunk/query/{candidates,error,execution,helpers,scan,search,tests}.rs`
set is gone. It is now split into:

- `repo_content_chunk/query/lookup/{mod,candidates,error,execution,helpers,route,scan}.rs`
- `repo_content_chunk/query/tests/ranking.rs`

---

## 📚 Theoretical Foundation (2025-2026)

Wendao is physically grounded in cutting-edge RAG research:

- **LightRAG (2025)**: Dual-level indexing (Logical + Entity).
- **RAGNET (Stanford 2025)**: End-to-end training for neural graph retrieval.
- **Columnar Knowledge Streams (2026)**: Zero-copy Arrow transport for scaling.

---

## 🛠 Architecture

- **Kernel**: Pure Rust (Tokio / Rayon)
- **Hot Cache**: Valkey (In-memory graph adjacency and saliency scores)
- **Cold Storage**: LanceDB (Persistent vector anchors and Arrow fragments)
- **Protocol**: Apache Arrow (Universal knowledge transport layer)

### Julia Arrow Adapter

The Julia Arrow rerank exchange is plugin-owned in
`xiuxian-wendao-julia`. Typed request rows, typed score rows, request-batch
assembly, response decoding, repository fetch helpers, and graph-structural
transport helpers now live in the plugin crate instead of under
`xiuxian-wendao::analyzers`.

`xiuxian-wendao` still re-exports the common WendaoArrow request or response
column-name constants and shared Arrow schema builders. For the memory-family
Julia lane, host callers should prefer the thin bridge at
`xiuxian_wendao::memory::julia`; anything below that bridge still belongs in
`xiuxian-wendao-julia`.

The memory-family Julia compute lane follows the same rule. Julia-specific
memory profile contracts, manifest semantics, validators, decoders, route
defaults, host staging, transport, and composed downcalls belong in
`xiuxian-wendao-julia`. `xiuxian-wendao` now keeps only the thin
`memory::julia` bridge that points at those plugin-owned surfaces.

For LinkGraph-to-WendaoGraph evidence, `xiuxian-wendao` provides the local
`link_graph::wendao_graph_evidence` adapter. The link-evidence bundle maps
`LinkGraphIndex` document links plus optional diffusion seeds and
either `semantic_neighbors` rows or host-precomputed `semantic_overlay` rows
into the existing `/graph/link/evidence` request tables by reusing the
`xiuxian-wendao-julia` contract mirror. The semantic paths validate node
identity, one-based vertex indices, rank, distance, overlay weight, and edge
kind before Arrow batch construction so Julia can derive `semantic_overlay`,
`diffusion_scores`, and `link_frontier` deterministically. The two semantic
input variants are mutually exclusive, matching the WendaoGraph.jl request
contract.
This is still transport-neutral: the adapter does not call Julia, does not add
a Flight route, and does not change search ranking behavior.

The WendaoGraph PageIndex reasoning path is separate. The same adapter module
also exposes a PageIndex sidecar builder that materializes `page_index_nodes`,
`page_index_edges`, and schema-stable `page_index_seeds` from `LinkGraphIndex`
for future WendaoGraph service integration. These PageIndex reasoning tables
are not mixed into `/graph/link/evidence`.

The same adapter now also exposes a semantic SSOT projection path for agent
reasoning-tree work. `xiuxian-wendao` projects parser-owned
`SemanticScopeBundle` facts into the existing PageIndex request tables so
WendaoGraph.jl can compute derived `reasoning_frontier`, `disclosure_trace`,
and planner-action evidence. The projection preserves semantic object ids,
relation kinds, source paths, status, confidence, owner, provenance, and
validation hints inside the PageIndex request rows. Julia does not own or
mutate SSOT authority; Rust validates the projection before any Julia dispatch.
An opt-in live proof materializes the semantic projection as temporary
PageIndex TSV fixture files and runs the real WendaoGraph.jl host entrypoint,
so this path is validated beyond schema-only tests while remaining
transport-neutral.
The stable design reference is
[`docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md`](../../../../docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md).

That boundary is backed by the git-stable host fixture under
`packages/rust/crates/xiuxian-wendao/tests/fixtures/wendaograph_page_index_reasoning_host`.
Rust tests compare the fixture against the PageIndex sidecar builder, and
WendaoGraph.jl can read the same fixture through its native PageIndex contract
and reasoning tests. The fixture is interop evidence only; it does not add a
Flight route or change runtime fallback behavior.

WendaoGraph.jl now also exposes `page_index_reasoning_from_request(...)` as a
host-request facade for those sidecar-shaped tables. Host callers can pass a
request object containing `page_index_nodes`, `page_index_edges`, and optional
`page_index_seeds`; table payloads supplied through keywords are rejected so
the transport boundary stays explicit.

For link-graph semantic retrieval, `VectorStoreSemanticIgnition` now also
provides `build_julia_rerank_request_batch(...)`, which reuses anchor ids as
the stable request-row identity and assembles a Julia-ready Arrow batch from
`QuantumAnchorHit` values plus the current query vector.

`OpenAiCompatibleSemanticIgnition` now exposes the same
`build_julia_rerank_request_batch(...)` surface. It resolves the effective
query vector from either an explicit `query_vector` or an
OpenAI-compatible embedding call, then builds the canonical WendaoArrow `v1`
request batch from the resulting anchors and stored embeddings.

For the link-graph runtime, `link_graph.retrieval.julia_rerank` is now the
dedicated config namespace for the future WendaoArrow post-processing step.
The runtime currently resolves `base_url`, `route`, `health_route`,
`schema_version`, and `timeout_secs`, and planned-search payloads now keep a
separate `julia_rerank` telemetry slot so remote Julia transport state stays
separate from `semantic_ignition`.

The OpenAI-compatible semantic-ignition runtime path now uses that config as an
optional post-processing stage. When configured, Wendao can build the
WendaoArrow `v1` request batch, call the remote Julia service, validate the
Arrow response contract, and overwrite `QuantumContext.saliency_score` with the
returned Julia `final_score`. Transport or contract failures degrade cleanly
back to the original Rust-side quantum-fusion ordering and are recorded in
`julia_rerank` telemetry.

That runtime path is now covered by a planned-search loopback integration test
that keeps a local mock only for `/v1/embeddings`, but sends the rerank Arrow
IPC request to the real `.data/WendaoArrow` Julia service, then asserts the
Julia `final_score` response actually reorders emitted `quantum_contexts`.

The vector-store semantic-ignition backend can now enter the same Julia rerank
path when the caller provides a precomputed query vector through the planned
search runtime. Wendao keeps that vector in the in-memory payload runtime state
only, uses it to build the WendaoArrow request batch, and still avoids
serializing it into the external payload contract.

That vector-store runtime path is also now covered against the real
`.data/WendaoArrow` Julia service rather than a Rust-side Arrow mock.

There is now also a dedicated planned-search integration that targets the
package-owned `.data/WendaoArrow/scripts/run_stream_scoring_server.sh`
example, so the main crate validates not only custom Julia rerank responses
but also the official stream scoring example surface.

A second package-owned integration now targets
`.data/WendaoAnalyzer/scripts/run_stream_linear_blend_server.sh`, so the main
crate also validates the first real analyzer package surface instead of only
transport-layer examples.

The Julia rerank runtime config can now also express analyzer-owned strategy
selection fields:

- `link_graph.retrieval.julia_rerank.service_mode`
- `link_graph.retrieval.julia_rerank.analyzer_config_path`
- `link_graph.retrieval.julia_rerank.analyzer_strategy`
- `link_graph.retrieval.julia_rerank.vector_weight`
- `link_graph.retrieval.julia_rerank.similarity_weight`

These fields are additive. They do not change the Arrow transport contract, and
they are currently validated through main integration coverage against the
analyzer-owned test server surface. `LinkGraphJuliaRerankRuntimeConfig` now
comes directly from the Julia-owned compatibility surface, and
`xiuxian-wendao` consumes it without keeping a host re-export shim. That
runtime record also exposes
`analyzer_service_descriptor()` so Rust-side code can derive the analyzer-owned
launch contract without repeating field mapping. The same runtime surface now
also exposes `analyzer_launch_manifest()`, which resolves the generic analyzer
launcher path and ordered Julia CLI args into one Julia-owned manifest.

For analyzer-owned deployments, the current service contract is intentionally
split:

- `xiuxian-wendao` owns remote Julia rerank addressing through
  `link_graph.retrieval.julia_rerank.*`
- `WendaoAnalyzer` owns analyzer strategy flags such as
  `--analyzer-config`, `--analyzer-strategy`, `--vector-weight`, and
  `--similarity-weight`
- remaining HTTP and Arrow transport flags are still passed through to
  `WendaoArrow`

The Rust integration support now consumes that launch manifest directly, so it
no longer has to hardcode the analyzer launcher script name or hand-assemble
ordered Julia args from repeated field mapping.

The same runtime surface now also exposes `deployment_artifact()`, which
packages the resolved transport coordinates and analyzer launch manifest into
one serializable artifact suitable for inspection or persistence. That
artifact now also owns `to_toml_string()` and `write_toml_file(...)`, so
runtime assembly can export the resolved Julia deployment contract without
repeating serialization or file-write conventions outside `xiuxian-wendao`.
The deployment artifact now also carries artifact-level inspection metadata:
`artifact_schema_version` identifies the artifact contract itself, while
`generated_at` records when a concrete JSON/TOML export instance was rendered.
On top of that, the remaining Julia-shaped launch and deployment DTOs are now
package-owned by `xiuxian-wendao-julia`, so callers that still need the
legacy Julia compatibility records should import them from the Julia package
rather than from `xiuxian-wendao` shims or selector re-exports.

For downstream Rust imports:

- prefer `xiuxian_wendao_julia::compatibility::link_graph::*` for Julia-owned
  deployment and launch compatibility DTOs such as
  `LinkGraphJuliaRerankRuntimeConfig` and
  `LinkGraphJuliaDeploymentArtifact`
- treat `xiuxian-wendao` itself as the owner of the generic plugin-artifact
  surfaces and runtime binding helpers
- do not expect a crate-root `xiuxian_wendao::compatibility::*` namespace;
  that host migration shim is retired

```rust
use xiuxian_wendao_julia::compatibility::link_graph::{
    DEFAULT_JULIA_ANALYZER_LAUNCHER_PATH, LinkGraphJuliaDeploymentArtifact,
    LinkGraphJuliaRerankRuntimeConfig,
};

let _ = core::mem::size_of::<LinkGraphJuliaRerankRuntimeConfig>();
let _ = core::mem::size_of::<LinkGraphJuliaDeploymentArtifact>();
let _launcher = DEFAULT_JULIA_ANALYZER_LAUNCHER_PATH;
```

For inspection surfaces, the same export is now visible through
`zhenfa_router` as `wendao.plugin_artifact`, which returns one resolved
artifact selected by `plugin_id` and `artifact_id` over the existing
native/RPC tool boundary. The tool defaults to TOML, also supports a
structured JSON variant via `output_format = "json"`, and accepts an optional
`output_path` for direct TOML or JSON export instead of only returning the
rendered payload inline.
Studio now also exposes the same resolved artifact at
`GET /api/ui/plugins/{plugin_id}/artifacts/{artifact_id}`, returning the
structured generic plugin-artifact JSON directly from the gateway debug
surface. The same endpoint also accepts `?format=toml` for parity with the
tool-side inspection surface. That Studio inspection surface is now also
formalized in the gateway OpenAPI route inventory around the generic
`UiPluginArtifact` payload, so the debug endpoint no longer leaks the raw
link-graph runtime artifact struct directly.
The Studio JSON payload includes the same artifact-level metadata fields, so
UI/debug consumers can distinguish artifact-contract versioning from the
underlying WendaoArrow transport `schema_version`.
The current frontend consumer surface now also exposes this endpoint through
typed API client methods, so Qianji Studio can request either the structured
JSON artifact or the TOML inspection view without issuing raw fetch calls.
That same frontend consumer path is now also rendered in the Studio workspace
StatusBar as a Julia rerank inspection chip, so the resolved deployment
artifact is visible in the live UI shell instead of only through API/debug
consumers.
That live inspection chip now also offers direct frontend-side export actions
for `Copy TOML` and `Download JSON`, reusing the same Studio endpoint rather
than introducing a second deployment-debug surface.
On the frontend side, the artifact formatting and export orchestration now sit
behind one dedicated inspection feature folder instead of staying split across
the workspace shell and status-bar component.
That frontend inspection folder now also exposes a dedicated subview for the
artifact popover, so the workspace `StatusBar` can remain a thin orchestration
surface on the UI side as well.
The same frontend cleanup now also extracts the repo-index diagnostics
chip/popover into its own status-bar subview, so the main status shell is
consistently acting as composition/orchestration rather than inline popover
rendering.
That status-bar cleanup now also moves derived labels and tones behind a
frontend `statusBar/model.ts` helper, so the top-level shell is no longer
mixing derivation with final rendering assembly.
The Julia deployment inspection surface now also carries its own controller for
copy/download feedback state, further reducing the amount of local UI state
owned by the top-level StatusBar shell.

A second WendaoSearch service-fixture integration now targets
`.data/WendaoArrow/scripts/run_stream_metadata_server.sh` to confirm additive
response columns derived from request metadata do not break the planned-search
Julia rerank path. The Julia response decoder now also surfaces optional
additive `trace_id` columns into `julia_rerank.trace_ids`, and the planned
search runtime writes a stable request-schema `trace_id` so the metadata
service fixture can roundtrip that context without changing the core
`doc_id / analyzer_score / final_score` contract.

The integration support layer now keeps those two concerns separate:

- custom-score tests launch a private Julia processor with explicit score maps
- WendaoSearch service-fixture tests launch `.data/WendaoArrow/scripts/run_stream_scoring_server.sh`

At the request boundary, both `zhenfa_router::WendaoSearchRequest` and the
planned HTTP request surface now accept an optional `query_vector`. When
present, the router forwards it into the planned-search runtime so the
vector-store backend can participate in Julia rerank without requiring an
extra embedding call on the Rust side.

The same optional `query_vector` is now accepted by the native
`wendao.search` Zhenfa tool arguments, so direct tool callers and bridge-based
LLM flows can reuse the vector-store Julia rerank path without changing the
serialized planned-payload contract.

This keeps transport ownership in the Arrow substrate while giving future
gateway and reranking paths one stable Rust-side integration surface.

### Query Core Phase 1

Wendao now starts the RFC-defined query-core split with a new internal
`query_core` module. In this layout:

- `xiuxian-wendao` owns typed query operators, execution context, graph
  adapters, and explain or telemetry contracts
- `xiuxian-db-store` exposes reusable retrieval-row shaping and
  payload-projection helpers used by the query core

The currently landed Phase-1 slice is no longer only a skeleton:

- repo-scoped `repo_content` code search now routes through
  `query_core::query_repo_content_relation(...)`
- repo-scoped `repo_entity` code search now routes through
  `query_core::query_repo_entity_relation(...)`
- Studio graph handlers now route `graph_neighbors` and `node_neighbors`
  through query-core-native graph projections rather than handler-local Arrow
  relation decoding
- repo-scoped code-search telemetry for both `repo_content_chunk` and
  `repo_entity` is now reflected back into Search Plane status
- graph-side explain events now emit stable internal summaries through the
  existing logging path instead of remaining test-only
- graph query-core backend naming is now neutral and API-first:
  `LinkGraphBackend` and `SearchPlaneBackend` replace the old `legacy-*`
  labels inside explain events and tests
- handlers no longer need to hand-assemble execution context directly; a
  thin query-core service facade now owns the common adapter wiring
- repo-wide `code_search` task fanout now reuses one per-repo
  `search_repo_code_hits(...)` entrypoint, so the join-set scheduler is
  responsible only for concurrency and timeout control, not entity/content
  fallback semantics
- the repo-scoped code-search policy now also has a query-core service-level
  entrypoint. Entity-first fallback ordering and repo publication gating are
  no longer re-decided in the scheduling layer
- repo-scoped retrieval callers now exercise the full Phase-1 relation path:
  `vector_search -> column_mask -> payload_fetch`. `ColumnMaskOp` and
  `PayloadFetchOp` are no longer test-only operators
- query-core now also has direct service-level regressions for repo code-search
  corpus selection, so entity-preferred routing and content fallback are pinned
  below the handler layer
- query-core graph integration is now also projection-first. The new
  `query_core::graph` module owns `WendaoGraphProjection`,
  `WendaoGraphNode`, and `WendaoGraphLink`, while
  `query_core::query_graph_neighbors_projection(...)` lets Studio graph
  handlers consume one typed internal graph shape instead of parsing Arrow
  columns in the route layer
- the Studio graph handler cluster now also follows the same Phase-2 shared
  service rule as repo and docs. `graph_neighbors`, `node_neighbors`, and
  `topology_3d` route through one graph service boundary, so route files no
  longer own query-core projection execution, explain logging, or topology
  assembly directly
- the remaining graph shared-state surface is now closed too. `graph/shared/`
  separates graph-neighbor query parsing from graph rendering and topology
  helpers, and `graph/shared/mod.rs` is now interface-only like the other
  Phase-2 feature folders
- the graph test boundary now matches that production shape. `graph/tests/support/`
  separates fixture runtime, response helpers, assertions, and snapshot
  shaping, while `graph/tests/mod.rs` stays interface-only
- the router export seam now matches docs as well. `graph_exports.rs` owns the
  outward-facing graph handler exports, so `handlers/mod.rs` no longer carries
  a flat graph re-export block
- the repo router export seam now matches the same pattern. `repo_exports.rs`
  owns the outward-facing repo handler exports, so `handlers/mod.rs` no longer
  carries a flat repo re-export block either
- the same outward-facing export rule now also covers analysis handlers.
  `analysis_exports.rs` owns the public analysis handler exports, so
  `handlers/mod.rs` no longer carries a flat analysis re-export block
- the remaining top-level router seams are now closed as well.
  `capabilities_exports.rs`, `ui_config_exports.rs`, and `vfs_exports.rs`
  now own the outward-facing exports for those handler clusters, so
  `handlers/mod.rs` no longer carries any flat capability, UI-config, or VFS
  re-export blocks
- the analysis handler body is now closed too. `analysis/` separates markdown
  routes, code-AST routes, shared loader logic, and query types, while
  `analysis/mod.rs` stays interface-only
- the capabilities handler body now follows the same pattern. `capabilities/`
  separates UI-capabilities routes, deployment-artifact routes, and query
  types, while `capabilities/mod.rs` stays interface-only
- repo-analysis search handlers now route their repo-entity fast path through
  `query_core::service`, and the repeated cache plus fast-path plus analyzer
  fallback control flow is now centralized in one shared repo-analysis search
  service helper with typed `run_repo_module_search(...)`,
  `run_repo_symbol_search(...)`, and `run_repo_example_search(...)` entrypoints.
  `module-search`, `symbol-search`, and `example-search` no longer duplicate
  that orchestration inline
- that same shared repo-analysis service now also owns the typed route-facing
  entry contract end to end. The route files for module, symbol, and example
  search no longer carry route-local fallback-builder closures; they only parse
  request parameters and return `Json(...)` from the shared typed service
- the shared repo-analysis service now also owns a lower-level typed contract
  for query construction, query-core fast-path dispatch, and analyzer-artifact
  fallback binding. Module, symbol, and example flows now vary by typed
  contract metadata instead of carrying near-duplicate builder wiring
- that fallback contract ownership has now been pushed one layer lower into
  `analyzers::service::search`. Gateway-side repo-analysis orchestration now
  composes analyzer-owned fallback contracts with query-core fast paths instead
  of directly binding analyzer query builders and artifact result constructors
- the repo-analysis fast-path contract has now been pushed lower as well.
  `query_core::service` owns typed repo-entity fast-path contracts for module,
  symbol, and example result surfaces, and the shared gateway repo-analysis
  service now composes query-core-owned fast-path contracts with
  analyzer-owned fallback contracts rather than binding three separate
  repo-entity helper functions
- that same shared Phase-2 repo-analysis service now also owns
  `repo/import-search`. Studio routing and OpenAPI inventory expose the import
  lane, analyzer search owns the typed import fallback contract, and the route
  layer now validates both `MISSING_REPO` and `MISSING_IMPORT_FILTER`
- the import lane has now landed its next Phase-2 hardening milestone too.
  Import cache identity is derived from one canonical fallback-contract query
  text that preserves both `package` and `module`, so combined filters no
  longer collapse onto the same cached search key
- `repo/import-search` is now also repo-entity aware. Import rows are
  materialized on the repo-entity plane, query-core exposes a publication-gated
  typed import fast path, and the shared Phase-2 repo-analysis service now
  prefers that fast path before falling back to analyzer-owned import search
- query-search regression coverage now starts using snapshot baselines under
  `tests/snapshots/wendao/`. The first landed snapshots pin
  query-core repo-code results, query-core graph projections, and typed
  repo-entity module/symbol/example query outputs
- repo-analysis gateway coverage now also keeps Studio-level snapshot baselines
  under `tests/snapshots/gateway/studio/` for the shared repo-entity fast-path
  payloads returned by `module-search`, `symbol-search`, and `example-search`
  helper flows
- snapshot coverage now also includes analyzer import search output and the
  gateway-facing `repo/import-search` payload surface, so the import lane is
  pinned at both the analyzer contract layer and the Studio response layer
- import fast-path coverage now also includes a dedicated query-core snapshot
  plus a gateway regression proving `run_repo_import_search(...)` can succeed
  from repo-entity publication alone, without requiring repo-config fallback
- the same Phase-2 service-consolidation pattern now also covers the non-search
  repo-analysis endpoints. `repo/overview` and `repo/doc-coverage` now route
  through one shared analysis service boundary instead of each handler owning
  its own `with_repo_analysis(...)` orchestration
- projected retrieval handlers now follow the same rule. `projected_retrieval`,
  `projected_retrieval_hit`, `projected_retrieval_context`,
  `projected_page_index_tree_search`, and `projected_page_search` now sit
  behind one shared projected-service boundary instead of carrying repeated
  route-local analysis orchestration
- the same shared projected-service boundary now also owns the projected page
  lookup cluster. `projected_pages`, `projected_gap_report`,
  `projected_page`, `projected_page_index_tree`,
  `projected_page_index_node`, and `projected_page_index_trees` are now thin
  route adapters over typed service entrypoints instead of direct
  `with_repo_analysis(...)` callers
- that projected-service boundary now also owns the projected family and
  navigation cluster. `projected_page_family_context`,
  `projected_page_family_search`, `projected_page_family_cluster`,
  `projected_page_navigation`, and `projected_page_navigation_search` now
  route through one shared typed service seam instead of carrying route-local
  repository-analysis orchestration
- the remaining repo command endpoints now follow the same rule. `repo_index`,
  `repo_index_status`, and `refine_entity_doc` now delegate through one shared
  repo-command service instead of each route binding its own command or
  repository-analysis orchestration
- the next repo-handler cleanup slice has started too. The old
  `repo/shared.rs` helper surface is now split into a `repo/shared/` feature
  folder with separate repository-resolution and execution modules. The shared
  repo handler boundary now follows the same feature-folder rule as the rest of
  the repo handler surface
- the same service-consolidation rule now also covers the docs surface.
  `docs/search`, `docs/retrieval`, `docs/page`, `docs/family`, and
  `docs/navigation` now delegate through one shared `docs/service.rs`
  boundary, so docs handlers no longer own repeated route-local
  `with_repo_analysis(...)` orchestration or retrieval-hit error translation
- that docs-service boundary is now closed over the planner and gap surfaces
  too. `docs/projected-gap-report` and the full `docs/planner-*` cluster now
  delegate through the same shared service instead of binding route-local
  repository-analysis orchestration
- the last flat docs shared-state surface is now closed too. The old
  `docs/types.rs` has been replaced by a `docs/types/` feature folder with
  separate planner and projected-gap query-param modules, and route imports now
  go through the crate-qualified docs types boundary
- the flat docs route surface has now been closed as well. Projection-oriented
  docs handlers now live under `docs/projection/`, planner routes live under
  `docs/planner/`, and router re-exports keep the external endpoint surface
  unchanged
- the same closure now applies to the internal docs service layer. The old
  flat `docs/service.rs` has been replaced by `docs/service/{projection,planner,runtime}.rs`,
  so service ownership now aligns with the route and query-param boundaries
- the top router export seam is now aligned too. Docs handler re-exports live
  in `handlers/docs_exports.rs`, so the final outward-facing docs symbol
  surface is grouped instead of inlined into `handlers/mod.rs`

Phase 1 keeps the external gateway and CLI contract unchanged, but the first
repo-facing internal milestone is now complete: repo code-search and
repo-analysis fast paths both sit on Wendao-owned typed internal service
boundaries instead of route-local adapter seams. Phase 2 has also landed its
first two repo-analysis orchestration milestones, so the route layer now acts
as a thin adapter over one shared typed repo-analysis service boundary, and the
import lane now participates in that same publication-aware fast-path model.
The same consolidation pattern now spans repo, projected, command, shared, and
docs handler clusters instead of stopping at repo-analysis search alone. The
docs handler surface is now fully closed over one shared docs-service seam, and
its shared query-param and route surfaces now both follow the same
feature-folder rule. The internal docs service layer and router export seam now
follow it too. The last legacy `docs_exports.rs` barrel is gone as well, so the
outward docs handler surface now binds directly to the `docs::planner` and
`docs::projection` feature folders without an extra forwarding hop. The same is
now true for repository handlers: `repo_exports.rs` is gone, and the outward
repository handler surface binds directly to `handlers/repo/mod.rs`. The router
test surface now also pins this cleanup state: `gateway/studio/router/tests`
contains a regression test that fails if any legacy `handlers/*_exports.rs`
files reappear. The shared OpenAPI inventory now has the same kind of guard:
`gateway/openapi/paths/shared/tests.rs` fails if retired Flight-only HTTP paths
such as `/api/search/ast` or `/api/analysis/*` ever re-enter the stable route
inventory.

---

## 📦 Usage

### As a CLI Tool (Standalone Binary)

Build the sovereign binary:

```bash
cargo build --release --bin wendao
```

Run common operations:

```bash
# Analyze document hierarchy
./target/release/wendao page-index --path ./my_notes/paper.md

# Execute hybrid search
./target/release/wendao search "Explain quantum entanglement" --hybrid

# Show graph neighbors
./target/release/wendao neighbors "Agentic_RAG"
```

### As a Library

Add **Wendao** to your `Cargo.toml`:

```toml
[dependencies]
xiuxian-wendao = { git = "https://github.com/tao3k/wenbdao.git" }
```

Initialize the engine:

```rust
let engine = WendaoEngine::builder()
    .with_storage(ValkeyConfig::default())
    .with_vectors(LanceConfig::at("./data/vectors"))
    .build()
    .await?;
```

### Optional Python Bindings

Enable the PyO3 surface only when you need Python interop:

```bash
cargo build -p xiuxian-wendao --features pybindings
```

This keeps the default build free of PyO3 and the Python-specific modules.

---

## 🛡️ License

Designed with the precision of a master artisan.

© 2026 Sovereign Forge. All Rights Reserved.
