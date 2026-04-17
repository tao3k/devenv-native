# xiuxian-wendao-julia

`xiuxian-wendao-julia` is the Julia-owned Repo Intelligence plugin crate for `xiuxian-wendao`.

The Modelica repo-intelligence lane now lives here too. There is no separate
`xiuxian-wendao-modelica` Rust crate to maintain; both Julia and Modelica
plugins ride the same Julia-owned parser and Arrow Flight integration line.
The default `xiuxian-wendao-builtin` registry bundle now links that shared
Julia plus Modelica line unconditionally, so the builtin registry no longer
needs a feature-gated second plugin bundle for these languages.

## Verification Status

- crate-wide strict clippy is green again under
  `direnv exec . cargo clippy -p xiuxian-wendao-julia --all-targets --all-features -- -D warnings`
- the graph-structural plugin and test baseline now closes without lint
  suppressions by using shared test-support panic helpers, moving the
  `entry.rs` test module behind production items, and keeping the staged
  graph-structural transport fixture aligned with the current request contract
- the targeted transport regression proof is green under
  `direnv exec . cargo test -p xiuxian-wendao-julia plugin::graph_structural_transport::tests::validate_graph_structural_request_batches_accepts_staged_shapes -- --exact --nocapture`
- the full crate test gate now completes again under
  `direnv exec . cargo test -p xiuxian-wendao-julia`; the current full pass is
  `147 passed` in `226.16s` after the slow solver-demo manifest-discovery
  graph-structural live proofs were regrouped so one `multi_route` Julia
  service lifecycle now covers multiple rerank or filter assertions instead of
  repeatedly re-spawning the same live service
- the graph-structural live proof surface has since been tightened again:
  the remaining `demo` and `solver_demo` pair or generic-topology live proofs
  now consolidate onto `multi_route` services, and the plugin test-support
  launches the Julia example services through explicit `julia --project=...`
  commands rather than repo-level `direnv` wrappers
- the remaining repeated `demo` capability-manifest live proofs are now also
  consolidated into one grouped test that covers manifest fetch, manifest
  preflight, graph-structural binding discovery, transport fallback, and
  plugin preflight against one live `WendaoSearch.jl` endpoint
- the current canonical full crate pass is now `139 passed` in `136.25s`,
  while preserving explicit transport, manifest-discovery, and grouped
  capability-manifest live coverage across the plugin lane
- Julia and Modelica parser-summary transport discovery now also works with
  plain repository plugin ids. `plugins = ["julia"]` and
  `plugins = ["modelica"]` default to the standard
  `WendaoSearch.jl --config config/live/parser_summary.toml` base URL
  `http://127.0.0.1:41081` for parser-summary routes, while tests pin the same
  contract through linked in-process base URLs instead of inlining
  `parser_summary_transport` into every repo fixture
- linked `WendaoSearch.jl` parser-summary test services now leave the live
  `gRPCServer` runtime dependency under the package's own
  `scripts/run_parser_summary_service.jl` launcher and the delegated
  `scripts/run_search_service.jl` bootstrap: the launcher still honors
  `WENDAO_FLIGHT_GRPCSERVER_PATH` when an explicit local checkout is needed,
  reuses a vendored `.cache/vendor/gRPCServer.jl` checkout when present, and
  otherwise reuses one depot-installed `gRPCServer.jl` source checkout that is
  already visible to the live Julia process before binding the Flight listener
- the repository now also exposes one process-managed
  `wendaosearch-solver-demo` background service, but its route and port
  semantics stay package-owned in `WendaoSearch.jl` TOML config and the crate
  test suites still self-spawn Julia services for isolated live proofs; the
  managed service now also mirrors stdout and stderr into repo-local runtime
  log files so background failures are inspectable without attaching to the
  live process manager UI, and absence of those files usually means the
  background service is still running from an older process-compose generation
  that predates the current launcher
- the repository now also exposes one canonical process-managed
  `wendaosearch-parser-summary` background service for the native Julia and Modelica
  parser-summary lane; unlike `wendaosearch-solver-demo`, it launches
  the package-owned `config/live/parser_summary.toml` through
  `scripts/run_parser_summary_service.jl` and is the intended managed-service
  surface for gateway `code_search` and `code_ast` integration
- the Rust gateway parser-summary test seam now also understands that managed
  service directly: setting `RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST=1` makes the
  linked parser-summary helper bind to `wendaosearch-parser-summary` instead of
  self-spawning an in-process Julia service, so focused gateway search or
  code-AST proofs can exercise the same process-managed service shape that
  `process.nix` owns
- Julia test support now lives under `tests/unit/plugin/` plus
  `tests/unit/memory/mod.rs` instead of production `src/` files, while
  `src/lib.rs` mounts `tests/unit/lib_policy.rs` and `tests/unit_test.rs`
  owns the root harness target so both `cargo test --lib` and
  `cargo test --test unit_test` execute the shared `xiuxian-testing` policy
  gate
- the process-managed `WendaoSearch.jl` background service now also has one
  opt-in Rust live proof under
  `RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST=1 cargo test -p xiuxian-wendao-julia plugin::graph_structural_exchange::tests::fetch_graph_structural_solver_demo_rows_for_repository_against_process_managed_wendaosearch_service -- --exact --nocapture`,
  while the existing self-spawn solver-demo proof remains the deterministic
  isolated baseline; that opt-in proof now first checks whether
  `wendaosearch-solver-demo` is already healthy, and otherwise starts the
  current `devenv` generation itself through `devenv processes up -d`
  instead of trusting an inherited `PC_CONFIG_FILES` shell variable, so the
  same proof path also protects the managed-service log sink against stale
  generation reuse
