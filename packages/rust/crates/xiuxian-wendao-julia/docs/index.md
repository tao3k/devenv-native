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

1. `src/polyglot.rs` translates Julia-owned profile, manifest, route, schema,
   warmup, benchmark, and admission-window facts into
   `xiuxian-polyglot-orchestrator` readiness contracts.
2. The active readiness coverage is mounted into the lib target from
   `tests/unit/polyglot.rs`.
3. `examples/wendaograph_search_strategy_flow.rs` is a CLI proof surface for
   Rust-owned SearchStrategyFlow dispatch into `WendaoGraph.jl`; it preserves
   Julia ownership of graph scoring, frontier pruning, and planner action
   generation. The bridge now has two Rust candidate sources before invoking
   Julia: a no-endpoint Markdown heading scan for local smoke tests and a
   Studio `/search/repos/main` Arrow Flight source when materialization config
   is present. Both sources pass section-level candidates into
   `WendaoGraph.jl` for deterministic SearchStrategyFlow scoring. Julia still
   owns the graph algorithm; Rust owns evidence discovery, Flight route
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
   `resolvedGraphNodeId` from the Studio display path.
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
:LAST_SYNC: 2026-05-09
:END:
