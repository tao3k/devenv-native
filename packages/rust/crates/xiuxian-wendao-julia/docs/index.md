# xiuxian-wendao-julia: Map of Content

:PROPERTIES:
:ID: 3ceefb88f9107b7e479ec3249ccc974e7cf77616
:TYPE: INDEX
:STATUS: ACTIVE
:END:

Standardized documentation index for the `xiuxian-wendao-julia` package.

This package owns Julia profile, schema, manifest, transport, readiness, and
memory-family helper contracts for Wendao compute integrations. Rust may use
these facts for admission and readiness evidence, but Julia execution,
profile semantics, and live worker behavior remain owned here.

Polyglot boundary:

1. `src/polyglot/` translates Julia-owned profile, manifest, route, schema,
   warmup, benchmark, and admission-window facts into
   `xiuxian-polyglot-orchestrator` readiness contracts.
2. The active readiness coverage is mounted into the lib target from
   `tests/unit/polyglot/`.
3. `src/bin/wendaograph_search_strategy_flow.rs` is a formal CLI entry point for
   Rust-owned SearchStrategyFlow dispatch into `WendaoGraph.jl`; it preserves
   Julia ownership of graph scoring, frontier pruning, and planner action
   generation. The bridge now has two Rust candidate sources before invoking
   Julia: a no-endpoint Markdown heading scan for local smoke tests and a
   Studio `/search/repos/main` Arrow Flight source when materialization config
   is present. The local scan remains Markdown-only. The Flight source leaves
   repo-search language filters empty by default and relies on query, path,
   page-index, parser availability, and authority overlays to decide the
   downstream evidence path. `xiuxian-ast`/ast-grep-backed search is the
   general AST baseline for supported source languages. The bridge consumes
   `AstParserRegistry` from `xiuxian-ast` and registers local overrides for
   `rust-lang-parser`, `markdown-lang-parser`, `julia-lang-parser`, and
   `modelica-lang-parser`. Those local/native/plugin parsers have priority
   over the general baseline when they own richer domain facts and become the
   effective parser for their surfaces, while the general AST baseline remains a
   comparable supporting evidence plane when available. Both sources pass
   bounded candidates into `WendaoGraph.jl` for deterministic
   SearchStrategyFlow scoring. The Flight source runs route-scoped authority
   attempts before broad repo-search windows, ranks merged candidates before
   truncation, and calibrates package owner docs, validation docs, and
   ownership RFCs as authority evidence so Julia frontier scoring does not
   promote generic LinkGraph mentions over required evidence. It keeps one best
   candidate per source path before Julia selection, so duplicate sections from
   one document cannot displace required validation or ownership branches. Julia
   still owns the graph algorithm; Rust owns evidence discovery, Flight route
   planning, and materialization receipts. The bridge enriches the returned
   trace with additive
   `retrievalRoutes` plans for Studio-owned Arrow Flight materialization routes
   so downstream agents can inspect the native route sequence without
   rebuilding local fixture plans. It must not mark those routes as executed
   until Rust has completed real Arrow Flight network materialization. When a
   Flight endpoint is configured, Rust decodes Arrow batches with
   `arrow-flight` and returns compact JSON receipts; TypeScript callers do not
   own JS Arrow decoding in this path. The bridge keeps page-index section
   nodes and link-graph document nodes in separate namespaces: retrieval
   context uses the resolved section node, while graph-neighbor expansion uses
   `resolvedGraphNodeId` from the Studio display path. Executed receipts keep
   `sourcePath`, `headingAnchor`, `repoSearchResolutionStatus`,
   `materializedRows`, route receipts, decoded payload receipts, and evidence
   anchors stable so downstream agents can consume section-level provenance
   without opening full Markdown files. A selected `sourcePath` remains the
   fallback structure contract when repo-search cannot rediscover the exact
   row; page-index, retrieval-context, and graph-neighbor routes still execute
   through the real Flight host. Graph-neighbor materialization is intentionally
   a compact one-hop relation proof rather than a two-hop neighborhood dump, so
   live traces preserve relation-path evidence without large graph fanout. The
   bridge also records additive `elapsedMs` timings for candidate-discovery
   attempts and executed materialization routes. Candidate discovery can stop
   after the required attempt floor once enough unique source paths are present,
   while the host-local projection cache keeps projected page-index and
   retrieval-context materialization in the millisecond range. The measured
   next boundary is the persistent batch host: one warm Julia process preserves
   the same frontier and route contracts while reducing repeated 32-family
   replay cold submits from the `7774 ms-8907 ms` range to warm submits in the
   `48 ms-164 ms` range. That host is now available as an integration-support
   surface for Rust-controlled Julia pod promotion, with
   `SearchStrategyFlowPersistentBatchHost::submit_with_flight_materialization`
   as the real Flight-backed warm-host entry point and
   `stabilize_with_flight_materialization` as the pre-release admission report
   path. The report exports a stable JSON evidence object for later harness or
   receipt archival. The `wendaograph_search_strategy_flow` binary exposes that
   report path through `--persistent-warm-samples <count>` without changing the
   default trace schema. Root-backed warm Flight evidence currently shows
   direct bridge repeats at `15.01s` and `13.11s`, while one persistent host
   report measured `19363.439 ms` prewarm and `54.343 ms` / `145.905 ms` warm
   submits. The binary also exposes `--serve-stdio` as a JSONL local-session
   adapter that keeps one Rust bridge and Julia host alive across requests; the
   first two-request root-backed proof measured `20557.705 ms` for the warmup
   request and `83.391 ms` for the second request while preserving required
   evidence coverage.
   The ontology read-model quality bridge is separate from SearchStrategyFlow:
   `build_wendaograph_ontology_read_model_quality_arrow_request(...)` packages
   accepted semantic read-model `RecordBatch` tables as Arrow IPC streams for
   `WendaoGraph.jl`'s `OntologyReadModelQuality` service contract. It keeps
   read-model materialization in `xiuxian-wendao-sql`, graph-quality scoring in
   `WendaoGraph.jl`, and Rust bridge responsibility limited to Arrow-native
   request packaging. The companion
   `build_wendaograph_ontology_read_model_quality_flight_request_batch(...)`
   wraps the three payloads into one Arrow request-bundle table for the
   WendaoGraph Flight route, and
   `build_wendaograph_ontology_read_model_quality_flight_descriptor(...)`
   builds the matching `FlightDescriptor` path.
   `build_wendaograph_ontology_read_model_quality_flight_binding(...)` builds
   the runtime-negotiable Arrow Flight binding for callers that use the shared
   Wendao transport client, and
   `roundtrip_wendaograph_ontology_read_model_quality_with_binding(...)` sends
   the request bundle through that runtime-owned negotiated client. The focused
   Rust smoke constructs those payloads
   from `xiuxian-wendao-sql` semantic read-model `RecordBatch` output before
   bundle packaging, proving the bridge consumes the SQL owner surface rather
   than a registry JSON file. The opt-in live loopback test starts the
   WendaoGraph ontology quality runner and sends the same bundle through the
   runtime Flight client; it is not part of the default test path because it
   launches a real Julia service process.
   The bridge also adds `rustProjectedEvidenceRows` to the JSON trace as
   additive research metadata over candidates, frontier selection, planner
   materialization, and planned route counts. That projection is a bridge
   receipt surface for SearchStrategyFlow research notebooks; it does not add a
   public Arrow schema or a new Flight route.
4. The bridge does not transfer Julia scheduling, route mutation, or schema
   authority to Rust.

Verification profile:

1. `cargo test -p xiuxian-wendao-julia --lib polyglot` covers the Julia
   polyglot readiness bridge.
2. `cargo test -p xiuxian-wendao-julia --lib enforce_rust_project_harness_gate`
   covers the shared harness profile gate.

---

:FOOTER:
:STANDARDS: v2.0
:LAST_SYNC: 2026-05-10
:END:
