#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${PRJ_ROOT:-${DEVENV_ROOT:-$(cd "${SCRIPT_DIR}/../.." && pwd)}}"
source "${SCRIPT_DIR}/process-runtime.sh"

if ! command -v npm >/dev/null 2>&1; then
  echo "Error: npm not found in PATH." >&2
  exit 1
fi

PROJECT_RUNTIME_ROOT="${PRJ_RUNTIME_DIR:-$PROJECT_ROOT/.run}"
PROJECT_DATA_ROOT="${PRJ_DATA_HOME:-$PROJECT_ROOT/.data}"
HOST="${WENDAO_FRONTEND_HOST:-127.0.0.1}"
PORT="${WENDAO_FRONTEND_PORT:-9518}"
REPO_URL="${WENDAO_FRONTEND_REPO_URL:-https://github.com/tao3k/wendao-frontend.git}"
FRONTEND_DIR="${WENDAO_FRONTEND_DIR:-$PROJECT_DATA_ROOT/wendao-frontend}"
RUNTIME_DIR="${WENDAO_FRONTEND_RUNTIME_DIR:-$PROJECT_RUNTIME_ROOT/wendao-frontend}"
PIDFILE="${WENDAO_FRONTEND_PIDFILE:-$RUNTIME_DIR/wendao-frontend.pid}"
LOG_DIR="${WENDAO_FRONTEND_LOG_DIR:-$PROJECT_RUNTIME_ROOT/logs}"
STDOUT_LOG="${WENDAO_FRONTEND_STDOUT_LOG:-$LOG_DIR/wendao-frontend.stdout.log}"
STDERR_LOG="${WENDAO_FRONTEND_STDERR_LOG:-$LOG_DIR/wendao-frontend.stderr.log}"
MANAGED="${WENDAO_FRONTEND_MANAGED:-0}"

mkdir -p "$RUNTIME_DIR" "$LOG_DIR"
managed_materialize_git_repo "$FRONTEND_DIR" "$REPO_URL" "" "wendao-frontend checkout"

if [ ! -x "$FRONTEND_DIR/node_modules/.bin/rspack" ]; then
  (
    cd "$FRONTEND_DIR"
    npm ci
  )
fi

if [ "$MANAGED" = "1" ]; then
  managed_cleanup_pidfile_process "$PIDFILE" wendao-frontend "rspack-node"
  managed_cleanup_listener "$PORT" wendao-frontend "rspack-node"
  rm -f "$PIDFILE"
fi

cd "$FRONTEND_DIR"

if [ "$MANAGED" != "1" ]; then
  exec ./node_modules/.bin/rspack dev --host "$HOST" --port "$PORT"
fi

./node_modules/.bin/rspack dev --host "$HOST" --port "$PORT" \
  > >(tee -a "$STDOUT_LOG") \
  2> >(tee -a "$STDERR_LOG" >&2) &
WENDAO_FRONTEND_CHILD_PID=$!
managed_write_pidfile "$PIDFILE" "$WENDAO_FRONTEND_CHILD_PID"

cleanup_child() {
  if managed_process_is_alive "$WENDAO_FRONTEND_CHILD_PID"; then
    kill "$WENDAO_FRONTEND_CHILD_PID" 2>/dev/null || true
    managed_wait_for_exit "$WENDAO_FRONTEND_CHILD_PID" 25 0.2 || true
  fi
  rm -f "$PIDFILE"
}

trap cleanup_child TERM INT

if wait "$WENDAO_FRONTEND_CHILD_PID"; then
  STATUS=0
else
  STATUS=$?
fi
rm -f "$PIDFILE"
exit "$STATUS"
