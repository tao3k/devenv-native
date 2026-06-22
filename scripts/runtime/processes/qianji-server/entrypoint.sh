#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../common.sh"
PROJECT_ROOT="$(process_project_root "$SCRIPT_DIR" "../../../..")"
PROJECT_RUNTIME_ROOT="$(process_runtime_root "$PROJECT_ROOT")"
LOG_DIR="${QIANJI_SERVER_LOG_DIR:-$PROJECT_RUNTIME_ROOT/logs}"
RUNTIME_DIR="${QIANJI_SERVER_RUNTIME_DIR:-$PROJECT_RUNTIME_ROOT/qianji-server}"
PIDFILE="${QIANJI_SERVER_PIDFILE:-$RUNTIME_DIR/qianji-server.pid}"
QIANJI_SERVER_BIN="${QIANJI_SERVER_BIN:-$PROJECT_ROOT/target/debug/qianji-server}"
QIANJI_SERVER_CARGO_FEATURES="${QIANJI_SERVER_CARGO_FEATURES:-valkey,qianji-full}"
BIND_ADDR="${QIANJI_SERVER_BIND_ADDR:-127.0.0.1:38130}"
FLIGHT_BIND_ADDR="${QIANJI_SERVER_FLIGHT_BIND_ADDR:-127.0.0.1:38131}"
QIANJI_VALKEY_URL="${QIANJI_SERVER_VALKEY_URL:-${VALKEY_URL:-redis://127.0.0.1:6379/0}}"
FLOWHUB_ROOT="${QIANJI_FLOWHUB_ROOT:-$PROJECT_ROOT/qianji-flowhub}"
CONTROL_LEDGER_PATH="${QIANJI_SERVER_CONTROL_LEDGER:-${QIANJI_SERVER_CONTROL_LEDGER_PATH:-$RUNTIME_DIR/control-ledger.duckdb}}"
STDOUT_LOG="${QIANJI_SERVER_STDOUT_LOG:-$LOG_DIR/qianji-server.stdout.log}"
STDERR_LOG="${QIANJI_SERVER_STDERR_LOG:-$LOG_DIR/qianji-server.stderr.log}"
BUILD_MODE="${QIANJI_SERVER_BUILD:-auto}"

LOG_DIR="$(process_abs_path "$PROJECT_ROOT" "$LOG_DIR")"
RUNTIME_DIR="$(process_abs_path "$PROJECT_ROOT" "$RUNTIME_DIR")"
PIDFILE="$(process_abs_path "$PROJECT_ROOT" "$PIDFILE")"
QIANJI_SERVER_BIN="$(process_abs_path "$PROJECT_ROOT" "$QIANJI_SERVER_BIN")"
FLOWHUB_ROOT="$(process_abs_path "$PROJECT_ROOT" "$FLOWHUB_ROOT")"
CONTROL_LEDGER_PATH="$(process_abs_path "$PROJECT_ROOT" "$CONTROL_LEDGER_PATH")"

source "$PROJECT_ROOT/scripts/runtime/process-runtime.sh"
mkdir -p "$RUNTIME_DIR" "$LOG_DIR"

PORT="${BIND_ADDR##*:}"
FLIGHT_PORT="${FLIGHT_BIND_ADDR##*:}"
managed_cleanup_pidfile_process "$PIDFILE" qianji-server "$QIANJI_SERVER_BIN" "qianji-server"
managed_cleanup_listener "$PORT" qianji-server "$QIANJI_SERVER_BIN" "qianji-server"
managed_cleanup_listener "$FLIGHT_PORT" qianji-server-flight "$QIANJI_SERVER_BIN" "qianji-server"
rm -f "$PIDFILE"

build_qianji_server_binary() {
  if ! command -v cargo >/dev/null 2>&1; then
    echo "Error: cargo not found and Qianji server binary needs to be built: $QIANJI_SERVER_BIN" >&2
    exit 1
  fi
  if [ -n "$QIANJI_SERVER_CARGO_FEATURES" ]; then
    cargo build -p xiuxian-qianji --bin qianji-server --features "$QIANJI_SERVER_CARGO_FEATURES" --locked
  else
    cargo build -p xiuxian-qianji --bin qianji-server --locked
  fi
}

qianji_server_binary_needs_build() {
  if [ ! -x "$QIANJI_SERVER_BIN" ]; then
    return 0
  fi
  for path in \
    "$PROJECT_ROOT/Cargo.lock" \
    "$PROJECT_ROOT/Cargo.toml" \
    "$PROJECT_ROOT/packages/rust/crates/xiuxian-qianji/Cargo.toml" \
    "$SCRIPT_DIR/entrypoint.sh"
  do
    if [ -e "$path" ] && [ "$path" -nt "$QIANJI_SERVER_BIN" ]; then
      return 0
    fi
  done
  for dir in \
    "$PROJECT_ROOT/packages/rust/crates/xiuxian-qianji/resources" \
    "$PROJECT_ROOT/packages/rust/crates/xiuxian-qianji/src"
  do
    if [ -d "$dir" ] && find "$dir" -type f -newer "$QIANJI_SERVER_BIN" -print -quit | grep -q .; then
      return 0
    fi
  done
  return 1
}

cd "$PROJECT_ROOT"
case "$BUILD_MODE" in
  0|false|False|FALSE|off|OFF)
    ;;
  1|true|True|TRUE|on|ON)
    build_qianji_server_binary
    ;;
  auto|"")
    if qianji_server_binary_needs_build; then
      build_qianji_server_binary
    fi
    ;;
  *)
    echo "Error: unsupported QIANJI_SERVER_BUILD value: $BUILD_MODE" >&2
    exit 1
    ;;
esac

"$QIANJI_SERVER_BIN" \
  --bind "$BIND_ADDR" \
  --flight-bind "$FLIGHT_BIND_ADDR" \
  --valkey-url "$QIANJI_VALKEY_URL" \
  --flowhub-root "$FLOWHUB_ROOT" \
  --control-ledger "$CONTROL_LEDGER_PATH" \
  --require-valkey-ready \
  > >(tee -a "$STDOUT_LOG") \
  2> >(tee -a "$STDERR_LOG" >&2) &
QIANJI_SERVER_CHILD_PID=$!
managed_write_pidfile "$PIDFILE" "$QIANJI_SERVER_CHILD_PID"

cleanup_child() {
  if managed_process_is_alive "$QIANJI_SERVER_CHILD_PID"; then
    kill "$QIANJI_SERVER_CHILD_PID" 2>/dev/null || true
    managed_wait_for_exit "$QIANJI_SERVER_CHILD_PID" 25 0.2 || true
  fi
  rm -f "$PIDFILE"
}

trap cleanup_child TERM INT

if wait "$QIANJI_SERVER_CHILD_PID"; then
  STATUS=0
else
  STATUS=$?
fi
rm -f "$PIDFILE"
exit "$STATUS"
