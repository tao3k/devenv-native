#!/usr/bin/env bash
set -euo pipefail

bash -n \
  scripts/ci/rust_quality_gate_ci.sh \
  scripts/ci/test_quick.sh \
  scripts/rust/check_lint_inheritance.sh \
  scripts/rust/xiuxian_qianji_scenario_audit_contracts.sh \
  scripts/rust/xiuxian_wendao_contract_feedback_consumer.sh \
  scripts/rust/wendao_retrieval_audits.sh \
  scripts/runtime/valkey_live_gate.sh \
  scripts/runtime/valkey-runtime.sh \
  scripts/runtime/valkey-launch.sh \
  scripts/runtime/valkey-healthcheck.sh \
  scripts/runtime/wendao-frontend-healthcheck.sh \
  scripts/gate_wendao_ppr.sh \
  scripts/wendao_ppr_rollout_ci.sh

just --dry-run rust-xiuxian-qianji-scenario-audit-contracts >/dev/null
just --dry-run rust-xiuxian-wendao-contract-feedback-consumer >/dev/null
just --dry-run rust-retrieval-audits >/dev/null
just --dry-run gate-wendao-ppr >/dev/null
just --dry-run validate-wendao-ppr-reports >/dev/null
just --dry-run wendao-ppr-rollout-status >/dev/null
just --dry-run valkey-live >/dev/null