- the current full crate pass is also green with that opt-in background lane
  enabled:
  `direnv exec . env RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST=1 cargo test -p xiuxian-wendao-julia`
  now completes with the managed-service proof enabled alongside the existing
  self-spawn live suites
- the host consumer still checks cleanly under
  `direnv exec . cargo check -p xiuxian-wendao --lib --features julia`
- the linked host gateway now also proves the native parser route all the way
  through Studio `intent = "code_search"` for both plain Julia and plain
  Modelica plugin repositories:
  `direnv exec . cargo test -p xiuxian-wendao search_intent_routes_code_search_to_plain_julia_plugin_repository --features julia,zhenfa-router -- --nocapture`
  and
  `direnv exec . cargo test -p xiuxian-wendao search_intent_routes_code_search_to_plain_modelica_plugin_repository --features julia,zhenfa-router -- --nocapture`
- the linked host gateway now also proves the same native parser route through
  the repo-aware Studio `analysis/code-ast` loader for both plain Julia and
  plain Modelica plugin repositories:
  `direnv exec . cargo test -p xiuxian-wendao load_code_ast_analysis_response_supports_plain_julia_plugin_repository --features julia,zhenfa-router -- --nocapture`
  and
  `direnv exec . cargo test -p xiuxian-wendao load_code_ast_analysis_response_supports_plain_modelica_plugin_repository --features julia,zhenfa-router -- --nocapture`
- the Rust parser-summary symbol seam now preserves parser-owned line spans and
  detail attributes all the way into `SymbolRecord`, and the Studio `code_ast`
  retrieval payload now keeps backend-issued `displayLabel`, `excerpt`, and
  `attributes` instead of collapsing those details before the frontend
  language modules can render them
- the Julia symbol materialization path now keeps same-name parser overloads as
  distinct Rust symbols instead of collapsing them onto one
  `repo:<id>:symbol:<module>.<name>` record; only colliding parser symbols pick
  up a stable disambiguating suffix, and export placeholders no longer survive
  when a real parser-owned symbol with the same name exists
- Julia parser-summary docstring attachments now also preserve parser-owned
  `target_path` and `target_line_start/end`, and the Rust docstring projection
  uses those fields to bind overload docs to the correct symbol instead of
  resolving only by `target_name`
- repo doc coverage transport now also projects parser-owned `doc_target`
  metadata from Julia docstring records, so the host Flight batch and frontend
  repo-intelligence doc facet can keep target kind, name, qualified path, and
  line spans instead of collapsing those docs back into generic `doc` rows
- the Modelica parser-summary seam now also preserves parser-owned symbol
  attributes such as visibility, variability, type name, owner path, class
  path, restriction, and equation text inside `ParsedDeclaration` and
  `SymbolRecord`, so downstream Studio `code_ast` retrieval atoms and the
  frontend language projection layer can render parser-backed structured
  detail instead of collapsing everything to generic fallback strings

## Ownership Boundary

- `xiuxian-wendao-runtime` owns the reusable Arrow Flight runtime client and negotiation seam.
- `xiuxian-wendao-julia` owns Julia-specific interpretation of repository plugin options and translates them into the runtime-owned Flight binding.
- `xiuxian-wendao-julia` also owns the Modelica repo-intelligence plugin and
  its native parser-summary transport. Rust no longer keeps a standalone
  Modelica crate or a second Modelica AST implementation surface.
- `xiuxian-wendao-julia` also owns the Julia parser-summary client seam for
  repo-intelligence and host incremental safety, including repository-scoped
  transport parsing, Arrow request or response validation, typed summary
  decoding, and the public helper
  `julia_parser_summary_allows_safe_incremental_file_for_repository`.
- `xiuxian-wendao-julia` also owns the parser-rich symbol identity seam for
  Julia repo intelligence, including parser-owned line spans, parser detail
  attributes, and overload-safe symbol materialization before those records are
  projected into Wendao host analysis or Studio `code_ast` retrieval atoms.
- `xiuxian-wendao-julia` also owns the parser-rich Julia docstring target seam,
  including native doc-target path and line metadata decoding plus overload-safe
  doc-to-symbol resolution before Wendao builds documentation relations.
- `xiuxian-wendao-julia` also owns the bounded projection from parser-rich
  Julia docstring targets into `DocRecord`, so downstream repo-doc coverage
  transport and frontend repo-intelligence doc hits can render parser-owned
  target identity without regex inference.
- `xiuxian-wendao-julia` also owns the parser-rich Modelica symbol attribute
  seam, including parser-summary column decoding, `ParsedDeclaration`
  attribute preservation, and projection of those attributes into
  `SymbolRecord` so downstream Studio consumers can render parser-backed
  structured detail without regex inference.
- The parser-summary boundary is Flight-only for the touched Julia cutover
  surface. `xiuxian-wendao-julia` does not keep a Rust-local
  Julia or Modelica AST fallback for repo-intelligence or the incremental
  safety probe; file-summary and root-summary now resolve through either an
  explicit `parser_summary_transport` binding or the standard mounted
  `WendaoSearch.jl` parser-summary endpoint, and if the native parser-summary
  route is unavailable or contract-invalid, the Rust caller fails that
  operation explicitly.
