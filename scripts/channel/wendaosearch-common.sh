#!/usr/bin/env bash

set -euo pipefail

wendaosearch_default_service_name() {
  printf '%s\n' "wendaosearch-parser-summary"
}

wendaosearch_default_runtime_dir() {
  printf '%s\n' ".run/wendaosearch"
}

wendaosearch_default_config() {
  printf '%s\n' ".data/WendaoSearch.jl/config/live/parser_summary.toml"
}

wendaosearch_default_script_name() {
  printf '%s\n' "run_parser_summary_service.jl"
}

wendaosearch_default_package_repo_url() {
  printf '%s\n' "https://github.com/tao3k/WendaoSearch.jl.git"
}

wendaosearch_default_host() {
  printf '%s\n' "127.0.0.1"
}

wendaosearch_default_port() {
  printf '%s\n' "41081"
}

wendaosearch_default_mode() {
  printf '%s\n' "solver_demo"
}

wendaosearch_default_route_names() {
  printf '%s\n' "capability_manifest,structural_rerank,constraint_filter"
}

wendaosearch_effective_service_name() {
  printf '%s\n' "${WENDAOSEARCH_SERVICE_NAME:-$(wendaosearch_default_service_name)}"
}

wendaosearch_effective_runtime_dir() {
  printf '%s\n' "${WENDAOSEARCH_RUNTIME_DIR:-$(wendaosearch_default_runtime_dir)}"
}

wendaosearch_effective_config() {
  printf '%s\n' "${WENDAOSEARCH_CONFIG:-$(wendaosearch_default_config)}"
}

wendaosearch_effective_pidfile() {
  if [ -n "${WENDAOSEARCH_PIDFILE:-}" ]; then
    printf '%s\n' "$WENDAOSEARCH_PIDFILE"
    return 0
  fi

  printf '%s/%s.pid\n' \
    "$(wendaosearch_effective_runtime_dir)" \
    "$(wendaosearch_effective_service_name)"
}

wendaosearch_service_kind_from_script_name() {
  local script_name="$1"
  case "$script_name" in
  run_parser_summary_service.jl)
    printf '%s\n' "parser_summary"
    ;;
  run_search_service.jl)
    printf '%s\n' "search"
    ;;
  *)
    return 1
    ;;
  esac
}

wendaosearch_service_kind_from_service_name() {
  local service_name="$1"
  case "$service_name" in
  wendaosearch-parser-summary)
    printf '%s\n' "parser_summary"
    ;;
  wendaosearch-solver-demo)
    printf '%s\n' "search"
    ;;
  *)
    return 1
    ;;
  esac
}

wendaosearch_detect_service_kind_from_config() {
  local root="$1"
  if wendaosearch_config_value "$root" "route_name" >/dev/null 2>&1 ||
    wendaosearch_config_value "$root" "route_names" >/dev/null 2>&1 ||
    wendaosearch_config_value "$root" "mode" >/dev/null 2>&1; then
    printf '%s\n' "search"
    return 0
  fi
  if wendaosearch_config_value "$root" "code_parser_route_name" >/dev/null 2>&1 ||
    wendaosearch_config_value "$root" "code_parser_route_names" >/dev/null 2>&1 ||
    wendaosearch_config_section_value "$root" "interface" "host" >/dev/null 2>&1 ||
    wendaosearch_config_section_value "$root" "interface" "port" >/dev/null 2>&1; then
    printf '%s\n' "parser_summary"
    return 0
  fi
  return 1
}

wendaosearch_effective_service_kind() {
  local root="${1:-$(pwd)}"
  if [ -n "${WENDAOSEARCH_SCRIPT:-}" ]; then
    wendaosearch_service_kind_from_script_name "$WENDAOSEARCH_SCRIPT" && return 0
  fi
  if [ -n "${WENDAOSEARCH_SERVICE_NAME:-}" ]; then
    wendaosearch_service_kind_from_service_name "$WENDAOSEARCH_SERVICE_NAME" && return 0
  fi
  wendaosearch_detect_service_kind_from_config "$root" 2>/dev/null || printf '%s\n' "parser_summary"
}

wendaosearch_effective_script_name() {
  local root="${1:-$(pwd)}"
  if [ -n "${WENDAOSEARCH_SCRIPT:-}" ]; then
    printf '%s\n' "$WENDAOSEARCH_SCRIPT"
    return 0
  fi
  case "$(wendaosearch_effective_service_kind "$root")" in
  search)
    printf '%s\n' "run_search_service.jl"
    ;;
  *)
    printf '%s\n' "run_parser_summary_service.jl"
    ;;
  esac
}

wendaosearch_effective_package_repo_url() {
  printf '%s\n' "${WENDAOSEARCH_PACKAGE_REPO_URL:-$(wendaosearch_default_package_repo_url)}"
}

wendaosearch_config_value() {
  local root="$1"
  local field="$2"
  local raw_value
  raw_value="$(wendaosearch_read_raw_config_value "$root" "" "$field")" || return 1
  wendaosearch_normalize_toml_value "$raw_value"
}

wendaosearch_config_section_value() {
  local root="$1"
  local section="$2"
  local field="$3"
  local raw_value
  raw_value="$(wendaosearch_read_raw_config_value "$root" "$section" "$field")" || return 1
  wendaosearch_normalize_toml_value "$raw_value"
}

