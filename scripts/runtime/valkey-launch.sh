#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
source "${SCRIPT_DIR}/valkey-common.sh"
source "${SCRIPT_DIR}/valkey-runtime.sh"

if ! command -v valkey-server >/dev/null 2>&1; then
  echo "Error: valkey-server not found in PATH." >&2
  exit 1
fi
if ! command -v valkey-cli >/dev/null 2>&1; then
  echo "Error: valkey-cli not found in PATH." >&2
  exit 1
fi

RUNTIME_DIR="$(valkey_resolve_path "$PROJECT_ROOT" "$(valkey_effective_runtime_dir)")"
DATA_DIR="$(valkey_resolve_path "$PROJECT_ROOT" "$(valkey_effective_data_dir)")"
PIDFILE="$(valkey_resolve_path "$PROJECT_ROOT" "$(valkey_effective_pidfile)")"
LOGFILE_VALUE="$(valkey_effective_logfile)"
LOGFILE=""
if [ -n "$LOGFILE_VALUE" ]; then
  LOGFILE="$(valkey_resolve_path "$PROJECT_ROOT" "$LOGFILE_VALUE")"
fi
BIND="$(valkey_effective_bind)"
PROTECTED_MODE="$(valkey_effective_protected_mode)"
PORT="$(valkey_effective_port)"
URL="$(valkey_effective_url)"
TCP_BACKLOG="$(valkey_effective_tcp_backlog)"
DAEMONIZE="$(valkey_effective_daemonize)"
INITIAL_DELAY_SECONDS="$(valkey_effective_startup_initial_delay_seconds)"
PERIOD_SECONDS="$(valkey_effective_startup_period_seconds)"
FAILURE_THRESHOLD="$(valkey_effective_startup_failure_threshold)"

mkdir -p "$RUNTIME_DIR" "$DATA_DIR"
rm -f "$PIDFILE"

server_args=(
  --port "$PORT"
  --bind "$BIND"
  --tcp-backlog "$TCP_BACKLOG"
  --dir "$DATA_DIR"
  --pidfile "$PIDFILE"
)

if [ -n "$LOGFILE" ]; then
  server_args+=(--logfile "$LOGFILE")
fi

if [ -n "$PROTECTED_MODE" ]; then
  server_args+=(--protected-mode "$PROTECTED_MODE")
fi

server_cmd=(valkey-server)

if [ "$DAEMONIZE" = "yes" ]; then
  "${server_cmd[@]}" "${server_args[@]}" --daemonize yes

  sleep "$INITIAL_DELAY_SECONDS"
  attempt=1
  while [ "$attempt" -le "$FAILURE_THRESHOLD" ]; do
    if valkey_listener_matches_pidfile "$PIDFILE" "$URL" && valkey-cli -u "$URL" ping >/dev/null 2>&1; then
      echo "Valkey started. pidfile=$PIDFILE logfile=${LOGFILE:-<stdout>} datadir=$DATA_DIR"
      exit 0
    fi
    sleep "$PERIOD_SECONDS"
    attempt=$((attempt + 1))
  done

  echo "Error: Valkey did not become healthy at $URL." >&2
  exit 1
fi

exec "${server_cmd[@]}" "${server_args[@]}" --daemonize no