- `xiuxian-wendao-julia` also owns the runtime-level memory-family thin compat
  surface under `src/memory/`, including staged memory profile metadata,
  manifest projection for the RFC `memory` family entry shape,
  runtime-to-binding normalization for `memory.julia_compute`, one optional
  family-level `health_route` propagation path, and typed `episodic_recall`,
  `memory_gate_score`, `memory_plan_tuning`, `memory_calibration`, and
  manifest Arrow request or response validation and decoding.
- the canonical staged defaults in that lane now use `/memory/calibration` for
  the calibration route and `promote_to_working_knowledge` for the
  recommendation-only working-knowledge promotion verdict in
  `memory_gate_score`
- `xiuxian-wendao-julia` now also owns the plugin-side host-adapter helpers
  under `src/memory/host/`, including the Rust-memory-engine projection or
  evidence inputs that build staged `episodic_recall`, `memory_gate_score`,
  `memory_plan_tuning`, and `memory_calibration` request rows or batches.
- `xiuxian-wendao-julia` also owns the runtime-facing memory-family transport
  seam under `src/memory/transport/`, including runtime-config-driven Flight
  client construction, request or response validation dispatch, the bounded
  `max_in_flight_requests` admission-control bridge, roundtrip execution, and
  typed fetch helpers for the four staged memory profiles.
- `xiuxian-wendao-julia` also owns the plugin-side memory-family composition
  seam under `src/memory/downcall/`, which combines `src/memory/host/` input
  staging with `src/memory/transport/` Flight execution so host consumers can
  call one thin plugin-owned downcall surface instead of manually stitching
  those layers together in `xiuxian-wendao`.
- `xiuxian-wendao-julia` owns the Julia Arrow rerank exchange seam only where it stays Julia-specific: repository plugin-option interpretation, remote fetch helpers, and plugin-local loopback tests.
- `xiuxian-wendao-julia` also owns Julia-specific graph-structural transport option parsing, route-kind dispatch defaults, and staged request or response validation for promoted structural-search downcalls.
- `xiuxian-wendao-julia` also owns manifest-driven graph-structural binding fallback, so graph-structural client construction can derive route bindings from the live Julia capability manifest when explicit graph-structural transport config is absent.
- `xiuxian-wendao-julia` also owns one grouped same-endpoint capability-manifest
  live proof that fetches the manifest, validates plugin preflight, derives
  graph-structural bindings, and builds manifest-fallback transport clients
  without re-spawning redundant `demo` services.
- `xiuxian-wendao-julia` also owns the plugin-side proof that one live `WendaoSearch.jl` endpoint can advertise the capability manifest and immediately serve graph-structural downcalls discovered from that same manifest.
- `xiuxian-wendao-julia` also owns the plugin-side proof that the same live
  `WendaoSearch.jl` endpoint can serve both heuristic `demo` and bounded
  solver-backed `solver_demo` graph-structural traffic for both
  `structural_rerank` and `constraint_filter` without widening the staged Rust
  graph-structural contract.
- `xiuxian-wendao-julia` also owns Julia-specific graph-structural route names, draft schema-version defaults, semantic projection DTOs, typed request or response row helpers, and Arrow batch validation for the mixed-graph structural plugin lane.
- `xiuxian-wendao-julia` also owns stable two-node pair projection helpers for that lane, including pair candidate id normalization, pair candidate subgraph projection, and pair-to-request-row builders.
- `xiuxian-wendao-julia` also owns simple keyword-or-tag query-context builders and binary keyword-or-tag rerank-signal builders for that lane, so host consumers do not manually create anchor DTOs or convert boolean matches into staged plane scores.
- `xiuxian-wendao-julia` also owns the next convenience layer above those helpers: combined keyword-or-tag pair-rerank request-row builders that compose query-context, rerank-signal, and pair-row projection in one plugin-owned call.
- `xiuxian-wendao-julia` also owns shared-tag overlap discovery for that lane, including normalized shared-tag anchor extraction and a tag-overlap-aware combined pair-rerank helper.
- `xiuxian-wendao-julia` also owns the metadata-aware convenience layer above that seam, including node-metadata input bundles and a metadata-aware overlap helper that keeps host consumers from passing ad hoc tag vectors into request projection.
- `xiuxian-wendao-julia` also owns the metadata-aware batch-assembly layer above that seam, including scored metadata-aware rerank input bundles and a batch helper that composes metadata projection and Arrow request materialization inside the plugin crate.
- `xiuxian-wendao-julia` also owns the higher-level candidate-input layer above that seam, including single-bundle keyword-overlap request inputs and a batch helper that composes query, metadata, pair, and score staging inside the plugin crate.
- `xiuxian-wendao-julia` also owns the shared-query and candidate-bundle layer above that seam, including one shared keyword-overlap query bundle, one plugin-owned per-pair candidate bundle, and a batch helper that derives the higher-level request inputs inside the plugin crate.
- `xiuxian-wendao-julia` also owns the raw-to-candidate staging helper above that seam, so host callers can hand over one pair-input DTO plus raw tag vectors and scores without manually constructing the node-metadata or candidate-bundle DTO layers first.
- `xiuxian-wendao-julia` also owns the raw-to-query staging helper above that seam, so host callers can hand over raw query identity, layer bounds, keyword anchors, and edge constraints without manually constructing the shared-query DTO layer first.
- `xiuxian-wendao-julia` also owns the raw-to-pair staging helper above that seam, so host callers can hand over raw pair ids and edge kinds without manually constructing the pair-input DTO layer first.
- `xiuxian-wendao-julia` also owns the raw pair-metadata-to-candidate staging helper above that seam, so host callers can hand over raw pair ids, edge kinds, left or right tags, and scores without manually composing the metadata-bundle helper and the candidate-bundle helper in sequence.
- `xiuxian-wendao-julia` also owns the raw-candidate collection batch or fetch seam above that layer, so host callers can hand over one shared query plus raw candidate bundles without manually normalizing each candidate before request-batch or repository-fetch dispatch.
- `xiuxian-wendao-julia` also now owns one generic explicit-edge topology seam above the pair helpers for structural rerank, so non-pair candidate graphs can be staged and fetched without pair normalization.
- `xiuxian-wendao-julia` also now owns one raw connected-pair staging seam above the scored pair-collection helper, so host callers can hand over connected pair ids plus semantic scores without first normalizing them into scored pair DTOs.
- `xiuxian-wendao-julia` also now owns the Julia capability-manifest Arrow seam, including route constants, typed manifest request or response rows, manifest transport option parsing, repository-scoped fetch helpers, manifest-to-binding decoding, and plugin-owned preflight validation against the live Julia capability-manifest route.
- the internal graph-structural projection surface now lives under the
  feature-folder `src/plugin/graph_structural_projection/` with interface-only
  `mod.rs` plus responsibility modules for core DTOs, generic topology,
  pair staging, overlap staging, request-row builders, and normalization
  support; that refactor preserved the existing public exports and live route
  proofs
- the graph-structural exchange test surface now follows the same pattern:
  `graph_structural_exchange.rs` keeps production code only, while
  `#[cfg(test)] #[path = "..."]` modules hold the unit and live proof suites
  in `graph_structural_exchange_tests.rs` and
  `graph_structural_exchange_generic_topology_tests.rs`
- `xiuxian-wendao-julia` also owns the legacy Julia link-graph compatibility semantics under `src/compatibility/link_graph/`, including Julia selector ids, the default analyzer package dir, launcher path, example-config path, the Julia rerank runtime record, service-descriptor and CLI-arg meaning, launch-manifest meaning, deployment-artifact meaning, and conversions to and from Wendao core plugin contracts.
- `xiuxian-wendao` hosts the analyzer registry and loads repository config, but it does not own a second transport implementation or a second graph-structural adapter layer.
- `xiuxian-wendao` gateway `code_search` now consumes only the shared
  repo-search seam plus the repo publications materialized from this crate's
  Julia-owned parser-summary line. It does not keep a second Rust-local Julia
  or Modelica AST execution path.
- `xiuxian-wendao` gateway `code_ast` now consumes the same Julia-owned native
  parser publications through the repo-aware analysis loader and does not keep
  a second Rust-local Julia or Modelica code-AST execution path.
- `xiuxian-wendao` now consumes this crate through a normal Cargo dependency instead of sibling-source inclusion.

## Public Surface

- `JuliaRepoIntelligencePlugin`
- `register_into`
- `build_julia_flight_transport_client`
- `process_julia_flight_batches`
- `julia_parser_summary_allows_safe_incremental_file_for_repository`
- `memory::*` for memory-family profile metadata, manifest projection helpers,
  runtime binding builders, and typed `episodic_recall`, `memory_gate_score`,
  `memory_plan_tuning`, and `memory_calibration` Arrow request or response
  helpers
- `memory::host::*` for plugin-owned host-adapter helpers over
  `xiuxian-memory-engine` read models, gate evidence, recall tuning inputs,
  and calibration job inputs
- `memory::transport::*` for memory-family Flight client construction,
  request or response validation dispatch, roundtrip execution, and typed
  fetch helpers for the four staged memory profiles
- `memory::downcall::*` for plugin-owned composition helpers that turn Rust
  memory-engine projection, evidence, tuning, or calibration inputs into one
  staged Julia downcall plus typed result rows
