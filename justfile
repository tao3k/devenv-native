# Justfile for xiuxian-artisan-workshop project
# https://github.com/casey/just
#
# Design principles:
# - Interactive commands for humans (e.g., `just commit`)
# - Agent-friendly commands with `agent-*` prefix (e.g., `just agent-commit "feat" "cli" "message"`)
# - SRE health checks with JSON output for machine parsing
# - Group annotations for clean `just --list` output
# ==============================================================================
# Global Settings
# ==============================================================================

set dotenv-load := true
set shell := ["bash", "-uc"]
set positional-arguments := true

# Enable JSON output mode via environment variable

json_output := if env_var_or_default("JUST_JSON", "false") == "true" { "true" } else { "false" }
xiuxian_wendao_gate_limit := env_var_or_default("XIUXIAN_WENDAO_GATE_LIMIT", "10")
xiuxian_wendao_gate_min_top3_rate := env_var_or_default("XIUXIAN_WENDAO_GATE_MIN_TOP3_RATE", "1.0")
xiuxian_wendao_gate_stem := env_var_or_default("XIUXIAN_WENDAO_GATE_STEM", "README")
xiuxian_wendao_gate_runs := env_var_or_default("XIUXIAN_WENDAO_GATE_RUNS", "5")
xiuxian_wendao_gate_warm_runs := env_var_or_default("XIUXIAN_WENDAO_GATE_WARM_RUNS", "1")
xiuxian_wendao_gate_profile := env_var_or_default("XIUXIAN_WENDAO_GATE_PROFILE", "debug")
xiuxian_wendao_gate_build_mode := env_var_or_default("XIUXIAN_WENDAO_GATE_BUILD_MODE", "no-build")
xiuxian_wendao_gate_subgraph_mode := env_var_or_default("XIUXIAN_WENDAO_GATE_SUBGRAPH_MODE", "auto")
xiuxian_wendao_gate_max_p95_ms := env_var_or_default("XIUXIAN_WENDAO_GATE_MAX_P95_MS", "1500")
xiuxian_wendao_gate_max_avg_ms := env_var_or_default("XIUXIAN_WENDAO_GATE_MAX_AVG_MS", "1200")
xiuxian_wendao_gate_expect_subgraph_count_min := env_var_or_default("XIUXIAN_WENDAO_GATE_EXPECT_SUBGRAPH_COUNT_MIN", "1")
xiuxian_wendao_gate_output_mode := env_var_or_default("XIUXIAN_WENDAO_GATE_OUTPUT_MODE", "text")
xiuxian_wendao_gate_report_dir := env_var_or_default("XIUXIAN_WENDAO_GATE_REPORT_DIR", ".run/reports/wendao-ppr-gate")
xiuxian_wendao_gate_query_prefix := env_var_or_default("XIUXIAN_WENDAO_GATE_QUERY_PREFIX", "")
xiuxian_wendao_gate_summary_strict_green := env_var_or_default("XIUXIAN_WENDAO_GATE_SUMMARY_STRICT_GREEN", "0")
xiuxian_wendao_mixed_canary_min_top3_rate := env_var_or_default("XIUXIAN_WENDAO_MIXED_CANARY_MIN_TOP3_RATE", "0.9")
xiuxian_wendao_mixed_canary_query_prefix := env_var_or_default("XIUXIAN_WENDAO_MIXED_CANARY_QUERY_PREFIX", "scope:mixed ")
xiuxian_wendao_mixed_canary_report_dir := env_var_or_default("XIUXIAN_WENDAO_MIXED_CANARY_REPORT_DIR", ".run/reports/wendao-ppr-mixed-canary")
xiuxian_wendao_rollout_required_runs := env_var_or_default("XIUXIAN_WENDAO_ROLLOUT_REQUIRED_RUNS", "7")
xiuxian_wendao_rollout_min_mixed_top3_rate := env_var_or_default("XIUXIAN_WENDAO_ROLLOUT_MIN_MIXED_TOP3_RATE", "0.9")
xiuxian_wendao_rollout_strict_ready := env_var_or_default("XIUXIAN_WENDAO_ROLLOUT_STRICT_READY", "0")
xiuxian_wendao_rollout_fetch_remote_status := env_var_or_default("XIUXIAN_WENDAO_ROLLOUT_FETCH_REMOTE_STATUS", "0")
xiuxian_wendao_rollout_remote_workflow_file := env_var_or_default("XIUXIAN_WENDAO_ROLLOUT_REMOTE_WORKFLOW_FILE", "ci.yaml")
xiuxian_wendao_rollout_remote_artifact_name := env_var_or_default("XIUXIAN_WENDAO_ROLLOUT_REMOTE_ARTIFACT_NAME", "")
xiuxian_wendao_rollout_remote_run_status := env_var_or_default("XIUXIAN_WENDAO_ROLLOUT_REMOTE_RUN_STATUS", "completed")
xiuxian_wendao_runner_os := env_var_or_default("RUNNER_OS", "local")
xiuxian_wendao_gateway_formal_filter := "test(performance::gateway_search::repo_module_search_perf_gate_reports_warm_cache_latency_formal_gate) | test(performance::gateway_search::repo_symbol_search_perf_gate_reports_warm_cache_latency_formal_gate) | test(performance::gateway_search::repo_example_search_perf_gate_reports_warm_cache_latency_formal_gate) | test(performance::gateway_search::repo_projected_page_search_perf_gate_reports_warm_cache_latency_formal_gate) | test(performance::gateway_search::studio_code_search_perf_gate_reports_warm_cache_latency_formal_gate) | test(performance::gateway_search::search_index_status_perf_gate_reports_query_telemetry_summary_formal_gate)"
xiuxian_wendao_gateway_perf_report_dir := env_var_or_default("XIUXIAN_WENDAO_GATEWAY_PERF_REPORT_DIR", ".run/reports/xiuxian-wendao/perf-gateway")
xiuxian_wendao_gateway_real_workspace_perf_report_dir := env_var_or_default("XIUXIAN_WENDAO_GATEWAY_REAL_WORKSPACE_PERF_REPORT_DIR", ".run/reports/xiuxian-wendao/perf-gateway-real-workspace")

# ==============================================================================
# Core Commands
# ==============================================================================

# Fetch Dots OCR vision weights and prune legacy model caches by default.
fetch-vision-models:
    uv run --group dev python scripts/fetch_vision_models.py --prune-legacy

# Backward-compatible alias
fetch-vision: fetch-vision-models

# Run OCR timeout/busy recovery simulation without starting channel runtimes.
test-ocr-recovery:
    bash scripts/channel/simulate_ocr_timeout_recovery.sh

# Run single-image OCR probe without starting webhook/channel runtimes.

# Usage: just probe-ocr-image /absolute/or/relative/path/to/image.png
probe-ocr-image image_path:
    bash scripts/channel/probe_single_image_ocr.sh "{{ image_path }}"

# Run single-image OCR probe with real-time guard (memory/process spike kill).
# Usage: just probe-ocr-image-guarded /absolute/or/relative/path/to/image.png
# Optional env overrides:
#   OCR_GUARD_LABEL (default ocr-probe)
#   OCR_GUARD_MAX_RSS_GB (default 10)
#   OCR_GUARD_MAX_GROWTH_GB_PER_MIN (default 0, disabled)
#   OCR_GUARD_GROWTH_WINDOW_SEC (default 20)
#   OCR_GUARD_GROWTH_WARMUP_SEC (default 5)
#   OCR_GUARD_MAX_PIDS (default 0, disabled)
#   OCR_GUARD_SINGLETON_SUBSTRINGS (default "", comma-separated)
#   OCR_GUARD_KILL_SUBSTRINGS (default "", comma-separated)
#   OCR_GUARD_PROCESS_SPIKE_RULES (default "exe=ld:2:3.5,exe=rustc:14:12,exe_prefix=llm-:1:10", comma-separated; each rule: substring:max_count:max_total_rss_gb)
#   OCR_GUARD_POLL_MS (default 500)
#   OCR_GUARD_GRACE_MS (default 1500)
#   OCR_GUARD_LOG_EVERY (default 4)

# OCR_GUARD_TRUNCATE_SAMPLES (default 1)
probe-ocr-image-guarded image_path:
    #!/usr/bin/env bash
    set -euo pipefail

    mkdir -p ".run/logs" ".run/reports/guarded-nextest"

    guard_args=(
      --label "${OCR_GUARD_LABEL:-ocr-probe}"
      --max-rss-gb "${OCR_GUARD_MAX_RSS_GB:-10}"
      --max-growth-gb-per-min "${OCR_GUARD_MAX_GROWTH_GB_PER_MIN:-0}"
      --growth-window-sec "${OCR_GUARD_GROWTH_WINDOW_SEC:-20}"
      --growth-warmup-sec "${OCR_GUARD_GROWTH_WARMUP_SEC:-5}"
      --max-pids "${OCR_GUARD_MAX_PIDS:-0}"
      --poll-ms "${OCR_GUARD_POLL_MS:-500}"
      --grace-ms "${OCR_GUARD_GRACE_MS:-1500}"
      --log-every "${OCR_GUARD_LOG_EVERY:-4}"
      --log-file "${OCR_GUARD_LOG_FILE:-.run/logs/guarded-nextest.log}"
      --report-json "${OCR_GUARD_REPORT_JSON:-.run/reports/guarded-nextest/latest.json}"
      --samples-jsonl "${OCR_GUARD_SAMPLES_JSONL:-.run/reports/guarded-nextest/samples.jsonl}"
      --history-jsonl "${OCR_GUARD_HISTORY_JSONL:-.run/reports/guarded-nextest/history.jsonl}"
    )

    if [ "${OCR_GUARD_TRUNCATE_SAMPLES:-1}" = "1" ]; then
      guard_args+=(--truncate-samples)
    fi

    add_csv_flags() {
      local csv_value="$1"
      local flag_name="$2"
      local item=""
      IFS=',' read -r -a raw_items <<< "${csv_value}"
      for item in "${raw_items[@]}"; do
        item="${item#"${item%%[![:space:]]*}"}"
        item="${item%"${item##*[![:space:]]}"}"
        if [ -n "${item}" ]; then
          guard_args+=("${flag_name}" "${item}")
        fi
      done
    }

    if [ -n "${OCR_GUARD_SINGLETON_SUBSTRINGS-}" ]; then
      add_csv_flags "${OCR_GUARD_SINGLETON_SUBSTRINGS-}" "--singleton-substring"
    fi
    if [ -n "${OCR_GUARD_KILL_SUBSTRINGS-}" ]; then
      add_csv_flags "${OCR_GUARD_KILL_SUBSTRINGS-}" "--kill-substring"
    fi
    if [ -n "${OCR_GUARD_PROCESS_SPIKE_RULES:-exe=ld:2:3.5,exe=rustc:14:12,exe_prefix=llm-:1:10}" ]; then
      add_csv_flags "${OCR_GUARD_PROCESS_SPIKE_RULES:-exe=ld:2:3.5,exe=rustc:14:12,exe_prefix=llm-:1:10}" "--process-spike-rule"
    fi

    UV_CACHE_DIR="${UV_CACHE_DIR:-.cache/uv}" \
      uv run python scripts/guarded_nextest.py \
      "${guard_args[@]}" \
      -- bash scripts/channel/probe_single_image_ocr.sh "{{ image_path }}"

# Run real OCR smoke against local Dots model with guarded memory limits.
# Default device:
#   - macOS: `XIUXIAN_VISION_DEVICE=metal`
#   - Linux: `XIUXIAN_VISION_DEVICE=cuda`
#   - others: `XIUXIAN_VISION_DEVICE=cpu`
# Usage:
#   just test-ocr-real-smoke
#   just test-ocr-real-smoke ".data/models/dots-ocr"
# Optional env:
#   NEXTTEST_GUARD_MAX_RSS_GB (default 20)
#   NEXTTEST_GUARD_MAX_GROWTH_GB_PER_MIN (default 0)
#   XIUXIAN_VISION_OCR_MAX_NEW_TOKENS (default 1024)

# XIUXIAN_VISION_MAX_TILES (default 12)
test-ocr-real-smoke model_root="":
    #!/usr/bin/env bash
    set -euo pipefail

    resolved_model_root="{{ model_root }}"
    if [ -z "${resolved_model_root}" ]; then
      if [ -d ".data/models/dots-ocr" ]; then
        resolved_model_root=".data/models/dots-ocr"
      elif [ -n "${PRJ_DATA_HOME:-}" ] && [ -d "${PRJ_DATA_HOME}/models/dots-ocr" ]; then
        resolved_model_root="${PRJ_DATA_HOME}/models/dots-ocr"
      else
        echo "Error: dots-ocr model directory not found. Run: just fetch-vision-models" >&2
        exit 1
      fi
    fi

    default_device="cpu"
    case "$(uname -s)" in
      Darwin)
        default_device="metal"
        ;;
      Linux)
        default_device="cuda"
        ;;
    esac

    export XIUXIAN_VISION_MODEL_KIND="${XIUXIAN_VISION_MODEL_KIND:-dots}"
    export XIUXIAN_VISION_MODEL_PATH="${XIUXIAN_VISION_MODEL_PATH:-${resolved_model_root}}"
    export XIUXIAN_VISION_DEVICE="${XIUXIAN_VISION_DEVICE:-${default_device}}"
    export XIUXIAN_VISION_REQUIRE_QUANTIZED="${XIUXIAN_VISION_REQUIRE_QUANTIZED:-0}"
    export XIUXIAN_VISION_OCR_MAX_NEW_TOKENS="${XIUXIAN_VISION_OCR_MAX_NEW_TOKENS:-1024}"
    export XIUXIAN_VISION_MAX_TILES="${XIUXIAN_VISION_MAX_TILES:-12}"

    echo "[ocr-real-smoke] model_root=${XIUXIAN_VISION_MODEL_PATH}"
    echo "[ocr-real-smoke] device=${XIUXIAN_VISION_DEVICE}"

    NEXTTEST_GUARD_LABEL="${NEXTTEST_GUARD_LABEL:-ocr-real-smoke}" \
    NEXTTEST_GUARD_MAX_RSS_GB="${NEXTTEST_GUARD_MAX_RSS_GB:-20}" \
    NEXTTEST_GUARD_MAX_GROWTH_GB_PER_MIN="${NEXTTEST_GUARD_MAX_GROWTH_GB_PER_MIN:-0}" \
    NEXTTEST_GUARD_PROCESS_SPIKE_RULES="${NEXTTEST_GUARD_PROCESS_SPIKE_RULES:-}" \
      just nextest-guarded \
      -p xiuxian-llm \
      --test llm_vision_deepseek_smoke \
      deepseek_smoke_runs_real_inference_from_local_model_cache

# Run cargo nextest with RSS guard; kill test tree on memory anomaly.
# Usage:
#   NEXTTEST_GUARD_MAX_RSS_GB=6 just nextest-guarded -p xiuxian-daochang --test llm -E 'test(litellm_ocr_)'
# Optional env overrides:
#   NEXTTEST_GUARD_MAX_RSS_GB (default 6)
#   NEXTTEST_GUARD_MAX_GROWTH_GB_PER_MIN (default 0, disabled)
#   NEXTTEST_GUARD_GROWTH_WINDOW_SEC (default 20)
#   NEXTTEST_GUARD_GROWTH_WARMUP_SEC (default 5)
#   NEXTTEST_GUARD_MAX_PIDS (default 0, disabled)
#   NEXTTEST_GUARD_SINGLETON_SUBSTRINGS (default "", comma-separated)
#   NEXTTEST_GUARD_KILL_SUBSTRINGS (default "", comma-separated)
#   NEXTTEST_GUARD_PROCESS_SPIKE_RULES (default "", comma-separated; each rule: substring:max_count:max_total_rss_gb)
#   NEXTTEST_GUARD_POLL_MS (default 500)
#   NEXTTEST_GUARD_GRACE_MS (default 1500)
#   NEXTTEST_GUARD_LOG_EVERY (default 4)
#   NEXTTEST_GUARD_TRUNCATE_SAMPLES (default 1)
#   NEXTTEST_GUARD_LABEL (default nextest)
#   NEXTTEST_GUARD_LOG_FILE (default .run/logs/guarded-nextest.log)
#   NEXTTEST_GUARD_REPORT_JSON (default .run/reports/guarded-nextest/latest.json)
#   NEXTTEST_GUARD_SAMPLES_JSONL (default .run/reports/guarded-nextest/samples.jsonl)

