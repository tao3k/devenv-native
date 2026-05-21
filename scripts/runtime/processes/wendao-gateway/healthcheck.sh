#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../common.sh"
PROJECT_ROOT="$(process_project_root "$SCRIPT_DIR" "../../../..")"
process_export_project_defaults "$PROJECT_ROOT"

exec bash "$PROJECT_ROOT/scripts/runtime/wendao-gateway-healthcheck.sh"
