#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../common.sh"
PROJECT_ROOT="$(process_project_root "$SCRIPT_DIR" "../../../..")"
export WENDAO_FRONTEND_MANAGED="${WENDAO_FRONTEND_MANAGED:-1}"

exec bash "$PROJECT_ROOT/scripts/channel/wendao-frontend-launch.sh"
