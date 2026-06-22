#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${PRJ_ROOT:-${DEVENV_ROOT:-$(cd "${SCRIPT_DIR}/../.." && pwd)}}"
PROJECT_RUNTIME_ROOT="${PRJ_RUNTIME_DIR:-$PROJECT_ROOT/.run}"
PYTHON_BIN="${WENDAO_GATEWAY_PYTHON:-}"

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

PIDFILE="${WENDAO_GATEWAY_PIDFILE:-$PROJECT_RUNTIME_ROOT/wendao-gateway/wendao.pid}"
LOGFILE="${WENDAO_GATEWAY_STDERR_LOG:-$PROJECT_RUNTIME_ROOT/logs/wendao-gateway.stderr.log}"
CONFIG_PATH="${WENDAO_GATEWAY_CONFIG:-$PROJECT_ROOT/wendao.toml}"
HOST="${WENDAO_GATEWAY_HOST:-127.0.0.1}"
TIMEOUT_SECS="${WENDAO_GATEWAY_HEALTH_TIMEOUT_SECS:-0.5}"
ATTEMPTS="${WENDAO_GATEWAY_HEALTH_ATTEMPTS:-3}"
RETRY_DELAY_SECS="${WENDAO_GATEWAY_HEALTH_RETRY_DELAY_SECS:-0.2}"

if [[ "$PIDFILE" != /* ]]; then
  PIDFILE="$PROJECT_ROOT/$PIDFILE"
fi
if [[ "$LOGFILE" != /* ]]; then
  LOGFILE="$PROJECT_ROOT/$LOGFILE"
fi
if [[ "$CONFIG_PATH" != /* ]]; then
  CONFIG_PATH="$PROJECT_ROOT/$CONFIG_PATH"
fi

PORT="$("$PYTHON_BIN" "$PROJECT_ROOT/scripts/runtime/resolve_wendao_gateway_port.py" --config "$CONFIG_PATH")"

"$PYTHON_BIN" "$PROJECT_ROOT/scripts/runtime/check_wendao_gateway_health.py" \
  --host "$HOST" \
  --port "$PORT" \
  --pidfile "$PIDFILE" \
  --logfile "$LOGFILE" \
  --timeout-secs "$TIMEOUT_SECS" \
  --attempts "$ATTEMPTS" \
  --retry-delay-secs "$RETRY_DELAY_SECS"