wendaosearch_read_raw_config_value() {
  local root="$1"
  local section="$2"
  local field="$3"
  local config_path
  config_path="$(wendaosearch_resolve_path "$root" "$(wendaosearch_effective_config)")"
  awk -v target_section="$section" -v target_field="$field" '
    function trim(value) {
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", value)
      return value
    }

    BEGIN {
      current_section = ""
      capture = 0
      found = 0
      value = ""
    }

    /^[[:space:]]*#/ || /^[[:space:]]*$/ {
      next
    }

    /^\[[^]]+\][[:space:]]*$/ {
      if (capture) {
        exit 1
      }
      current_section = $0
      gsub(/^[[:space:]]*\[/, "", current_section)
      gsub(/\][[:space:]]*$/, "", current_section)
      current_section = trim(current_section)
      next
    }

    {
      line = $0
      if (!capture) {
        if (target_section == "" && current_section != "") {
          next
        }
        if (target_section != "" && current_section != target_section) {
          next
        }
        pattern = "^[[:space:]]*" target_field "[[:space:]]*="
        if (line !~ pattern) {
          next
        }
        sub(pattern, "", line)
        value = trim(line)
        if (value ~ /^\[/ && value !~ /\][[:space:]]*$/) {
          capture = 1
          next
        }
        found = 1
        print value
        exit 0
      }

      value = value "\n" trim(line)
      if (line ~ /\][[:space:]]*$/) {
        found = 1
        print value
        exit 0
      }
    }

    END {
      if (!found) {
        exit 1
      }
    }
  ' "$config_path"
}

wendaosearch_normalize_toml_value() {
  local raw_value="$1"
  if [[ $raw_value == \[* ]]; then
    printf '%s\n' "$raw_value" |
      tr '\n' ' ' |
      sed -E 's/^\[//; s/\][[:space:]]*$//; s/"//g; s/[[:space:]]*,[[:space:]]*/,/g; s/^[[:space:],]+//; s/[[:space:],]+$//'
    return 0
  fi
  printf '%s\n' "$raw_value" | sed -E 's/^"(.*)"$/\1/'
}

wendaosearch_effective_host() {
  if [ -n "${WENDAOSEARCH_HOST:-}" ]; then
    printf '%s\n' "$WENDAOSEARCH_HOST"
    return 0
  fi
  if [ "$(wendaosearch_effective_service_kind "$1")" = "parser_summary" ]; then
    wendaosearch_config_section_value "$1" "interface" "host" 2>/dev/null ||
      wendaosearch_config_value "$1" "host" 2>/dev/null ||
      wendaosearch_default_host
    return 0
  fi
  wendaosearch_config_value "$1" "host" 2>/dev/null || wendaosearch_default_host
}

wendaosearch_effective_port() {
  if [ -n "${WENDAOSEARCH_PORT:-}" ]; then
    printf '%s\n' "$WENDAOSEARCH_PORT"
    return 0
  fi
  if [ "$(wendaosearch_effective_service_kind "$1")" = "parser_summary" ]; then
    wendaosearch_config_section_value "$1" "interface" "port" 2>/dev/null ||
      wendaosearch_config_value "$1" "port" 2>/dev/null ||
      wendaosearch_default_port
    return 0
  fi
  wendaosearch_config_value "$1" "port" 2>/dev/null || wendaosearch_default_port
}

wendaosearch_effective_mode() {
  if [ -n "${WENDAOSEARCH_MODE:-}" ]; then
    printf '%s\n' "$WENDAOSEARCH_MODE"
    return 0
  fi
  wendaosearch_config_value "$1" "mode" 2>/dev/null || wendaosearch_default_mode
}

wendaosearch_effective_route_name() {
  if [ -n "${WENDAOSEARCH_ROUTE_NAME:-}" ]; then
    printf '%s\n' "$WENDAOSEARCH_ROUTE_NAME"
    return 0
  fi
  wendaosearch_config_value "$1" "route_name" 2>/dev/null || printf '%s\n' ""
}

wendaosearch_effective_route_names() {
  if [ -n "${WENDAOSEARCH_ROUTE_NAMES:-}" ]; then
    printf '%s\n' "$WENDAOSEARCH_ROUTE_NAMES"
    return 0
  fi
  wendaosearch_config_value "$1" "route_names" 2>/dev/null || wendaosearch_default_route_names
}

wendaosearch_effective_julia_load_path() {
  printf '%s\n' "${WENDAOSEARCH_JULIA_LOAD_PATH:-@:@stdlib}"
}

wendaosearch_resolve_path() {
  local root="$1"
  local path="$2"
  if [[ $path == /* ]]; then
    printf '%s\n' "$path"
  else
    printf '%s\n' "$root/$path"
  fi
}

wendaosearch_package_dir() {
  local root="$1"
  wendaosearch_resolve_path "$root" ".data/WendaoSearch.jl"
}

wendaosearch_materialize_package_repo() {
  local root="$1"
  local package_dir
  package_dir="$(wendaosearch_package_dir "$root")"
  managed_materialize_git_repo \
    "$package_dir" \
    "$(wendaosearch_effective_package_repo_url)" \
    "" \
    "WendaoSearch package checkout"
}

wendaosearch_script_path() {
  local root="$1"
  local package_dir
  package_dir="$(wendaosearch_package_dir "$root")"
  printf '%s\n' "$package_dir/scripts/$(wendaosearch_effective_script_name "$root")"
}
