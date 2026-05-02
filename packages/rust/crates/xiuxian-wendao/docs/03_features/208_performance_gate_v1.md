# Wendao Performance Gate V1

:PROPERTIES:
:ID: feat-wendao-performance-gate-v1
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: feature, performance, gate, nextest, criterion
:STATUS: ACTIVE
:VERSION: 1.0
:END:

## Overview

`xiuxian-wendao` now owns its performance gate helpers locally under
`tests/performance/support/perf.rs` and keeps a single gate entrypoint at
`tests/wendao-validation-gate.rs`.

No default test semantics were changed: performance suites run only when
feature flags are enabled.

## Feature Flags

- `performance`: enables performance gate tests.
- `performance-stress`: depends on `performance` and enables long-running
  ignored stress suites.

## Test Mounting Strategy

The unified gate mounts:

- `tests/performance/*` under `#[cfg(feature = "performance")]`
- `tests/performance/stress/*` under `#[cfg(feature = "performance-stress")]`
- required integration suites via `#[path = "integration/*.rs"]` under
  `#[cfg(not(feature = "performance"))]`
- a Rust project harness gate backed by `rust-lang-project-harness`

Root-level test wrappers are intentionally minimized. Integration and
performance tests are mounted from `tests/wendao-validation-gate.rs` instead of duplicated
`tests/*_test.rs` pass-through files.
Current root Rust entry files are:
`tests/wendao-validation-gate.rs`.

Suite layout:

- `latency_*`: PR-fast p95 latency gates.
- `throughput_*`: PR-fast throughput floor gates.
- `stress/*`: Nightly-only ignored stress gates.
- `gateway_search`: formal `tests/performance/gateway_search.rs` now mounts six
  serialized warm-cache gateway cases under the `performance` feature
  (`repo_module_search`, `repo_symbol_search`, `repo_example_search`,
  `repo_projected_page_search`, `studio_code_search`, and
  `search_index_status`) through the narrow
  `gateway::studio::perf_support` fixture surface.

## Budget Strategy

Default budgets are auditable Rust constants in the formal performance target
and still support environment overrides in CI.

This keeps the gateway perf lane explicit and reviewable inside
`tests/performance/gateway_search.rs`, where per-case budgets stay aligned
with the formal warm-cache cases instead of drifting behind a separate
calibration surface.

The formal gateway warm-cache lane resolves defaults through `RUNNER_OS`
runner profiles and accepts per-case overrides via:

- `XIUXIAN_WENDAO_GATEWAY_PERF_<CASE>_P50_MS`
- `XIUXIAN_WENDAO_GATEWAY_PERF_<CASE>_P95_MS`
- `XIUXIAN_WENDAO_GATEWAY_PERF_<CASE>_P99_MS`
- `XIUXIAN_WENDAO_GATEWAY_PERF_<CASE>_MIN_QPS`
- `XIUXIAN_WENDAO_GATEWAY_PERF_<CASE>_MAX_ERROR_RATE`

`<CASE>` is the uppercase formal gateway case id without the `_formal` suffix,
such as `REPO_MODULE_SEARCH`, `REPO_SYMBOL_SEARCH`, `REPO_EXAMPLE_SEARCH`,
`REPO_PROJECTED_PAGE_SEARCH`, `STUDIO_CODE_SEARCH`, or
`STUDIO_SEARCH_INDEX_STATUS`.

`Linux` keeps the stricter CI-oriented baseline constants, while `Local` and
`Other` use looser defaults for workstation noise without reopening a second
gateway calibration lane.

Current workstation-safe local defaults are:

- `repo_module_search`: `p95 <= 1.25ms`, `qps >= 500`
- `repo_symbol_search`: `p95 <= 1.25ms`, `qps >= 700`
- `repo_example_search`: `p95 <= 1.5ms`, `qps >= 600`
- `repo_projected_page_search`: `p95 <= 1.5ms`, `qps >= 700`
- `studio_code_search`: `p95 <= 10.0ms`, `qps >= 100`
- `search_index_status`: `p95 <= 0.48ms`, `qps >= 1250`

## Reporting Contract

Each run persists a JSON report under:

- `.run/reports/xiuxian-wendao/perf/*`
- `.run/reports/xiuxian-wendao/perf/stress/*`
- `.run/reports/xiuxian-wendao/perf-gateway-real-workspace/*` for the manual
  large-workspace gateway sample lane

