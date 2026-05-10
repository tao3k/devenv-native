#!/usr/bin/env bash

valkey_process_id_for_url() {
  local url="$1"

  valkey-cli -u "$url" info server | awk -F: '/^[[:space:]]*process_id:/ { gsub(/[[:space:]\r]/, "", $2); print $2; exit }'
}

valkey_pidfile_process_id() {
  local pidfile="$1"

  [ -s "$pidfile" ] || return 1
  cat "$pidfile"
}

valkey_listener_matches_pidfile() {
  local pidfile="$1"
  local url="$2"
  local expected_pid
  local actual_pid

  expected_pid="$(valkey_pidfile_process_id "$pidfile")" || return 1
  actual_pid="$(valkey_process_id_for_url "$url")" || return 1

  [ -n "$actual_pid" ] && [ "$actual_pid" = "$expected_pid" ]
}