- `GraphStructuralRouteKind`
- `JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION`
- `graph_structural_route_kind`
- `is_graph_structural_route`
- `validate_graph_structural_*`
- `GraphStructuralQueryAnchor`
- `GraphStructuralQueryContext`
- `GraphStructuralCandidateSubgraph`
- `GraphStructuralKeywordTagQueryInputs`
- `GraphStructuralNodeMetadataInputs`
- `GraphStructuralKeywordOverlapPairInputs`
- `GraphStructuralKeywordOverlapPairRerankInputs`
- `GraphStructuralKeywordOverlapPairRequestInputs`
- `GraphStructuralPairCandidateInputs`
- `GraphStructuralKeywordOverlapQueryInputs`
- `GraphStructuralKeywordOverlapRawCandidateInputs`
- `GraphStructuralKeywordOverlapCandidateInputs`
- `GraphStructuralRawConnectedPairInputs`
- `GraphStructuralGenericTopologyCandidateMetadataInputs`
- `GraphStructuralGenericTopologyCandidateInputs`
- `GraphStructuralRerankSignals`
- `GraphStructuralFilterConstraint`
- `GraphStructural*RequestRow`
- `GraphStructural*ScoreRow`
- `graph_structural_pair_candidate_id`
- `graph_structural_shared_tag_anchors`
- `build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_inputs`
- `build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_metadata`
- `build_graph_structural_keyword_overlap_candidate_inputs`
- `build_graph_structural_keyword_overlap_raw_candidate_inputs`
- `build_graph_structural_keyword_overlap_pair_candidate_inputs_from_raw`
- `build_graph_structural_keyword_overlap_query_inputs`
- `build_graph_structural_pair_candidate_inputs`
- `build_graph_structural_raw_connected_pair_inputs`
- `build_graph_structural_keyword_overlap_pair_request_input`
- `build_graph_structural_keyword_overlap_pair_rerank_request_batch`
- `build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_raw_candidates`
- `build_graph_structural_generic_topology_candidate_metadata_inputs`
- `build_graph_structural_generic_topology_candidate_inputs`
- `build_graph_structural_generic_topology_candidate_inputs_from_raw_connected_pairs`
- `build_graph_structural_generic_topology_candidate_subgraph`
- `build_graph_structural_generic_topology_rerank_request_row`
- `build_graph_structural_generic_topology_rerank_request_batch`
- `build_graph_structural_keyword_overlap_pair_rerank_request_row`
- `build_graph_structural_keyword_overlap_pair_rerank_request_row_from_metadata`
- `build_graph_structural_keyword_tag_query_context`
- `build_graph_structural_keyword_tag_pair_rerank_request_row`
- `build_graph_structural_keyword_tag_rerank_signals`
- `build_graph_structural_pair_candidate_subgraph`
- `build_graph_structural_pair_*_request_row`
- `build_graph_structural_*_request_row`
- `build_graph_structural_*_request_batch`
- `decode_graph_structural_*_score_rows`
- `fetch_graph_structural_*_rows_for_repository`
- `fetch_graph_structural_generic_topology_rerank_rows_for_repository`
- `fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates`
- `JULIA_PLUGIN_CAPABILITY_MANIFEST_*`
- `JuliaPluginCapabilityManifestRequestRow`
- `JuliaPluginCapabilityManifestRow`
- `build_julia_capability_manifest_flight_transport_client`
- `build_julia_plugin_capability_manifest_request_batch`
- `decode_julia_plugin_capability_manifest_rows`
- `fetch_julia_plugin_capability_manifest_rows_for_repository`
- `process_julia_capability_manifest_flight_batches`
- `process_julia_capability_manifest_flight_batches_for_repository`
- `validate_julia_plugin_capability_manifest_*`
- `build_graph_structural_flight_transport_client`
- `process_graph_structural_flight_batches`
- `process_graph_structural_flight_batches_for_repository`
- `compatibility::link_graph::*` for Julia-owned legacy launch/deployment compatibility DTOs, the Julia rerank runtime record, selector helpers, and analyzer package-path defaults

The transport builder consumes repository plugin entries that resolve to:

```toml
[link_graph.projects.sample]
root = "/path/to/repo"
plugins = [
  "julia",
  { id = "julia", flight_transport = { base_url = "http://127.0.0.1:8815", route = "/rerank", health_route = "/healthz", timeout_secs = 15, max_in_flight_requests = 32 } }
]
```

The inline object is materialized by `xiuxian-wendao` as
`RepositoryPluginConfig::Config`, then interpreted here to construct a
runtime-owned Arrow Flight binding and negotiated Flight client.

The graph-structural transport surface now stages from a separate repository
plugin option block so Search downcalls can stay Julia-plugin-owned as well:

```toml
[link_graph.projects.sample]
root = "/path/to/repo"
plugins = [
  "julia",
  { id = "julia", graph_structural_transport = { base_url = "http://127.0.0.1:8815", max_in_flight_requests = 32, structural_rerank = { route = "/graph/structural/rerank", schema_version = "v0-draft" }, constraint_filter = { route = "/graph/structural/filter", timeout_secs = 20 } } }
]
```

That block is interpreted in `xiuxian-wendao-julia` rather than in
`xiuxian-wendao-runtime`. The runtime still owns generic Arrow Flight
negotiation only.
When that block is absent but `capability_manifest_transport` is configured,
`xiuxian-wendao-julia` now falls back to the live `/plugin/capabilities`
manifest and derives the graph-structural binding for the requested variant
inside the plugin crate.
That fallback is now also covered against one real same-port multi-route
`WendaoSearch.jl` demo service, so manifest discovery and structural-rerank
fetch are proven to work through the same Julia endpoint.
That same plugin-owned proof now also covers the bounded
`WendaoSearch.jl --mode solver_demo` rerank and filter lanes, both through
explicit graph-structural transport config and through capability-manifest
discovery, and the staged request shape now carries explicit edge endpoints.

The same ownership rule now also applies to plugin capability discovery. Rust
keeps static plugin identity registration, while the Julia plugin crate owns
the Arrow contract for a dedicated capability-manifest route:

```toml
[link_graph.projects.sample]
root = "/path/to/repo"
plugins = [
  "julia",
  { id = "julia", capability_manifest_transport = { base_url = "http://127.0.0.1:8815", route = "/plugin/capabilities", health_route = "/healthz", schema_version = "v0-draft", timeout_secs = 15 } }
]
```

