#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
source "${SCRIPT_DIR}/valkey-common.sh"
source "${SCRIPT_DIR}/valkey-runtime.sh"

if ! command -v valkey-cli >/dev/null 2>&1; then
  echo "Error: valkey-cli not found in PATH." >&2
  exit 1
fi

PIDFILE="$(valkey_resolved_pidfile "$PROJECT_ROOT")"
URL="$(valkey_effective_url)"

if valkey_matching_pidfile "$PROJECT_ROOT" "$URL" >/dev/null && valkey-cli -u "$URL" ping >/dev/null 2>&1; then
  echo "PONG"
  exit 0
fi

if valkey-cli -u "$URL" ping >/dev/null 2>&1; then
  echo "Error: Valkey is reachable at $URL but pidfile $PIDFILE does not match the listener." >&2
  exit 1
fi

echo "Error: Valkey is not running at $URL." >&2
exit 1
