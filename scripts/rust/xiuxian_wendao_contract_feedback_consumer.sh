#!/usr/bin/env bash
set -euo pipefail

run_cargo() {
  if [[ -n ${CARGO_BIN:-} ]]; then
    "${CARGO_BIN}" "$@"
  else
    direnv exec . cargo "$@"
  fi
}
target_dir="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}"

# Downstream consumer lane:
# - rest_docs warning findings remain mappable to Wendao reference knowledge
# - modularity warning findings remain mappable to Wendao architecture knowledge
# - qianji persisted rest_docs flow writes Wendao-native entries through a sink
CARGO_TARGET_DIR="${target_dir}" run_cargo test -p xiuxian-wendao --lib \
  contract_feedback_adapter_

CARGO_TARGET_DIR="${target_dir}" run_cargo test -p xiuxian-qianji \
  --test wendao_persisted_rest_docs_contract_feedback
