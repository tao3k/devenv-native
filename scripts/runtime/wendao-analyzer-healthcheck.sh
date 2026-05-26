#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${PRJ_ROOT:-${DEVENV_ROOT:-$(cd "${SCRIPT_DIR}/../.." && pwd)}}"

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

CONFIG_PATH="${WENDAO_GATEWAY_CONFIG:-$PROJECT_ROOT/wendao.toml}"
if [[ "$CONFIG_PATH" != /* ]]; then
  CONFIG_PATH="$PROJECT_ROOT/$CONFIG_PATH"
fi

HOST="$("$PYTHON_BIN" "$PROJECT_ROOT/scripts/runtime/resolve_wendao_document_extract_endpoint.py" --config "$CONFIG_PATH" --field host)"
PORT="$("$PYTHON_BIN" "$PROJECT_ROOT/scripts/runtime/resolve_wendao_document_extract_endpoint.py" --config "$CONFIG_PATH" --field port)"
TIMEOUT_SECS="${WENDAO_ANALYZER_HEALTH_TIMEOUT_SECS:-0.5}"
ATTEMPTS="${WENDAO_ANALYZER_HEALTH_ATTEMPTS:-3}"
RETRY_DELAY_SECS="${WENDAO_ANALYZER_HEALTH_RETRY_DELAY_SECS:-0.2}"

"$PYTHON_BIN" - "$HOST" "$PORT" "$TIMEOUT_SECS" "$ATTEMPTS" "$RETRY_DELAY_SECS" <<'PY'
import socket
import sys
import time

host = sys.argv[1]
port = int(sys.argv[2])
timeout_secs = max(0.1, float(sys.argv[3]))
attempts = max(1, int(sys.argv[4]))
retry_delay_secs = max(0.0, float(sys.argv[5]))
last_error = None

for attempt_index in range(attempts):
    try:
        with socket.create_connection((host, port), timeout=timeout_secs):
            raise SystemExit(0)
    except OSError as error:
        last_error = error
    if attempt_index + 1 < attempts:
        time.sleep(retry_delay_secs)

print(
    "Error: analyzer endpoint unreachable: "
    f"{host}:{port} after {attempts} attempt(s): {last_error}",
    file=sys.stderr,
)
raise SystemExit(1)
PY
