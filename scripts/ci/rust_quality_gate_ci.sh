#!/usr/bin/env bash
set -euo pipefail

timeout_secs="${1:-3600}"

just rust-lint-inheritance-check
just rust-test-layout
just rust-check "${timeout_secs}"
just rust-clippy
just rust-nextest
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
just rust-test-xiuxian-core-rs
just rust-security-gate
