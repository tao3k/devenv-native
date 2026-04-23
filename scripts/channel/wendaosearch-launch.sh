#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${PRJ_ROOT:-${DEVENV_ROOT:-$(cd "${SCRIPT_DIR}/../.." && pwd)}}"
source "${SCRIPT_DIR}/wendaosearch-common.sh"
source "${SCRIPT_DIR}/process-runtime.sh"

if ! command -v julia >/dev/null 2>&1; then
  echo "Error: julia not found in PATH." >&2
  exit 1
fi

RUNTIME_DIR="$(wendaosearch_resolve_path "$PROJECT_ROOT" "$(wendaosearch_effective_runtime_dir)")"
PACKAGE_DIR="$(wendaosearch_package_dir "$PROJECT_ROOT")"
wendaosearch_materialize_package_repo "$PROJECT_ROOT"
SCRIPT_PATH="$(wendaosearch_script_path "$PROJECT_ROOT")"
CONFIG_PATH="$(wendaosearch_resolve_path "$PROJECT_ROOT" "$(wendaosearch_effective_config)")"
JULIA_LOAD_PATH_VALUE="$(wendaosearch_effective_julia_load_path)"
JULIA_PROJECT="${WENDAOSEARCH_JULIA_PROJECT:-$PACKAGE_DIR}"
PIDFILE="$(wendaosearch_resolve_path "$PROJECT_ROOT" "$(wendaosearch_effective_pidfile)")"
SERVICE_NAME="$(wendaosearch_effective_service_name)"
PORT="$(wendaosearch_effective_port "$PROJECT_ROOT")"

mkdir -p "$RUNTIME_DIR"
test -f "$CONFIG_PATH" || {
  echo "Error: WendaoSearch config does not exist: $CONFIG_PATH" >&2
  exit 1
}
export JULIA_LOAD_PATH="$JULIA_LOAD_PATH_VALUE"
export WENDAOSEARCH_PIDFILE="$PIDFILE"

managed_cleanup_pidfile_process "$PIDFILE" "$SERVICE_NAME" "julia" "$SCRIPT_PATH"
managed_cleanup_listener "$PORT" "$SERVICE_NAME" "julia" "$SCRIPT_PATH"

command=(julia "--project=${JULIA_PROJECT}" "$SCRIPT_PATH" "--config" "$CONFIG_PATH")

"${command[@]}" &
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
