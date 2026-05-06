#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${PRJ_ROOT:-${DEVENV_ROOT:-$(cd "${SCRIPT_DIR}/../.." && pwd)}}"
PROJECT_RUNTIME_ROOT="${PRJ_RUNTIME_DIR:-$PROJECT_ROOT/.run}"
SERVICE_NAME="${WENDAOSEARCH_SERVICE_NAME:-wendaosearch}"
RUNTIME_DIR="${WENDAOSEARCH_RUNTIME_DIR:-$PROJECT_RUNTIME_ROOT/wendaosearch}"
PIDFILE="${WENDAOSEARCH_PIDFILE:-$RUNTIME_DIR/$SERVICE_NAME.pid}"

if [[ "$RUNTIME_DIR" != /* ]]; then
  RUNTIME_DIR="$PROJECT_ROOT/$RUNTIME_DIR"
fi
if [[ "$PIDFILE" != /* ]]; then
  PIDFILE="$PROJECT_ROOT/$PIDFILE"
fi

if [ ! -s "$PIDFILE" ]; then
  echo "Error: WendaoSearch pidfile missing: $PIDFILE" >&2
  exit 1
fi

PID="$(tr -d '[:space:]' <"$PIDFILE")"
if [ -z "$PID" ] || ! kill -0 "$PID" 2>/dev/null; then
  echo "Error: WendaoSearch process is not running." >&2
  exit 1
fi