# NEXTTEST_GUARD_HISTORY_JSONL (default .run/reports/guarded-nextest/history.jsonl)
nextest-guarded +nextest_args:
    #!/usr/bin/env bash
    set -euo pipefail

    mkdir -p ".run/logs" ".run/reports/guarded-nextest"

    guard_args=(
      --label "${NEXTTEST_GUARD_LABEL:-nextest}"
      --max-rss-gb "${NEXTTEST_GUARD_MAX_RSS_GB:-6}"
      --max-growth-gb-per-min "${NEXTTEST_GUARD_MAX_GROWTH_GB_PER_MIN:-0}"
      --growth-window-sec "${NEXTTEST_GUARD_GROWTH_WINDOW_SEC:-20}"
      --growth-warmup-sec "${NEXTTEST_GUARD_GROWTH_WARMUP_SEC:-5}"
      --max-pids "${NEXTTEST_GUARD_MAX_PIDS:-0}"
      --poll-ms "${NEXTTEST_GUARD_POLL_MS:-500}"
      --grace-ms "${NEXTTEST_GUARD_GRACE_MS:-1500}"
      --log-every "${NEXTTEST_GUARD_LOG_EVERY:-4}"
      --log-file "${NEXTTEST_GUARD_LOG_FILE:-.run/logs/guarded-nextest.log}"
      --report-json "${NEXTTEST_GUARD_REPORT_JSON:-.run/reports/guarded-nextest/latest.json}"
      --samples-jsonl "${NEXTTEST_GUARD_SAMPLES_JSONL:-.run/reports/guarded-nextest/samples.jsonl}"
      --history-jsonl "${NEXTTEST_GUARD_HISTORY_JSONL:-.run/reports/guarded-nextest/history.jsonl}"
    )

    if [ "${NEXTTEST_GUARD_TRUNCATE_SAMPLES:-1}" = "1" ]; then
      guard_args+=(--truncate-samples)
    fi

    add_csv_flags() {
      local csv_value="$1"
      local flag_name="$2"
      local item=""
      IFS=',' read -r -a raw_items <<< "${csv_value}"
      for item in "${raw_items[@]}"; do
        item="${item#"${item%%[![:space:]]*}"}"
        item="${item%"${item##*[![:space:]]}"}"
        if [ -n "${item}" ]; then
          guard_args+=("${flag_name}" "${item}")
        fi
      done
    }

    if [ -n "${NEXTTEST_GUARD_SINGLETON_SUBSTRINGS:-}" ]; then
      add_csv_flags "${NEXTTEST_GUARD_SINGLETON_SUBSTRINGS}" "--singleton-substring"
    fi
    if [ -n "${NEXTTEST_GUARD_KILL_SUBSTRINGS:-}" ]; then
      add_csv_flags "${NEXTTEST_GUARD_KILL_SUBSTRINGS}" "--kill-substring"
    fi
    if [ -n "${NEXTTEST_GUARD_PROCESS_SPIKE_RULES:-}" ]; then
      add_csv_flags "${NEXTTEST_GUARD_PROCESS_SPIKE_RULES}" "--process-spike-rule"
    fi

    UV_CACHE_DIR="${UV_CACHE_DIR:-.cache/uv}" \
      uv run python scripts/guarded_nextest.py \
      "${guard_args[@]}" \
      -- cargo nextest run {{ nextest_args }}

# Run OCR-focused nextest lanes with OCR-friendly RSS defaults.

# Usage: just nextest-guarded-ocr -p xiuxian-daochang --test llm -E 'test(litellm_ocr_)'
nextest-guarded-ocr +nextest_args:
    NEXTTEST_GUARD_LABEL="${NEXTTEST_GUARD_LABEL-nextest-ocr}" \
    NEXTTEST_GUARD_MAX_RSS_GB="${NEXTTEST_GUARD_MAX_RSS_GB-8}" \
    NEXTTEST_GUARD_SINGLETON_SUBSTRINGS="${NEXTTEST_GUARD_SINGLETON_SUBSTRINGS-}" \
    NEXTTEST_GUARD_KILL_SUBSTRINGS="${NEXTTEST_GUARD_KILL_SUBSTRINGS-}" \
    NEXTTEST_GUARD_PROCESS_SPIKE_RULES="${NEXTTEST_GUARD_PROCESS_SPIKE_RULES-}" \
    just nextest-guarded {{ nextest_args }}

# Run nextest with AGGRESSIVE process monitoring.
# Kills everything immediately if any process exceeds limits.
# Usage: just nextest-aggressive -p xiuxian-llm test_real_metal
# Env overrides:
#   NEXTTEST_AGGR_MAX_RSS_GB (default 10)
#   NEXTTEST_AGGR_MAX_VSZ_GB (default 0, disabled - macOS VSZ is misleading)
#   NEXTTEST_AGGR_SYSTEM_MAX_RSS_GB (default 0) - system-wide RSS guard, kills ANY process exceeding this
#   NEXTTEST_AGGR_LINKER_MAX (default 2) - max linker processes
#   NEXTTEST_AGGR_LINKER_RSS_GB (default 2) - max RSS per linker
#   NEXTTEST_AGGR_POLL_MS (default 50) - polling interval (very fast)

# NEXTTEST_AGGR_WATCH_BINARY (default "") - binary name:max_rss_gb to watch (e.g. "llm_vision_deepseek_real_metal:4")
nextest-aggressive +nextest_args:
    #!/usr/bin/env bash
    set -euo pipefail

    max_rss="${NEXTTEST_AGGR_MAX_RSS_GB:-10}"
    max_vsz="${NEXTTEST_AGGR_MAX_VSZ_GB:-0}"  # Disabled by default - macOS VSZ is ~400GB per process
    system_max_rss="${NEXTTEST_AGGR_SYSTEM_MAX_RSS_GB:-0}"  # System-wide RSS guard
    linker_max="${NEXTTEST_AGGR_LINKER_MAX:-2}"
    linker_rss="${NEXTTEST_AGGR_LINKER_RSS_GB:-2}"
    poll_ms="${NEXTTEST_AGGR_POLL_MS:-50}"
    watch_binary="${NEXTTEST_AGGR_WATCH_BINARY:-}"

    mkdir -p ".run/logs" ".run/reports/guarded-nextest"

    watch_args=()
    if [ -n "${watch_binary}" ]; then
        watch_args+=(--watch-binary "${watch_binary}")
    fi

    system_rss_args=()
    if [ "${system_max_rss}" != "0" ]; then
        system_rss_args+=(--system-max-rss-gb "${system_max_rss}")
    fi

    uv run python scripts/guarded_nextest.py \
        --label "aggressive-${NEXTTEST_GUARD_LABEL:-nextest}" \
        --max-rss-gb "${max_rss}" \
        --max-vsz-gb "${max_vsz}" \
        --poll-ms "${poll_ms}" \
        --grace-ms 0 \
        --aggressive-poll-ms "${poll_ms}" \
        --aggressive-kill "exe=ld:${linker_max}:${linker_rss}" \
        "${watch_args[@]}" \
        "${system_rss_args[@]}" \
        --log-file ".run/logs/guarded-nextest-aggressive.log" \
        -- cargo nextest run {{ nextest_args }}

# Tier-1 test lane: fast logic-only Rust feedback loop.
# Excludes tests whose names contain `vision` or `ocr`.
# Example:

# just test-logic
test-logic:
    bash scripts/rust/test_logic_lane.sh

# Tier-2 test lane: vision wiring smoke lane (non-heavy, no full OCR model inference).
# This validates vision request/response contracts and runtime config logic.
# Example:

# just test-vision-smoke
test-vision-smoke:
    bash scripts/rust/test_vision_smoke_lane.sh

# Tier-3 test lane: real OCR integration against local model (release + serialized).
# Example:
#   just test-vision-heavy
#   just test-vision-heavy ".data/models/dots-ocr"
# Optional env:
#   VISION_HEAVY_TEST_THREADS (default 1)

# NEXTTEST_GUARD_MAX_RSS_GB (default 24)
test-vision-heavy model_root="":
    bash scripts/rust/test_vision_heavy_lane.sh "{{ model_root }}"

# ==============================================================================
# [AIP] Metal Hardware Limit & DSQ Compatibility Stress Tests
# ==============================================================================
# Metal stress test environment variables

metal_stress_report_dir := env_var_or_default("METAL_STRESS_REPORT_DIR", ".run/reports/metal-stress")
metal_stress_max_dimension := env_var_or_default("METAL_STRESS_MAX_DIMENSION", "2048")
metal_stress_iterations := env_var_or_default("METAL_STRESS_ITERATIONS", "5")

# Run full Metal stress test suite (macOS only).
# Usage: just test-metal-stress
# Optional env:
#   METAL_STRESS_REPORT_DIR - Directory for test reports (default: .run/reports/metal-stress)
#   METAL_STRESS_MAX_DIMENSION - Maximum image dimension to test (default: 2048)

# METAL_STRESS_ITERATIONS - Number of inference iterations for memory test (default: 5)
test-metal-stress:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "=========================================="
    echo "[AIP] Metal Hardware Limit Stress Test"
    echo "=========================================="

    mkdir -p "{{ metal_stress_report_dir }}"
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)

    # Check environment
    if [[ "$(uname)" != "Darwin" ]]; then
        echo "ERROR: This test must run on macOS for Metal testing"
        exit 1
    fi

    # Step 1: DSQ alignment matrix test
    echo "=== Step 1: DSQ Alignment Matrix Test ==="
    cargo nextest run -p xiuxian-llm --test llm_vision_deepseek_dsq_repair_unit --nocapture 2>&1 | \
        tee "{{ metal_stress_report_dir }}/dsq_alignment_${TIMESTAMP}.log"

    # Step 2: Metal smoke test with telemetry
    echo "=== Step 2: Metal Smoke Test with Telemetry ==="
    RUST_LOG=xiuxian_llm=debug \
    cargo nextest run -p xiuxian-llm --test llm_vision_deepseek_smoke --nocapture 2>&1 | \
        tee "{{ metal_stress_report_dir }}/metal_smoke_${TIMESTAMP}.log" || true

    # Step 3: Buffer boundary probe
    echo "=== Step 3: Buffer Boundary Probe ==="
    for dim in 512 768 1024 1280 1536 1792 2048; do
        echo "Testing max_dimension=$dim"
        XIUXIAN_VISION_OCR_MAX_DIMENSION=$dim \
        cargo nextest run -p xiuxian-llm --test llm_vision_deepseek_smoke --nocapture 2>&1 | \
            grep -E "(PASS|FAIL|Buffer|Metal)" | tee -a "{{ metal_stress_report_dir }}/buffer_boundary_${TIMESTAMP}.log" || true
    done

    # Step 4: Memory reclamation test
    echo "=== Step 4: Memory Reclamation Test ==="
    for i in $(seq 1 {{ metal_stress_iterations }}); do
        echo "Iteration $i/{{ metal_stress_iterations }}"
        echo "Memory before: $(vm_stat | grep -E 'free|active' | head -2)" | \
            tee -a "{{ metal_stress_report_dir }}/memory_reclaim_${TIMESTAMP}.log"
        cargo nextest run -p xiuxian-llm --test llm_vision_deepseek_smoke --nocapture 2>&1 | \
            tee -a "{{ metal_stress_report_dir }}/memory_reclaim_${TIMESTAMP}.log" || true
        echo "Memory after: $(vm_stat | grep -E 'free|active' | head -2)" | \
            tee -a "{{ metal_stress_report_dir }}/memory_reclaim_${TIMESTAMP}.log"
        sleep 2
    done

    echo "=========================================="
    echo "Stress tests completed!"
    echo "Reports: {{ metal_stress_report_dir }}"
    echo "=========================================="

# Quick DSQ alignment validation test.

# Usage: just test-dsq-alignment
test-dsq-alignment:
    cargo nextest run -p xiuxian-llm --test llm_vision_deepseek_dsq_repair_unit --nocapture

# Metal buffer boundary test with custom dimension.

# Usage: just test-metal-buffer 1024
test-metal-buffer dimension="1024":
    XIUXIAN_VISION_OCR_MAX_DIMENSION={{ dimension }} \
    cargo nextest run -p xiuxian-llm --test llm_vision_deepseek_smoke --nocapture

# Real Metal inference test with capacity check and runtime memory guard.
# 1. Uses capfox to check capacity before starting
# 2. Monitors memory during execution and kills if exceeded
# Usage:
#   just test-real-metal              # Metal GPU inference (10GB limit)
#   just test-real-metal --cpu        # Force CPU (12GB limit, no GPU)
#   just test-real-metal --max-rss=8  # Custom memory limit
# Exit codes:
#   0   - Test passed
#   75  - No capacity (capfox denied)

# 137 - Killed by memory guard
test-real-metal *args:
    #!/usr/bin/env bash
    set -euo pipefail

    # Ensure test binary exists
    if ! ls target/debug/deps/llm_vision_deepseek_real_metal-* 2>/dev/null | head -1 | xargs -I{} test -x {}; then
        echo "Building test binary..."
        cargo build -p xiuxian-llm --tests
    fi

    # Run with capfox check + memory guard
    uv run python scripts/run_real_metal_test.py {{ args }}

# Unified local-model safe interface (embedding warmup + vision).
# Profiles:
#   - safe: embedding warmup + vision smoke (default)
#   - full: embedding warmup + real OCR heavy lane
#   - embed-only / vision-only / vision-heavy-only
# Usage:
#   just test-local-model-safe

# just test-local-model-safe "full" ".data/models/dots-ocr"
test-local-model-safe profile="safe" model_root="":
    bash scripts/rust/test_local_models_safe.sh "{{ profile }}" "{{ model_root }}"

default:
    @just --list

# Test LLM Proxy multiple providers
test-llm-proxy:
    uv run python scripts/test_llm_proxy.py

# Run custom-base multimodal regression without starting webhook/channel runtimes.

# Verifies image URL -> base64 inlining path used by Anthropic custom-base fallbacks.
test-llm-custom-base-image:
    cargo nextest run -p xiuxian-llm --features provider-litellm --test llm_providers -E 'test(inline_openai_compatible_image_urls_converts_and_caches_image_urls)'

# Run OpenAI `/responses` tool-schema regression for strict provider gateways.

# Guards against: "object schema missing properties" (HTTP 400 invalid_function_parameters).
test-llm-responses-schema:
    cargo nextest run -p xiuxian-llm --features provider-litellm --test llm_openai_responses_payload -E 'test(responses_payload_injects_empty_properties_for_object_schema_without_properties)'

# Run OpenAI `/responses` tool-call chain regressions for strict provider gateways.

# Guards against orphaned `function_call_output` payloads and missing assistant call items.
test-llm-responses-tool-chain:
    cargo nextest run -p xiuxian-llm --features provider-litellm --test llm_openai_responses_payload -E 'test(responses_payload_serializes_assistant_tool_calls_before_tool_outputs) or test(responses_payload_skips_tool_output_without_call_id)'

# Run Anthropic `messages` tool-use/result chain regression.

# Guards against missing `tool_result.tool_use_id` mapping for tool responses.
test-llm-anthropic-tool-chain:
    cargo nextest run -p xiuxian-llm --features provider-litellm --test llm_providers -E 'test(build_anthropic_messages_body_maps_tool_call_chain_to_tool_use_and_tool_result)'

# Rust LLM smoke bundle (protocol-critical lanes only).

# Includes image custom-base path plus OpenAI/Anthropic tool-call protocol contracts.
test-rust-llm-smoke:
    just test-llm-custom-base-image
    just test-llm-responses-schema
    just test-llm-responses-tool-chain
    just test-llm-anthropic-tool-chain

# Live provider smoke test from xiuxian.toml (real LLM network calls).
# Example:
#   just llm-provider-smoke
#   just llm-provider-smoke "$PRJ_CONFIG_HOME/xiuxian-artisan-workshop/xiuxian.toml" "all" "" "60" "" "" ""
#   just llm-provider-smoke "$PRJ_CONFIG_HOME/xiuxian-artisan-workshop/xiuxian.toml" "all" "" "90" "https://example.com/a.png" "responses" "flower|bloom|hibiscus,red|hibiscus"
# image:
#   - empty string => text-only test
#   - file path / URL / data URI => text + image test
# wire_api:
#   - empty string => use xiuxian.toml provider value
#   - chat_completions | responses => force override
# image_contains:

