#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../common.sh"
PROJECT_ROOT="$(process_project_root "$SCRIPT_DIR" "../../../..")"
process_export_project_defaults "$PROJECT_ROOT"
PROJECT_RUNTIME_ROOT="$(process_runtime_root "$PROJECT_ROOT")"

export WENDAOSEARCH_SERVICE_NAME="${WENDAOSEARCH_SERVICE_NAME:-wendaocodeparser-parser-summary}"
export WENDAOSEARCH_RUNTIME_DIR="${WENDAOSEARCH_RUNTIME_DIR:-$PROJECT_RUNTIME_ROOT/wendaocodeparser}"

exec bash "$PROJECT_ROOT/scripts/runtime/wendaosearch-healthcheck.sh"
