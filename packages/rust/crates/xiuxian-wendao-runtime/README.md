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

Arrow remains a default substrate in this crate rather than a transport-only
optional dependency gate. The transport feature still gates transport-facing
logic, but Arrow and Arrow Flight stay first-class runtime dependencies.

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

## Selection Rule

If the code reads environment state, touches config files, negotiates
transport, materializes clients/servers, or otherwise depends on deployment
context, prefer `xiuxian-wendao-runtime`.

For the full three-package boundary matrix, see
[`../xiuxian-wendao/docs/06_roadmap/417_wendao_package_boundary_matrix.md`](../xiuxian-wendao/docs/06_roadmap/417_wendao_package_boundary_matrix.md).

## Transport Server Test Layout

The transport server tests now follow a feature-folder layout under
`src/tests/transport/server/` instead of a single flat `server.rs`.

- `assertions.rs`: shared test assertions and Flight decoding helpers
- `construction.rs`: service-construction boundaries
- `fixtures.rs`: shared service builders and Flight batch decode helpers
- `metadata.rs`: request-header validation coverage
- `providers.rs`: recording route-provider doubles
- `request_headers.rs`: shared metadata/header builders
- `rerank.rs`: rerank contract tests
- `routes/`: route-family integration coverage split by concern

## Transport Query Contract Layout

The query-contract surface now follows the same folder-first rule.
`src/transport/query_contract.rs` is only the stable re-export seam, while the
implementation lives under `src/transport/query_contract/` and the contract
tests live under `src/transport/query_contract/tests/`.

- `common.rs`: route normalization plus descriptor helpers
- `search/`: repo search, attachments, definition, autocomplete, and AST
  contract constants
- `query/`: SQL query contract
- `query/sql/headers.rs`: stable SQL route and metadata-header constants
- `query/sql/validation.rs`: DataFusion-backed read-only SQL validation
- `vfs/`: content/resolve/scan contracts
- `graph/`: neighbors and topology contracts
- `analysis/`: markdown and code-AST request validation
- `repo/`: repo analysis and refine-doc contracts
- `rerank/`: rerank schema, batch validation, and scoring
- `tests/`: query contract coverage split by the same feature families

## Verification

Current runtime verification for this lane:

- `direnv exec . cargo clippy -p xiuxian-wendao-runtime --tests --features transport -- -D warnings`
- `direnv exec . cargo test -p xiuxian-wendao-runtime --features transport`
- `direnv exec . cargo test -p xiuxian-wendao-runtime query_contract --features transport`
- `direnv exec . cargo clippy -p xiuxian-wendao -p xiuxian-wendao-runtime --all-targets --all-features -- -D warnings`

The `plugin_arrow_exchange` transport tests now satisfy strict clippy without
`expect_err(...)`-style assertions, so test-scope warning closure is back to a
green baseline for this crate.
