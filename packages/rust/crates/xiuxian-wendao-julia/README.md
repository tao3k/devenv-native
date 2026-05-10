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
  launches the managed Julia services through explicit `julia --project=...`
  commands rather than repo-level `direnv` wrappers
- the remaining repeated `demo` capability-manifest live proofs are now also
  consolidated into one grouped test that covers manifest fetch, manifest
  preflight, graph-structural binding discovery, transport fallback, and
  plugin preflight against one live `WendaoSearch.jl` endpoint
- the current canonical full crate pass is now `139 passed` in `136.25s`,
  while preserving explicit transport, manifest-discovery, and grouped
  capability-manifest live coverage across the plugin lane
- Julia and Modelica parser-summary transport discovery now also works with
  plain repository plugin ids. `plugins = ["julia-code-parser"]` and
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
  `wendaocodeparser-parser-summary` background service for the native Julia and
  Modelica parser-summary lane; unlike `wendaosearch-solver-demo`, it launches
  the package-owned `config/live/parser_summary.toml` through
  `scripts/run_service.jl` and is the intended managed-service surface for
  gateway `code_search` and `code_ast` integration
- the Rust gateway parser-summary test seam now also understands that managed
  service directly: setting `RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST=1` makes the
  linked parser-summary helper bind to `wendaocodeparser-parser-summary` instead of
  self-spawning an in-process Julia service, so focused gateway search or
  code-AST proofs can exercise the same process-managed service shape that
  `process.nix` owns
- SearchStrategyFlow Flight materialization now keeps route namespaces
  explicit. Repo search resolves the candidate document, projected page-index
  and retrieval-context routes use the resolved page and section node ids, and
  graph-neighbor expansion uses a separate `resolvedGraphNodeId` based on the
  Studio display path. This prevents page-index section ids from being sent as
  link-graph node ids.
- Julia test support now lives under `tests/unit/plugin/` plus
  `tests/unit/memory/mod.rs` instead of production `src/` files, while
  `src/lib.rs` owns the root harness target so `cargo test --lib` executes the
  shared `rust-lang-project-harness` policy gate
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
- the self-spawn `WendaoSearch.jl --mode solver_demo` graph-structural seam
  now also has an opt-in communication profile under
  `RUN_WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_TEST=1 cargo test -p xiuxian-wendao-julia graph_structural_live_perf -- --nocapture`.
  That profile records startup, first-route, release-probe, sequential, and
  concurrent rerank/filter timings while preserving the existing Flight routes,
  Arrow schemas, Rust fallback boundary, and Julia execution ownership.
- that same graph-structural profile now accepts
  `WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_RUNS` and
  `WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_WARM_SAMPLES` to emit per-run samples
  plus min, median, p95, max, and spread-ratio summaries. The latest single-run
  macOS probes showed large cross-run variance, so the recorded numbers must
  be treated as instability evidence rather than a stable promotion baseline.
  A two-run, three-warm-sample probe kept release-prewarmed first explicit
  rerank at median `3.880 ms` while the release probe itself measured median
  `3868.476 ms`; however warm concurrent tails still reached `44.622 ms` with
  release prewarm and `136.179 ms` without it. The supported architectural
  direction is therefore Rust selecting and holding Julia pods until
  owner-supplied route probes prove warm readiness, with future promotion gates
  based on p95, max, and spread ratio rather than one observed latency. Julia
  still owns JIT warmup, optional thread pinning, and internal numerical
  scheduling.
- the graph-structural release-prewarm owner bridge now also exposes
  `stabilize_wendaosearch_solver_demo_graph_structural_routes(...)`. It runs
  the all-route release probe, samples sequential and concurrent warm paths,
  returns p95/max/spread summaries, and recommends an initial Rust
  `max_in_flight` budget. This keeps performance and stability coupled at pod
  release time: a warm Julia pod only receives a degraded admission budget when
  p95 or max latency crosses the configured tail budget. Low millisecond-level
  spread is recorded as scheduler evidence, not treated as user-visible
  instability by itself. The report also includes an explicit stability reason
  so downstream admission code can distinguish a true tail-budget overflow
  from harmless low-latency spread. In the first short real validation after
  this gate, the release gate reported `stable=true` and
  `recommended_max_in_flight=4`; first explicit rerank after release was
  `2.315 ms`, sequential warm p95 was `12.312 ms`, and concurrent warm p95 was
  `11.097 ms`.
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
- for
  [RFC: Polyglot Compute Orchestrator](../../../../../docs/rfcs/2026-05-04-polyglot-compute-orchestrator-rfc.md)
  Phase 3 readiness evidence, `xiuxian-wendao-julia` owns the Julia-side
  profile, schema, manifest, route-validation, warmup, benchmark, and
  readiness-evidence boundary. Rust may gate requests by profile, route,
  timeout, and in-flight budget, but this crate does not transfer Julia thread
  scheduling to Rust. The approved `xiuxian-polyglot-orchestrator` crate may
  define shared lane, admission, readiness, and snapshot contracts, but it must
  reference Julia profile and readiness facts through this package boundary.