## Criterion Layer

A Criterion bench target is available at:

- `benches/wendao_performance.rs`

It mirrors gate themes (`related_ppr`, `narration`) for trend analysis but is
not used as a PR blocker.

## CI Topology

- PR mainline CI (`.github/workflows/ci.yaml`, `.github/workflows/checks.yaml`)
  intentionally does not run Wendao performance lanes.
- Wendao performance gates run in the dedicated workflow:
  `.github/workflows/xiuxian-wendao-performance-gates.yaml`.
- `quick` profile is manual-only (`workflow_dispatch`).
- `nightly` profile runs on schedule and supports manual dispatch.
- Bench compile in nightly uses the fast lane and stays advisory
  (`continue-on-error`) to avoid blocking stability gates on runner noise.

## Validation Commands

- Preferred quick entrypoint:
  `direnv exec . just rust-wendao-performance-gate`
- Quick target only:
  `direnv exec . just rust-wendao-performance-quick`
- Formal gateway six-case proof:
  `direnv exec . just rust-wendao-performance-gateway-formal`
- Manual real-workspace gateway sample:
  `direnv exec . just rust-wendao-performance-gateway-real-workspace`
  This ignored lane defaults to `.data/wendao-frontend`, bootstraps the real
  repo-index until the workspace is query-ready, then samples cross-repo
  `code_search` and `repo/index/status`. The current local `wendao-frontend`
  sample covers `179` configured repositories.
- Direct formal gateway nextest proof:
  `direnv exec . cargo nextest run -p xiuxian-wendao --features performance --test wendao-validation-gate -E "test(performance::gateway_search::repo_module_search_perf_gate_reports_warm_cache_latency_formal_gate) | test(performance::gateway_search::repo_symbol_search_perf_gate_reports_warm_cache_latency_formal_gate) | test(performance::gateway_search::repo_example_search_perf_gate_reports_warm_cache_latency_formal_gate) | test(performance::gateway_search::repo_projected_page_search_perf_gate_reports_warm_cache_latency_formal_gate) | test(performance::gateway_search::studio_code_search_perf_gate_reports_warm_cache_latency_formal_gate) | test(performance::gateway_search::search_index_status_perf_gate_reports_query_telemetry_summary_formal_gate)"`
- PR quick gate:
  `direnv exec . cargo nextest run -p xiuxian-wendao --features performance --test wendao-validation-gate -E "not (test(performance::gateway_search::repo_module_search_perf_gate_reports_warm_cache_latency_formal_gate) | test(performance::gateway_search::repo_symbol_search_perf_gate_reports_warm_cache_latency_formal_gate) | test(performance::gateway_search::repo_example_search_perf_gate_reports_warm_cache_latency_formal_gate) | test(performance::gateway_search::repo_projected_page_search_perf_gate_reports_warm_cache_latency_formal_gate) | test(performance::gateway_search::studio_code_search_perf_gate_reports_warm_cache_latency_formal_gate) | test(performance::gateway_search::search_index_status_perf_gate_reports_query_telemetry_summary_formal_gate))"`
- Formal gateway perf listing:
  `direnv exec . cargo nextest list -p xiuxian-wendao --features performance --test wendao-validation-gate`
- Formal gateway six-case proof:
  `direnv exec . just rust-wendao-performance-gateway-formal`
- Default integration + structure gate:
  `direnv exec . cargo test -p xiuxian-wendao --test wendao-validation-gate`
- Nightly stress gate:
  `direnv exec . cargo nextest run -p xiuxian-wendao --features "performance performance-stress" --test wendao-validation-gate --run-ignored ignored-only`
- Bench fast compile proof (recommended):
  `direnv exec . env CARGO_PROFILE_BENCH_LTO=off CARGO_PROFILE_BENCH_CODEGEN_UNITS=16 CARGO_PROFILE_BENCH_DEBUG=0 cargo check -p xiuxian-wendao --features performance --benches`
- Bench no-run lane (heavy, advisory):
  `direnv exec . env CARGO_PROFILE_BENCH_LTO=off CARGO_PROFILE_BENCH_CODEGEN_UNITS=16 CARGO_PROFILE_BENCH_DEBUG=0 CARGO_TARGET_DIR=.cache/cargo-target/xiuxian-wendao-bench cargo bench -p xiuxian-wendao --features performance --bench wendao_performance --no-run`
