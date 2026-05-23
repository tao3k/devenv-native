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
BIND_ADDR="${QIANJI_SERVER_BIND_ADDR:-127.0.0.1:38130}"
QIANJI_VALKEY_URL="${QIANJI_SERVER_VALKEY_URL:-${VALKEY_URL:-redis://127.0.0.1:6379/0}}"
STDOUT_LOG="${QIANJI_SERVER_STDOUT_LOG:-$LOG_DIR/qianji-server.stdout.log}"
STDERR_LOG="${QIANJI_SERVER_STDERR_LOG:-$LOG_DIR/qianji-server.stderr.log}"
BUILD_MODE="${QIANJI_SERVER_BUILD:-auto}"

LOG_DIR="$(process_abs_path "$PROJECT_ROOT" "$LOG_DIR")"
RUNTIME_DIR="$(process_abs_path "$PROJECT_ROOT" "$RUNTIME_DIR")"
PIDFILE="$(process_abs_path "$PROJECT_ROOT" "$PIDFILE")"
QIANJI_SERVER_BIN="$(process_abs_path "$PROJECT_ROOT" "$QIANJI_SERVER_BIN")"

source "$PROJECT_ROOT/scripts/runtime/process-runtime.sh"
mkdir -p "$RUNTIME_DIR" "$LOG_DIR"

PORT="${BIND_ADDR##*:}"
managed_cleanup_pidfile_process "$PIDFILE" qianji-server "$QIANJI_SERVER_BIN" "qianji-server"
managed_cleanup_listener "$PORT" qianji-server "$QIANJI_SERVER_BIN" "qianji-server"
rm -f "$PIDFILE"

cd "$PROJECT_ROOT"
case "$BUILD_MODE" in
  0|false|False|FALSE|off|OFF)
    ;;
  1|true|True|TRUE|on|ON)
    if command -v cargo >/dev/null 2>&1; then
      cargo build -p xiuxian-qianji --bin qianji-server --locked
    elif [ ! -x "$QIANJI_SERVER_BIN" ]; then
      echo "Error: cargo not found and Qianji server binary is missing: $QIANJI_SERVER_BIN" >&2
      exit 1
    fi
    ;;
  auto|"")
    if [ ! -x "$QIANJI_SERVER_BIN" ]; then
      if command -v cargo >/dev/null 2>&1; then
        cargo build -p xiuxian-qianji --bin qianji-server --locked
      else
        echo "Error: cargo not found and Qianji server binary is missing: $QIANJI_SERVER_BIN" >&2
        exit 1
      fi
    fi
    ;;
  *)
    echo "Error: unsupported QIANJI_SERVER_BUILD value: $BUILD_MODE" >&2
    exit 1
    ;;
esac

"$QIANJI_SERVER_BIN" \
  --bind "$BIND_ADDR" \
  --valkey-url "$QIANJI_VALKEY_URL" \
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
