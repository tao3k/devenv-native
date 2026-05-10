#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

resolve_valkey_field() {
  uv run python "${PROJECT_ROOT}/scripts/runtime/resolve_valkey_endpoint.py" --field "$1"
}

port="${1:-$(resolve_valkey_field port)}"
valkey_url="${2:-$(resolve_valkey_field url)}"

cleanup() {
  bash scripts/runtime/valkey-stop.sh "${port}" || true
}
trap cleanup EXIT

bash scripts/runtime/valkey-start.sh "${port}"
VALKEY_URL="${valkey_url}" bash scripts/runtime/valkey-healthcheck.sh