- `xiuxian-wendao-julia` also owns the Julia profile schedule-plan projection
  for WendaoGraph link evidence, WendaoSearch graph-structural routes, legacy
  WendaoSearch rerank, and memory-family compute profiles. These helpers only
  convert owner-supplied runtime stats, admission counters, fallback
  availability, task shape, and latency constraints into inert
  `JuliaSchedulePlan` values; they do not call Julia, mutate queues, or execute
  Rust fallback algorithms.
- `xiuxian-wendao-julia` also mirrors the WendaoGraph PageIndex reasoning
  table contracts as a separate owner boundary from `/graph/link/evidence`.
  The PageIndex request contract covers `page_index_nodes`, `page_index_edges`,
  and `page_index_seeds`; the response contract covers those tables plus
  `reasoning_frontier` and `disclosure_trace`. Host crates may build sidecar
  batches against this mirror, but they must not widen the existing
  link-evidence route with PageIndex reasoning tables.
- the PageIndex reasoning mirror now has a git-stable host fixture under
  `packages/rust/crates/xiuxian-wendao/tests/fixtures/wendaograph_page_index_reasoning_host`.
  `xiuxian-wendao` compares that fixture against its Rust sidecar builder, and
  WendaoGraph.jl can consume the same tables through its native PageIndex
  reasoning test. This remains a fixture-level interop proof, not a Flight
  transport change.
- WendaoGraph.jl also exposes `page_index_reasoning_from_request(...)` for
  host-owned PageIndex request objects. This keeps PageIndex reasoning
  sidecars separate from `/graph/link/evidence` while letting host integration
  tests exercise the native Julia pipeline directly.
- the polyglot owner bridge now projects WendaoGraph PageIndex reasoning as
  the `wendao_graph_page_index_reasoning` Julia profile using the
  `WendaoGraph.page_index_reasoning_from_request` host entrypoint. The entry
  is scheduling evidence only; it is not a live Flight route.
- semantic SSOT-driven agent reasoning uses the same PageIndex owner boundary.
  `xiuxian-wendao` may project parser-owned semantic-scope facts into
  `page_index_nodes`, `page_index_edges`, and `page_index_seeds`; this package
  continues to own only the Julia profile and table-contract mirror. SSOT
  authority remains with the repo-native semantic artifacts and parser
  validation described in
  [`docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md`](../../../../docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md).
- the same PageIndex host entrypoint now has an opt-in real Julia process
  probe:
  `RUN_WENDAOGRAPH_PAGE_INDEX_HOST_PROBE_TEST=1 WENDAOGRAPH_PACKAGE_DIR=<WendaoGraph.jl checkout> cargo test -p xiuxian-wendao-julia --lib wendaograph_page_index_host_probe -- --nocapture`.
  The probe runs the host fixture through `page_index_reasoning_from_request`,
  validates frontier and disclosure-trace counts, and prints first-call plus
  warm-call timing evidence. It deliberately avoids creating a Flight service
  or route before a later service-boundary ExecPlan approves one. Set
  `WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_ACTIONS=1` on the same command to also
  compute `page_index_planner_action_table(...)` from the returned frontier and
  report planner action rows plus expand, compare, jump, and stop counts.
- the PageIndex host-probe helpers also accept an explicit fixture directory.
  This lets `xiuxian-wendao` materialize semantic SSOT projections as
  temporary PageIndex TSV fixtures and validate them against the real
  WendaoGraph.jl host entrypoint without mutating global environment state.
- the PageIndex host-probe evidence also projects into the owner-side polyglot
  readiness bridge for the existing `wendao_graph_page_index_reasoning` Julia
  profile. The bridge maps warm median and p95 probe timings into
  `JuliaRuntimeStats`, preserves positive sub-millisecond observations as
  `1 ms` scheduler facts, and keeps planner action counts as validation
  evidence rather than direct readiness gates.