That block is interpreted in `xiuxian-wendao-julia` and decoded into manifest
rows plus runtime `PluginCapabilityBinding` values. The host does not need a
second Julia-specific registration adapter layer for this discovery step.
When the block is configured, `JuliaRepoIntelligencePlugin::preflight_repository`
now also performs one plugin-owned live discovery roundtrip against
`/plugin/capabilities` before repository layout analysis continues.

The repository plugin config id remains `julia`, while the capability-manifest
rows themselves advertise the canonical provider id
`xiuxian-wendao-julia` so runtime provider selectors stay stable.

The same ownership rule now applies to the typed Rust exchange helpers for
these structural routes:

- semantic projection DTOs live in `xiuxian-wendao-julia`
- request-row structs and Arrow batch builders live in `xiuxian-wendao-julia`
- response-row structs and Arrow batch decoders live in `xiuxian-wendao-julia`
- repository-configured fetch helpers also live in `xiuxian-wendao-julia`
- `xiuxian-wendao` should consume or re-export that surface rather than grow a
  host-local graph-structural adapter module

The same rule also now has a bounded host-side proof in
`xiuxian-wendao`: the integration target
`packages/rust/crates/xiuxian-wendao/tests/xiuxian-testing-gate.rs`
through the `link_graph_agentic_expansion` unit module projects a real
`LinkGraphIndex` agentic-expansion pair through these Julia-owned pair helpers
and DTOs, then into a validated structural-rerank request batch, without
introducing a new production graph-structural adapter in the host crate.

That bounded proof now also consumes Julia-owned keyword-or-tag query and
binary rerank-signal helpers, so the host no longer manually creates
`GraphStructuralQueryAnchor` rows or converts boolean keyword-or-tag matches
into `1.0` or `0.0` plane scores by hand.

The same proof now also consumes a single Julia-owned combined helper for the
final staged rerank row, so the host no longer manually composes
`query context -> rerank signals -> pair rerank row` as three separate steps.
That convenience helper now accepts dedicated query and pair input bundles,
which keeps the public surface below the clippy argument-count ceiling without
moving the normalization logic back into host crates.

The same proof now also leaves shared-tag overlap discovery inside
`xiuxian-wendao-julia`, so the host only forwards raw left or right tag
metadata instead of finding the overlap itself.

The same proof now also stages those raw metadata slices through
plugin-owned metadata input bundles before building the staged rerank row, so
the host no longer threads raw tag vectors directly into the overlap helper.

The same proof now also consumes a plugin-owned metadata-aware batch helper,
so the host no longer assembles `Vec<GraphStructuralRerankRequestRow>` before
calling the staged Arrow batch builder.

The same proof now also consumes a single higher-level candidate-input bundle
per pair, so the host no longer manually composes query-input, metadata-input,
pair-input, and scored-rerank-input DTOs before building the staged request
batch.

The same proof now also consumes one shared query bundle plus one
plugin-owned per-candidate bundle per pair, so the host no longer constructs
`GraphStructuralKeywordOverlapPairRequestInputs` by hand before staging the
request batch.

