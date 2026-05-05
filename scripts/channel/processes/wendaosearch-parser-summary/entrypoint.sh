#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../common.sh"
PROJECT_ROOT="$(process_project_root "$SCRIPT_DIR" "../../../..")"

export WENDAOSEARCH_SERVICE_NAME="${WENDAOSEARCH_SERVICE_NAME:-wendaosearch-parser-summary}"
export WENDAOSEARCH_CONFIG="${WENDAOSEARCH_CONFIG:-$PROJECT_ROOT/.data/WendaoSearch.jl/config/live/parser_summary.toml}"
export WENDAOSEARCH_SCRIPT="${WENDAOSEARCH_SCRIPT:-run_parser_summary_service.jl}"

exec bash "$PROJECT_ROOT/scripts/channel/processes/wendaosearch/entrypoint.sh"
