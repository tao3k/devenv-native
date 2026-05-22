#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${PRJ_ROOT:-${DEVENV_ROOT:-$(cd "${SCRIPT_DIR}/../.." && pwd)}}"
source "${SCRIPT_DIR}/process-runtime.sh"

PYTHON_BIN="${WENDAO_ANALYZER_PYTHON:-}"
if [ -z "$PYTHON_BIN" ]; then
  PYTHON_BIN="${PYO3_PYTHON:-}"
fi
if [ -z "$PYTHON_BIN" ]; then
  PYTHON_BIN="$(command -v python3 2>/dev/null || true)"
fi
if [ -z "$PYTHON_BIN" ]; then
  echo "Error: python3 not found in PATH." >&2
  exit 1
fi
if ! command -v uv >/dev/null 2>&1; then
  echo "Error: uv not found in PATH." >&2
  exit 1
fi

PROJECT_RUNTIME_ROOT="${PRJ_RUNTIME_DIR:-$PROJECT_ROOT/.run}"
if [[ "$PROJECT_RUNTIME_ROOT" != /* ]]; then
  PROJECT_RUNTIME_ROOT="$PROJECT_ROOT/$PROJECT_RUNTIME_ROOT"
fi

CONFIG_PATH="${WENDAO_GATEWAY_CONFIG:-$PROJECT_ROOT/wendao.toml}"
RUNTIME_DIR="${WENDAO_ANALYZER_RUNTIME_DIR:-$PROJECT_RUNTIME_ROOT/wendao-analyzer}"
PIDFILE="${WENDAO_ANALYZER_PIDFILE:-$RUNTIME_DIR/wendao-analyzer.pid}"
LOG_DIR="${WENDAO_ANALYZER_LOG_DIR:-$PROJECT_RUNTIME_ROOT/logs}"
STDOUT_LOG="${WENDAO_ANALYZER_STDOUT_LOG:-$LOG_DIR/wendao-analyzer.stdout.log}"
STDERR_LOG="${WENDAO_ANALYZER_STDERR_LOG:-$LOG_DIR/wendao-analyzer.stderr.log}"

if [[ "$CONFIG_PATH" != /* ]]; then
  CONFIG_PATH="$PROJECT_ROOT/$CONFIG_PATH"
fi
if [[ "$RUNTIME_DIR" != /* ]]; then
  RUNTIME_DIR="$PROJECT_ROOT/$RUNTIME_DIR"
fi
if [[ "$PIDFILE" != /* ]]; then
  PIDFILE="$PROJECT_ROOT/$PIDFILE"
fi
if [[ "$LOG_DIR" != /* ]]; then
  LOG_DIR="$PROJECT_ROOT/$LOG_DIR"
fi
if [[ "$STDOUT_LOG" != /* ]]; then
  STDOUT_LOG="$PROJECT_ROOT/$STDOUT_LOG"
fi
if [[ "$STDERR_LOG" != /* ]]; then
  STDERR_LOG="$PROJECT_ROOT/$STDERR_LOG"
fi

HOST="$("$PYTHON_BIN" "$PROJECT_ROOT/scripts/runtime/resolve_wendao_document_extract_endpoint.py" --config "$CONFIG_PATH" --field host)"
PORT="$("$PYTHON_BIN" "$PROJECT_ROOT/scripts/runtime/resolve_wendao_document_extract_endpoint.py" --config "$CONFIG_PATH" --field port)"

mkdir -p "$RUNTIME_DIR" "$LOG_DIR"

cleanup_analyzer_listener() {
  local listener_pid command
  listener_pid="$(managed_listener_pid "$PORT")"
  [ -n "$listener_pid" ] || return 0

  if ! managed_process_is_alive "$listener_pid"; then
    return 0
  fi

  command="$(managed_process_command "$listener_pid")"
  if [[ $command != *"wendao-analyzer"* ]] \
    && [[ $command != *"wendao-document-extract"* ]] \
    && [[ $command != *"xiuxian-wendao-analyzer"* ]] \
    && [[ $command != *"xiuxian_wendao_analyzer"* ]] \
    && [[ $command != *"DocumentExtractFlightServer"* ]]; then
    echo "Error: refusing to clean wendao-analyzer listener on port ${PORT} because pid ${listener_pid} is not owned by this service." >&2
    echo "Command: ${command}" >&2
    return 1
  fi

  managed_terminate_pid "$listener_pid" wendao-analyzer
}

managed_cleanup_pidfile_process "$PIDFILE" wendao-analyzer "wendao-analyzer"
cleanup_analyzer_listener
rm -f "$PIDFILE"

cd "$PROJECT_ROOT"
uv run --package xiuxian-wendao-analyzer --extra documents \
  wendao-document-extract \
  --host "$HOST" \
  --port "$PORT" \
  > >(tee -a "$STDOUT_LOG") \
  2> >(tee -a "$STDERR_LOG" >&2) &
WENDAO_ANALYZER_CHILD_PID=$!
managed_write_pidfile "$PIDFILE" "$WENDAO_ANALYZER_CHILD_PID"

cleanup_child() {
  if managed_process_is_alive "$WENDAO_ANALYZER_CHILD_PID"; then
    kill "$WENDAO_ANALYZER_CHILD_PID" 2>/dev/null || true
    managed_wait_for_exit "$WENDAO_ANALYZER_CHILD_PID" 25 0.2 || true
  fi
  rm -f "$PIDFILE"
}

trap cleanup_child TERM INT

if wait "$WENDAO_ANALYZER_CHILD_PID"; then
  STATUS=0
else
  STATUS=$?
fi
rm -f "$PIDFILE"
exit "$STATUS"
