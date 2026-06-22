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

# Live OpenAPI artifact lane:
# - bundled Wendao gateway OpenAPI stays aligned to the declared route inventory
# - qianji rest_docs helper accepts the real bundled Wendao artifact without findings
CARGO_TARGET_DIR="${target_dir}" run_cargo test -p xiuxian-wendao --lib \
  bundled_gateway_openapi_document_

CARGO_TARGET_DIR="${target_dir}" run_cargo test -p xiuxian-qianji \
  --test wendao_live_rest_docs_contract_feedback