The same Julia-owned seam now also has a repository-fetch convenience helper,
`fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository(...)`,
so a future host caller with query-plus-candidate DTOs can skip manual batch
materialization before calling the configured structural-rerank transport.
That bounded proof now consumes the graph-structural helper surface through
`xiuxian_wendao::analyzers::languages`, which keeps the host on the intended
thin language seam instead of importing the Julia crate directly.
That same proof now also consumes
`build_graph_structural_keyword_overlap_candidate_inputs(...)`, so the host
no longer manually constructs `GraphStructuralNodeMetadataInputs` or
`GraphStructuralKeywordOverlapCandidateInputs` before staging the rerank
request or repository fetch.
That same proof now also consumes
`build_graph_structural_keyword_overlap_query_inputs(...)`, so the host no
longer manually constructs `GraphStructuralKeywordOverlapQueryInputs` before
staging the rerank request or repository fetch.
That same thin-seam host proof now also covers the live
`WendaoSearch.jl --mode solver_demo` rerank and filter services without
widening the staged Rust request contract.
That staged Rust request contract now includes explicit edge endpoints, so the
same proof no longer relies on the Julia service's projected-path topology
assumption.
That same proof now also consumes
`build_graph_structural_pair_candidate_inputs(...)`, so the host no longer
manually constructs `GraphStructuralPairCandidateInputs` before staging the
rerank request or repository fetch.
That same proof now also consumes
`build_graph_structural_keyword_overlap_pair_candidate_inputs_from_raw(...)`,
so the host no longer manually composes
`build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(...)`
and `build_graph_structural_keyword_overlap_candidate_inputs(...)` before
staging the rerank request or repository fetch.
That same proof now also consumes
`build_graph_structural_keyword_overlap_raw_candidate_inputs(...)`,
`build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_raw_candidates(...)`,
and
`fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates(...)`,
so the host no longer manually normalizes each raw candidate before batch or
repository-fetch dispatch.
That same thin-seam live lane now also promotes one connected
`LinkGraphAgenticExpansionPlan` pair collection into the generic explicit-edge
topology helper path, so the three-node `solver_demo` proof no longer
hand-builds node and edge arrays in either the plugin crate or the host proof.
That same live lane now also owns one scored pair-collection helper above the
pair DTO seam, so generic-topology proofs no longer manually average pair
priorities or manually normalize connected pairs into
`GraphStructuralPairCandidateInputs`.
That same live lane now also owns one raw connected-pair helper above the
scored pair-collection seam, so host proofs no longer map
`LinkGraphAgenticCandidatePair` into scored pair DTOs before the generic
topology downcall.
That same live lane now also proves one multi-candidate generic-topology
batch against the same `WendaoSearch.jl --mode solver_demo` endpoint, both in
the plugin crate and through the host language seam, so the connected-pair
collection path is no longer limited to one candidate per request.
That same host-through-language-seam proof now also relies on a dedicated
host test-support extractor for connected pair collections, so
`link_graph_agentic/expansion.rs` no longer carries that collection-selection
algorithm inline while the live downcall behavior stays unchanged.
That same host-side proof now also relies on dedicated host test-support for
generic-topology manifest-discovery repository setup, shared query-context
setup, and baseline solver-demo row assertions, so `expansion.rs` keeps only
test intent plus pin-specific assertions while the Julia-owned fetch seam and
live contract stay unchanged.
That same live lane now also proves one higher-level seed-centered candidate
batch derived from a real `LinkGraphAgenticExpansionPlan`, so host proofs can
promote one more realistic mixed-graph batch above connected-pair collections
without changing the Julia-owned generic-topology fetch seam.
That same host-through-language-seam live lane now also proves one
worker-partition generic-topology batch derived from real
`LinkGraphAgenticWorkerPlan` partitions, so the current solver-demo route now
covers one more planner-shaped candidate batch above seed-centered groups.
That worker-partition proof now accepts mixed feasible and infeasible solver
rows inside the same batch, while still requiring at least one feasible live
result from the returned candidate set.
That same host live lane now also derives one batch-level generic-topology
query context from the real expansion-plan query plus selected worker seed
metadata, so the host proof no longer hard-codes `"alpha"` or `"related"`
inside the final manifest-discovered solver-demo downcall helper.
That same host live lane now also derives worker-batch dependency, keyword,
and tag scores from real plan-aware batch semantics before the downcall, and
validates those staged request-batch columns against the outgoing
generic-topology Arrow batch while the Julia-owned live contract remains
unchanged.
That same host live lane now also validates the staged `semantic_score`
request column derived from real worker-partition pair semantics, so the
outgoing generic-topology Arrow batch is now proven above one less implicit
Julia-owned normalization step while the live solver-demo contract remains
unchanged.
That same host live lane now also validates the staged `query_id`,
`retrieval_layer`, `query_max_layers`, `anchor_planes`, `anchor_values`, and
`edge_constraint_kinds` request columns against the same plan-aware batch
fixture before the live downcall, so the outgoing generic-topology Arrow batch
is now proven above one less implicit host-to-Julia query-context handoff.
That same host live lane now also validates the staged
`candidate_node_ids`, `candidate_edge_sources`,
`candidate_edge_destinations`, and `candidate_edge_kinds` request columns
against the same plan-aware batch fixture before the live downcall, so the
outgoing generic-topology Arrow batch is now proven above one less implicit
host-to-Julia topology handoff.
That same host live lane now also proves one plan-aware worker-partition
generic-topology `constraint_filter` batch above the same raw connected-pair
collection seam, and it now validates the staged `constraint_kind` and
`required_boundary_size` request columns before reusing that batch against the
manifest-discovered `WendaoSearch.jl --mode solver_demo` filter route.
That same host filter lane now also derives that staged
`required_boundary_size` from the current plan-aware anchor and candidate-
topology semantics, and it validates filter-side anchor and topology list
columns before the same live downcall.
That same host filter lane now also derives the staged `constraint_kind` from
that same batch shape, while the paired plugin live proof now exercises the
non-default `boundary_match` filter mode against the real solver-demo multi-
route endpoint.
That same plugin-owned live lane now also proves one multi-candidate generic-
topology `constraint_filter` batch against that same manifest-discovered
`WendaoSearch.jl --mode solver_demo` multi-route endpoint, and the real Julia
service tests are now serialized with a shared file lock so default
`cargo test -p xiuxian-wendao-julia graph_structural_exchange --lib` remains
stable under the repo's normal parallel Rust harness.
That same host generic-topology live lane now also derives its fallback edge
labels and staged `edge_constraint_kinds` from the normalized Wendao agentic
execution relation, so the manifest-discovered solver-demo downcall no longer
keeps a placeholder `"related"` edge semantic in host test support.
The capability-manifest response validator and generic-topology scored-pair
normalization are now also clippy-clean under `-D warnings`, so this live lane
no longer depends on local lint suppressions or precision-loss casts.
That bounded host-side proof now also exercises that public fetch helper
directly and confirms that the missing-transport failure still resolves through
the Julia-owned structural-rerank route instead of a host-local adapter layer.
This crate now also has a plugin-owned live loopback for that same fetch seam:
the `graph_structural_exchange` test module launches the real
`.data/WendaoSearch.jl/scripts/run_search_service.jl` entrypoint in demo mode,
waits for `/graph/structural/rerank` to accept Flight connections, and proves
`fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates(...)`
can decode a live structural-rerank response without any host-side adapter.

The transport client now sends `x-wendao-schema-version` and defaults to the
`v1` WendaoArrow contract unless the repository plugin config overrides
`schema_version`. This crate also stamps `wendao.schema_version` onto outgoing
request batch metadata so the managed Julia Flight services see the same
request-side contract boundary as the Rust rerank path.

