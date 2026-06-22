#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${PRJ_ROOT:-${DEVENV_ROOT:-$(cd "${SCRIPT_DIR}/../.." && pwd)}}"
PROJECT_RUNTIME_ROOT="${PRJ_RUNTIME_DIR:-$PROJECT_ROOT/.run}"
PYTHON_BIN="${WENDAO_AI_PYTHON:-}"

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

if [[ "$PROJECT_RUNTIME_ROOT" != /* ]]; then
  PROJECT_RUNTIME_ROOT="$PROJECT_ROOT/$PROJECT_RUNTIME_ROOT"
fi

PIDFILE="${WENDAO_AI_PIDFILE:-$PROJECT_RUNTIME_ROOT/wendao-ai/wendao-ai.pid}"
HOST="${WENDAO_AI_HOST:-127.0.0.1}"
PORT="${WENDAO_AI_PORT:-9518}"
TIMEOUT_SECS="${WENDAO_AI_HEALTH_TIMEOUT_SECS:-2}"

if [[ "$PIDFILE" != /* ]]; then
  PIDFILE="$PROJECT_ROOT/$PIDFILE"
fi

"$PYTHON_BIN" "$PROJECT_ROOT/scripts/runtime/check_runtime_web_health.py" \
  --service-name "wendao.ai" \
  --host "$HOST" \
  --port "$PORT" \
  --pidfile "$PIDFILE" \
  --timeout-secs "$TIMEOUT_SECS"
