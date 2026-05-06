#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../common.sh"
PROJECT_ROOT="$(process_project_root "$SCRIPT_DIR" "../../../..")"
PROJECT_RUNTIME_ROOT="$(process_runtime_root "$PROJECT_ROOT")"
PROJECT_DATA_ROOT="$(process_data_root "$PROJECT_ROOT")"

export VALKEY_RUNTIME_DIR="${VALKEY_RUNTIME_DIR:-$PROJECT_RUNTIME_ROOT/valkey}"
export VALKEY_DATA_DIR="${VALKEY_DATA_DIR:-$PROJECT_DATA_ROOT/valkey}"
export VALKEY_PIDFILE="${VALKEY_PIDFILE:-$VALKEY_RUNTIME_DIR/valkey.pid}"
export VALKEY_PORT="${VALKEY_PORT:-6379}"
export VALKEY_HOST="${VALKEY_HOST:-127.0.0.1}"
export VALKEY_BIND="${VALKEY_BIND:-$VALKEY_HOST}"
export VALKEY_DAEMONIZE="${VALKEY_DAEMONIZE:-no}"

source "$PROJECT_ROOT/scripts/channel/process-runtime.sh"
mkdir -p "$VALKEY_RUNTIME_DIR" "$VALKEY_DATA_DIR"
managed_cleanup_pidfile_process "$VALKEY_PIDFILE" valkey "valkey-server"
managed_cleanup_listener "$VALKEY_PORT" valkey "valkey-server"
rm -f "$VALKEY_PIDFILE"

exec bash "$PROJECT_ROOT/scripts/channel/valkey-launch.sh"
