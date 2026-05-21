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

PIDFILE="$(valkey_resolved_pidfile "$PROJECT_ROOT")"
SHARED_PIDFILE="$(valkey_resolved_shared_pidfile "$PROJECT_ROOT")"
URL="redis://${HOST}:${PORT}/${DB}"

MATCHED_PIDFILE=""
if MATCHED_PIDFILE="$(valkey_matching_pidfile "$PROJECT_ROOT" "$URL")"; then
  valkey-cli -u "$URL" shutdown nosave >/dev/null 2>&1 || true
  sleep 0.2
  rm -f "$MATCHED_PIDFILE"
fi

if valkey-cli -u "$URL" ping >/dev/null 2>&1; then
  echo "Warning: Valkey is reachable at $URL but pidfile ${PIDFILE} does not match the listener." >&2
  exit 1
fi

rm -f "$PIDFILE"
rm -f "$SHARED_PIDFILE"
echo "Valkey stopped on port ${PORT}."
