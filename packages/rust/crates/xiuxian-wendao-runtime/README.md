# xiuxian-wendao-runtime

Runtime support crate for Wendao transport contracts, Flight server/client
plumbing, runtime config resolution, and artifact rendering helpers.

## Responsibility

`xiuxian-wendao-runtime` owns generic host behavior for the Wendao split.
If a boundary depends on runtime state, config resolution, transport
negotiation, or live host assembly, it belongs here instead of in
`xiuxian-wendao-core`.

Current ownership:

- typed host config models and resolvers
- settings merge, parse, and directory helpers
- transport negotiation and Flight client/server helpers
- runtime artifact resolve/render helpers
- embedded zhixing artifact path, mount, and text helpers that only depend on
  embedded resources, parser-derived frontmatter metadata, and Wendao URI
  parsing
- bundled Wendao gateway `OpenAPI` text, path, and parsed-document helpers
  that only depend on checked-in artifact access and JSON parsing

## Bounded DuckDB Runtime Lane

The bounded DuckDB direction is tracked in
[RFC: DuckDB as a Bounded In-Process Analytic Lane for Wendao and Qianji](../../../../../docs/rfcs/2026-04-08-wendao-qianji-duckdb-bounded-analytics-rfc.md).

The first runtime-owned slice for that lane is now landed under
`src/config/duckdb/`. `xiuxian-wendao-runtime` owns only the Wendao
host-side runtime concerns for this lane:

- typed host config resolution
- temp/spill directory policy
- connection/bootstrap helpers that depend on deployment context

By default, Wendao-owned DuckDB files and spill directories resolve under
`$PRJ_DATA_HOME/xiuxian-wendao`. Gateway deployments may override
`search.duckdb.database_path` and `search.duckdb.temp_directory` through the
root `wendao.toml`; package resource config leaves those paths unset so the
runtime default remains authoritative.

Arrow remains a default substrate in this crate rather than a transport-only
optional dependency gate. The transport feature still gates transport-facing
logic, but Arrow and Arrow Flight stay first-class runtime dependencies.
Arrow Flight is the canonical cross-language data plane. Local stdio, process
arguments, JSON, JSONL, and REST metadata may coordinate control flow, but they
must not carry Arrow table payloads through ad hoc JSON/base64 wrappers. Shared
callers should build runtime-negotiated Flight bindings and exchange
`RecordBatch` data through the Arrow Flight client/server boundary. The
canonical tokens and validators are exported from `transport::data_plane`:
`arrow-flight` and `arrow-record-batch` are data-plane tokens; `json-control`,
`jsonl-stdio-control`, `process-args-control`, and `rest-metadata-control` are
control-only tokens.

It must not become the home for DuckDB query semantics, search-plane
registration logic, or Qianji workflow-stage orchestration.

## Non-Goals

Do not use `xiuxian-wendao-runtime` as the home for:

- stable contract record ownership that plugins can share directly
- knowledge-graph, retrieval, or storage semantics
- language-specific intelligence implementation

Those belong in `xiuxian-wendao-core` or `xiuxian-wendao` respectively.

## Config Layout

The crate keeps raw config access and typed resolved config separate.

- `src/settings/`: raw merged-setting access, normalization, and parse helpers
- `src/config/`: typed host config records and resolver logic

This avoids repeating `runtime` in a `runtime_config` namespace inside
`xiuxian-wendao-runtime` itself.

## Memory Julia Compute Host Seam

The first memory-family Julia compute host seam now lives under:

- `src/config/memory/julia/compute.rs`

This surface is intentionally runtime-owned and compute-only:

- `memory.julia_compute` resolves runtime-level host config
- the runtime config now also carries one optional family-level `health_route`
  for the Julia compute provider
- the runtime config now also carries one bounded
  `max_in_flight_requests` budget so Rust can cap queued Julia compute
  roundtrips explicitly instead of allowing unbounded waiters behind one
  shared Flight client
- the module does not own host lifecycle or state mutation
- recommendation-only memory profiles stay outside host authority until Rust
  commits them

## Polyglot Compute Boundary

The Polyglot Compute Orchestrator boundary is tracked in
[RFC: Polyglot Compute Orchestrator](../../../../../docs/rfcs/2026-05-04-polyglot-compute-orchestrator-rfc.md)
and its
[audit](../../../../../docs/rfcs/2026-05-04-polyglot-compute-orchestrator-audit.md).

For that lane, `xiuxian-wendao-runtime` owns the Rust control-plane substrate:
runtime config resolution, reusable Arrow Flight client configuration,
route-level request gates, timeout policy, schema-version metadata, and
transport validation. This crate does not own Python Docling execution, Julia
compute schemas, document/OCR cache policy, or polyglot schedule contracts. The
approved
`xiuxian-polyglot-orchestrator` crate owns shared lane, admission, evidence,
reference, and pure Docling scheduling-plan contracts. Its optional
`wendao-contracts` surface may depend on this crate to project runtime-owned
route/config facts into polyglot contracts. This crate must not depend on the
orchestrator.

