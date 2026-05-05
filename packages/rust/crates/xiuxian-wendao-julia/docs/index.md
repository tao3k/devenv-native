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
3. The bridge does not transfer Julia scheduling, route mutation, or schema
   authority to Rust.

Verification profile:

1. `cargo test -p xiuxian-wendao-julia --lib polyglot` covers the Julia
   polyglot readiness bridge.
2. `cargo test -p xiuxian-wendao-julia --lib enforce_rust_project_harness_gate`
   covers the shared harness profile gate.

---

:FOOTER:
:STANDARDS: v2.0
:LAST_SYNC: 2026-05-05
:END:
