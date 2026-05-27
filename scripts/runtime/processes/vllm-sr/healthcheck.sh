#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../common.sh"
PROJECT_ROOT="$(process_project_root "$SCRIPT_DIR" "../../../..")"
process_export_project_defaults "$PROJECT_ROOT"

MODE="${WENDAO_MODEL_ROUTING_MODE:-deterministic}"
case "$MODE" in
  deterministic)
    exit 0
    ;;
  vllm-sr)
    ;;
  *)
    echo "unsupported WENDAO_MODEL_ROUTING_MODE value: $MODE" >&2
    exit 1
    ;;
esac

PYTHON_BIN="$(process_require_python_bin "${WENDAO_VLLM_SR_PYTHON:-}")"
BASE_URL="${WENDAO_VLLM_SR_BASE_URL:-http://127.0.0.1:8888}"
TIMEOUT_SECS="${WENDAO_VLLM_SR_HEALTH_TIMEOUT_SECS:-1.0}"

exec "$PYTHON_BIN" - "$BASE_URL" "$TIMEOUT_SECS" <<'PY'
from __future__ import annotations

import socket
import sys
from urllib.parse import urlparse

base_url = sys.argv[1]
timeout = float(sys.argv[2])
parsed = urlparse(base_url)
host = parsed.hostname
port = parsed.port or (443 if parsed.scheme == "https" else 80)
if not host:
    raise SystemExit(f"invalid WENDAO_VLLM_SR_BASE_URL: {base_url}")

try:
    with socket.create_connection((host, port), timeout=timeout):
        pass
except OSError as error:
    raise SystemExit(f"vLLM-SR is not accepting connections at {host}:{port}: {error}") from error
PY
