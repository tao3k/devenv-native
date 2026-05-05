#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../common.sh"
PROJECT_ROOT="$(process_project_root "$SCRIPT_DIR" "../../../..")"
PROJECT_RUNTIME_ROOT="$(process_runtime_root "$PROJECT_ROOT")"
PROJECT_DATA_ROOT="$(process_data_root "$PROJECT_ROOT")"
LOG_DIR="${WENDAOSEARCH_LOG_DIR:-$PROJECT_RUNTIME_ROOT/logs}"
SERVICE_NAME="${WENDAOSEARCH_SERVICE_NAME:-wendaosearch}"
RUNTIME_DIR="${WENDAOSEARCH_RUNTIME_DIR:-$PROJECT_RUNTIME_ROOT/wendaosearch}"
PACKAGE_DIR="${WENDAOSEARCH_PACKAGE_DIR:-$PROJECT_DATA_ROOT/WendaoSearch.jl}"
PACKAGE_REPO_URL="${WENDAOSEARCH_PACKAGE_REPO_URL:-https://github.com/tao3k/WendaoSearch.jl.git}"
CONFIG_PATH="${WENDAOSEARCH_CONFIG:-$PACKAGE_DIR/config/live/solver_demo.toml}"
SCRIPT_NAME="${WENDAOSEARCH_SCRIPT:-run_search_service.jl}"
JULIA_BIN="${WENDAOSEARCH_JULIA:-julia}"
PIDFILE="${WENDAOSEARCH_PIDFILE:-$RUNTIME_DIR/$SERVICE_NAME.pid}"
STDOUT_LOG="${WENDAOSEARCH_STDOUT_LOG:-$LOG_DIR/$SERVICE_NAME.stdout.log}"
STDERR_LOG="${WENDAOSEARCH_STDERR_LOG:-$LOG_DIR/$SERVICE_NAME.stderr.log}"

RUNTIME_DIR="$(process_abs_path "$PROJECT_ROOT" "$RUNTIME_DIR")"
LOG_DIR="$(process_abs_path "$PROJECT_ROOT" "$LOG_DIR")"
PIDFILE="$(process_abs_path "$PROJECT_ROOT" "$PIDFILE")"
CONFIG_PATH="$(process_abs_path "$PROJECT_ROOT" "$CONFIG_PATH")"
PACKAGE_DIR="$(process_abs_path "$PROJECT_ROOT" "$PACKAGE_DIR")"

source "$PROJECT_ROOT/scripts/channel/process-runtime.sh"
mkdir -p "$RUNTIME_DIR" "$LOG_DIR"
managed_materialize_git_repo "$PACKAGE_DIR" "$PACKAGE_REPO_URL" "" "WendaoSearch.jl checkout"

SCRIPT_PATH="$PACKAGE_DIR/scripts/$SCRIPT_NAME"
if [ ! -f "$SCRIPT_PATH" ]; then
  echo "Error: WendaoSearch service script not found: $SCRIPT_PATH" >&2
  exit 1
fi
if [ ! -f "$CONFIG_PATH" ]; then
  echo "Error: WendaoSearch service config not found: $CONFIG_PATH" >&2
  exit 1
fi

managed_cleanup_pidfile_process "$PIDFILE" "$SERVICE_NAME" "$SCRIPT_PATH"
rm -f "$PIDFILE"

"$JULIA_BIN" --project="$PACKAGE_DIR" "$SCRIPT_PATH" --config "$CONFIG_PATH" \
  > >(tee -a "$STDOUT_LOG") \
  2> >(tee -a "$STDERR_LOG" >&2) &
WENDAOSEARCH_CHILD_PID=$!
managed_write_pidfile "$PIDFILE" "$WENDAOSEARCH_CHILD_PID"

cleanup_child() {
  if managed_process_is_alive "$WENDAOSEARCH_CHILD_PID"; then
    kill "$WENDAOSEARCH_CHILD_PID" 2>/dev/null || true
    managed_wait_for_exit "$WENDAOSEARCH_CHILD_PID" 25 0.2 || true
  fi
  rm -f "$PIDFILE"
}

trap cleanup_child TERM INT

if wait "$WENDAOSEARCH_CHILD_PID"; then
  STATUS=0
else
  STATUS=$?
fi
rm -f "$PIDFILE"
exit "$STATUS"