The runtime-owned `validate_plugin_arrow_response_batches(...)` helper enforces
the current `v1` response shape before a future gateway integration accepts
analyzer output:

- required columns: `doc_id`, `analyzer_score`, `final_score`
- `doc_id` must be unique and non-null
- `final_score` must be finite

`process_julia_flight_batches` is the thin runtime hook for future gateway
integration. It performs:

- Arrow Flight roundtrip via `xiuxian-wendao-runtime`'s negotiated client
- response schema-version enforcement
- runtime-owned `v1` plugin Arrow response validation before returning
  decoded record batches

## Graph-Structural Draft Contract

The first mixed-graph structural plugin routes now stage from this crate
instead of `xiuxian-wendao-runtime`.

- schema version: `v0-draft`
- structural rerank route: `/graph/structural/rerank`
- constraint filter route: `/graph/structural/filter`

That means:

- `xiuxian-wendao-runtime` still owns generic Flight transport mechanics such
  as route normalization and negotiated clients
- `xiuxian-wendao-julia` owns the Julia-specific semantic contract and
  repository-config interpretation for these structural plugin exchanges
- future host dispatch should import these Julia-owned route and validation
  surfaces from this crate rather than adding another runtime-local contract

## Validation

- `direnv exec . cargo test -p xiuxian-wendao-julia transport --lib`
- `direnv exec . cargo test -p xiuxian-wendao-julia --lib rerank_exchange`
- `direnv exec . cargo test -p xiuxian-wendao-julia graph_structural_exchange --lib`
- `direnv exec . cargo test -p xiuxian-wendao-julia graph_structural_projection --lib`
- `direnv exec . cargo test -p xiuxian-wendao-julia process_julia_flight_batches_against_real_wendaoarrow_service --lib`
- `direnv exec . cargo test -p xiuxian-wendao-julia real_wendaoarrow_metadata_example_roundtrip_decodes_trace_id_column --lib`
- `direnv exec . cargo test -p xiuxian-wendao-julia fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates_against_real_wendaosearch_demo_service --lib`
- `direnv exec . cargo check -p xiuxian-wendao-julia --lib`
- `direnv exec . cargo test -p xiuxian-wendao --test xiuxian-testing-gate test_agentic_expansion_pair_projects_into_julia_graph_structural_request`
- `direnv exec . cargo test -p xiuxian-wendao --test xiuxian-testing-gate test_agentic_expansion_pair_uses_julia_graph_structural_fetch_helper`
- `direnv exec . cargo check -p xiuxian-wendao --features julia --test xiuxian-testing-gate`

The real loopback tests now speak only to the Flight examples. They spawn
`.data/WendaoArrow.jl/scripts/run_stream_scoring_flight_server.sh` and
`.data/WendaoArrow.jl/scripts/run_stream_metadata_flight_server.sh`, wait for the
Flight socket to accept connections, then send the canonical request batches
through the runtime-owned negotiated Flight client. Those fixtures now use the
shared `julia_arrow_request_schema(...)` builder as well, so the official
example roundtrip receives the full WendaoArrow `v1` request shape instead of a
test-local reduced schema.

There is also a metadata-aware real loopback that targets
`.data/WendaoArrow.jl/scripts/run_stream_metadata_flight_server.sh`, sends a
request whose Arrow schema metadata includes `trace_id`, and asserts the Rust
side can decode the additive `trace_id` response column. That path now goes
through the production Flight client, so the test verifies request schema
metadata survives the real Flight API instead of only a hand-written HTTP
fixture.

The corresponding test support is now split under `tests/unit/plugin/`
plus `tests/unit/memory/mod.rs`, mirroring the same semantic split used by
`xiuxian-wendao` integration support while keeping helper code out of the
production `src/` tree.
The custom WendaoArrow scoring helper in `integration_support/custom_service.rs`
now also emits its temporary Julia source files under project-cache ownership
rooted at `PRJ_CACHE_HOME`, and the cache-local namespace under that root is
declared in
`resources/integration_support/wendaoarrow_custom_service.toml` instead of
being hard-coded into the helper itself. It no longer writes numbered scripts
into the `WendaoArrow.jl` package git tree.
That official-example layer now includes real `WendaoSearch.jl` structural
launchers for both `demo` and `solver_demo`, so plugin-owned
graph-structural fetch helpers can be proven against a live Search child
service without moving route logic back into `xiuxian-wendao`.
Those live proofs now cover both hand-built generic topology smoke and the
real pair-collection promotion path above `LinkGraphAgenticExpansionPlan`.
They now also cover plugin-owned candidate-level semantic aggregation from that
raw pair collection before the generic-topology downcall.
They now also cover the one-step-higher raw connected-pair seam above that
aggregation path, so host proofs can forward only connected pair ids plus
semantic scores into Julia-owned staging before live downcall.
They now also keep the exchange implementation file lean by externalizing the
remaining unit and live proof modules behind `#[cfg(test)] #[path = "..."]`
without changing the green live baseline.
The crate now also follows the canonical shared gate shape:
`src/lib.rs -> tests/unit/lib_policy.rs` covers `cargo test --lib`, and
`tests/unit_test.rs` covers the explicit Cargo test target. The former inline
test debt in `src/integration_support/`, `src/memory/`, and `src/plugin/` is
now fully externalized into canonical `tests/unit/...` mounts, so the shared
crate test-policy harness passes without crate-local allowlists.