# - comma-separated semantic expectation groups; every group must match, use | for alternatives
llm-provider-smoke config_path="" provider="all" model_override="" timeout_secs="60" image="" wire_api="" image_contains="":
    #!/usr/bin/env bash
    set -euo pipefail
    resolved_config_path="{{ config_path }}"
    if [ -z "$resolved_config_path" ]; then
      resolved_config_path="${PRJ_CONFIG_HOME:-.config}/xiuxian-artisan-workshop/xiuxian.toml"
    fi
    uv run python scripts/llm_provider_smoke.py \
      --config-path "$resolved_config_path" \
      --provider "{{ provider }}" \
      --model-override "{{ model_override }}" \
      --timeout-secs "{{ timeout_secs }}" \
      --image "{{ image }}" \
      --wire-api "{{ wire_api }}" \
      --image-contains "{{ image_contains }}"

# Live smoke test for all configured providers (text only).
llm-provider-smoke-all config_path="" model_override="" timeout_secs="60" wire_api="":
    just llm-provider-smoke "{{ config_path }}" "all" "{{ model_override }}" "{{ timeout_secs }}" "" "{{ wire_api }}" ""

# Live smoke test for all configured providers (text + image).

# image must be a file path, URL, or data URI.
llm-provider-smoke-image config_path="" model_override="" timeout_secs="90" image="" wire_api="" image_contains="":
    #!/usr/bin/env bash
    set -euo pipefail
    resolved_config_path="{{ config_path }}"
    if [ -z "$resolved_config_path" ]; then
      resolved_config_path="${PRJ_CONFIG_HOME:-.config}/xiuxian-artisan-workshop/xiuxian.toml"
    fi
    if [ -z "{{ image }}" ]; then
      echo "Error: image argument is required for llm-provider-smoke-image" >&2
      exit 2
    fi
    uv run python scripts/llm_provider_smoke.py \
      --config-path "$resolved_config_path" \
      --provider "all" \
      --model-override "{{ model_override }}" \
      --timeout-secs "{{ timeout_secs }}" \
      --image "{{ image }}" \
      --wire-api "{{ wire_api }}" \
      --image-contains "{{ image_contains }}"

# Canonical semantic image smoke using a known flower JPEG.

# Fails providers that return unrelated image descriptions.
llm-provider-smoke-image-semantic config_path="" provider="all" model_override="" timeout_secs="90" image="https://upload.wikimedia.org/wikipedia/commons/3/3f/JPEG_example_flower.jpg" wire_api="" image_contains="flower|bloom|hibiscus,red|hibiscus":
    just llm-provider-smoke "{{ config_path }}" "{{ provider }}" "{{ model_override }}" "{{ timeout_secs }}" "{{ image }}" "{{ wire_api }}" "{{ image_contains }}"

# ==============================================================================
# AGENT INTERFACE (Non-interactive, argument-based)
# Designed for AI agents - accepts parameters, no interactive prompts
# ==============================================================================
# Non-interactive commit for agents
# Usage: just agent-commit

# Reads commit message from token file generated by the smart_commit tool workflow
agent-commit:
    #!/usr/bin/env bash
    set -euo pipefail

    # Token file path (must match commit.py)
    TOKEN_FILE="/tmp/.xiuxian_commit_token"

    # Check if token exists and is valid (only from smart_commit workflow)
    if [ ! -f "$TOKEN_FILE" ]; then
        echo "Error: No authorization token found." >&2
        echo "" >&2
        echo "To commit, you must:" >&2
        echo "1. Use the smart_commit tool first: @xiuxian-orchestrator smart_commit(context='...')" >&2
        echo "2. Then run: just agent-commit" >&2
        exit 1
    fi

    # Read token content: format is session_id:token:timestamp:message
    TOKEN_CONTENT=$(cat "$TOKEN_FILE")
    SESSION_ID=$(echo "$TOKEN_CONTENT" | cut -d':' -f1)
    TOKEN=$(echo "$TOKEN_CONTENT" | cut -d':' -f2)
    TIMESTAMP=$(echo "$TOKEN_CONTENT" | cut -d':' -f3)
    # Message may contain colons, so use the 4th field to end
    MSG=$(echo "$TOKEN_CONTENT" | cut -d':' -f4-)

    # Validate token is not empty
    if [ -z "$TOKEN" ]; then
        echo "Error: Invalid authorization token (empty)." >&2
        rm -f "$TOKEN_FILE"
        exit 1
    fi

    # Check token expiration (5 minutes)
    TOKEN_EPOCH=$(date -d "$TIMESTAMP" +%s 2>/dev/null || date +%s)
    NOW_EPOCH=$(date +%s)
    ELAPSED=$((NOW_EPOCH - TOKEN_EPOCH))
    if [ $ELAPSED -gt 300 ]; then
        echo "Error: Authorization token has expired." >&2
        rm -f "$TOKEN_FILE"
        exit 1
    fi

    echo "Commit message: $MSG"
    echo ""

    # Run lefthook first to apply all formatting fixes (prevents unstaged files after commit)
    echo "Running pre-commit hooks..."
    lefthook run pre-commit --all-files --no-tty

    # Re-stage all files after formatting (prettier modifies files but doesn't auto-stage)
    echo "Re-staging all modified files..."
    git add -A

    # Run tests before commit
    echo "Running tests..."
    devenv test

    # Stage again after tests (in case tests modify files)
    echo "Staging all files..."
    git add -A

    # Commit with staged changes
    git commit -m "$MSG"

    # Consume the token (invalidate it)
    rm -f "$TOKEN_FILE"

    echo ""
    echo "Committed: $MSG"

# Agent-friendly validate (non-interactive)
agent-validate:
    @echo "Running validation..." && lefthook run pre-commit --all-files --no-tty && devenv test

# Agent-friendly validate with git status output (safe - no commit)
# Usage: just agent-test-status

# This command runs tests and outputs git status for agent to read
agent-test-status:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== TEST_START ==="
    devenv test
    TEST_RESULT=$?
    echo "=== TEST_END ==="
    echo "=== GIT_STATUS_START ==="
    git status --short
    echo "=== GIT_STATUS_END ==="
    echo "=== GIT_LOG_START ==="
    git log --oneline -3
    echo "=== GIT_LOG_END ==="
    if [ $TEST_RESULT -eq 0 ]; then
        echo "Tests passed"
    else
        echo "Tests failed"
    fi
    exit $TEST_RESULT

# Agent-friendly format (apply fixes)
agent-fmt:
    @echo "Applying formatting fixes..." && lefthook run pre-commit --all-files --no-tty

# Agent-friendly version bump
agent-bump type="auto":
    #!/usr/bin/env bash
    set -euo pipefail
    BUMP_TYPE="{{ type }}"
    if [ "$BUMP_TYPE" = "auto" ]; then
        cog bump --auto
    else
        cog bump --$BUMP_TYPE
    fi

# Live LLM pre-release gate (protocol smoke + real provider smoke).

# Default scope is all configured providers and canonical semantic flower image.
llm-provider-release-gate config_path="" provider="all" model_override="" text_timeout_secs="60" image_timeout_secs="90" wire_api="" image="https://upload.wikimedia.org/wikipedia/commons/3/3f/JPEG_example_flower.jpg" image_contains="flower|bloom|hibiscus,red|hibiscus":
    just test-rust-llm-smoke
    just llm-provider-smoke "{{ config_path }}" "{{ provider }}" "{{ model_override }}" "{{ text_timeout_secs }}" "" "{{ wire_api }}" ""
    just llm-provider-smoke-image-semantic "{{ config_path }}" "{{ provider }}" "{{ model_override }}" "{{ image_timeout_secs }}" "{{ image }}" "{{ wire_api }}" "{{ image_contains }}"

# Agent-friendly release validation (local validation + live LLM gate)
agent-validate-release:
    @echo "Running release validation..." && just agent-validate && just llm-provider-release-gate

# Agent-friendly release publish
agent-publish-release version="latest":
    #!/usr/bin/env bash
    set -euo pipefail
    VERSION="{{ version }}"
    if [ "$VERSION" = "latest" ]; then
        VERSION=$(git describe --tags --abbrev=0)
    fi
    NOTES=$(mktemp)
    just release-notes "$VERSION" > "$NOTES"
    gh release create "$VERSION" --title "Release $VERSION" --notes-file "$NOTES" --verify-tag
    rm -f "$NOTES"

# Agent-friendly complete release workflow
agent-release type="auto" version="latest":
    #!/usr/bin/env bash
    set -euo pipefail
    just agent-validate-release
    just agent-bump {{ type }}
    just agent-publish-release {{ version }}

# ==============================================================================
# 🤖 AGENT WORKFLOW AUTOMATION
# ==============================================================================

