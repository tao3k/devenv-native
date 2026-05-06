#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=../common.sh
# shellcheck disable=SC1091
source "$SCRIPT_DIR/../common.sh"
PROJECT_ROOT="$(process_project_root "$SCRIPT_DIR" "../../../..")"
PROJECT_RUNTIME_ROOT="$(process_runtime_root "$PROJECT_ROOT")"
LOG_DIR="${WENDAO_SEMANTIC_REFRESH_LOG_DIR:-$PROJECT_RUNTIME_ROOT/logs}"
RUNTIME_DIR="${WENDAO_SEMANTIC_REFRESH_RUNTIME_DIR:-$PROJECT_RUNTIME_ROOT/wendao-semantic-refresh}"
PIDFILE="${WENDAO_SEMANTIC_REFRESH_PIDFILE:-$RUNTIME_DIR/wendao-semantic-refresh.pid}"
WENDAO_CLIENT_BIN="${WENDAO_SEMANTIC_REFRESH_BIN:-$PROJECT_ROOT/target/debug/wendao-client}"
STDOUT_LOG="${WENDAO_SEMANTIC_REFRESH_STDOUT_LOG:-$LOG_DIR/wendao-semantic-refresh.stdout.log}"
STDERR_LOG="${WENDAO_SEMANTIC_REFRESH_STDERR_LOG:-$LOG_DIR/wendao-semantic-refresh.stderr.log}"
BUILD_ENABLED="${WENDAO_SEMANTIC_REFRESH_BUILD:-1}"
INTERVAL_SECS="${WENDAO_SEMANTIC_REFRESH_INTERVAL_SECS:-300}"

LOG_DIR="$(process_abs_path "$PROJECT_ROOT" "$LOG_DIR")"
RUNTIME_DIR="$(process_abs_path "$PROJECT_ROOT" "$RUNTIME_DIR")"
PIDFILE="$(process_abs_path "$PROJECT_ROOT" "$PIDFILE")"
WENDAO_CLIENT_BIN="$(process_abs_path "$PROJECT_ROOT" "$WENDAO_CLIENT_BIN")"

# shellcheck source=../../process-runtime.sh
# shellcheck disable=SC1091
source "$PROJECT_ROOT/scripts/channel/process-runtime.sh"
mkdir -p "$RUNTIME_DIR" "$LOG_DIR"

managed_cleanup_pidfile_process \
  "$PIDFILE" \
  wendao-semantic-refresh \
  "$WENDAO_CLIENT_BIN" \
  " semantic refresh-projections"
rm -f "$PIDFILE"

cd "$PROJECT_ROOT"
if [ "$BUILD_ENABLED" != "0" ]; then
  cargo build -p xiuxian-wendao-client --bin wendao-client --locked
fi

refresh_args=(
  semantic
  refresh-projections
  --require-clean-worktree
  --interval-secs
  "$INTERVAL_SECS"
)

if [ -n "${WENDAO_SEMANTIC_REFRESH_MAX_RUNS:-}" ]; then
  refresh_args+=(--max-runs "$WENDAO_SEMANTIC_REFRESH_MAX_RUNS")
fi

"$WENDAO_CLIENT_BIN" "${refresh_args[@]}" \
  > >(tee -a "$STDOUT_LOG") \
  2> >(tee -a "$STDERR_LOG" >&2) &
WENDAO_SEMANTIC_REFRESH_CHILD_PID=$!
managed_write_pidfile "$PIDFILE" "$WENDAO_SEMANTIC_REFRESH_CHILD_PID"

# shellcheck disable=SC2329
cleanup_child() {
  if managed_process_is_alive "$WENDAO_SEMANTIC_REFRESH_CHILD_PID"; then
    kill "$WENDAO_SEMANTIC_REFRESH_CHILD_PID" 2>/dev/null || true
    managed_wait_for_exit "$WENDAO_SEMANTIC_REFRESH_CHILD_PID" 25 0.2 || true
  fi
  rm -f "$PIDFILE"
}

trap cleanup_child TERM INT

if wait "$WENDAO_SEMANTIC_REFRESH_CHILD_PID"; then
  STATUS=0
else
  STATUS=$?
fi
rm -f "$PIDFILE"
exit "$STATUS"
