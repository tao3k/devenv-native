#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../common.sh"
PROJECT_ROOT="$(process_project_root "$SCRIPT_DIR" "../../../..")"
process_export_project_defaults "$PROJECT_ROOT"
export WENDAOSEARCH_SERVICE_NAME="${WENDAOSEARCH_SERVICE_NAME:-wendaosearch-parser-summary}"

exec bash "$PROJECT_ROOT/scripts/channel/wendaosearch-healthcheck.sh"
