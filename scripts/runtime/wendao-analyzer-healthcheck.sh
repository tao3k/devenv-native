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

"$PYTHON_BIN" - "$HOST" "$PORT" <<'PY'
import socket
import sys

host = sys.argv[1]
port = int(sys.argv[2])
with socket.create_connection((host, port), timeout=2):
    pass
PY
