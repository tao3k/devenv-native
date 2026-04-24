---
type: knowledge
title: "Testing Documentation"
category: "testing"
tags:
  - testing
  - README
saliency_base: 6.5
decay_rate: 0.04
metadata:
  title: "Testing Documentation"
---

# Testing Documentation

This folder holds testing-related docs: evaluation reports, decision records, and guides.

## Canonical references

| Document                                                                                            | Use                                                                                                           |
| --------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| [修仙道场 (Xiuxian Daochang) Live Multi-Group Runbook](xiuxian-daochang-live-multigroup-runbook.md) | Canonical live validation flow for `Test1/Test2/Test3` session isolation + memory evolution + trace evidence. |

## Reports (historical or one-off)

The following are evaluation reports, statistical evidence, or one-off comparisons. Kept for traceability when they still describe active surfaces; for current behavior use the canonical docs and [Testing Guide](../developer/testing.md).

- `routing-quality-analysis.md`, `router-file-discovery-intent-report.md` — Routing quality and intent reports
- `llm_comprehension_test.md`, `test_kit_modernization.md`, `scenario-test-driven-autofix-loop.md` — Test design and modernization notes

Retired note:

- The vector-owned keyword-backend comparison corpus and report-generation
  loop have been retired with the old `xiuxian-vector` search surface. Use the
  Wendao/DuckDB retrieval utilities below for current search quality work.

## Main testing guide

For how to run tests, write tests, and use the test kit: [Developer Testing Guide](../developer/testing.md) and [Test Kit](../reference/test-kit.md).

## Performance Regression Utilities

- `scripts/benchmark_wendao_search.py` — local latency benchmark for `wendao search`.
- `scripts/evaluate_wendao_retrieval.py` — query-matrix regression for Top1/Top3/Top10 quality.
- `scripts/validate_wendao_gate_reports.py` — contract validation for PPR gate JSON artifacts.
- `scripts/render_wendao_ppr_rollout_status.py` — sustained-green rollout readiness evaluation.
- `docs/testing/wendao-query-regression-matrix.json` — canonical query matrix for retrieval regression.

Recommended usage:

```bash
# just wrapper (defaults: architecture, runs=5, warm_runs=2, debug, no-build)
just benchmark-wendao-search

# quick local sanity check (uses existing target/debug/wendao when present)
python scripts/benchmark_wendao_search.py --root . --query architecture --runs 5 --warm-runs 2 --no-build

# release-profile benchmark (recommended for performance review)
python scripts/benchmark_wendao_search.py --root . --query architecture --runs 5 --warm-runs 2 --release --no-build

# CI-style gate (fails on threshold breach)
python scripts/benchmark_wendao_search.py \
  --root . \
  --query architecture \
  --runs 7 \
  --warm-runs 2 \
  --no-build \
  --max-p95-ms 1500 \
  --max-avg-ms 1000

# retrieval quality regression (Top1/Top3/Top10 on fixed matrix)
python scripts/evaluate_wendao_retrieval.py \
  --binary .cache/target-codex-wendao/debug/wendao \
  --no-build \
  --json

# just wrappers
just evaluate-wendao-retrieval
just gate-wendao-ppr
just gate-wendao-ppr-report
just gate-wendao-ppr-mixed-canary
just validate-wendao-ppr-reports
just wendao-ppr-gate-summary
just wendao-ppr-rollout-status
just no-inline-python-guard

# gate scripts emit one-line CI diagnostics to stderr:
# - WENDAO_PPR_GATE ...
# - WENDAO_ROLLOUT ...
#
# CI default:
# - `XIUXIAN_WENDAO_GATE_SUMMARY_STRICT_GREEN=1` is enabled in CI, so gate-summary render fails unless both base and mixed are green.
# - for local advisory runs, set `XIUXIAN_WENDAO_GATE_SUMMARY_STRICT_GREEN=0`.

# default validation path (both local `just validate` and `just ci`)
# now includes:
# - unified WG2/WG3 gate via `test` -> `gate-wendao-ppr`
# - channel cursor contract gate via `test` -> `test-channel-cursor-contracts`
# - execution-path guard via `scripts/test_no_inline_python_exec_patterns.py`
#   (prevents `python -c` / inline `<<'PY'` patterns in shell/workflow/task files)
# - dedicated CI fail-fast task: `ci:no-inline-python-guard` -> `just no-inline-python-guard`
```

Developer-mode rule:

- Only rebuild Rust Python bindings when related code changes:
  `uv sync --reinstall-package xiuxian-core-rs`

## Mixed-Scope Rollout Sign-Off

Promote default retrieval scope from `doc_only` to `mixed` only after both conditions hold:

1. Mandatory gate (`just gate-wendao-ppr`) stays green for 7 consecutive CI runs.
2. Advisory mixed canary (`just gate-wendao-ppr-mixed-canary`) reports `top3_rate >= 0.90` for the same 7-run window, with valid report contracts (`just validate-wendao-ppr-reports`).

Rollout tracker artifacts (under `.run/reports/wendao-ppr-gate/`):

- `report_validation.json`
- `wendao_gate_status_summary.json`
- `wendao_gate_status_summary.md`
- `wendao_gate_rollout_status.md` (single-page gate + rollout summary)
- `wendao_rollout_status.json`
- `wendao_rollout_status.md`
- `wendao_rollout_status.previous.json` (when previous CI status fetch is enabled)
- `wendao_rollout_status.json` includes `readiness.gate_log_line`, and `scripts/wendao_ppr_rollout_ci.sh` prints the same one-line status to stderr for compact CI logs.
- CI step summary consumes `wendao_gate_rollout_status.md` (fallback: `wendao_rollout_status.md`).

Optional strict mode:

- CI now runs with `XIUXIAN_WENDAO_ROLLOUT_STRICT_READY=1` (hard gate).
- To avoid streak deadlock while strict is enabled, rollout tracker fetches previous status from `run-status=completed` history.
- Rollout strict check is placed near the end of CI, so full core test coverage still runs before readiness enforcement fails the job.
- For local dry runs, keep `XIUXIAN_WENDAO_ROLLOUT_STRICT_READY=0` unless you want strict failure behavior.
