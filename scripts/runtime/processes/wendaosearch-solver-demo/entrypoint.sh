#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../common.sh"
PROJECT_ROOT="$(process_project_root "$SCRIPT_DIR" "../../../..")"

export WENDAOSEARCH_SERVICE_NAME="${WENDAOSEARCH_SERVICE_NAME:-wendaosearch-solver-demo}"
export WENDAOSEARCH_CONFIG="${WENDAOSEARCH_CONFIG:-$PROJECT_ROOT/.data/WendaoSearch.jl/config/live/solver_demo.toml}"

exec bash "$PROJECT_ROOT/scripts/runtime/processes/wendaosearch/entrypoint.sh"
