#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../common.sh"
PROJECT_ROOT="$(process_project_root "$SCRIPT_DIR" "../../../..")"
PROJECT_RUNTIME_ROOT="$(process_runtime_root "$PROJECT_ROOT")"
LOG_DIR="${WENDAO_GATEWAY_LOG_DIR:-$PROJECT_RUNTIME_ROOT/logs}"
RUNTIME_DIR="${WENDAO_GATEWAY_RUNTIME_DIR:-$PROJECT_RUNTIME_ROOT/wendao-gateway}"
PIDFILE="${WENDAO_GATEWAY_PIDFILE:-$RUNTIME_DIR/wendao.pid}"
CONFIG_PATH="${WENDAO_GATEWAY_CONFIG:-$PROJECT_ROOT/wendao.toml}"
PYTHON_BIN="${WENDAO_GATEWAY_PYTHON:-${PYO3_PYTHON:-python3}}"
PORT_RESOLVER="${WENDAO_GATEWAY_PORT_RESOLVER:-$PROJECT_ROOT/scripts/runtime/resolve_wendao_gateway_port.py}"
WENDAO_BIN="${WENDAO_GATEWAY_BIN:-$PROJECT_ROOT/target/debug/wendao}"
STDOUT_LOG="${WENDAO_GATEWAY_STDOUT_LOG:-$LOG_DIR/wendao-gateway.stdout.log}"
STDERR_LOG="${WENDAO_GATEWAY_STDERR_LOG:-$LOG_DIR/wendao-gateway.stderr.log}"
BUILD_MODE="${WENDAO_GATEWAY_BUILD:-auto}"

CONFIG_PATH="$(process_abs_path "$PROJECT_ROOT" "$CONFIG_PATH")"
LOG_DIR="$(process_abs_path "$PROJECT_ROOT" "$LOG_DIR")"
RUNTIME_DIR="$(process_abs_path "$PROJECT_ROOT" "$RUNTIME_DIR")"
PIDFILE="$(process_abs_path "$PROJECT_ROOT" "$PIDFILE")"
WENDAO_BIN="$(process_abs_path "$PROJECT_ROOT" "$WENDAO_BIN")"

source "$PROJECT_ROOT/scripts/runtime/process-runtime.sh"
mkdir -p "$RUNTIME_DIR" "$LOG_DIR"

PORT="$("$PYTHON_BIN" "$PORT_RESOLVER" --config "$CONFIG_PATH")"
managed_cleanup_pidfile_process "$PIDFILE" wendao-gateway "$WENDAO_BIN" " gateway start"
managed_cleanup_listener "$PORT" wendao-gateway "$WENDAO_BIN" " gateway start"
rm -f "$PIDFILE"

export VALKEY_URL="${VALKEY_URL:-redis://127.0.0.1:6379/0}"
export XIUXIAN_WENDAO_GATEWAY_BOOTSTRAP_BACKGROUND_INDEXING="${XIUXIAN_WENDAO_GATEWAY_BOOTSTRAP_BACKGROUND_INDEXING:-1}"
export XIUXIAN_WENDAO_GATEWAY_FLIGHT_GRPC_WEB_ENABLED="${XIUXIAN_WENDAO_GATEWAY_FLIGHT_GRPC_WEB_ENABLED:-true}"
export XIUXIAN_WENDAO_GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS="${XIUXIAN_WENDAO_GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS:-600}"
export WENDAO_GATEWAY_PIDFILE="$PIDFILE"

cd "$PROJECT_ROOT"
case "$BUILD_MODE" in
  0|false|False|FALSE|off|OFF)
    ;;
  1|true|True|TRUE|on|ON)
    if command -v cargo >/dev/null 2>&1; then
      cargo build -p xiuxian-wendao-studio --bin wendao --features cli-bin-support,zhenfa-router --locked
    elif [ ! -x "$WENDAO_BIN" ]; then
      echo "Error: cargo not found and Wendao gateway binary is missing: $WENDAO_BIN" >&2
      exit 1
    fi
    ;;
  auto|"")
    if [ ! -x "$WENDAO_BIN" ]; then
      if command -v cargo >/dev/null 2>&1; then
        cargo build -p xiuxian-wendao-studio --bin wendao --features cli-bin-support,zhenfa-router --locked
      else
        echo "Error: cargo not found and Wendao gateway binary is missing: $WENDAO_BIN" >&2
        exit 1
      fi
    fi
    ;;
  *)
    echo "Error: unsupported WENDAO_GATEWAY_BUILD value: $BUILD_MODE" >&2
    exit 1
    ;;
esac

"$WENDAO_BIN" --conf "$CONFIG_PATH" gateway start \
  > >(tee -a "$STDOUT_LOG") \
  2> >(tee -a "$STDERR_LOG" >&2) &
WENDAO_GATEWAY_CHILD_PID=$!
managed_write_pidfile "$PIDFILE" "$WENDAO_GATEWAY_CHILD_PID"

cleanup_child() {
  if managed_process_is_alive "$WENDAO_GATEWAY_CHILD_PID"; then
    kill "$WENDAO_GATEWAY_CHILD_PID" 2>/dev/null || true
    managed_wait_for_exit "$WENDAO_GATEWAY_CHILD_PID" 25 0.2 || true
  fi
  rm -f "$PIDFILE"
}

trap cleanup_child TERM INT

if wait "$WENDAO_GATEWAY_CHILD_PID"; then
  STATUS=0
else
  STATUS=$?
fi
rm -f "$PIDFILE"
exit "$STATUS"
