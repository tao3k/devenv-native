#!/usr/bin/env bash

managed_pidfile_process_id() {
  local pidfile="$1"

  [ -s "$pidfile" ] || return 1
  tr -d '[:space:]' <"$pidfile"
}

managed_process_is_alive() {
  local pid="$1"
  kill -0 "$pid" 2>/dev/null
}

managed_process_command() {
  local pid="$1"
  ps -p "$pid" -o command= 2>/dev/null || true
}

managed_project_root() {
  if [ -n "${PRJ_ROOT:-}" ]; then
    printf '%s\n' "$PRJ_ROOT"
    return 0
  fi
  if [ -n "${DEVENV_ROOT:-}" ]; then
    printf '%s\n' "$DEVENV_ROOT"
    return 0
  fi
  return 1
}

managed_command_matches_pattern() {
  local command="$1"
  local pattern="$2"

  [ -n "$pattern" ] || return 0
  [[ $command == *"$pattern"* ]] && return 0

  local project_root relative_pattern
  project_root="$(managed_project_root 2>/dev/null || true)"
  if [ -n "$project_root" ] && [[ $pattern == "$project_root/"* ]]; then
    relative_pattern="${pattern#"$project_root"/}"
    [[ $command == *"$relative_pattern"* ]] && return 0
  fi

  return 1
}

managed_pid_matches_patterns() {
  local pid="$1"
  shift

  local command
  command="$(managed_process_command "$pid")"
  [ -n "$command" ] || return 1

  local pattern
  for pattern in "$@"; do
    managed_command_matches_pattern "$command" "$pattern" || return 1
  done
}

managed_listener_pid() {
  local port="$1"
  lsof -nP -iTCP:"$port" -sTCP:LISTEN -t 2>/dev/null | head -n 1 || true
}

managed_listener_matches_pidfile() {
  local pidfile="$1"
  local port="$2"

  local expected_pid listener_pid
  expected_pid="$(managed_pidfile_process_id "$pidfile")" || return 1
  listener_pid="$(managed_listener_pid "$port")"
  [ -n "$listener_pid" ] || return 1
  [ "$listener_pid" = "$expected_pid" ]
}

managed_wait_for_exit() {
  local pid="$1"
  local attempts="${2:-50}"
  local sleep_secs="${3:-0.2}"
  local attempt=1

  while [ "$attempt" -le "$attempts" ]; do
    managed_process_is_alive "$pid" || return 0
    sleep "$sleep_secs"
    attempt=$((attempt + 1))
  done

  return 1
}

managed_terminate_pid() {
  local pid="$1"
  local label="$2"

  managed_process_is_alive "$pid" || return 0

  kill "$pid" 2>/dev/null || true
  if managed_wait_for_exit "$pid" 50 0.2; then
    return 0
  fi

  kill -9 "$pid" 2>/dev/null || true
  if managed_wait_for_exit "$pid" 10 0.1; then
    return 0
  fi

  echo "Error: failed to terminate ${label} process ${pid}." >&2
  return 1
}

managed_cleanup_pidfile_process() {
  local pidfile="$1"
  local label="$2"
  shift 2

  [ -e "$pidfile" ] || return 0

  local pid
  pid="$(managed_pidfile_process_id "$pidfile")" || {
    rm -f "$pidfile"
    return 0
  }

  if ! managed_process_is_alive "$pid"; then
    rm -f "$pidfile"
    return 0
  fi

  if ! managed_pid_matches_patterns "$pid" "$@"; then
    echo "Error: refusing to clean ${label} pidfile process ${pid} because it is not owned by this service." >&2
    echo "Command: $(managed_process_command "$pid")" >&2
    return 1
  fi

  managed_terminate_pid "$pid" "$label"
  rm -f "$pidfile"
}

managed_cleanup_listener() {
  local port="$1"
  local label="$2"
  shift 2

  local listener_pid
  listener_pid="$(managed_listener_pid "$port")"
  [ -n "$listener_pid" ] || return 0

  if ! managed_process_is_alive "$listener_pid"; then
    return 0
  fi

  if ! managed_pid_matches_patterns "$listener_pid" "$@"; then
    echo "Error: refusing to clean ${label} listener on port ${port} because pid ${listener_pid} is not owned by this service." >&2
    echo "Command: $(managed_process_command "$listener_pid")" >&2
    return 1
  fi

  managed_terminate_pid "$listener_pid" "$label"
}

managed_write_pidfile() {
  local pidfile="$1"
  local pid="$2"
  mkdir -p "$(dirname "$pidfile")"
  printf '%s\n' "$pid" >"$pidfile"
}

managed_git_checkout_exists() {
  local path="$1"
  git -C "$path" rev-parse --is-inside-work-tree >/dev/null 2>&1
}

managed_materialize_git_repo() {
  local target_dir="$1"
  local repo_url="$2"
  local repo_rev="$3"
  local label="$4"

  if [ -d "$target_dir" ]; then
    if managed_git_checkout_exists "$target_dir"; then
      return 0
    fi
    echo "Error: ${label} path exists but is not a git checkout: ${target_dir}" >&2
    return 1
  fi

  command -v git >/dev/null 2>&1 || {
    echo "Error: git not found while materializing ${label}." >&2
    return 1
  }

  mkdir -p "$(dirname "$target_dir")"
  git clone "$repo_url" "$target_dir"
  if [ -n "$repo_rev" ]; then
    git -C "$target_dir" checkout --detach "$repo_rev"
  fi
}
