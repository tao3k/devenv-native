#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../common.sh"
PROJECT_ROOT="$(process_project_root "$SCRIPT_DIR" "../../../..")"
PROJECT_RUNTIME_ROOT="$(process_runtime_root "$PROJECT_ROOT")"
PYTHON_BIN="$(process_require_python_bin "${WENDAO_SENTINEL_PYTHON:-${WENDAO_GATEWAY_PYTHON:-}}")"

CONFIG_PATH="${WENDAO_GATEWAY_CONFIG:-$PROJECT_ROOT/wendao.toml}"
PIDFILE="${WENDAO_SENTINEL_PIDFILE:-$PROJECT_RUNTIME_ROOT/wendao-sentinel/wendao-sentinel.pid}"

CONFIG_PATH="$(process_abs_path "$PROJECT_ROOT" "$CONFIG_PATH")"
PIDFILE="$(process_abs_path "$PROJECT_ROOT" "$PIDFILE")"

exec "$PYTHON_BIN" "$PROJECT_ROOT/scripts/channel/check_wendao_sentinel_health.py" \
  --project-root "$PROJECT_ROOT" \
  --config "$CONFIG_PATH" \
  --pidfile "$PIDFILE"
