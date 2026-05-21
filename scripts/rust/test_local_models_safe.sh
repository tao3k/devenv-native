#!/usr/bin/env bash
set -euo pipefail

profile="${1:-safe}"
model_root="${2:-}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

run_vision_safe() {
  bash "${PROJECT_ROOT}/scripts/rust/test_vision_smoke_lane.sh"
}

run_vision_full() {
  bash "${PROJECT_ROOT}/scripts/rust/test_vision_heavy_lane.sh" "${model_root}"
}

case "${profile}" in
safe)
  run_vision_safe
  ;;
full)
  run_vision_full
  ;;
vision-only)
  run_vision_safe
  ;;
vision-heavy-only)
  run_vision_full
  ;;
*)
  cat <<'USAGE' >&2
Usage: scripts/rust/test_local_models_safe.sh [safe|full|vision-only|vision-heavy-only] [model_root]
USAGE
  exit 2
  ;;
esac
