#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../common.sh"
PROJECT_ROOT="$(process_project_root "$SCRIPT_DIR" "../../../..")"
PROJECT_RUNTIME_ROOT="$(process_runtime_root "$PROJECT_ROOT")"
RUNTIME_DIR="${WENDAO_SENTINEL_RUNTIME_DIR:-$PROJECT_RUNTIME_ROOT/wendao-sentinel}"
PIDFILE="${WENDAO_SENTINEL_PIDFILE:-$RUNTIME_DIR/wendao-sentinel.pid}"
CONFIG_PATH="${WENDAO_GATEWAY_CONFIG:-$PROJECT_ROOT/wendao.toml}"
WENDAO_BIN="${WENDAO_SENTINEL_BIN:-${WENDAO_GATEWAY_BIN:-$PROJECT_ROOT/target/debug/wendao}}"
BUILD_ENABLED="${WENDAO_SENTINEL_BUILD:-1}"
SENTINEL_FEATURES="${WENDAO_SENTINEL_FEATURES:-${WENDAO_GATEWAY_FEATURES:-cli-bin-support,zhenfa-router,document-extract-attachment-audit,document-extract-pdf-render,document-extract-audio-shards}}"

CONFIG_PATH="$(process_abs_path "$PROJECT_ROOT" "$CONFIG_PATH")"
RUNTIME_DIR="$(process_abs_path "$PROJECT_ROOT" "$RUNTIME_DIR")"
PIDFILE="$(process_abs_path "$PROJECT_ROOT" "$PIDFILE")"
WENDAO_BIN="$(process_abs_path "$PROJECT_ROOT" "$WENDAO_BIN")"

source "$PROJECT_ROOT/scripts/runtime/process-runtime.sh"
mkdir -p "$RUNTIME_DIR"
managed_cleanup_pidfile_process "$PIDFILE" wendao-sentinel "$WENDAO_BIN" " sentinel watch"
rm -f "$PIDFILE"

export VALKEY_URL="${VALKEY_URL:-redis://127.0.0.1:6379/0}"

cd "$PROJECT_ROOT"
if [ "$BUILD_ENABLED" != "0" ]; then
  cargo build -p xiuxian-wendao-studio --bin wendao --features "$SENTINEL_FEATURES" --locked
fi

"$WENDAO_BIN" --conf "$CONFIG_PATH" sentinel watch &
WENDAO_SENTINEL_CHILD_PID=$!
managed_write_pidfile "$PIDFILE" "$WENDAO_SENTINEL_CHILD_PID"

cleanup_child() {
  if managed_process_is_alive "$WENDAO_SENTINEL_CHILD_PID"; then
    kill "$WENDAO_SENTINEL_CHILD_PID" 2>/dev/null || true
    managed_wait_for_exit "$WENDAO_SENTINEL_CHILD_PID" 25 0.2 || true
  fi
  rm -f "$PIDFILE"
}

trap cleanup_child TERM INT

if wait "$WENDAO_SENTINEL_CHILD_PID"; then
  STATUS=0
else
  STATUS=$?
fi
rm -f "$PIDFILE"
exit "$STATUS"