- the owner bridge also exposes an inert `WendaoGraph.jl` algorithm catalog.
  The catalog maps LinkGraph structural, semantic-overlay, diffusion, and
  frontier helpers; PageIndex frontier, disclosure-trace, and planner-action
  helpers; SearchStrategyFlow candidate, transition, frontier, and table
  helpers; and GNN feature, graph, score, and frontier helpers to their owning
  Julia profile, Julia entrypoint, output table when applicable, and scheduler
  complexity hint. Rust can use this as capability evidence for later
  algorithm-aware planning, but the catalog does not call Julia, add a route,
  widen a schema, or gate admission by itself.
- the SearchStrategyFlow catalog entries mirror the graph-owned pure contract
  in WendaoGraph.jl: `WendaoGraph.strategy_flow_candidate_rows`,
  `WendaoGraph.strategy_flow_transition_rows`,
  `WendaoGraph.strategy_flow_frontier_rows`, and
  `WendaoGraph.strategy_flow_tables`. They intentionally reuse the existing
  `wendao_graph_page_index_reasoning` profile until a live SearchStrategyFlow
  route is proven; this is static owner evidence, not live Julia execution.
- the Rust bridge for SearchStrategyFlow now builds real candidate inputs from
  Markdown heading sections under the configured search root before invoking
  Julia. Rust scores only task-local discovery evidence such as intent-term
  coverage, path/title matches, section context cost, and edge-kind hints.
  `WendaoGraph.jl` remains the owner of SearchStrategyFlow scoring, frontier
  pruning, transition inference, and planner actions. The returned trace
  records `candidateInputSource="rust-markdown-headings"` when the bridge is
  using real search-root candidates instead of the fixed proof fallback.
- when a SearchStrategyFlow Flight materialization endpoint is configured, the
  bridge first asks the Studio `/search/repos/main` Arrow Flight route for
  repo-native candidate rows and passes those rows to Julia with
  `candidateInputSource="rust-flight-repo-search"`. This keeps section
  discovery on the indexed Rust/Studio side while Julia owns graph strategy
  flow decisions. The local Markdown heading scan remains the no-endpoint
  smoke path and is not a TypeScript or `pi-wendao` responsibility.
- the algorithm catalog now also exposes a relationship-search subset for
  HNSW semantic fanout, MOC-style community grouping, PPR-like relatedness,
  graph search ranking, and large object-graph traversal. These entries map to
  Julia-owned `WendaoGraph.hnsw_neighbor_rows`,
  `WendaoGraph.topology_community_rows`,
  `WendaoGraph.multi_plane_diffusion_scores`,
  `WendaoGraph.link_frontier_rows`, `WendaoGraph.sparse_adjacency`, and
  `WendaoGraph.build_graph_snapshot` surfaces while preserving the existing
  `wendao_graph_link_evidence` profile boundary. They are scheduling evidence
  only and do not create a new Flight route or Arrow schema.
- relationship-search algorithm ids can now also be projected from the
  existing full structural LinkGraph host-probe report into per-algorithm
  scheduling evidence. The projection records the backing probe table, observed
  row count, warm p50/p95 stats, and the existing `JuliaSchedulePlan` result
  for each relationship-search id. HNSW semantic fanout is backed by the
  semantic-overlay probe surface, PPR-like relatedness by `diffusion_scores`,
  ranking by `link_frontier` or `topology_candidates`, MOC-style grouping by
  community tables, and large traversal by component or graph-metric evidence.
  The projection is observational only; row counts do not become hard
  readiness gates.
- the same relationship-search evidence projection has an opt-in real
  WendaoGraph host-process proof:
  `RUN_WENDAOGRAPH_RELATIONSHIP_SEARCH_LIVE_PERF_TEST=1 WENDAOGRAPH_PACKAGE_DIR=<WendaoGraph.jl checkout> cargo test -p xiuxian-wendao-julia --lib wendaograph_relationship_search_live_perf -- --nocapture`.
  The proof runs the full structural LinkGraph host probe, projects all
  relationship-search ids, and prints compact evidence rows containing
  algorithm id, backing probe table, row count, p50/p95, schedule action,
  confidence, and selected batch size. The first local run against the small
  host fixture projected all ten relationship-search ids to `Dispatch` with
  p50/p95 `1 ms` scheduler facts and `batch_size=4`; those numbers are live
  evidence for the fixture, not a large-graph promotion baseline.
- that relationship-search proof can now drive an env-sized synthetic LinkGraph
  workload through the same host-process entrypoint by setting
  `WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_MODE=synthetic-large` plus
  `WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_NODES`,
  `WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_FANOUT`, and
  `WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_SEMANTIC_NEIGHBORS`. The probe report
  records the selected mode, input node count, input edge count, and semantic
  neighbor count, and the relationship-search schedule evidence derives its
  task shape from those observed counts.
- the synthetic relationship-search proof now also has an opt-in repeated-run
  stability mode:
  `RUN_WENDAOGRAPH_RELATIONSHIP_SEARCH_SYNTHETIC_STABILITY_TEST=1 WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_MODE=synthetic-large WENDAOGRAPH_PACKAGE_DIR=<WendaoGraph.jl checkout> cargo test -p xiuxian-wendao-julia --lib wendaograph_relationship_search_synthetic_stability -- --nocapture`.
  Set `WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_STABILITY_RUNS` to control the number
  of repeated host-process probes. The proof prints run count, algorithm count,
  action counts, latency p50/p95, warm max, warm spread ratio, selected batch
  size, and observed graph size. It also writes a cache-local JSON receipt
  under the project cache by default; set
  `WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_STABILITY_RECEIPT` to redirect that
  receipt for focused validation. The same proof now reads the receipt back and
  emits a receipt-backed `Candidate` or `Reject` gate verdict from action
  counts, row counts, latency p95, and warm spread ratio, so later automation
  can consume the result without scraping stdout. The first receipt-backed
  local run over 128 nodes and 512 edges projected all twenty evidence rows to
  `Dispatch`, reported scheduler p50/p95 facts of `3/5 ms`, and emitted
  `Candidate` with warm spread ratio `1.710`. These synthetic runs remain
  local evidence probes rather than final promotion baselines until persisted
  benchmark artifacts and p99 gates are added.
- the relationship-search stability, receipt, promotion-gate, and opt-in live
  proof tests now live under
  `tests/unit/integration_support/wendaograph/relationship_search.rs`. The
  parent `wendaograph.rs` test module keeps parser, PageIndex, and LinkGraph
  host-probe contracts, so future p99 gate work can evolve without expanding
  the mixed integration-support file. The live proof prints the receipt-backed
  gate verdict by default; set
  `WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_STABILITY_REQUIRE_CANDIDATE=1` when a
  promotion run should fail unless the receipt is a `Candidate`.
- the catalog now has a shape bridge: host code can provide a
  `WendaoGraphAlgorithmWorkload` for a specific algorithm id and receive a
  `JuliaComputeTaskShape` with the catalog complexity hint and a stable
  profile/algorithm batchability key. A thin algorithm schedule-plan helper
  routes known LinkGraph, PageIndex, SearchStrategyFlow, and GNN algorithm ids
  through the existing profile-specific schedule helpers; unknown algorithm ids
  return `None` rather than creating an admission rejection.
- the WendaoGraph LinkGraph host-request entrypoint now has an opt-in real
  Julia process probe:
  `RUN_WENDAOGRAPH_LINK_GRAPH_HOST_PROBE_TEST=1 WENDAOGRAPH_PACKAGE_DIR=<WendaoGraph.jl checkout> cargo test -p xiuxian-wendao-julia --lib wendaograph_link_graph_host_probe -- --nocapture`.
  The probe runs `link_graph_evidence_from_request(...)` with
  `semantic_neighbors` by default. Set
  `WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_MODE=semantic-overlay` to run the same
  host-process probe with precomputed `semantic_overlay` rows instead. Both
  modes validate the full 17-table LinkGraph response bundle and print
  first-call plus warm-call timing evidence. The full structural probe report
  records graph metrics, components, topology profile/candidate/bottleneck,
  community, cover, core, boundary, transition, gateway, community summary,
  community link, community frontier, semantic overlay, diffusion, and
  link-frontier row counts. They remain host-process probes, not Flight routes.
- the LinkGraph full structural probe evidence also projects into the
  owner-side polyglot readiness bridge for the existing
  `wendao_graph_link_evidence` Julia profile. The bridge maps warm median and
  p95 probe timings into `JuliaRuntimeStats` and leaves the full structural row
  counts as validation evidence. It does not add a Flight route, widen the
  Arrow schema, or turn row counts into direct readiness gates.
- the WendaoGraph GNN reasoning surface now has a separate opt-in real Julia
  process probe:
  `RUN_WENDAOGRAPH_GNN_HOST_PROBE_TEST=1 WENDAOGRAPH_PACKAGE_DIR=<WendaoGraph.jl checkout> cargo test -p xiuxian-wendao-julia --lib wendaograph_gnn_host_probe -- --nocapture`.
  The probe builds a deterministic graph, computes topology-plus-embedding
  node features, constructs the `GNNGraph`, runs seeded CPU GCN scores,
  projects GNN frontier rows, and records backend diagnostics for Metal, CUDA,
  and AMDGPU. Metal functionality is diagnostic only; unavailable Metal does
  not fail the host probe. This remains a host-process proof, not a Flight
  route.
