#!/usr/bin/env bash
set -euo pipefail

uv run pytest packages/python/foundation/tests/ packages/python/core/tests/ -q --tb=short

uv run pytest \
  scripts/test_active_cargo_subprocess_env_imports.py \
  scripts/test_render_wendao_ppr_rollout_status.py \
  scripts/test_wendao_ppr_rollout_ci.py \
  scripts/test_gate_wendao_ppr_ci.py \
  scripts/test_render_wendao_gate_status_summary.py \
  scripts/test_render_wendao_gate_line.py \
  scripts/test_render_wendao_gateway_perf_summary.py \
  scripts/test_render_wendao_rollout_gate_line.py \
  scripts/test_llm_provider_smoke.py \
  scripts/test_no_inline_python_exec_patterns.py \
  scripts/test_run_real_metal_test.py \
  scripts/runtime/test_epoch_millis.py \
  scripts/runtime/test_config_resolver_compat.py \
  scripts/runtime/test_resolve_tool_port_from_settings.py \
  scripts/runtime/test_check_tool_health.py \
  scripts/runtime/test_generate_secret_token.py \
  scripts/runtime/test_read_telegram_setting.py \
  scripts/runtime/test_extract_ngrok_public_url.py \
  scripts/rust/test_resolve_libpython_path.py \
  packages/ncl/scripts/test_print_skill_summary.py \
  -q --tb=short
