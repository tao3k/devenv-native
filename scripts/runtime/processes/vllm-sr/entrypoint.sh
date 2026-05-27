#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../common.sh"
PROJECT_ROOT="$(process_project_root "$SCRIPT_DIR" "../../../..")"
process_export_project_defaults "$PROJECT_ROOT"

PROJECT_CONFIG_ROOT="$(process_abs_path "$PROJECT_ROOT" "${PRJ_CONFIG_HOME:-$PROJECT_ROOT/.config}")"
PROJECT_RUNTIME_ROOT="$(process_runtime_root "$PROJECT_ROOT")"
LOG_DIR="${WENDAO_VLLM_SR_LOG_DIR:-$PROJECT_RUNTIME_ROOT/logs}"
CONFIG_PATH="${WENDAO_VLLM_SR_CONFIG_PATH:-$PROJECT_CONFIG_ROOT/vllm-sr/config.yaml}"
BASE_URL="${WENDAO_VLLM_SR_BASE_URL:-http://127.0.0.1:8888}"
MODE="${WENDAO_MODEL_ROUTING_MODE:-vllm-sr}"
PYTHON_BIN="$(process_require_python_bin "${WENDAO_VLLM_SR_PYTHON:-}")"

CONFIG_PATH="$(process_abs_path "$PROJECT_ROOT" "$CONFIG_PATH")"
LOG_DIR="$(process_abs_path "$PROJECT_ROOT" "$LOG_DIR")"
mkdir -p "$LOG_DIR" "$(dirname "$CONFIG_PATH")"

case "$MODE" in
  vllm-sr)
    ;;
  deterministic)
    echo "Wendao model routing mode is deterministic; vLLM-SR process is intentionally idle."
    exec bash -c 'while :; do sleep 3600; done'
    ;;
  *)
    echo "Error: unsupported WENDAO_MODEL_ROUTING_MODE value: $MODE" >&2
    exit 1
    ;;
esac

if [ ! -f "$CONFIG_PATH" ]; then
  "$PYTHON_BIN" - "$CONFIG_PATH" <<'PY'
from __future__ import annotations

import os
import sys
import json
from pathlib import Path

config_path = Path(sys.argv[1])
default_model = os.environ.get("WENDAO_VLLM_SR_DEFAULT_MODEL", "deepseek/deepseek-v4-pro")
provider_name = os.environ.get("WENDAO_VLLM_SR_DEFAULT_PROVIDER", "openrouter")
backend_base_url = os.environ.get(
    "WENDAO_VLLM_SR_DEFAULT_BACKEND_BASE_URL",
    "https://openrouter.ai/api/v1",
)
api_key_env = os.environ.get("WENDAO_VLLM_SR_DEFAULT_API_KEY_ENV", "OPENROUTER_API_KEY")
listener_port = int(os.environ.get("WENDAO_VLLM_SR_LISTENER_PORT", "8888"))

def ystr(value: str) -> str:
    return json.dumps(value)

config = f"""version: v0.3
listeners:
  - name: wendao-routing-proxy
    address: 127.0.0.1
    port: {listener_port}
    timeout: 300s
providers:
  defaults:
    default_model: {ystr(default_model)}
  models:
    - name: {ystr(default_model)}
      provider_model_id: {ystr(default_model)}
      api_format: openai
      external_model_ids:
        {provider_name}: {ystr(default_model)}
      backend_refs:
        - name: {ystr(provider_name)}
          provider: {ystr(provider_name)}
          base_url: {ystr(backend_base_url)}
          auth_header: Authorization
          auth_prefix: Bearer
          chat_path: /chat/completions
          api_key_env: {ystr(api_key_env)}
          weight: 100
routing:
  modelCards:
    - name: {ystr(default_model)}
      description: Wendao default chat model routed through vLLM-SR.
      capabilities: [chat]
      quality_score: 0.9
      modality: ar
      tags: [default, wendao]
global:
  router:
    config_source: file
    strategy: priority
  services:
    observability:
      metrics:
        enabled: true
"""

config_path.write_text(config, encoding="utf-8")
PY
  echo "Generated default vLLM-SR config at $CONFIG_PATH"
fi

if ! command -v vllm-sr >/dev/null 2>&1; then
  echo "Error: vllm-sr command not found; install the vLLM Semantic Router runtime before starting Wendao in vllm-sr mode." >&2
  exit 1
fi

export WENDAO_VLLM_SR_BASE_URL="$BASE_URL"
exec vllm-sr serve --config "$CONFIG_PATH" \
  > >(tee -a "$LOG_DIR/vllm-sr.stdout.log") \
  2> >(tee -a "$LOG_DIR/vllm-sr.stderr.log" >&2)
