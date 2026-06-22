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
HOST="${WENDAO_AI_HOST:-127.0.0.1}"
PORT="${WENDAO_AI_PORT:-9518}"
REPO_URL="${WENDAO_AI_REPO_URL:-https://github.com/tao3k/wendao.ai.git}"
APP_DIR="${WENDAO_AI_DIR:-$PROJECT_DATA_ROOT/wendao.ai}"
RUNTIME_DIR="${WENDAO_AI_RUNTIME_DIR:-$PROJECT_RUNTIME_ROOT/wendao-ai}"
PIDFILE="${WENDAO_AI_PIDFILE:-$RUNTIME_DIR/wendao-ai.pid}"
LOG_DIR="${WENDAO_AI_LOG_DIR:-$PROJECT_RUNTIME_ROOT/logs}"
STDOUT_LOG="${WENDAO_AI_STDOUT_LOG:-$LOG_DIR/wendao-ai.stdout.log}"
STDERR_LOG="${WENDAO_AI_STDERR_LOG:-$LOG_DIR/wendao-ai.stderr.log}"
MANAGED="${WENDAO_AI_MANAGED:-0}"
PROCESS_PATTERN="${WENDAO_AI_PROCESS_PATTERN:-rspack-node}"

mkdir -p "$RUNTIME_DIR" "$LOG_DIR"
managed_materialize_git_repo "$APP_DIR" "$REPO_URL" "" "wendao.ai checkout"

RSBUILD_BIN="$APP_DIR/node_modules/.bin/rsbuild"
if [ ! -x "$RSBUILD_BIN" ]; then
  (
    cd "$APP_DIR"
    npm ci
  )
fi

if [ "$MANAGED" = "1" ]; then
  managed_cleanup_pidfile_process "$PIDFILE" wendao-ai "$PROCESS_PATTERN"
  managed_cleanup_listener "$PORT" wendao-ai "$PROCESS_PATTERN"
  rm -f "$PIDFILE"
fi

cd "$APP_DIR"

if [ "$MANAGED" != "1" ]; then
  exec "$RSBUILD_BIN" dev --host "$HOST" --port "$PORT"
fi

"$RSBUILD_BIN" dev --host "$HOST" --port "$PORT" \
  > >(tee -a "$STDOUT_LOG") \
  2> >(tee -a "$STDERR_LOG" >&2) &
WENDAO_AI_CHILD_PID=$!
managed_write_pidfile "$PIDFILE" "$WENDAO_AI_CHILD_PID"

cleanup_child() {
  if managed_process_is_alive "$WENDAO_AI_CHILD_PID"; then
    kill "$WENDAO_AI_CHILD_PID" 2>/dev/null || true
    managed_wait_for_exit "$WENDAO_AI_CHILD_PID" 25 0.2 || true
  fi
  rm -f "$PIDFILE"
}

trap cleanup_child TERM INT

if wait "$WENDAO_AI_CHILD_PID"; then
  STATUS=0
else
  STATUS=$?
fi
rm -f "$PIDFILE"
exit "$STATUS"
