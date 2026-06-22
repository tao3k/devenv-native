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
PROJECT_CACHE_ROOT="${PRJ_CACHE_HOME:-$PROJECT_ROOT/.cache}"
if [[ "$PROJECT_CACHE_ROOT" != /* ]]; then
  PROJECT_CACHE_ROOT="$PROJECT_ROOT/$PROJECT_CACHE_ROOT"
fi

CONFIG_PATH="${WENDAO_GATEWAY_CONFIG:-$PROJECT_ROOT/wendao.toml}"
RUNTIME_DIR="${WENDAO_ANALYZER_RUNTIME_DIR:-$PROJECT_RUNTIME_ROOT/wendao-analyzer}"
PIDFILE="${WENDAO_ANALYZER_PIDFILE:-$RUNTIME_DIR/wendao-analyzer.pid}"
LOG_DIR="${WENDAO_ANALYZER_LOG_DIR:-$PROJECT_RUNTIME_ROOT/logs}"
STDOUT_LOG="${WENDAO_ANALYZER_STDOUT_LOG:-$LOG_DIR/wendao-analyzer.stdout.log}"
STDERR_LOG="${WENDAO_ANALYZER_STDERR_LOG:-$LOG_DIR/wendao-analyzer.stderr.log}"
PDF_OCR_WORKER="${WENDAO_PDF_OCR_WORKER:-docling}"
PDF_OCR_WORKERS="${WENDAO_PDF_OCR_WORKERS:-auto}"
AUDIO_WORKER="${WENDAO_AUDIO_WORKER:-hosted}"
AUDIO_WORKERS="${WENDAO_AUDIO_WORKERS:-auto}"
AUDIO_LOCAL_BACKEND="${WENDAO_AUDIO_LOCAL_BACKEND:-auto}"
AUDIO_LOCAL_BACKEND_RUNNER="${WENDAO_AUDIO_BACKEND_RUNNER:-qwen3-asr-mlx}"
AUDIO_LOCAL_HOST="${WENDAO_AUDIO_LOCAL_HOST:-127.0.0.1}"
AUDIO_LOCAL_PORT="${WENDAO_AUDIO_LOCAL_PORT:-8010}"
AUDIO_LOCAL_MODEL_PATH="${WENDAO_AUDIO_LOCAL_MODEL_PATH:-}"
AUDIO_LOCAL_BACKEND_RUNTIME_DIR="${WENDAO_AUDIO_LOCAL_BACKEND_RUNTIME_DIR:-$PROJECT_RUNTIME_ROOT/wendao-audio-local-backend}"
AUDIO_LOCAL_BACKEND_PIDFILE="${WENDAO_AUDIO_LOCAL_BACKEND_PIDFILE:-$AUDIO_LOCAL_BACKEND_RUNTIME_DIR/wendao-audio-local-backend.pid}"
AUDIO_LOCAL_BACKEND_STDOUT_LOG="${WENDAO_AUDIO_LOCAL_BACKEND_STDOUT_LOG:-$LOG_DIR/wendao-audio-local-backend.stdout.log}"
AUDIO_LOCAL_BACKEND_STDERR_LOG="${WENDAO_AUDIO_LOCAL_BACKEND_STDERR_LOG:-$LOG_DIR/wendao-audio-local-backend.stderr.log}"
AUDIO_LOCAL_BACKEND_CHILD_PID=""

if [ "$AUDIO_WORKER" = "hosted" ]; then
  export WENDAO_AUDIO_HOSTED_PROVIDER="${WENDAO_AUDIO_HOSTED_PROVIDER:-openrouter}"
  export WENDAO_AUDIO_HOSTED_TRACE_PATH="${WENDAO_AUDIO_HOSTED_TRACE_PATH:-$PROJECT_CACHE_ROOT/agent/evidence/audio_shards/wendao-analyzer.hosted-audio.jsonl}"
  if [ "$WENDAO_AUDIO_HOSTED_PROVIDER" = "openrouter" ]; then
    export WENDAO_AUDIO_HOSTED_MODEL="${WENDAO_AUDIO_HOSTED_MODEL:-qwen/qwen3-asr-flash-2026-02-10}"
    export WENDAO_AUDIO_HOSTED_ENDPOINT="${WENDAO_AUDIO_HOSTED_ENDPOINT:-audio-transcriptions}"
  fi
fi

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
if [[ "$AUDIO_LOCAL_BACKEND_RUNTIME_DIR" != /* ]]; then
  AUDIO_LOCAL_BACKEND_RUNTIME_DIR="$PROJECT_ROOT/$AUDIO_LOCAL_BACKEND_RUNTIME_DIR"
fi
if [[ "$AUDIO_LOCAL_BACKEND_PIDFILE" != /* ]]; then
  AUDIO_LOCAL_BACKEND_PIDFILE="$PROJECT_ROOT/$AUDIO_LOCAL_BACKEND_PIDFILE"
fi
if [[ "$AUDIO_LOCAL_BACKEND_STDOUT_LOG" != /* ]]; then
  AUDIO_LOCAL_BACKEND_STDOUT_LOG="$PROJECT_ROOT/$AUDIO_LOCAL_BACKEND_STDOUT_LOG"
fi
if [[ "$AUDIO_LOCAL_BACKEND_STDERR_LOG" != /* ]]; then
  AUDIO_LOCAL_BACKEND_STDERR_LOG="$PROJECT_ROOT/$AUDIO_LOCAL_BACKEND_STDERR_LOG"
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

audio_local_backend_requested() {
  local mode
  mode="$(printf '%s' "$AUDIO_LOCAL_BACKEND" | tr '[:upper:]' '[:lower:]')"
  case "$mode" in
  0 | false | off | disabled | skip) return 1 ;;
  *) return 0 ;;
  esac
}

audio_local_backend_ready() {
  "$PYTHON_BIN" - "$AUDIO_LOCAL_HOST" "$AUDIO_LOCAL_PORT" <<'PY'
import json
import sys
import urllib.error
import urllib.request

host = sys.argv[1]
port = sys.argv[2]
url = f"http://{host}:{port}/v1/models"
try:
    with urllib.request.urlopen(url, timeout=2) as response:
        payload = json.loads(response.read().decode("utf-8"))
except (OSError, urllib.error.URLError, json.JSONDecodeError):
    raise SystemExit(1)
if not isinstance(payload, dict) or "data" not in payload:
    raise SystemExit(1)
PY
}

cleanup_audio_local_backend_listener() {
  managed_cleanup_listener \
    "$AUDIO_LOCAL_PORT" \
    "wendao-audio-local-backend" \
    "qwen3_asr_mlx_openai_adapter.py"
}

start_audio_local_backend() {
  audio_local_backend_requested || return 0
  if audio_local_backend_ready; then
    echo "wendao-audio-local-backend already ready at http://${AUDIO_LOCAL_HOST}:${AUDIO_LOCAL_PORT}/v1"
    return 0
  fi

  mkdir -p "$AUDIO_LOCAL_BACKEND_RUNTIME_DIR" "$LOG_DIR"
  managed_cleanup_pidfile_process \
    "$AUDIO_LOCAL_BACKEND_PIDFILE" \
    "wendao-audio-local-backend" \
    "qwen3_asr_mlx_openai_adapter.py"
  cleanup_audio_local_backend_listener
  rm -f "$AUDIO_LOCAL_BACKEND_PIDFILE"

  uv run --package xiuxian-wendao-analyzer \
    wendao-document-extract \
    --audio-start-backend \
    --audio-backend-runner "$AUDIO_LOCAL_BACKEND_RUNNER" \
    --audio-backend-host "$AUDIO_LOCAL_HOST" \
    --audio-backend-port "$AUDIO_LOCAL_PORT" \
    --audio-backend-model-path "$AUDIO_LOCAL_MODEL_PATH" \
    > >(tee -a "$AUDIO_LOCAL_BACKEND_STDOUT_LOG") \
    2> >(tee -a "$AUDIO_LOCAL_BACKEND_STDERR_LOG" >&2) &
  AUDIO_LOCAL_BACKEND_CHILD_PID=$!
  managed_write_pidfile "$AUDIO_LOCAL_BACKEND_PIDFILE" "$AUDIO_LOCAL_BACKEND_CHILD_PID"
}

managed_cleanup_pidfile_process "$PIDFILE" wendao-analyzer "wendao-analyzer"
cleanup_analyzer_listener
rm -f "$PIDFILE"

cd "$PROJECT_ROOT"
start_audio_local_backend
uv run --package xiuxian-wendao-analyzer --extra documents-audio \
  wendao-document-extract \
  --host "$HOST" \
  --port "$PORT" \
  --pdf-ocr-worker "$PDF_OCR_WORKER" \
  --pdf-ocr-workers "$PDF_OCR_WORKERS" \
  --audio-worker "$AUDIO_WORKER" \
  --audio-workers "$AUDIO_WORKERS" \
  > >(tee -a "$STDOUT_LOG") \
  2> >(tee -a "$STDERR_LOG" >&2) &
WENDAO_ANALYZER_CHILD_PID=$!
managed_write_pidfile "$PIDFILE" "$WENDAO_ANALYZER_CHILD_PID"

cleanup_child() {
  if [ -n "$AUDIO_LOCAL_BACKEND_CHILD_PID" ] \
    && managed_process_is_alive "$AUDIO_LOCAL_BACKEND_CHILD_PID"; then
    kill "$AUDIO_LOCAL_BACKEND_CHILD_PID" 2>/dev/null || true
    managed_wait_for_exit "$AUDIO_LOCAL_BACKEND_CHILD_PID" 25 0.2 || true
  fi
  rm -f "$AUDIO_LOCAL_BACKEND_PIDFILE"
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