- the GNN probe evidence also projects into the owner-side polyglot readiness
  bridge as the `wendao_graph_gnn_reasoning` Julia profile. The bridge maps
  warm median and p95 probe timings into `JuliaRuntimeStats`, carries
  Metal/CUDA/AMDGPU diagnostics as non-gating readiness evidence, and leaves
  cold-start handling to Julia pod warmup or release policy. It does not add a
  Flight route or move accelerator selection into Rust.
- `xiuxian-wendao-julia` also owns the Julia-side warmup and thread-diagnostic
  evidence seam for WendaoSearch and WendaoGraph profile readiness. Rust may
  request pod-level prewarm and record `ThreadPinning.jl` availability or
  policy diagnostics, but Julia remains the only owner of JIT warmup execution,
  thread-pinning application, internal queues, and numerical work scheduling.
- `xiuxian-wendao-julia` also exposes the owner-side
  `prewarm_wendaosearch_solver_demo_graph_structural_routes(...)` helper for
  process or pod release probes. The helper warms the stable WendaoSearch
  solver-demo graph-structural route family through existing Flight contracts
  and returns typed route-count, elapsed-time, and candidate-id evidence;
  orchestration layers may consume that evidence but must not duplicate
  WendaoSearch-specific request construction.
- the active `rust-lang-project-harness` profile marks `src/polyglot/` as the
  Julia polyglot bridge for readiness evidence projection. That profile records
  Julia profile/schema/manifest/readiness ownership without moving live Julia
  scheduling into Rust.
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
- `WENDAO_GRAPH_LINK_EVIDENCE_ROUTE`
- `WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION`
- `WENDAO_GRAPH_EVIDENCE_*_TABLE_NAMES`
- `WENDAO_GRAPH_EVIDENCE_*_TABLE_CONTRACTS`
- `wendao_graph_evidence_table_schema`
- `validate_wendao_graph_evidence_*_schema`
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
  { id = "julia-code-parser", flight_transport = { base_url = "http://127.0.0.1:8815", route = "/rerank", health_route = "/healthz", timeout_secs = 15, max_in_flight_requests = 32 } }
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
  { id = "julia-code-parser", graph_structural_transport = { base_url = "http://127.0.0.1:8815", max_in_flight_requests = 32, structural_rerank = { route = "/graph/structural/rerank", schema_version = "v0-draft" }, constraint_filter = { route = "/graph/structural/filter", timeout_secs = 20 } } }
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
  { id = "julia-code-parser", capability_manifest_transport = { base_url = "http://127.0.0.1:8815", route = "/plugin/capabilities", health_route = "/healthz", schema_version = "v0-draft", timeout_secs = 15 } }
]
```

That block is interpreted in `xiuxian-wendao-julia` and decoded into manifest
rows plus runtime `PluginCapabilityBinding` values. The host does not need a
second Julia-specific registration adapter layer for this discovery step.
When the block is configured, `JuliaRepoIntelligencePlugin::preflight_repository`
now also performs one plugin-owned live discovery roundtrip against
`/plugin/capabilities` before repository layout analysis continues.

The repository plugin config id is `julia-code-parser`, while the
capability-manifest rows themselves advertise the canonical provider id
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
`packages/rust/crates/xiuxian-wendao/tests/wendao-validation-gate.rs`
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
- `direnv exec . cargo test -p xiuxian-wendao --test wendao-validation-gate test_agentic_expansion_pair_projects_into_julia_graph_structural_request`
- `direnv exec . cargo test -p xiuxian-wendao --test wendao-validation-gate test_agentic_expansion_pair_uses_julia_graph_structural_fetch_helper`
- `direnv exec . cargo check -p xiuxian-wendao --features julia --test wendao-validation-gate`

The real loopback tests now speak only to the managed Flight services. They spawn
`.data/WendaoArrow.jl/scripts/run_stream_scoring_flight_server.sh` and
`.data/WendaoArrow.jl/scripts/run_stream_metadata_flight_server.sh`, wait for the
Flight socket to accept connections, then send the canonical request batches
through the runtime-owned negotiated Flight client. Those fixtures now use the
shared `julia_arrow_request_schema(...)` builder as well, so the managed
roundtrip receives the full WendaoArrow `v1` request shape instead of a
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
That managed service layer now includes real `WendaoSearch.jl` structural
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
`src/lib.rs` covers `cargo test --lib`. The former inline test debt in
`src/integration_support/`, `src/memory/`, and `src/plugin/` is now fully
externalized into canonical `tests/unit/...` mounts, so the shared crate
test-policy harness passes without crate-local allowlists.
