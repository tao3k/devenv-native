#!/usr/bin/env bash

process_project_root() {
  local script_dir="$1"
  local fallback_relative="$2"

  if [ -n "${PRJ_ROOT:-}" ]; then
    printf '%s' "$PRJ_ROOT"
    return 0
  fi
  if [ -n "${DEVENV_ROOT:-}" ]; then
    printf '%s' "$DEVENV_ROOT"
    return 0
  fi

  cd "$script_dir/$fallback_relative" && pwd
}

process_abs_path() {
  local base="$1"
  local path="$2"

  if [ -z "$path" ]; then
    return 0
  fi

  case "$path" in
  /*) printf '%s' "$path" ;;
  *) printf '%s/%s' "$base" "$path" ;;
  esac
}

process_runtime_root() {
  local project_root="$1"

  process_abs_path "$project_root" "${PRJ_RUNTIME_DIR:-$project_root/.run}"
}

process_data_root() {
  local project_root="$1"

  process_abs_path "$project_root" "${PRJ_DATA_HOME:-$project_root/.data}"
}

process_python_bin() {
  local preferred="${1:-}"

  if [ -n "$preferred" ]; then
    printf '%s' "$preferred"
    return 0
  fi
  if [ -n "${PYO3_PYTHON:-}" ]; then
    printf '%s' "$PYO3_PYTHON"
    return 0
  fi

  command -v python3 2>/dev/null || true
}

process_require_python_bin() {
  local preferred="${1:-}"
  local python_bin

  python_bin="$(process_python_bin "$preferred")"
  if [ -z "$python_bin" ]; then
    echo "Error: python3 not found in PATH." >&2
    return 1
  fi

  printf '%s' "$python_bin"
}

process_export_project_defaults() {
  local project_root="$1"

  export PRJ_ROOT="${PRJ_ROOT:-$project_root}"
  export PRJ_RUNTIME_DIR="${PRJ_RUNTIME_DIR:-$project_root/.run}"
  export PRJ_DATA_HOME="${PRJ_DATA_HOME:-$project_root/.data}"
}