For document extraction, runtime owns route admission, timeout behavior,
request-header translation, and any live Flight dispatch decisions. Studio
supplies pressure facts and caller-local worker or shard bounds to the
orchestrator `wendao-contracts` scheduler. The returned plan is inert.
The page-range Docling structure fallback uses the runtime/server transport
header `x-wendao-document-extract-page-range` as an internal 1-based inclusive
range contract. Runtime only exposes the stable header name; Studio decides
when to request page-range structure and analyzer performs the Docling
conversion.

Studio consumes the orchestrator-generated Wendao contract plan for
full-document Docling dispatch before selecting from the existing endpoint
pool. The owner budget is still Studio's existing conversion semaphore; runtime
does not create a queue, launch workers, or mutate Python routes.

## Selection Rule

If the code reads environment state, touches config files, negotiates
transport, materializes clients/servers, or otherwise depends on deployment
context, prefer `xiuxian-wendao-runtime`.

For the full three-package boundary matrix, see
[`../xiuxian-wendao/docs/06_roadmap/417_wendao_package_boundary_matrix.md`](../xiuxian-wendao/docs/06_roadmap/417_wendao_package_boundary_matrix.md).

## Transport Server Test Layout

The transport server tests now follow a feature-folder layout under
`tests/unit/lib/transport/server/` instead of a single flat `server.rs`.

- `assertions.rs`: shared test assertions and Flight decoding helpers
- `construction.rs`: service-construction boundaries
- `fixtures.rs`: shared service builders and Flight batch decode helpers
- `metadata.rs`: request-header validation coverage
- `providers.rs`: recording route-provider doubles
- `request_headers.rs`: shared metadata/header builders
- `rerank.rs`: rerank contract tests
- `routes/`: route-family integration coverage split by concern

The dataset-to-ontology materialization route now has server-side admission
coverage for its multi-table manifest metadata. Runtime validates the manifest
and cache/admission key for `/ontology/dataset/materialize`. If no provider is
configured, the route returns an explicit unimplemented status. If a provider
is configured, runtime passes the admitted manifest through
`DatasetOntologyMaterializeFlightRouteProvider` and streams the returned Arrow
batches through the existing Flight payload path. DuckDB execution and
source-contract SQL orchestration remain outside this crate.

## Transport Query Contract Layout

The query-contract surface now follows the same folder-first rule.
`src/transport/query_contract.rs` is only the stable re-export seam, while the
implementation lives under `src/transport/query_contract/` and the contract
tests live under `tests/unit/transport/query_contract/`.

- `metadata.rs`: route normalization plus descriptor helpers
- `search/`: repo search, attachments, definition, autocomplete, and AST
  contract constants
- `query/`: SQL query contract
- `query/sql/headers.rs`: stable SQL route and metadata-header constants
- `query/sql/validation.rs`: DataFusion-backed read-only SQL validation
- `vfs/`: content/resolve/scan contracts
- `graph/`: neighbors and topology contracts
- `analysis/`: markdown and code-AST request validation
- `repo/`: repo analysis and refine-doc contracts
- `ontology/`: dataset-to-ontology multi-table Arrow handoff route,
  metadata headers, and manifest validation
- `rerank/`: rerank schema, batch validation, and scoring
- `tests/`: query contract coverage split by the same feature families

## Project Policy Gate

The crate uses `rust-lang-project-harness` as its active project-policy gate
with no disabled-rule baseline. The gate is mounted from the library, root
unit-test target, and shared lib-policy test target.

The strict gate requires all diagnostics to be closed, including informational
agent-policy output. Current owner-boundary fixes include:

- `src/bin/wendao_flight_server.rs` stays a thin entrypoint and delegates to
  `src/transport/server/sample_host.rs`
- the Flight service implementation lives in
  `src/transport/server/flight/service.rs` rather than a repeated
  `transport` module segment
- scalar config helpers and shared query metadata use responsibility-specific
  module names
- runtime source modules carry concise intent docs for agent traversal
- tests use explicit owner paths instead of deep `super::super` imports
- the polyglot bridge is profiled as runtime-owned route and admission
  projection, not as a live scheduler

## Verification

Current runtime verification for this lane:

- `cargo test -p xiuxian-wendao-runtime --lib enforce_rust_project_harness_gate`
- `cargo test -p xiuxian-wendao-runtime --lib --test unit_test`
- `cargo test -p xiuxian-wendao-runtime --all-features`
- `cargo fmt --package xiuxian-wendao-runtime --check`
- `cargo clippy -p xiuxian-wendao-runtime --all-targets --all-features -- -D warnings`

The `plugin_arrow_exchange` transport tests now satisfy strict clippy without
`expect_err(...)`-style assertions, so test-scope warning closure is back to a
green baseline for this crate.