# Generate high-density context dump for agent startup
[no-exit-message]
agent-context:
    @echo "<project_context_dump>"
    @echo "=== 📋 CURRENT MISSION (Backlog Top 20) ==="
    @head -n 20 Backlog.md 2>/dev/null || echo "⚠️ No Backlog.md found. Create one to drive the agent."
    @echo ""
    @echo "=== 🚦 GIT STATUS ==="
    @git status --short --branch
    @echo ""
    @echo "=== ⚙️ RULES (cog.toml scopes) ==="
    @grep -A 20 "scopes =" cog.toml 2>/dev/null | head -15 || echo "cog.toml not found"
    @echo ""
    @echo "=== 📝 RECENT COMMITS ==="
    @git log --oneline -5
    @echo ""
    @echo "=== ✍️ WRITING STYLE (agent/writing-style/) ==="
    @ls -1 agent/writing-style/*.md 2>/dev/null | xargs -I {} basename {} .md | sed 's/^/  - /' || echo "  No style guides found"
    @echo "  Hint: Use 'writer.polish_text' to enforce these rules"
    @echo ""
    @echo "</project_context_dump>"
    @echo ""
    @echo "💡 INSTRUCTION: Read the context above. Identify the active task from Backlog.md and check if it aligns with git status. Await user command."

# ==============================================================================
# 🧠 COGNITION & SPECS
# ==============================================================================
# Focus Mode: Load specific Spec and prepare for development

# Usage: just agent-focus assets/specs/feature_name.md
agent-focus spec_path:
    @echo "🚀 Focusing Agent on Spec: {{ spec_path }}..."
    @echo ""
    @echo "=== 🎯 FOCUS TARGET: {{ spec_path }} ==="
    @cat {{ spec_path }}
    @echo ""
    @echo "=== 🏗️ RELATED CODE STRUCTURE ==="
    @echo "packages/python/wendao-core-lib modules:"
    @ls -1 packages/python/wendao-core-lib/src/wendao_core_lib/*.py 2>/dev/null | xargs -I {} basename {} .py | sed 's/^/  - /' || echo "  No modules found"
    @echo "agent/skills modules:"
    @ls -1 agent/skills/ 2>/dev/null | grep -v "^_" | sed 's/^/  - /' || echo "  No skills found"
    @echo ""
    @echo "=== 📋 BACKLOG ALIGNMENT ==="
    @grep -i "$(basename {{ spec_path }} .md)" Backlog.md 2>/dev/null || echo "  No matching backlog entry found"
    @echo ""
    @echo "💡 INSTRUCTION: Review the Spec above. Create a PLAN in 'SCRATCHPAD.md' before modifying any code."
    @echo "SCRATCHPAD Location: ${PRJ_CACHE_HOME:-.cache}/xiuxian-artisan-workshop/.memory/active_context/SCRATCHPAD.md"

# Quick create new Spec from template

# Usage: just spec-new "feature_name" "Feature description..."
spec-new name description:
    @echo "🏗️ Scaffolding Spec: {{ name }}..."
    @cp assets/specs/template.md assets/specs/{{ name }}.md
    @( \
        echo "=== 📝 TASK: DRAFT SPEC ==="; \
        echo "Target File: assets/specs/{{ name }}.md"; \
        echo "Feature Name: {{ name }}"; \
        echo "User Description: {{ description }}"; \
        echo ""; \
        echo "💡 INSTRUCTION: Read the 'Target File' template. Fill in Sections 1 (Context) and 2 (Architecture) based on the 'User Description'. Leave Section 3 (Plan) for later."; \
    ) | claude
    @echo "✅ Spec draft created at assets/specs/{{ name }}.md"

# Start Claude with automatic context injection
agent-start:
    @echo "🚀 Initializing Agent with Context..."
    @just agent-context | claude

# ==============================================================================
# HUMAN INTERFACE (Interactive commands preserved)
# Commands with user prompts for manual operations
# ==============================================================================

# Interactive commit helper (for humans - uses select/read)
[group('git')]
commit:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Interactive Conventional Commit"
    echo "================================"
    select TYPE in feat fix docs style refactor perf test build ci chore; do
        [ -n "$TYPE" ] && break
    done
    read -p "Enter scope (optional): " SCOPE
    SCOPE_STR=""
    if [ -n "$SCOPE" ]; then
        SCOPE_STR="($SCOPE)"
    fi
    read -p "Enter short description: " DESC
    read -p "Add detailed body? [y/N]: " -n 1 ADD_BODY
    echo
    BODY=""
    if [[ $ADD_BODY =~ ^[Yy]$ ]]; then
        echo "Enter body (Ctrl+D when done):"
        BODY=$(cat)
    fi
    read -p "Breaking change? [y/N]: " -n 1 BREAKING
    echo
    FOOTER=""
    if [[ $BREAKING =~ ^[Yy]$ ]]; then
        read -p "Describe breaking change: " BREAKING_DESC
        FOOTER="BREAKING CHANGE: $BREAKING_DESC"
    fi
    MSG="$TYPE$SCOPE_STR: $DESC"
    if [ -n "$BODY" ]; then
        MSG="$MSG\n\n$BODY"
    fi
    if [ -n "$FOOTER" ]; then
        MSG="$MSG\n\n$FOOTER"
    fi
    echo ""
    echo "Preview:"
    echo -e "$MSG"
    echo ""
    read -p "Commit? [Y/n]: " -n 1 CONFIRM
    echo
    if [[ ! $CONFIRM =~ ^[Nn]$ ]]; then
        git commit -m "$(echo -e "$MSG")"
        echo "Committed!"
    else
        echo "Cancelled"
        exit 1
    fi

# ==============================================================================
# SETUP & VALIDATION
# ==============================================================================

[group('setup')]
setup:
    @echo "🚀 Setting up development environment..."
    @echo ""
    @echo "Step 1/3: Checking secrets configuration..."
    -@secretspec check --profile development 2>/dev/null || true
    @if ! secretspec check --profile development >/dev/null 2>&1; then \
        echo "⚠️  Secrets not configured."; \
        echo "   Checking if claude module needs to be disabled..."; \
        if grep -q "^    nixosModules.claude$" devenv.nix; then \
            echo "   Disabling claude module for initial setup..."; \
            sed -i '' 's/^    nixosModules.claude$/    # nixosModules.claude  # Disabled: configure secrets first/g' devenv.nix; \
            echo "   ✅ claude module disabled."; \
        else \
            echo "   ✅ claude module already disabled."; \
        fi; \
        echo ""; \
        echo "Step 2/3: Activating direnv (without claude module)..."; \
        direnv allow 2>/dev/null || true; \
        echo ""; \
        echo "Step 3/3: Environment ready (limited mode)."; \
        echo ""; \
        echo "📝 Next steps:"; \
        echo "   1. Configure secrets: https://secretspec.dev/concepts/providers/"; \
        echo "   2. Verify: just secrets-check"; \
        echo "   3. Re-run: just setup"; \
        echo ""; \
        echo "Run 'just' to see available commands."; \
    else \
        echo "✅ Secrets OK!"; \
        echo ""; \
        echo "Step 2/3: Restoring claude module if needed..."; \
        if grep -q "^    # nixosModules.claude  # Disabled:" devenv.nix; then \
            sed -i '' 's/^    # nixosModules.claude  # Disabled:/    nixosModules.claude/g' devenv.nix; \
            echo "   ✅ claude module restored!"; \
        else \
            echo "   ✅ claude module already enabled."; \
        fi; \
        echo ""; \
        echo "Step 3/3: Activating direnv..."; \
        direnv allow; \
        echo ""; \
        echo "🎉 Environment fully ready!"; \
        echo ""; \
        echo "Run 'just' to see available commands."; \
    fi

# Import Apple Metal toolchain from local Xcode into Nix Store
[group('nix')]
nix-import-metal-toolchain:
    @echo "📦 Starting Metal toolchain import..."
    @bash scripts/nix/import-metal-toolchain.sh

# Build xiuxian-llm with Metal pre-compilation
[group('nix')]
nix-build-xiuxian-llm:
    @echo "🚀 Building xiuxian-llm..."
    nix build .#xiuxian-llm-dev

[group('validate')]
validate: check-format check-commits lint test
    @echo "All validation checks passed!"

[group('validate')]
check-format:
    @echo "Checking code formatting..."
    @lefthook run pre-commit --all-files --no-tty

[group('validate')]
lint:
    @echo "Linting files..."
    @lefthook run pre-commit --all-files --no-tty

[group('validate')]
rust-check timeout_secs="1800":
    #!/usr/bin/env bash
    set -euo pipefail
    target_dir="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}"
    timeout_secs="{{ timeout_secs }}"
    echo "Running Rust compile checks (timeout=${timeout_secs}s, CARGO_TARGET_DIR=${target_dir})..."
    CARGO_TARGET_DIR="${target_dir}" python3 scripts/rust/cargo_check_with_timeout.py "${timeout_secs}"

[group('validate')]
rust-lint-inheritance-check:
    @bash scripts/rust/check_lint_inheritance.sh

[group('validate')]
rust-test-layout:
    @bash scripts/rust/check_test_layout.sh

[group('validate')]
rust-clippy:
    @echo "Running Rust clippy across the full workspace (warnings denied)..."
    @CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}" scripts/rust/cargo_exec.sh clippy --workspace -- -D warnings

[group('validate')]
rust-nextest:
    @echo "Running Rust tests via cargo-nextest..."
    @if ! command -v cargo-nextest >/dev/null 2>&1; then \
        echo "cargo-nextest is required but not installed."; \
        echo "Install with: nix profile add nixpkgs#cargo-nextest"; \
        exit 1; \
    fi
    @CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}" scripts/rust/cargo_exec.sh nextest run --workspace --exclude xiuxian-core-rs --no-fail-fast

[group('validate')]
rust-security-audit:
    @echo "Running Rust vulnerability audit (cargo-audit)..."
    @if ! command -v cargo-audit >/dev/null 2>&1; then \
        echo "cargo-audit is required but not installed."; \
        echo "Install with: nix profile add nixpkgs#cargo-audit"; \
        exit 1; \
    fi
    @bash scripts/rust/cargo_audit_gate.sh

[group('validate')]
rust-security-deny:
    @echo "Running Rust dependency policy gate (cargo-deny)..."
    @if ! command -v cargo-deny >/dev/null 2>&1; then \
        echo "cargo-deny is required but not installed."; \
        echo "Install with: nix profile add nixpkgs#cargo-deny"; \
        exit 1; \
    fi
    @scripts/rust/cargo_exec.sh deny check advisories bans sources

[group('validate')]
rust-security-gate: rust-security-audit rust-security-deny
    @echo "Rust dependency security gates passed (cargo-audit + cargo-deny)."

[group('validate')]
rust-contract-semver-core baseline_rev="":
    @echo "Running xiuxian-wendao-core semver gate (cargo-semver-checks)..."
    @if ! command -v cargo-semver-checks >/dev/null 2>&1; then \
        echo "cargo-semver-checks is required but not installed."; \
        echo "Install with: nix profile add nixpkgs#cargo-semver-checks"; \
        exit 1; \
    fi
    @WENDAO_CORE_SEMVER_BASELINE_REV="{{ baseline_rev }}" bash scripts/rust/wendao_contract_dependency_governance.sh semver-core

[group('validate')]
rust-dependency-hygiene-machete-wendao:
    @echo "Running Wendao dependency-hygiene advisory lane (cargo-machete)..."
    @if ! command -v cargo-machete >/dev/null 2>&1; then \
        echo "cargo-machete is required but not installed."; \
        echo "Install with: nix profile add nixpkgs#cargo-machete"; \
        exit 1; \
    fi
    @bash scripts/rust/wendao_contract_dependency_governance.sh machete-wendao

[group('validate')]
rust-dependency-hygiene-udeps-wendao:
    @echo "Running bounded Wendao unused-dependency advisory lane (cargo-udeps)..."
    @if ! command -v cargo-udeps >/dev/null 2>&1; then \
        echo "cargo-udeps is required but not installed."; \
        echo "Install with: nix profile add nixpkgs#cargo-udeps"; \
        exit 1; \
    fi
    @bash scripts/rust/wendao_contract_dependency_governance.sh udeps-wendao

[group('validate')]
rust-contract-dependency-governance: rust-contract-semver-core rust-dependency-hygiene-machete-wendao rust-dependency-hygiene-udeps-wendao
    @echo "Rust contract and dependency governance lanes completed (semver + advisory hygiene)."

[group('validate')]
rust-test-xiuxian-core-rs cargo_args="--no-fail-fast":
    @echo "Running xiuxian-core-rs test lane (runtime-linking-safe wrapper)..."
    @CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}" scripts/rust/test_xiuxian_core_rs.sh {{ cargo_args }}

[group('validate')]
rust-quality-gate: rust-lint-inheritance-check rust-test-layout rust-check rust-clippy rust-nextest rust-test-xiuxian-core-rs rust-security-gate
    @echo "Rust quality gates passed (check + strict clippy + nextest + xiuxian-core-rs runtime lane + dependency security)."

[group('validate')]
rust-quality-gate-ci timeout_secs="3600":
    @echo "Running CI rust quality gate (timeout={{ timeout_secs }}s)..."
    @bash scripts/ci/rust_quality_gate_ci.sh "{{ timeout_secs }}"

[group('validate')]
rust-xiuxian-core-rs-lib:
    @echo "Running xiuxian-core-rs library lane..."
    @CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}" scripts/rust/test_xiuxian_core_rs.sh --lib --no-fail-fast

[group('validate')]
rust-xiuxian-daochang-profiles:
    @echo "Running xiuxian-daochang profile checks..."
    @bash scripts/rust/xiuxian_daochang_profiles_check.sh

[group('validate')]
rust-xiuxian-daochang-dependency-assertions:
    @echo "Running xiuxian-daochang dependency assertions..."
    @bash scripts/rust/xiuxian_daochang_dependency_assertions.sh

[group('validate')]
rust-xiuxian-daochang-backend-role-contracts:
    @echo "Running xiuxian-daochang backend role contract tests..."
    @bash scripts/rust/xiuxian_daochang_backend_role_contracts.sh

[group('validate')]
rust-xiuxian-qianji-scenario-audit-contracts:
    @echo "Running xiuxian-qianji scenario-audit contract tests..."
    @bash scripts/rust/xiuxian_qianji_scenario_audit_contracts.sh

[group('validate')]
rust-xiuxian-wendao-contract-feedback-consumer:
    @echo "Running xiuxian-wendao contract-feedback consumer tests..."
    @bash scripts/rust/xiuxian_wendao_contract_feedback_consumer.sh

[group('validate')]
rust-xiuxian-daochang-embedding-role-perf-smoke single_runs="20" batch_runs="10" concurrent_total="64" concurrent_width="8" max_single_p95_ms="" max_batch8_p95_ms="" min_concurrent_rps="" report_json="":
    @bash scripts/rust/xiuxian_daochang_embedding_role_perf_smoke.sh \
      "{{ single_runs }}" \
      "{{ batch_runs }}" \
      "{{ concurrent_total }}" \
      "{{ concurrent_width }}" \
      "{{ max_single_p95_ms }}" \
      "{{ max_batch8_p95_ms }}" \
      "{{ min_concurrent_rps }}" \
      "{{ report_json }}"

[group('validate')]
rust-xiuxian-daochang-embedding-role-perf-medium-gate:
    @bash scripts/rust/xiuxian_daochang_embedding_role_perf_medium_gate.sh

[group('validate')]
rust-xiuxian-daochang-embedding-role-perf-heavy-gate:
    @bash scripts/rust/xiuxian_daochang_embedding_role_perf_heavy_gate.sh

[group('validate')]
rust-retrieval-audits:
    @echo "Running Wendao retrieval audits..."
    @bash scripts/rust/wendao_retrieval_audits.sh

[group('validate')]
rust-wendao-performance-quick:
    @echo "Running Wendao performance quick gate via nextest..."
    @RUNNER_OS="{{ xiuxian_wendao_runner_os }}" CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}" scripts/rust/cargo_exec.sh nextest run -p xiuxian-wendao --features performance --test xiuxian-testing-gate -E "not ({{ xiuxian_wendao_gateway_formal_filter }})"

[group('validate')]
rust-wendao-performance-gateway-formal:
    @echo "Running Wendao formal gateway warm-cache perf cases..."
    @RUNNER_OS="{{ xiuxian_wendao_runner_os }}" CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}" scripts/rust/cargo_exec.sh nextest run -p xiuxian-wendao --features performance --test xiuxian-testing-gate -E "{{ xiuxian_wendao_gateway_formal_filter }}"

[group('validate')]
wendao-gateway-perf-summary:
    @uv run python scripts/render_wendao_gateway_perf_summary.py \
      --report-dir "{{ xiuxian_wendao_gateway_perf_report_dir }}" \
      --real-workspace-report-dir "{{ xiuxian_wendao_gateway_real_workspace_perf_report_dir }}" \
      --runner-os "{{ xiuxian_wendao_runner_os }}" \
      --output-json "{{ xiuxian_wendao_gateway_perf_report_dir }}/gateway_perf_summary.json" \
      --output-markdown "{{ xiuxian_wendao_gateway_perf_report_dir }}/gateway_perf_summary.md"

[group('validate')]
wendao-gateway-perf-summary-real-workspace:
    @uv run python scripts/render_wendao_gateway_perf_summary.py \
      --report-dir "{{ xiuxian_wendao_gateway_perf_report_dir }}" \
      --real-workspace-report-dir "{{ xiuxian_wendao_gateway_real_workspace_perf_report_dir }}" \
      --runner-os "{{ xiuxian_wendao_runner_os }}" \
      --output-json "{{ xiuxian_wendao_gateway_perf_report_dir }}/gateway_perf_summary.json" \
      --output-markdown "{{ xiuxian_wendao_gateway_perf_report_dir }}/gateway_perf_summary.md" \
      --mirror-output-dir "{{ xiuxian_wendao_gateway_real_workspace_perf_report_dir }}"

[group('validate')]
rust-wendao-performance-gateway-real-workspace:
    @echo "Running Wendao manual real-workspace gateway perf samples..."
    @RUNNER_OS="{{ xiuxian_wendao_runner_os }}" XIUXIAN_WENDAO_GATEWAY_PERF_WORKSPACE_ROOT="${XIUXIAN_WENDAO_GATEWAY_PERF_WORKSPACE_ROOT:-.data/wendao-frontend}" CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}" scripts/rust/cargo_exec.sh test -p xiuxian-wendao --features performance --test xiuxian-testing-gate real_workspace -- --ignored --nocapture
    @just wendao-gateway-perf-summary-real-workspace

[group('validate')]
rust-wendao-performance-gate:
    @just rust-wendao-performance-quick
    @just rust-wendao-performance-gateway-formal

[group('validate')]
rust-wendao-performance-stress:
    @echo "Running Wendao performance stress gate (ignored-only) via nextest..."
    @CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}" scripts/rust/cargo_exec.sh nextest run -p xiuxian-wendao --features "performance performance-stress" --test xiuxian-testing-gate --run-ignored ignored-only

[group('validate')]
rust-wendao-performance-bench:
    @echo "Compiling Wendao Criterion benches (CI default: fast lane)..."
    @just rust-wendao-performance-bench-fast

[group('validate')]
rust-wendao-performance-bench-fast:
    @echo "Compiling Wendao Criterion benches (fast profile overrides)..."
    @CARGO_PROFILE_BENCH_LTO="${CARGO_PROFILE_BENCH_LTO:-off}" \
      CARGO_PROFILE_BENCH_CODEGEN_UNITS="${CARGO_PROFILE_BENCH_CODEGEN_UNITS:-16}" \
      CARGO_PROFILE_BENCH_DEBUG="${CARGO_PROFILE_BENCH_DEBUG:-0}" \
      CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}" \
      scripts/rust/cargo_exec.sh bench -p xiuxian-wendao --features performance --bench wendao_performance --no-run

[group('validate')]
telegram-session-isolation-rust:
    @echo "Running Telegram session isolation tests (Rust)..."
    @bash scripts/rust/telegram_session_isolation_rust.sh

[group('validate')]
telegram-session-isolation-python:
    @echo "Running Telegram session isolation tests (Python)..."
    @uv run pytest -q \
        packages/python/test-kit/tests/test_agent_channel_command_events.py \
        packages/python/test-kit/tests/test_agent_channel_session_matrix.py

# KG cache (xiuxian-wendao) and Lance storage-shell cache tests.
[group('validate')]
rust-test-cache:
    @echo "Running Rust cache tests (test_kg_cache, vector storage search_cache)..."
    @scripts/rust/cargo_exec.sh test -p xiuxian-wendao --test test_kg_cache -- --test-threads=1
    @scripts/rust/cargo_exec.sh test -p xiuxian-vector --test test_search_cache -- --test-threads=1

# xiuxian-daochang: config, session, tool runtime, gateway (HTTP 400/404), agent loop
[group('validate')]
rust-test-agent:
    @echo "Running Rust agent tests (xiuxian-daochang)..."
    @scripts/rust/cargo_exec.sh test -p xiuxian-daochang

[group('validate')]
test:
    @echo "TEST PIPELINE"
    @echo "========================================"
    @echo "[1/6] Rust compile gate"
    @just rust-check
    @echo ""
    @echo "[2/6] Rust cache tests (kg_cache, vector storage cache)"
    @just rust-test-cache
    @echo ""
    @echo "[3/6] Rust agent tests (xiuxian-daochang)"
    @just rust-test-agent
    @echo ""
    @echo "[4/6] Wendao PPR quality/perf gate"
    @just gate-wendao-ppr
    @echo ""
    @echo "[5/6] Python test suites"
    @uv run pytest packages/python/foundation/tests/ packages/python/core/tests/ \
        -v --tb=short
    @echo ""
    @echo "[6/6] Channel cursor contract gate"
    @just test-channel-cursor-contracts
    @echo ""
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "                              COMPLETE"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

[group('validate')]
test-python:
    @echo "PYTHON TEST PIPELINE"
    @echo "========================================"
    @echo "[1/6] foundation"
    @uv run pytest packages/python/foundation -q
    @echo ""
    @echo "[2/6] core"
    @uv run pytest packages/python/core -q
    @echo ""
    @echo "[3/6] wendao-core-lib"
    @uv run pytest packages/python/wendao-core-lib -q
    @echo ""
    @echo "[4/6] wendao-arrow-interface"
    @cd packages/python/wendao-arrow-interface && uv run pytest tests -q
    @echo ""
    @echo "[5/6] xiuxian-wendao-analyzer"
    @cd packages/python/xiuxian-wendao-analyzer && uv run pytest tests -q
    @echo ""
    @echo "[6/6] test-kit"
    @uv run pytest packages/python/test-kit -q
    @echo ""
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "                         PYTHON TESTS COMPLETE"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

[group('validate')]
test-channel-cursor-contracts:
    @echo "Running channel cursor contract regressions..."
    @uv run pytest -q scripts/channel/test_log_io.py scripts/channel/test_memory_ci_gate.py \
        scripts/channel/test_discord_ingress_stress_config.py \
        scripts/channel/test_discord_ingress_stress_runtime.py \
        packages/python/test-kit/tests/test_agent_channel_blackbox.py \
        packages/python/test-kit/tests/test_agent_channel_memory_benchmark.py \
        packages/python/test-kit/tests/test_agent_channel_concurrent_sessions.py \
        packages/python/test-kit/tests/test_agent_channel_dedup_events.py \
        packages/python/test-kit/tests/test_agent_channel_discord_ingress_stress.py

[group('validate')]
test-quick:
    @echo "TEST PIPELINE (QUICK)"
    @echo "========================================"
    @bash scripts/ci/test_quick.sh

[group('validate')]
no-inline-python-guard:
    @uv run pytest -q scripts/test_no_inline_python_exec_patterns.py --tb=short

[group('validate')]
contract-e2e-route-test-json:
    @uv run pytest \
        packages/python/foundation/tests/unit/services/test_contract_consistency.py::test_route_test_cli_json_validates_against_schema \
        -v

[group('validate')]
ci-scripts-smoke:
    @bash scripts/ci_scripts_smoke.sh

[group('validate')]
valkey-live:
    #!/usr/bin/env bash
    set -euo pipefail
    valkey_url="$(uv run python scripts/channel/resolve_valkey_endpoint.py --field url)"
    valkey_port="$(uv run python scripts/channel/resolve_valkey_endpoint.py --field port)"
    bash scripts/channel/valkey_live_gate.sh "${valkey_port}" "${valkey_url}"

[group('validate')]
test-contract-freeze:
    @bash scripts/ci-contract-freeze.sh

[group('validate')]
benchmark-wendao-search query="architecture" runs="5" warm_runs="2":
    @uv run python scripts/benchmark_wendao_search.py --root . --query "{{ query }}" --runs "{{ runs }}" --warm-runs "{{ warm_runs }}" --no-build

[group('validate')]
benchmark-wendao-search-build query="architecture" runs="5" warm_runs="2":
    @uv run python scripts/benchmark_wendao_search.py --root . --query "{{ query }}" --runs "{{ runs }}" --warm-runs "{{ warm_runs }}"

[group('validate')]
benchmark-wendao-search-release query="architecture" runs="5" warm_runs="2":
    @uv run python scripts/benchmark_wendao_search.py --root . --query "{{ query }}" --runs "{{ runs }}" --warm-runs "{{ warm_runs }}" --release --no-build

[group('validate')]
benchmark-wendao-search-release-build query="architecture" runs="5" warm_runs="2":
    @uv run python scripts/benchmark_wendao_search.py --root . --query "{{ query }}" --runs "{{ runs }}" --warm-runs "{{ warm_runs }}" --release

[group('validate')]
evaluate-wendao-retrieval limit="10" min_top3_rate="0.0":
    @uv run python scripts/evaluate_wendao_retrieval.py --root . --matrix-file "docs/testing/wendao-query-regression-matrix.json" --limit "{{ limit }}" --min-top3-rate "{{ min_top3_rate }}" --no-build

[group('validate')]
evaluate-wendao-retrieval-build limit="10" min_top3_rate="0.0":
    @uv run python scripts/evaluate_wendao_retrieval.py --root . --matrix-file "docs/testing/wendao-query-regression-matrix.json" --limit "{{ limit }}" --min-top3-rate "{{ min_top3_rate }}"

[group('validate')]
evaluate-wendao-retrieval-release limit="10" min_top3_rate="0.0":
    @uv run python scripts/evaluate_wendao_retrieval.py --root . --matrix-file "docs/testing/wendao-query-regression-matrix.json" --limit "{{ limit }}" --min-top3-rate "{{ min_top3_rate }}" --release --no-build

[group('validate')]
evaluate-wendao-retrieval-release-build limit="10" min_top3_rate="0.0":
    @uv run python scripts/evaluate_wendao_retrieval.py --root . --matrix-file "docs/testing/wendao-query-regression-matrix.json" --limit "{{ limit }}" --min-top3-rate "{{ min_top3_rate }}" --release

[group('validate')]
gate-wendao-ppr:
    @XIUXIAN_WENDAO_GATE_QUERY_PREFIX="{{ xiuxian_wendao_gate_query_prefix }}" \
      bash scripts/gate_wendao_ppr.sh \
        "docs/testing/wendao-query-regression-matrix.json" \
        "{{ xiuxian_wendao_gate_limit }}" \
        "{{ xiuxian_wendao_gate_profile }}" \
        "{{ xiuxian_wendao_gate_build_mode }}" \
        "{{ xiuxian_wendao_gate_min_top3_rate }}" \
        "{{ xiuxian_wendao_gate_stem }}" \
        "{{ xiuxian_wendao_gate_runs }}" \
        "{{ xiuxian_wendao_gate_warm_runs }}" \
        "{{ xiuxian_wendao_gate_subgraph_mode }}" \
        "{{ xiuxian_wendao_gate_max_p95_ms }}" \
        "{{ xiuxian_wendao_gate_max_avg_ms }}" \
        "{{ xiuxian_wendao_gate_expect_subgraph_count_min }}" \
        "{{ xiuxian_wendao_gate_output_mode }}"

[group('validate')]
gate-wendao-ppr-report:
    @XIUXIAN_WENDAO_GATE_QUERY_PREFIX="{{ xiuxian_wendao_gate_query_prefix }}" \
      XIUXIAN_WENDAO_GATE_REPORT_DIR="{{ xiuxian_wendao_gate_report_dir }}" \
      bash scripts/gate_wendao_ppr.sh \
        "docs/testing/wendao-query-regression-matrix.json" \
        "{{ xiuxian_wendao_gate_limit }}" \
        "{{ xiuxian_wendao_gate_profile }}" \
        "{{ xiuxian_wendao_gate_build_mode }}" \
        "{{ xiuxian_wendao_gate_min_top3_rate }}" \
        "{{ xiuxian_wendao_gate_stem }}" \
        "{{ xiuxian_wendao_gate_runs }}" \
        "{{ xiuxian_wendao_gate_warm_runs }}" \
        "{{ xiuxian_wendao_gate_subgraph_mode }}" \
        "{{ xiuxian_wendao_gate_max_p95_ms }}" \
        "{{ xiuxian_wendao_gate_max_avg_ms }}" \
        "{{ xiuxian_wendao_gate_expect_subgraph_count_min }}" \
        "json" >/dev/null

[group('validate')]
gate-wendao-ppr-mixed-canary:
    @XIUXIAN_WENDAO_GATE_QUERY_PREFIX="{{ xiuxian_wendao_mixed_canary_query_prefix }}" \
      XIUXIAN_WENDAO_GATE_REPORT_DIR="{{ xiuxian_wendao_mixed_canary_report_dir }}" \
      bash scripts/gate_wendao_ppr.sh \
        "docs/testing/wendao-query-regression-matrix.json" \
        "{{ xiuxian_wendao_gate_limit }}" \
        "{{ xiuxian_wendao_gate_profile }}" \
        "{{ xiuxian_wendao_gate_build_mode }}" \
        "{{ xiuxian_wendao_mixed_canary_min_top3_rate }}" \
        "{{ xiuxian_wendao_gate_stem }}" \
        "{{ xiuxian_wendao_gate_runs }}" \
        "{{ xiuxian_wendao_gate_warm_runs }}" \
        "{{ xiuxian_wendao_gate_subgraph_mode }}" \
        "{{ xiuxian_wendao_gate_max_p95_ms }}" \
        "{{ xiuxian_wendao_gate_max_avg_ms }}" \
        "{{ xiuxian_wendao_gate_expect_subgraph_count_min }}" \
        "json" >/dev/null

[group('validate')]
validate-wendao-ppr-reports:
    @uv run python scripts/validate_wendao_gate_reports.py \
      --root . \
      --report-dir "{{ xiuxian_wendao_gate_report_dir }}" \
      --mixed-report-dir "{{ xiuxian_wendao_mixed_canary_report_dir }}"

[group('validate')]
wendao-ppr-gate-summary:
    @strict_flag=""; \
      case "{{ xiuxian_wendao_gate_summary_strict_green }}" in \
        1|true|TRUE|yes|YES|on|ON) strict_flag="--strict-green" ;; \
      esac; \
      uv run python scripts/render_wendao_gate_status_summary.py \
        --base-report-dir "{{ xiuxian_wendao_gate_report_dir }}" \
        --mixed-report-dir "{{ xiuxian_wendao_mixed_canary_report_dir }}" \
        --min-base-top3-rate "{{ xiuxian_wendao_gate_min_top3_rate }}" \
        --min-mixed-top3-rate "{{ xiuxian_wendao_rollout_min_mixed_top3_rate }}" \
        --runner-os "{{ xiuxian_wendao_runner_os }}" \
        --output-json "{{ xiuxian_wendao_gate_report_dir }}/wendao_gate_status_summary.json" \
        --output-markdown "{{ xiuxian_wendao_gate_report_dir }}/wendao_gate_status_summary.md" \
        ${strict_flag}

[group('validate')]
wendao-ppr-rollout-status:
    @XIUXIAN_WENDAO_GATE_MIN_TOP3_RATE="{{ xiuxian_wendao_gate_min_top3_rate }}" \
      XIUXIAN_WENDAO_GATE_SUMMARY_STRICT_GREEN="{{ xiuxian_wendao_gate_summary_strict_green }}" \
      XIUXIAN_WENDAO_ROLLOUT_FETCH_REMOTE_STATUS="{{ xiuxian_wendao_rollout_fetch_remote_status }}" \
      XIUXIAN_WENDAO_ROLLOUT_REMOTE_WORKFLOW_FILE="{{ xiuxian_wendao_rollout_remote_workflow_file }}" \
      XIUXIAN_WENDAO_ROLLOUT_REMOTE_ARTIFACT_NAME="{{ xiuxian_wendao_rollout_remote_artifact_name }}" \
      XIUXIAN_WENDAO_ROLLOUT_REMOTE_RUN_STATUS="{{ xiuxian_wendao_rollout_remote_run_status }}" \
      bash scripts/wendao_ppr_rollout_ci.sh \
        "{{ xiuxian_wendao_gate_report_dir }}" \
        "{{ xiuxian_wendao_mixed_canary_report_dir }}" \
        "{{ xiuxian_wendao_rollout_required_runs }}" \
        "{{ xiuxian_wendao_rollout_min_mixed_top3_rate }}" \
        "{{ xiuxian_wendao_rollout_strict_ready }}"

[group('validate')]
test-parallel:
    @echo "Running tests in parallel (faster)..."
    @uv run pytest packages/python/foundation/tests/ packages/python/core/tests/ packages/python/wendao-core-lib/tests/ -n auto --tb=short

[group('validate')]
vulture:
    @echo "Checking for dead code with vulture..."
    @uvx vulture || echo "Dead code detected - review above items"

[group('validate')]
test-stress:
    @echo "Running stress tests (slow)..."
    @uv run pytest packages/python/test-kit/tests/benchmarks/ -v

# Contract tests: data interface shape for run_skill, reindex, sync, run_entry (no xdist)
[group('validate')]
test-contracts:
    @echo "Running data interface contract tests..."
    @uv run pytest packages/python/foundation/tests/unit/services/test_runtime_contract_schemas.py -v --tb=short --override-ini addopts="-v --tb=short"

# Scale benchmarks: in test-kit (run_skill, reindex_status, sync; latency thresholds)
[group('validate')]
test-benchmarks:
    @echo "Running scale benchmarks (xiuxian-test-kit)..."
    @cd packages/python/test-kit && uv run pytest tests/benchmarks/ -v --tb=short

# ==============================================================================
# CHANGELOG MANAGEMENT
# =============================================================================

[group('changelog')]
changelog-preview:
    @echo "Changelog Preview (since last tag)"
    @echo "===================================="
    @cog log
    @echo ""
    @echo "Commit breakdown:"
    @cog log | grep -oE "^(feat|fix|docs|style|refactor|perf|test|build|ci|chore)" | sort | uniq -c

[group('changelog')]
changelog-stats:
    @echo "Changelog Statistics"
    @echo "===================="
    @echo "Commits by type:"
    @cog log | grep -oE "^(feat|fix|docs|style|refactor|perf|test|build|ci|chore)" | sort | uniq -c | sort -rn
    @echo ""
    @echo "Commits by author:"
    @git log --format='%an' $(git describe --tags --abbrev=0 2>/dev/null || echo "HEAD")..HEAD | sort | uniq -c | sort -rn
    @echo ""
    @echo "Changes since last release:"
    @git diff --stat $(git describe --tags --abbrev=0 2>/dev/null || echo "HEAD")..HEAD

[group('validate')]
check-commits:
    @echo "Validating commit messages..."
    @cog check

[group('validate')]
check-commits-range from to:
    @cog check {{ from }}..{{ to }}

[group('changelog')]
changelog:
    @echo "Generating changelog..."
    @cog changelog

[group('changelog')]
changelog-at version:
    @cog changelog --at {{ version }}

[group('changelog')]
changelog-export version="latest":
    #!/usr/bin/env bash
    set -euo pipefail
    VERSION={{ version }}
    if [ "$VERSION" = "latest" ]; then
        VERSION=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
    fi
    echo "Exporting changelog for $VERSION..."
    cog changelog --at "$VERSION" > "CHANGELOG_${VERSION}.md"
    echo "  Markdown: CHANGELOG_${VERSION}.md"
    cog log --format json > "CHANGELOG_${VERSION}.json"
    echo "  JSON: CHANGELOG_${VERSION}.json"
    cog changelog --at "$VERSION" | sed 's/\[//' | sed 's/\](.*)$//' > "CHANGELOG_${VERSION}.txt"
    echo "  Plain text: CHANGELOG_${VERSION}.txt"

# ==============================================================================
# VERSION MANAGEMENT & RELEASES
# ==============================================================================

[group('version')]
version:
    @echo "Current version: $(cat VERSION 2>/dev/null || git describe --tags --abbrev=0 2>/dev/null || echo 'No version found')"

[group('version')]
bump-auto: validate
    @echo "Auto-bumping version..."
    @cog bump --auto
    @just _sync-versions

[group('version')]
bump-patch: validate
    @echo "Bumping patch version..."
    @cog bump --patch
    @just _sync-versions

[group('version')]
bump-minor: validate
    @echo "Bumping minor version..."
    @cog bump --minor
    @just _sync-versions

[group('version')]
bump-major: validate
    @echo "Bumping major version..."
    @cog bump --major
    @just _sync-versions

# Sync versions across all packages from VERSION file

# Internal helper - called by bump-*
[private]
_sync-versions:
    #!/usr/bin/env bash
    set -euo pipefail
    NEW_VERSION=$(cat VERSION)
    echo "Syncing version $NEW_VERSION across all packages..."
    # Update core pyproject.toml
    sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" packages/python/core/pyproject.toml && rm -f packages/python/core/pyproject.toml.bak
    echo "  ✓ Core: packages/python/core/pyproject.toml"
    # Update foundation pyproject.toml
    sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" packages/python/foundation/pyproject.toml && rm -f packages/python/foundation/pyproject.toml.bak
    echo "  ✓ Foundation: packages/python/foundation/pyproject.toml"
    # Note: Root pyproject.toml uses dynamic version (hatch-vcs), no sync needed
    echo ""
    echo "All packages updated to version $NEW_VERSION!"

[group('version')]
bump-dry:
    @echo "Previewing version bump (dry run)..."
    @cog bump --auto --dry-run

[group('version')]
bump-pre type="alpha":
    @echo "Creating pre-release ({{ type }})..."
    @cog bump --pre {{ type }}

# Set explicit version across all packages

# Usage: just bump-set 0.3.0
[group('version')]
bump-set version:
    #!/usr/bin/env bash
    set -euo pipefail
    NEW_VERSION="{{ version }}"
    echo "Setting version to $NEW_VERSION across all packages..."
    # Update VERSION file
    echo "$NEW_VERSION" > VERSION
    # Sync to all pyproject.toml files
    just _sync-versions
    echo ""
    echo "Next steps:"
    echo "  1. Run: git add -A && git commit -m 'chore: bump version to $NEW_VERSION'"
    echo "  2. Run: git tag v$NEW_VERSION"
    echo "  3. Run: git push origin main v$NEW_VERSION"

[group('version')]
release-notes version="latest":
    #!/usr/bin/env bash
    set -euo pipefail
    VERSION={{ version }}
    if [ "$VERSION" = "latest" ]; then
        VERSION=$(git describe --tags --abbrev=0)
    fi
    echo "# Release $VERSION"
    echo ""
    cog changelog --at "$VERSION" | sed -n "/^## \[v${VERSION#v}\]/,/^## \[v/p" | sed '$d'
    echo ""
    echo "---"
    echo "**Full Changelog**: https://github.com/tao3k/xiuxian-artisan-workshop/compare/$(git describe --tags --abbrev=0 $VERSION^ 2>/dev/null)...$VERSION"

[group('version')]
publish-release version="latest":
    #!/usr/bin/env bash
    set -euo pipefail
    VERSION={{ version }}
    if [ "$VERSION" = "latest" ]; then
        VERSION=$(git describe --tags --abbrev=0)
    fi
    NOTES=$(mktemp)
    just release-notes "$VERSION" > "$NOTES"
    echo "Publishing release $VERSION to GitHub..."
    gh release create "$VERSION" --title "Release $VERSION" --notes-file "$NOTES" --verify-tag
    rm -f "$NOTES"
    echo "Published release $VERSION"

[group('version')]
release type="auto":
    @echo "Starting release workflow..."
    @just bump-{{ type }}
    @just publish-release

# ==============================================================================
# GIT OPERATIONS
# ==============================================================================

[group('git')]
status:
    @echo "Repository Status"
    @echo "=================="
    @git status
    @echo ""
    @echo "Branch: $(git branch --show-current)"
    @echo "Last commit: $(git log -1 --oneline)"
    @echo "Last tag: $(git describe --tags --abbrev=0 2>/dev/null || echo 'No tags')"

[group('git')]
log n="10":
    @cog log --no-pager | head -n {{ n }}

# ==============================================================================
# DEVELOPMENT HELPERS
# ==============================================================================

[group('dev')]
fmt:
    @echo "Formatting code..."
    @lefthook run pre-commit --all-files

[group('dev')]
fmt-py:
    @echo "Formatting Python with ruff..."
    @uvx ruff format packages/python/

[group('dev')]
clean-generated:
    @echo "Cleaning generated changelog artifacts..."
    @rm -f CHANGELOG_*.md CHANGELOG_*.json CHANGELOG_*.txt RELEASE_NOTES_*.md
    @echo "Generated artifacts cleaned"

# Generate TypeScript bindings from Rust Specta types for Qianji Studio
[group('dev')]
generate-bindings:
    @echo "Generating TypeScript bindings from Rust Specta types..."
    cargo run --bin export_types --features zhenfa-router
    @echo "Bindings written to .data/wendao-frontend/src/api/bindings.ts"

[group('dev')]
clean-rust:
    @echo "Cleaning Rust build artifacts via cargo clean..."
    @scripts/rust/cargo_exec.sh clean
    @echo "Rust build artifacts cleaned"

[group('dev')]
clean-all:
    @just clean-generated
    @just clean-rust

[group('dev')]
clean:
    @just clean-generated

[group('dev')]
update:
    @echo "Updating dependencies..."
    @devenv update
    @echo "Updated"

[group('dev')]
julia-search mode="dev":
    @python scripts/sync_wendao_julia_locks.py --mode "{{ mode }}"

[group('dev')]
info:
    @echo "Environment Information"
    @echo "======================"
    @echo "devenv version: $(devenv version)"
    @echo "nix version: $(nix --version)"
    @echo "just version: $(just --version)"
    @echo "cog version: $(cog --version 2>/dev/null || echo 'not found')"
    @echo "git version: $(git --version)"
    @echo ""
    @echo "Repository: $(git remote get-url origin 2>/dev/null || echo 'no remote')"
    @echo "Branch: $(git branch --show-current)"
    @echo "Version: $(cat VERSION 2>/dev/null || git describe --tags --abbrev=0 2>/dev/null || echo 'unknown')"

[group('dev')]
watch:
    @echo "Watching for changes..."
    @watchexec -e nix,md,sh -c "just check-format"

# ==============================================================================
# TOOL RUNTIME COMMANDS
# ==============================================================================
# ==============================================================================
# SRE HEALTH CHECKS
# Outputs machine-parseable JSON for AI agents and CI/CD
# ==============================================================================

[group('sre')]
health: health-git health-nix health-secrets health-devenv
    @echo ""
    @echo "Health check complete!"

# Git repository health (JSON optional)
[group('sre')]
health-git:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "${JUST_JSON:-}" = "true" ]; then
        BRANCH=$(git branch --show-current)
        UNCOMMITTED=$(git status --porcelain | wc -l)
        LAST_COMMIT=$(git log -1 --oneline)
        BEHIND=0
        git fetch --quiet 2>/dev/null && BEHIND=$(git log HEAD..origin/$BRANCH 2>/dev/null | wc -l)
        jq -n --arg branch "$BRANCH" --argjson uncommitted "$UNCOMMITTED" --arg last_commit "$LAST_COMMIT" --argjson behind "$BEHIND" \
            '{component: "git", branch: $branch, uncommitted_files: $uncommitted, last_commit: $last_commit, commits_behind: $behind}'
    else
        echo "Git Health"
        echo "=========="
        echo "Branch: $(git branch --show-current)"
        echo "Status: $(git status --porcelain | wc -l) uncommitted files"
        echo "Last commit: $(git log -1 --oneline)"
    fi

# Nix/Devenv health
[group('sre')]
health-nix:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "${JUST_JSON:-}" = "true" ]; then
        NIX_VERSION=$(nix --version 2>/dev/null || echo "")
        jq -n --arg version "$NIX_VERSION" '{component: "nix", version: $version}'
    else
        echo "Nix Health"
        echo "=========="
        echo "Nix version: $(nix --version)"
    fi

# Devenv health
[group('sre')]
health-devenv:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "${JUST_JSON:-}" = "true" ]; then
        VERSION=$(devenv version 2>/dev/null || echo "")
        NIXPKGS=$(devenv nixpkgs-version 2>/dev/null || echo "")
        jq -n --arg version "$VERSION" --arg nixpkgs "$NIXPKGS" '{component: "devenv", version: $version, nixpkgs: $nixpkgs}'
    else
        echo "Devenv Health"
        echo "============="
        echo "Version: $(devenv version 2>/dev/null || echo 'not found')"
        echo "Nixpkgs: $(devenv nixpkgs-version 2>/dev/null || echo 'unknown')"
    fi

# Secrets health check (validates presence, never echoes values)
[group('sre')]
health-secrets:
    #!/usr/bin/env bash
    set -euo pipefail
    MISSING=""
    if [ -z "${MINIMAX_API_KEY:-}" ]; then
        MISSING="MINIMAX_API_KEY"
    fi
    if [ "${JUST_JSON:-}" = "true" ]; then
        if [ -z "$MISSING" ]; then
            jq -n '{component: "secrets", status: "pass", message: "All required secrets present"}'
        else
            jq -n --arg missing "$MISSING" \
                '{component: "secrets", status: "fail", message: "Missing secrets", missing_keys: [$missing]}'
            exit 1
        fi
    else
        echo "Secrets Health"
        echo "=============="
        echo "Provider: dotenv"
        if [ -z "$MISSING" ]; then
            echo "Status: OK"
        else
            echo "Status: MISSING - $MISSING"
        fi
    fi

# API keys health (presence check only)
[group('sre')]
health-api-keys:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "${JUST_JSON:-}" = "true" ]; then
        if [ -n "${MINIMAX_API_KEY:-}" ]; then
            jq -n '{component: "api_keys", minimax: "present"}'
        else
            jq -n '{component: "api_keys", minimax: "missing"}'
            exit 1
        fi
    else
        echo "API Keys Health"
        echo "==============="
        if [ -n "${MINIMAX_API_KEY:-}" ]; then
            echo "MINIMAX_API_KEY: Set"
        else
            echo "MINIMAX_API_KEY: Not set"
        fi
    fi

# Composite health report for agents
[group('sre')]
health-report:
    #!/usr/bin/env bash
    set -euo pipefail
    JUST_JSON=true just health-git
    JUST_JSON=true just health-devenv
    JUST_JSON=true just health-secrets

# ==============================================================================
# CI/CD COMMANDS
# ==============================================================================

[group('ci')]
ci: validate changelog-preview
    @echo "CI checks passed!"

[group('ci')]
pre-release: validate changelog-preview changelog-stats
    @echo ""
    @echo "Pre-release checks complete!"
    @echo ""
    @echo "Next steps:"
    @echo "  1. Review changelog preview"
    @echo "  2. Run: just bump-auto (or bump-patch/minor/major)"
    @echo "  3. Run: just publish-release"

# ==============================================================================
# SECRET MANAGEMENT (secretspec)
# ==============================================================================

[group('secrets')]
secrets-check:
    @echo "Checking secrets status..."
    @secretspec check --profile development

[group('secrets')]
secrets-info:
    @echo "Secret Management Info"
    @echo "======================"
    @echo "Provider: dotenv"
    @echo "Profile: development"
    @echo ""
    @echo "Configured secrets:"
    @secretspec check --profile development | grep -E "^\s+[A-Z]" || echo "  (none)"

[group('secrets')]
secrets-set-minimax:
    #!/usr/bin/env bash
    set -euo pipefail
    read -p "Enter MINIMAX_API_KEY: " -s API_KEY
    echo
    secretspec set MINIMAX_API_KEY --value "$API_KEY" --profile development
    echo "MINIMAX_API_KEY set"

[group('secrets')]
secrets-get-minimax:
    @secretspec get MINIMAX_API_KEY

# ==============================================================================
# DOCUMENTATION
# ==============================================================================

[group('docs')]
docs:
    @echo "Documentation Index"
    @echo "==================="
    @echo ""
    @echo "Available documentation:"
    @ls -1 *.md | sed 's/^/  - /'

[group('docs')]
examples:
    @echo "Commit Message Examples"
    @echo "======================="
    @echo ""
    @echo "feat: add new feature"
    @echo "feat(cli): add command"
    @echo "fix: correct bug"
    @echo "docs: update documentation"
    @echo "refactor: reorganize code"
    @echo "chore: maintenance tasks"
    @echo ""
    @echo "feat(api)!: breaking change"
    @echo "BREAKING CHANGE: description"

# ==============================================================================
# SPEC KIT (Spec-Driven Development)
# ==============================================================================

[group('spec')]
spec-list:
    @echo "Available Specs"
    @echo "================"
    @ls -1 assets/specs/*.md 2>/dev/null | sed 's|^assets/specs/||' | sed 's/\.md$//' | sed 's/^/  - /' || echo "  No specs found"

[group('spec')]
spec-template:
    @echo "Spec Template"
    @echo "============="
    @cat assets/specs/TEMPLATE.md

[group('spec')]
archive spec_path target_category="explanation":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "{{ spec_path }}" ]; then
        echo "Usage: just archive <spec-path> [category]"
        echo "Example: just archive assets/specs/auth_module.md explanation"
        exit 1
    fi
    if [ ! -f "{{ spec_path }}" ]; then
        echo "Error: Spec '{{ spec_path }}' not found"
        exit 1
    fi
    echo "============================================"
    echo "📦 Archiving completed spec..."
    echo "============================================"
    echo "Spec: {{ spec_path }}"
    echo "Category: {{ target_category }}"
    echo ""
    echo "Ask the Agent to archive:"
    echo ""
    echo "  @xiuxian-orchestrator archive_spec_to_doc spec_path=\"{{ spec_path }}\" target_category=\"{{ target_category }}\""
    echo ""
    echo "============================================"

# ==============================================================================
# ALIASES (using recipe definitions instead of variable assignments)
# ==============================================================================

check: validate

cl: changelog-preview

c: commit

s: status

ship: release

# Compatibility aliases for agent-* pattern
agent-ci: agent-validate

agent-test: test

agent-lint: lint

agent-format: fmt

# ==============================================================================
# RUST BUILD
# ==============================================================================
# ==============================================================================
# TELEGRAM CHANNEL
# ==============================================================================
# Run Telegram channel in polling mode (no tunnel needed; for local testing).
# Bootstraps local Valkey automatically before starting the agent.

# Usage: TELEGRAM_BOT_TOKEN=xxx just agent-channel [valkey_port]
[group('channel')]
agent-channel valkey_port="":
    #!/usr/bin/env bash
    set -euo pipefail
    resolved_valkey_port="{{ valkey_port }}"
    if [ -z "$resolved_valkey_port" ]; then
      resolved_valkey_port="$(uv run python scripts/channel/resolve_valkey_endpoint.py --field port)"
    fi
    bash scripts/channel/agent-channel-polling.sh "$resolved_valkey_port"

# Run Telegram channel in webhook mode via modular script entrypoint.
# By default this also starts Discord ingress runtime (from `discord.ingress_*` settings)
# unless `DISCORD_INGRESS_ENABLED=0` is set.
# Usage: TELEGRAM_BOT_TOKEN=xxx just agent-channel-webhook [valkey_port] [webhook_port] [gateway_port]
# Requires: ngrok installed, TELEGRAM_BOT_TOKEN in env, valkey-server in PATH
# Note: defaults to verbose debug logs (`--log-verbose`, `RUST_LOG=xiuxian_daochang=debug` when unset).

# Logs are mirrored to `${OMNI_CHANNEL_LOG_FILE:-.run/logs/xiuxian-daochang-webhook.log}` for black-box probes.
[group('channel')]
agent-channel-webhook valkey_port="" webhook_port="" gateway_port="":
    #!/usr/bin/env bash
    set -euo pipefail
    resolved_valkey_port="{{ valkey_port }}"
    if [ -z "$resolved_valkey_port" ]; then
      resolved_valkey_port="$(uv run python scripts/channel/resolve_valkey_endpoint.py --field port)"
    fi
    if [ -n "{{ webhook_port }}" ]; then \
        WEBHOOK_PORT="{{ webhook_port }}" GATEWAY_PORT="{{ gateway_port }}" bash scripts/channel/agent-channel-webhook.sh "$resolved_valkey_port"; \
    else \
        GATEWAY_PORT="{{ gateway_port }}" bash scripts/channel/agent-channel-webhook.sh "$resolved_valkey_port"; \
    fi

# Run Telegram webhook with Dots OCR performance defaults.
# Keeps existing user overrides (env takes precedence).
# Usage:

# TELEGRAM_BOT_TOKEN=xxx just agent-channel-webhook-ocr-fast [valkey_port] [webhook_port] [gateway_port] [model_root]
[group('channel')]
agent-channel-webhook-ocr-fast valkey_port="" webhook_port="" gateway_port="" model_root="":
    #!/usr/bin/env bash
    set -euo pipefail

    resolved_model_root="{{ model_root }}"
    if [ -z "${resolved_model_root}" ]; then
      if [ -d ".data/models/dots-ocr" ]; then
        resolved_model_root=".data/models/dots-ocr"
      elif [ -n "${PRJ_DATA_HOME:-}" ] && [ -d "${PRJ_DATA_HOME}/models/dots-ocr" ]; then
        resolved_model_root="${PRJ_DATA_HOME}/models/dots-ocr"
      else
        echo "Error: dots-ocr model directory not found. Run: just fetch-vision-models" >&2
        exit 1
      fi
    fi

    default_device="cpu"
    case "$(uname -s)" in
      Darwin) default_device="metal" ;;
      Linux) default_device="cuda" ;;
    esac

    export XIUXIAN_VISION_MODEL_KIND="${XIUXIAN_VISION_MODEL_KIND:-dots}"
    export XIUXIAN_VISION_MODEL_PATH="${XIUXIAN_VISION_MODEL_PATH:-${resolved_model_root}}"
    export XIUXIAN_VISION_DEVICE="${XIUXIAN_VISION_DEVICE:-${default_device}}"
    export XIUXIAN_VISION_REQUIRE_QUANTIZED="${XIUXIAN_VISION_REQUIRE_QUANTIZED:-0}"
    export XIUXIAN_VISION_MAX_TILES="${XIUXIAN_VISION_MAX_TILES:-12}"
    export XIUXIAN_VISION_OCR_MAX_NEW_TOKENS="${XIUXIAN_VISION_OCR_MAX_NEW_TOKENS:-1024}"
    export XIUXIAN_VISION_OCR_BATCH_WINDOW_MS="${XIUXIAN_VISION_OCR_BATCH_WINDOW_MS:-50}"
    export XIUXIAN_VISION_OCR_BATCH_MAX_SIZE="${XIUXIAN_VISION_OCR_BATCH_MAX_SIZE:-8}"

    echo "[webhook-ocr-fast] model_root=${XIUXIAN_VISION_MODEL_PATH}"
    echo "[webhook-ocr-fast] device=${XIUXIAN_VISION_DEVICE} model_kind=${XIUXIAN_VISION_MODEL_KIND}"

    just agent-channel-webhook "{{ valkey_port }}" "{{ webhook_port }}" "{{ gateway_port }}"

# Stop webhook launcher and reclaim lock directory safely.

# Usage: just agent-channel-webhook-stop
[group('channel')]
agent-channel-webhook-stop:
    #!/usr/bin/env bash
    set -euo pipefail
    lock_dir="${XIUXIAN_CHANNEL_WEBHOOK_LOCK_DIR:-.run/locks/xiuxian-daochang-webhook.lock}"
    pid_file="${lock_dir}/pid"

    if [ ! -f "$pid_file" ]; then
      echo "No webhook launcher pid file found at ${pid_file}."
      [ -d "$lock_dir" ] && rm -rf "$lock_dir"
      exit 0
    fi

    holder_pid="$(tr -d '[:space:]' < "$pid_file")"
    if [ -z "${holder_pid}" ] || ! ps -p "${holder_pid}" >/dev/null 2>&1; then
      echo "Stale webhook launcher lock detected; reclaiming ${lock_dir}."
      rm -rf "$lock_dir"
      exit 0
    fi

    echo "Stopping webhook launcher pid=${holder_pid}..."
    kill "${holder_pid}" >/dev/null 2>&1 || true

    for _ in $(seq 1 40); do
      if ! ps -p "${holder_pid}" >/dev/null 2>&1; then
        break
      fi
      sleep 0.25
    done

    if ps -p "${holder_pid}" >/dev/null 2>&1; then
      echo "Force stopping webhook launcher pid=${holder_pid}..."
      kill -9 "${holder_pid}" >/dev/null 2>&1 || true
    fi

    # Best-effort cleanup for orphaned webhook worker processes.
    while read -r pid; do
      [ -n "$pid" ] && kill "$pid" >/dev/null 2>&1 || true
    done < <(pgrep -f 'agent_channel_runtime_monitor.py.*xiuxian-daochang-webhook|xiuxian-daochang channel --mode webhook' || true)

    rm -rf "$lock_dir"
    rm -f .run/xiuxian-daochang-webhook.pid
    echo "Webhook launcher stopped and lock reclaimed."

# Restart webhook launcher (stop first to avoid lock conflicts).

# Usage: just agent-channel-webhook-restart [valkey_port] [webhook_port] [gateway_port]
[group('channel')]
agent-channel-webhook-restart valkey_port="" webhook_port="" gateway_port="":
    #!/usr/bin/env bash
    set -euo pipefail
    just agent-channel-webhook-stop
    just agent-channel-webhook "{{ valkey_port }}" "{{ webhook_port }}" "{{ gateway_port }}"

# Run Discord channel in ingress mode for synthetic ingress replay/ACL probes.

# Usage: DISCORD_BOT_TOKEN=xxx just agent-channel-discord-ingress
[group('channel')]
agent-channel-discord-ingress:
    #!/usr/bin/env bash
    set -euo pipefail
    resolved_features=""
    if [ "${XIUXIAN_DAOCHANG_CARGO_FEATURES+x}" = "x" ]; then
      resolved_features="${XIUXIAN_DAOCHANG_CARGO_FEATURES}"
    fi

    runtime_target_dir="${XIUXIAN_DAOCHANG_RUNTIME_TARGET_DIR:-${CARGO_TARGET_DIR:-target}}"
    export CARGO_TARGET_DIR="${runtime_target_dir}"
    xiuxian_daochang_bin="${XIUXIAN_DAOCHANG_BIN:-${CARGO_TARGET_DIR}/debug/xiuxian-daochang}"

    if [ -n "${resolved_features}" ]; then
      scripts/rust/cargo_exec.sh build -p xiuxian-daochang --features "${resolved_features}" --bin xiuxian-daochang
    else
      scripts/rust/cargo_exec.sh build -p xiuxian-daochang --bin xiuxian-daochang
    fi

    if [ ! -x "${xiuxian_daochang_bin}" ]; then
      echo "Error: xiuxian-daochang binary not found at ${xiuxian_daochang_bin} after build." >&2
      exit 1
    fi

    "${xiuxian_daochang_bin}" channel --provider discord --discord-runtime-mode ingress --log-verbose

# Black-box probe: inject one synthetic Telegram update into local webhook and wait for bot reply log.
# Usage: just agent-channel-blackbox "your prompt" [max_wait_secs]
# Behavior: event-driven by default (no hard timeout when max_wait_secs is omitted).
# Optional env: OMNI_TEST_CHAT_ID, OMNI_TEST_USER_ID, OMNI_TEST_USERNAME, OMNI_TEST_THREAD_ID, OMNI_WEBHOOK_URL,
#               OMNI_BLACKBOX_MAX_WAIT_SECS, OMNI_BLACKBOX_MAX_IDLE_SECS
# Advanced flags (expect/forbid regex, allow-no-bot, fail-fast tuning):
#   bash scripts/channel/agent-channel-blackbox.sh --help

# Implementation: Python (`scripts/channel/agent_channel_blackbox.py`) via shell wrapper.
[group('channel')]
agent-channel-blackbox prompt max_wait_secs="":
    if [ -n "{{ max_wait_secs }}" ]; then \
        bash scripts/channel/agent-channel-blackbox.sh --prompt "{{ prompt }}" --max-wait "{{ max_wait_secs }}"; \
    else \
        bash scripts/channel/agent-channel-blackbox.sh --prompt "{{ prompt }}"; \
    fi

# Black-box probe specialized for native tool dispatch validation.
# This adds strict runtime assertions that native tool dispatch succeeds and
# legacy dispatch wording does not appear in the same probe.

# Usage: just agent-channel-blackbox-native "crawl https://example.com" [max_wait_secs]
[group('channel')]
agent-channel-blackbox-native prompt max_wait_secs="":
    if [ -n "{{ max_wait_secs }}" ]; then \
        bash scripts/channel/agent-channel-blackbox.sh --prompt "{{ prompt }}" --max-wait "{{ max_wait_secs }}" --native-tools-only; \
    else \
        bash scripts/channel/agent-channel-blackbox.sh --prompt "{{ prompt }}" --native-tools-only; \
    fi

# Run strict black-box command matrix with command-level event assertions.
# Usage: just agent-channel-blackbox-commands [max_wait_secs] [max_idle_secs]

# Optional env: OMNI_TEST_USERNAME, OMNI_TEST_CHAT_ID, OMNI_TEST_USER_ID, OMNI_TEST_THREAD_ID, OMNI_WEBHOOK_URL
[group('channel')]
agent-channel-blackbox-commands max_wait_secs="25" max_idle_secs="25":
    bash scripts/channel/test-xiuxian-daochang-command-events.sh --max-wait "{{ max_wait_secs }}" --max-idle-secs "{{ max_idle_secs }}"

# Run strict Discord ingress ACL black-box probes (managed command permission-denied paths).
# Usage:
#   just agent-channel-discord-acl [max_wait_secs] [max_idle_secs] [channel_id] [user_id] [guild_id]
# Optional env: OMNI_DISCORD_INGRESS_URL, OMNI_TEST_DISCORD_CHANNEL_ID, OMNI_TEST_DISCORD_USER_ID,

# OMNI_TEST_DISCORD_GUILD_ID, OMNI_TEST_DISCORD_INGRESS_SECRET
[group('channel')]
agent-channel-discord-acl max_wait_secs="25" max_idle_secs="25" channel_id="" user_id="" guild_id="":
    #!/usr/bin/env bash
    set -euo pipefail
    args=(--max-wait "{{ max_wait_secs }}" --max-idle-secs "{{ max_idle_secs }}")
    if [ -n "{{ channel_id }}" ]; then
      args+=(--channel-id "{{ channel_id }}")
    fi
    if [ -n "{{ user_id }}" ]; then
      args+=(--user-id "{{ user_id }}")
    fi
    if [ -n "{{ guild_id }}" ]; then
      args+=(--guild-id "{{ guild_id }}")
    fi
    bash scripts/channel/test-xiuxian-daochang-discord-acl-events.sh "${args[@]}"

# Stress Discord ingress with concurrent synthetic events and queue-pressure telemetry.
# Usage:
#   just agent-channel-discord-ingress-stress
#   just agent-channel-discord-ingress-stress 6 1 8 20 10 0.2 "" "2001" "1001" "3001"
# Reports:
#   .run/reports/xiuxian-daochang-discord-ingress-stress.json

# .run/reports/xiuxian-daochang-discord-ingress-stress.md
[group('channel')]
agent-channel-discord-ingress-stress rounds="6" warmup_rounds="1" parallel="8" requests_per_worker="20" timeout_secs="10" cooldown_secs="0.2" ingress_url="" channel_id="" user_id="" guild_id="" username="" secret_token="" quality_max_failure_rate="0.0" quality_max_p95_ms="0" quality_min_rps="0" output_json=".run/reports/xiuxian-daochang-discord-ingress-stress.json" output_markdown=".run/reports/xiuxian-daochang-discord-ingress-stress.md":
    #!/usr/bin/env bash
    set -euo pipefail
    args=(--rounds "{{ rounds }}" --warmup-rounds "{{ warmup_rounds }}" --parallel "{{ parallel }}" --requests-per-worker "{{ requests_per_worker }}" --timeout-secs "{{ timeout_secs }}" --cooldown-secs "{{ cooldown_secs }}" --quality-max-failure-rate "{{ quality_max_failure_rate }}" --quality-max-p95-ms "{{ quality_max_p95_ms }}" --quality-min-rps "{{ quality_min_rps }}" --output-json "{{ output_json }}" --output-markdown "{{ output_markdown }}")
    if [ -n "{{ ingress_url }}" ]; then
      args+=(--ingress-url "{{ ingress_url }}")
    fi
    if [ -n "{{ channel_id }}" ]; then
      args+=(--channel-id "{{ channel_id }}")
    fi
    if [ -n "{{ user_id }}" ]; then
      args+=(--user-id "{{ user_id }}")
    fi
    if [ -n "{{ guild_id }}" ]; then
      args+=(--guild-id "{{ guild_id }}")
    fi
    if [ -n "{{ username }}" ]; then
      args+=(--username "{{ username }}")
    fi
    if [ -n "{{ secret_token }}" ]; then
      args+=(--secret-token "{{ secret_token }}")
    fi
    bash scripts/channel/test-xiuxian-daochang-discord-ingress-stress.sh "${args[@]}"

# Run dedup black-box probe by posting the same update_id twice and asserting accepted/duplicate events.
# Usage: just agent-channel-blackbox-dedup [max_wait_secs]

# Optional env: OMNI_TEST_CHAT_ID, OMNI_TEST_USER_ID, OMNI_TEST_USERNAME, OMNI_WEBHOOK_URL
[group('channel')]
agent-channel-blackbox-dedup max_wait_secs="25":
    bash scripts/channel/test-xiuxian-daochang-dedup-events.sh --max-wait "{{ max_wait_secs }}"

# Run concurrent dual-session black-box probe (same chat, different users).
# Usage: just agent-channel-blackbox-concurrent [max_wait_secs]

# Optional env: OMNI_TEST_CHAT_ID, OMNI_TEST_USER_ID, OMNI_TEST_USERNAME, OMNI_WEBHOOK_URL
[group('channel')]
agent-channel-blackbox-concurrent max_wait_secs="30":
    bash scripts/channel/test-xiuxian-daochang-concurrent-sessions.sh --max-wait "{{ max_wait_secs }}"

# Capture and persist Telegram test-group mappings (for Test1/Test2/Test3 workflows).
# Usage:
#   just agent-channel-capture-groups
#   just agent-channel-capture-groups "Test1,Test2,Test3"
# Outputs:
#   .run/config/agent-channel-groups.json

# .run/config/agent-channel-groups.env
[group('channel')]
agent-channel-capture-groups titles="Test1,Test2,Test3" log_file=".run/logs/xiuxian-daochang-webhook.log" output_json=".run/config/agent-channel-groups.json" output_env=".run/config/agent-channel-groups.env" user_id="":
    #!/usr/bin/env bash
    set -euo pipefail
    args=(--titles "{{ titles }}" --log-file "{{ log_file }}" --output-json "{{ output_json }}" --output-env "{{ output_env }}")
    if [ -n "{{ user_id }}" ]; then
      args+=(--user-id "{{ user_id }}")
    fi
    python3 scripts/channel/capture_telegram_group_profile.py "${args[@]}"

# Run end-to-end channel acceptance pipeline and emit one summary report.
# Pipeline:
#   capture-groups -> commands -> dedup -> concurrent -> matrix -> complex -> memory-evolution
# Usage:
#   just agent-channel-acceptance
# Reports:
#   .run/reports/agent-channel-acceptance.json

# .run/reports/agent-channel-acceptance.md
[group('channel')]
agent-channel-acceptance max_wait_secs="40" max_idle_secs="25" evolution_max_wait_secs="90" evolution_max_idle_secs="60" evolution_max_parallel="4" titles="Test1,Test2,Test3" log_file=".run/logs/xiuxian-daochang-webhook.log" output_json=".run/reports/agent-channel-acceptance.json" output_markdown=".run/reports/agent-channel-acceptance.md" retries="2":
    bash scripts/channel/agent-channel-acceptance.sh "{{ max_wait_secs }}" "{{ max_idle_secs }}" "{{ evolution_max_wait_secs }}" "{{ evolution_max_idle_secs }}" "{{ evolution_max_parallel }}" "{{ titles }}" "{{ log_file }}" "{{ output_json }}" "{{ output_markdown }}" "{{ retries }}"

# Run session isolation matrix (concurrent baseline + cross reset/resume validation).
# Usage: just agent-channel-blackbox-matrix [max_wait_secs] [max_idle_secs]
# Advanced:
#   just agent-channel-blackbox-matrix 35 25 "-1002000000001" "-1002000000002" "1001" "1002" "1304799692" "1304799693" "Please reply ok"
# Reports:
#   .run/reports/agent-channel-session-matrix.json

# .run/reports/agent-channel-session-matrix.md
[group('channel')]
agent-channel-blackbox-matrix max_wait_secs="35" max_idle_secs="25" chat_b="" chat_c="" thread_b="" thread_c="" user_b="" user_c="" mixed_plain_prompt="":
    #!/usr/bin/env bash
    set -euo pipefail
    args=(--max-wait "{{ max_wait_secs }}" --max-idle-secs "{{ max_idle_secs }}")
    if [ -n "{{ chat_b }}" ]; then
      args+=(--chat-b "{{ chat_b }}")
    fi
    if [ -n "{{ chat_c }}" ]; then
      args+=(--chat-c "{{ chat_c }}")
    fi
    if [ -n "{{ thread_b }}" ]; then
      args+=(--thread-b "{{ thread_b }}")
    fi
    if [ -n "{{ thread_c }}" ]; then
      args+=(--thread-c "{{ thread_c }}")
    fi
    if [ -n "{{ user_b }}" ]; then
      args+=(--user-b "{{ user_b }}")
    fi
    if [ -n "{{ user_c }}" ]; then
      args+=(--user-c "{{ user_c }}")
    fi
    if [ -n "{{ mixed_plain_prompt }}" ]; then
      args+=(--mixed-plain-prompt "{{ mixed_plain_prompt }}")
    fi
    bash scripts/channel/test-xiuxian-daochang-session-matrix.sh "${args[@]}"

# Run complex workflow black-box scenarios with dependency-graph complexity gates.
# Complexity is evaluated by workflow structure:
#   - step count
#   - dependency edges
#   - critical path length
#   - parallel wave count
# Usage:
#   just agent-channel-blackbox-complex
#   just agent-channel-blackbox-complex "scripts/channel/fixtures/complex_blackbox_scenarios.json" "" 40 30 4 14 14 6 3
# Reports:
#   .run/reports/agent-channel-complex-scenarios.json

# .run/reports/agent-channel-complex-scenarios.md
[group('channel')]
agent-channel-blackbox-complex dataset="scripts/channel/fixtures/complex_blackbox_scenarios.json" scenario="" max_wait_secs="40" max_idle_secs="30" max_parallel="4" execute_wave_parallel="false" min_steps="14" min_dependency_edges="14" min_critical_path="6" min_parallel_waves="3" output_json=".run/reports/agent-channel-complex-scenarios.json" output_markdown=".run/reports/agent-channel-complex-scenarios.md":
    #!/usr/bin/env bash
    set -euo pipefail
    args=(--dataset "{{ dataset }}" --max-wait "{{ max_wait_secs }}" --max-idle-secs "{{ max_idle_secs }}" --max-parallel "{{ max_parallel }}" --min-steps "{{ min_steps }}" --min-dependency-edges "{{ min_dependency_edges }}" --min-critical-path "{{ min_critical_path }}" --min-parallel-waves "{{ min_parallel_waves }}" --min-error-signals "0" --min-negative-feedback-events "0" --min-correction-checks "0" --min-successful-corrections "0" --min-planned-hits "0" --min-natural-language-steps "0" --output-json "{{ output_json }}" --output-markdown "{{ output_markdown }}")
    if [ -n "{{ scenario }}" ]; then
      args+=(--scenario "{{ scenario }}")
    fi
    if [ "{{ execute_wave_parallel }}" = "true" ]; then
      args+=(--execute-wave-parallel)
    fi
    bash scripts/channel/test-xiuxian-daochang-complex-scenarios.sh "${args[@]}"

# Run behavior-first memory evolution / self-correction black-box scenario.
# This suite validates:
#   - corrected memory persists across delayed turns
#   - feedback updates are observed in runtime logs
#   - cross-session distractors do not pollute target session memory
# Usage:
#   just agent-channel-blackbox-memory-evolution
# Reports:
#   .run/reports/agent-channel-memory-evolution.json

# .run/reports/agent-channel-memory-evolution.md
[group('channel')]
agent-channel-blackbox-memory-evolution scenario="memory_self_correction_high_complexity_dag" max_wait_secs="80" max_idle_secs="60" max_parallel="4" execute_wave_parallel="false" output_json=".run/reports/agent-channel-memory-evolution.json" output_markdown=".run/reports/agent-channel-memory-evolution.md":
    #!/usr/bin/env bash
    set -euo pipefail
    args=(--dataset "scripts/channel/fixtures/memory_evolution_complex_scenarios.json" --scenario "{{ scenario }}" --max-wait "{{ max_wait_secs }}" --max-idle-secs "{{ max_idle_secs }}" --max-parallel "{{ max_parallel }}" --output-json "{{ output_json }}" --output-markdown "{{ output_markdown }}")
    if [ "{{ execute_wave_parallel }}" = "true" ]; then
      args+=(--execute-wave-parallel)
    fi
    bash scripts/channel/test-xiuxian-daochang-complex-scenarios.sh "${args[@]}"

# Run memory-focused black-box + regression suite.
# Usage:
#   just test-xiuxian-daochang-memory-suite

# just test-xiuxian-daochang-memory-suite full 30 30 tao3k true true "<valkey_url>" false false false scripts/channel/fixtures/memory_evolution_complex_scenarios.json memory_self_correction_high_complexity_dag 1 "" ""
[group('channel')]
test-xiuxian-daochang-memory-suite suite="quick" max_wait_secs="25" max_idle_secs="25" username="" require_live_turn="false" with_valkey="false" valkey_url="" skip_blackbox="false" skip_rust="false" skip_evolution="false" evolution_dataset="scripts/channel/fixtures/memory_evolution_complex_scenarios.json" evolution_scenario="memory_self_correction_high_complexity_dag" evolution_max_parallel="1" evolution_output_json="" evolution_output_markdown="":
    #!/usr/bin/env bash
    set -euo pipefail
    args=(--suite "{{ suite }}" --max-wait "{{ max_wait_secs }}" --max-idle-secs "{{ max_idle_secs }}")
    if [ -n "{{ username }}" ]; then
      args+=(--username "{{ username }}")
    fi
    if [ "{{ require_live_turn }}" = "true" ]; then
      args+=(--require-live-turn)
    fi
    if [ "{{ with_valkey }}" = "true" ]; then
      resolved_valkey_url="{{ valkey_url }}"
      if [ -z "$resolved_valkey_url" ]; then
        resolved_valkey_url="$(uv run python scripts/channel/resolve_valkey_endpoint.py --field url)"
      fi
      args+=(--with-valkey --valkey-url "$resolved_valkey_url")
    fi
    if [ "{{ skip_blackbox }}" = "true" ]; then
      args+=(--skip-blackbox)
    fi
    if [ "{{ skip_rust }}" = "true" ]; then
      args+=(--skip-rust)
    fi
    if [ "{{ skip_evolution }}" = "true" ]; then
      args+=(--skip-evolution)
    fi
    if [ -n "{{ evolution_dataset }}" ]; then
      args+=(--evolution-dataset "{{ evolution_dataset }}")
    fi
    if [ -n "{{ evolution_scenario }}" ]; then
      args+=(--evolution-scenario "{{ evolution_scenario }}")
    fi
    if [ -n "{{ evolution_max_parallel }}" ]; then
      args+=(--evolution-max-parallel "{{ evolution_max_parallel }}")
    fi
    if [ -n "{{ evolution_output_json }}" ]; then
      args+=(--evolution-output-json "{{ evolution_output_json }}")
    fi
    if [ -n "{{ evolution_output_markdown }}" ]; then
      args+=(--evolution-output-markdown "{{ evolution_output_markdown }}")
    fi
    bash scripts/channel/test-xiuxian-daochang-memory-suite.sh "${args[@]}"

# Run memory A/B benchmark suite (baseline vs adaptive feedback).
# Usage:
#   just test-xiuxian-daochang-memory-benchmark

# just test-xiuxian-daochang-memory-benchmark baseline 1 60 40 tao3k scripts/channel/fixtures/memory_benchmark_scenarios.json
[group('channel')]
test-xiuxian-daochang-memory-benchmark mode="both" iterations="1" max_wait_secs="40" max_idle_secs="30" username="" dataset="scripts/channel/fixtures/memory_benchmark_scenarios.json" output_json="" output_markdown="" skip_reset="false" fail_on_tool_error="false" feedback_policy="deadband" feedback_down_threshold="0.34":
    #!/usr/bin/env bash
    set -euo pipefail
    args=(--iterations "{{ iterations }}" --max-wait "{{ max_wait_secs }}" --max-idle-secs "{{ max_idle_secs }}" --dataset "{{ dataset }}" --feedback-policy "{{ feedback_policy }}" --feedback-down-threshold "{{ feedback_down_threshold }}")
    if [ -n "{{ username }}" ]; then
      args+=(--username "{{ username }}")
    fi
    if [ "{{ mode }}" = "baseline" ]; then
      args+=(--mode baseline)
    elif [ "{{ mode }}" = "adaptive" ]; then
      args+=(--mode adaptive)
    elif [ "{{ mode }}" != "both" ]; then
      echo "invalid mode: {{ mode }} (expected: both|baseline|adaptive)" >&2
      exit 2
    fi
    if [ -n "{{ output_json }}" ]; then
      args+=(--output-json "{{ output_json }}")
    fi
    if [ -n "{{ output_markdown }}" ]; then
      args+=(--output-markdown "{{ output_markdown }}")
    fi
    if [ "{{ skip_reset }}" = "true" ]; then
      args+=(--skip-reset)
    fi
    if [ "{{ fail_on_tool_error }}" = "true" ]; then
      args+=(--fail-on-tool-error)
    fi
    bash scripts/channel/test-xiuxian-daochang-memory-benchmark.sh "${args[@]}"

# Aggregate evolution + benchmark + session matrix into one SLO gate report.
# Usage:
#   just test-xiuxian-daochang-memory-slo-report

# just test-xiuxian-daochang-memory-slo-report .run/reports/xiuxian-daochang-memory-evolution.json .run/reports/xiuxian-daochang-memory-benchmark.json .run/reports/agent-channel-session-matrix.json .run/logs/xiuxian-daochang-webhook.log true
[group('channel')]
test-xiuxian-daochang-memory-slo-report evolution_report_json=".run/reports/xiuxian-daochang-memory-evolution.json" benchmark_report_json=".run/reports/xiuxian-daochang-memory-benchmark.json" session_matrix_report_json=".run/reports/agent-channel-session-matrix.json" runtime_log_file="" enable_stream_gate="false" output_json=".run/reports/xiuxian-daochang-memory-slo-report.json" output_markdown=".run/reports/xiuxian-daochang-memory-slo-report.md":
    bash scripts/channel/test-xiuxian-daochang-memory-slo-report.sh "{{ evolution_report_json }}" "{{ benchmark_report_json }}" "{{ session_matrix_report_json }}" "{{ runtime_log_file }}" "{{ enable_stream_gate }}" "{{ output_json }}" "{{ output_markdown }}"

# Start local Valkey daemon for webhook dedup / stress tests.

# Usage: just valkey-start [port]
[group('channel')]
valkey-start port="":
    #!/usr/bin/env bash
    set -euo pipefail
    resolved_valkey_port="{{ port }}"
    if [ -z "$resolved_valkey_port" ]; then
      resolved_valkey_port="$(uv run python scripts/channel/resolve_valkey_endpoint.py --field port)"
    fi
    bash scripts/channel/valkey-start.sh "$resolved_valkey_port"

# Stop local Valkey daemon started by `just valkey-start`.

# Usage: just valkey-stop [port]
[group('channel')]
valkey-stop port="":
    #!/usr/bin/env bash
    set -euo pipefail
    resolved_valkey_port="{{ port }}"
    if [ -z "$resolved_valkey_port" ]; then
      resolved_valkey_port="$(uv run python scripts/channel/resolve_valkey_endpoint.py --field port)"
    fi
    bash scripts/channel/valkey-stop.sh "$resolved_valkey_port"

# Show local Valkey status for a given port.

# Usage: just valkey-status [port]
[group('channel')]
valkey-status port="":
    #!/usr/bin/env bash
    set -euo pipefail
    resolved_valkey_port="{{ port }}"
    if [ -z "$resolved_valkey_port" ]; then
      resolved_valkey_port="$(uv run python scripts/channel/resolve_valkey_endpoint.py --field port)"
    fi
    bash scripts/channel/valkey-status.sh "$resolved_valkey_port"

# Run ignored xiuxian-daochang stress tests that require live Valkey.

# Usage: just test-xiuxian-daochang-valkey-stress [valkey_url]
[group('channel')]
test-xiuxian-daochang-valkey-stress valkey_url="":
    #!/usr/bin/env bash
    set -euo pipefail
    resolved_valkey_url="{{ valkey_url }}"
    if [ -z "$resolved_valkey_url" ]; then
      resolved_valkey_url="$(uv run python scripts/channel/resolve_valkey_endpoint.py --field url)"
    fi
    uv run python scripts/channel/test_xiuxian_daochang_valkey_suite.py --suite stress "${resolved_valkey_url}"

# Run focused distributed SessionGate verification against live Valkey.

# Usage: just test-xiuxian-daochang-valkey-session-gate [valkey_url]
[group('channel')]
test-xiuxian-daochang-valkey-session-gate valkey_url="":
    #!/usr/bin/env bash
    set -euo pipefail
    resolved_valkey_url="{{ valkey_url }}"
    if [ -z "$resolved_valkey_url" ]; then
      resolved_valkey_url="$(uv run python scripts/channel/resolve_valkey_endpoint.py --field url)"
    fi
    uv run python scripts/channel/test_xiuxian_daochang_valkey_suite.py --suite session-gate "${resolved_valkey_url}"

# Run focused cross-instance session-context restore verification against live Valkey.

# Usage: just test-xiuxian-daochang-valkey-session-context [valkey_url]
[group('channel')]
test-xiuxian-daochang-valkey-session-context valkey_url="":
    #!/usr/bin/env bash
    set -euo pipefail
    resolved_valkey_url="{{ valkey_url }}"
    if [ -z "$resolved_valkey_url" ]; then
      resolved_valkey_url="$(uv run python scripts/channel/resolve_valkey_endpoint.py --field url)"
    fi
    uv run python scripts/channel/test_xiuxian_daochang_valkey_suite.py --suite session-context "${resolved_valkey_url}"

# Run focused multi-HTTP Valkey dedup verification.

# Usage: just test-xiuxian-daochang-valkey-multi-http [valkey_url]
[group('channel')]
test-xiuxian-daochang-valkey-multi-http valkey_url="":
    #!/usr/bin/env bash
    set -euo pipefail
    resolved_valkey_url="{{ valkey_url }}"
    if [ -z "$resolved_valkey_url" ]; then
      resolved_valkey_url="$(uv run python scripts/channel/resolve_valkey_endpoint.py --field url)"
    fi
    uv run python scripts/channel/test_xiuxian_daochang_valkey_suite.py --suite multi-http "${resolved_valkey_url}"

# Run focused multi-process Valkey dedup verification.

# Usage: just test-xiuxian-daochang-valkey-multi-process [valkey_url]
[group('channel')]
test-xiuxian-daochang-valkey-multi-process valkey_url="":
    #!/usr/bin/env bash
    set -euo pipefail
    resolved_valkey_url="{{ valkey_url }}"
    if [ -z "$resolved_valkey_url" ]; then
      resolved_valkey_url="$(uv run python scripts/channel/resolve_valkey_endpoint.py --field url)"
    fi
    uv run python scripts/channel/test_xiuxian_daochang_valkey_suite.py --suite multi-process "${resolved_valkey_url}"

# Run full live Valkey webhook verification suite
# (stress + distributed session gate + session-context + multi-http + multi-process).

# Usage: just test-xiuxian-daochang-valkey-full [valkey_url]
[group('channel')]
test-xiuxian-daochang-valkey-full valkey_url="":
    #!/usr/bin/env bash
    set -euo pipefail
    resolved_valkey_url="{{ valkey_url }}"
    if [ -z "$resolved_valkey_url" ]; then
      resolved_valkey_url="$(uv run python scripts/channel/resolve_valkey_endpoint.py --field url)"
    fi
    uv run python scripts/channel/test_xiuxian_daochang_valkey_suite.py --suite full "${resolved_valkey_url}"

# Validate observability event sequence from a captured agent log file.
# Usage:
#   just check-xiuxian-daochang-event-sequence <log_file>

# just check-xiuxian-daochang-event-sequence <log_file> true true valkey
[group('channel')]
check-xiuxian-daochang-event-sequence log_file strict="false" require_memory="false" expect_memory_backend="":
    #!/usr/bin/env bash
    set -euo pipefail
    args=()
    if [ "{{ strict }}" = "true" ]; then
      args+=(--strict)
    fi
    if [ "{{ require_memory }}" = "true" ]; then
      args+=(--require-memory)
    fi
    if [ -n "{{ expect_memory_backend }}" ]; then
      args+=(--expect-memory-backend "{{ expect_memory_backend }}")
    fi
    uv run python scripts/channel/check_xiuxian_daochang_event_sequence.py "{{ log_file }}" "${args[@]}"

# ==============================================================================
# RUST BUILD
# ==============================================================================

[group('rust')]
build-rust:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "🔨 Building Rust core library..."
    cd packages/rust/bindings/python

    # Build wheel (faster than maturin develop)
    echo "📦 Creating wheel..."
    maturin build --release

    # Find and install the wheel
    WHEEL_PATH=$(find ../../target/wheels -name "*.whl" 2>/dev/null | head -1)
    if [ -z "$WHEEL_PATH" ]; then
        echo "❌ Error: Could not find built wheel"
        exit 1
    fi

    echo "📦 Installing wheel: $WHEEL_PATH"
    uv pip install --force-reinstall --no-deps "$WHEEL_PATH"

    echo "✅ Rust library installed to venv"

[group('rust')]
build-rust-dev:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "🔨 Building Rust core library (DEBUG mode - fast)..."
    root_dir="$(git rev-parse --show-toplevel)"
    cd packages/rust/bindings/python

    # maturin develop uses debug build by default (much faster than release)
    # First cargo build, then install
    "${root_dir}/scripts/rust/cargo_exec.sh" build && maturin develop

    echo "✅ Rust debug library installed to venv"

[group('rust')]
build-rust-wheel:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "📦 Building Rust wheel (no recompile)..."
    cd packages/rust/bindings/python

    # Only build wheel (assumes cargo build --release already done)
    maturin build --release --skip-auditwheel

    # Find and show the wheel
    WHEEL_PATH=$(find ../../target/wheels -name "*.whl" 2>/dev/null | head -1)
    if [ -n "$WHEEL_PATH" ]; then
        echo "✅ Wheel: $WHEEL_PATH"
        ls -lh "$WHEEL_PATH"
    else
        echo "❌ Error: Could not find built wheel"
        exit 1
    fi

# Run Qianji JS Aesthetic Evolution Test (PaperBanana Soul)
test-qianji-evolution:
    @echo "🚀 Initiating Sovereign Evolution Test..."
    @python3 scripts/research/simulate_evolution.py

# ==============================================================================
# Qianji Studio (Thousand Mechanisms Studio)
# ==============================================================================

# Launch the high-performance React + Rspack laboratory for visual evolution.
studio:
    @echo "🚀 Launching Qianji Sovereign Studio (Rspack + React + TS)..."
    @bash scripts/channel/wendao-frontend-launch.sh

# Run REAL LLM evolution test for Qianji JS (requires OPENAI_API_KEY).

# Usage: just test-evolution "Your visual intent"
test-evolution intent="Forge ultimate academic topology for a multi-agent system.":
    @echo "🌀 Initiating REAL Evolution Forge..."
    export OPENAI_API_KEY=$OPENAI_API_KEY && cargo run -p xiuxian-qianji --bin qianji --features llm -- . packages/rust/crates/xiuxian-qianji/resources/omega_react_paper_banana.toml '{"User_Intent": "{{ intent }}"}'
