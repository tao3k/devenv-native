#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=../common.sh
# shellcheck disable=SC1091
source "$SCRIPT_DIR/../common.sh"
PROJECT_ROOT="$(process_project_root "$SCRIPT_DIR" "../../../..")"
PROJECT_RUNTIME_ROOT="$(process_runtime_root "$PROJECT_ROOT")"
RUNTIME_DIR="${WENDAO_SEMANTIC_REFRESH_RUNTIME_DIR:-$PROJECT_RUNTIME_ROOT/wendao-semantic-refresh}"
PIDFILE="${WENDAO_SEMANTIC_REFRESH_PIDFILE:-$RUNTIME_DIR/wendao-semantic-refresh.pid}"
WENDAO_CLIENT_BIN="${WENDAO_SEMANTIC_REFRESH_BIN:-$PROJECT_ROOT/target/debug/wendao-client}"

RUNTIME_DIR="$(process_abs_path "$PROJECT_ROOT" "$RUNTIME_DIR")"
PIDFILE="$(process_abs_path "$PROJECT_ROOT" "$PIDFILE")"
WENDAO_CLIENT_BIN="$(process_abs_path "$PROJECT_ROOT" "$WENDAO_CLIENT_BIN")"

# shellcheck source=../../process-runtime.sh
# shellcheck disable=SC1091
source "$PROJECT_ROOT/scripts/channel/process-runtime.sh"

pid="$(managed_pidfile_process_id "$PIDFILE")"
managed_process_is_alive "$pid"
managed_pid_matches_patterns \
  "$pid" \
  "$WENDAO_CLIENT_BIN" \
  " semantic refresh-projections"
