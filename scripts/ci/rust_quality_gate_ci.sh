#!/usr/bin/env bash
set -euo pipefail

timeout_secs="${1:-3600}"

just rust-lint-inheritance-check
just rust-test-layout
just rust-check "${timeout_secs}"
just rust-clippy
just rust-nextest
just rust-xiuxian-daochang-backend-role-contracts
just rust-xiuxian-qianji-scenario-audit-contracts
if ! command -v cargo-nextest >/dev/null 2>&1; then
  echo "cargo-nextest is required but not installed." >&2
  echo "Install with: nix profile add nixpkgs#cargo-nextest" >&2
  exit 1
fi

CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}" cargo nextest run -p xiuxian-vector \
  --test integration_test \
  --test performance_test \
  --no-fail-fast
just rust-xiuxian-wendao-contract-feedback-consumer
if [[ ${OMNI_ENABLE_EMBED_ROLE_PERF_GATE:-0} == "1" ]]; then
  profile="${OMNI_EMBED_ROLE_PERF_GATE_PROFILE:-medium}"
  case "${profile}" in
  medium)
    just rust-xiuxian-daochang-embedding-role-perf-medium-gate
    ;;
  heavy)
    just rust-xiuxian-daochang-embedding-role-perf-heavy-gate
    ;;
  *)
    echo "Unsupported OMNI_EMBED_ROLE_PERF_GATE_PROFILE='${profile}' (expected: medium|heavy)." >&2
    exit 1
    ;;
  esac
fi
just rust-test-xiuxian-core-rs
just rust-security-gate
