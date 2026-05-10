#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
source "${SCRIPT_DIR}/valkey-common.sh"
source "${SCRIPT_DIR}/valkey-runtime.sh"

resolve_valkey_field() {
  uv run python "${PROJECT_ROOT}/scripts/runtime/resolve_valkey_endpoint.py" --field "$1"
}

DEFAULT_PORT="$(resolve_valkey_field port)"
DEFAULT_HOST="$(resolve_valkey_field host)"
DEFAULT_DB="$(resolve_valkey_field db)"

PORT="${1:-${VALKEY_PORT:-${DEFAULT_PORT}}}"
HOST="${VALKEY_HOST:-${DEFAULT_HOST}}"
DB="${VALKEY_DB:-${DEFAULT_DB}}"

export VALKEY_PORT="$PORT"
export VALKEY_HOST="$HOST"
export VALKEY_DB="$DB"

if ! command -v valkey-server >/dev/null 2>&1; then
  echo "Error: valkey-server not found in PATH." >&2
  exit 1
fi
if ! command -v valkey-cli >/dev/null 2>&1; then
  echo "Error: valkey-cli not found in PATH." >&2
  exit 1
fi

RUNTIME_DIR="${PRJ_RUNTIME_DIR:-.run}/valkey"
DATA_DIR="${PRJ_CACHE_HOME:-.cache}/valkey"
RUNTIME_DIR="$(valkey_resolve_path "$PROJECT_ROOT" "$RUNTIME_DIR")"
DATA_DIR="$(valkey_resolve_path "$PROJECT_ROOT" "$DATA_DIR")"
mkdir -p "$RUNTIME_DIR" "$DATA_DIR"
PIDFILE="$(valkey_resolved_pidfile "$PROJECT_ROOT")"
LOGFILE="$RUNTIME_DIR/valkey-${PORT}.log"
URL="redis://${HOST}:${PORT}/${DB}"

MATCHED_PIDFILE=""
if MATCHED_PIDFILE="$(valkey_matching_pidfile "$PROJECT_ROOT" "$URL")" && valkey-cli -u "$URL" ping >/dev/null 2>&1; then
  echo "Valkey already running on ${PORT} (pid $(valkey_pidfile_process_id "$MATCHED_PIDFILE"))."
  exit 0
fi

if valkey-cli -u "$URL" ping >/dev/null 2>&1; then
  echo "Error: Valkey is reachable at $URL but pidfile $PIDFILE does not match the listener." >&2
  exit 1
fi

export VALKEY_BIND="$HOST"
export VALKEY_RUNTIME_DIR="$RUNTIME_DIR"
export VALKEY_DATA_DIR="$DATA_DIR"
export VALKEY_PIDFILE="$PIDFILE"
export VALKEY_LOGFILE="$LOGFILE"
export VALKEY_DAEMONIZE=yes

bash "${SCRIPT_DIR}/valkey-launch.sh"
