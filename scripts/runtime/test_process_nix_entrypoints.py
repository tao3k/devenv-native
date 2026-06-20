from __future__ import annotations

import os
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[2]
PROCESS_NIX = PROJECT_ROOT / "nix/modules/process.nix"
PROCESS_ROOT = PROJECT_ROOT / "scripts/runtime/processes"

ENTRYPOINT_PROCESSES = (
    "carfox",
    "qianji-server",
    "valkey",
    "vllm-sr",
    "wendao-analyzer",
    "wendao-ai",
    "wendao-frontend",
    "wendao-gateway",
    "wendao-sentinel",
    "wendao-semantic-refresh",
)

HEALTHCHECK_PROCESSES = (
    "qianji-server",
    "valkey",
    "vllm-sr",
    "wendao-analyzer",
    "wendao-ai",
    "wendao-frontend",
    "wendao-gateway",
    "wendao-sentinel",
    "wendao-semantic-refresh",
)


def test_process_nix_delegates_startup_to_process_entrypoints() -> None:
    process_nix = PROCESS_NIX.read_text(encoding="utf-8")

    assert (
        'exec bash "$ROOT_DIR/scripts/runtime/processes/${processName}/${scriptName}.sh"'
        in process_nix
    )
    assert "wendao-audio-local-backend = {" not in process_nix
    for forbidden in (
        ".run",
        ".data",
        "127.0.0.1",
        "6379",
        "9518",
        "WENDAO_",
        "WENDAOSEARCH_",
        "VALKEY_",
        "wendao.toml",
        "WendaoSearch.jl",
        "WendaoCodeParser.jl",
    ):
        assert forbidden not in process_nix


def test_process_entrypoints_are_owned_by_process_directories() -> None:
    for process_name in ENTRYPOINT_PROCESSES:
        entrypoint = PROCESS_ROOT / process_name / "entrypoint.sh"

        assert entrypoint.is_file()
        assert os.access(entrypoint, os.X_OK)


def test_process_healthchecks_are_owned_by_process_directories() -> None:
    for process_name in HEALTHCHECK_PROCESSES:
        healthcheck = PROCESS_ROOT / process_name / "healthcheck.sh"

        assert healthcheck.is_file()
        assert os.access(healthcheck, os.X_OK)


def test_vllm_sr_entrypoint_preflights_required_runtime() -> None:
    entrypoint = (PROCESS_ROOT / "vllm-sr" / "entrypoint.sh").read_text(encoding="utf-8")

    assert 'TARGET="${WENDAO_VLLM_SR_TARGET:-docker}"' in entrypoint
    assert "block_vllm_sr_infra()" in entrypoint
    assert "while :; do sleep 3600; done" in entrypoint
    assert "vLLM-SR local Docker target requires Docker" in entrypoint
    assert "docker info" in entrypoint
    assert 'vllm-sr validate --config "$CONFIG_PATH"' in entrypoint
    assert 'SERVE_ARGS=(serve --config "$CONFIG_PATH" --target "$TARGET"' in entrypoint
    assert "WENDAO_VLLM_SR_MINIMAL" in entrypoint


def test_wendao_analyzer_launch_cleans_legacy_document_extract_listener() -> None:
    launch_script = (PROJECT_ROOT / "scripts/runtime/wendao-analyzer-launch.sh").read_text(
        encoding="utf-8"
    )

    assert "cleanup_analyzer_listener" in launch_script
    assert "wendao-document-extract" in launch_script
    assert "xiuxian_wendao_analyzer" in launch_script
    assert "DocumentExtractFlightServer" in launch_script


def test_wendao_analyzer_launch_enables_full_attachment_workers() -> None:
    launch_script = (PROJECT_ROOT / "scripts/runtime/wendao-analyzer-launch.sh").read_text(
        encoding="utf-8"
    )

    assert 'PDF_OCR_WORKER="${WENDAO_PDF_OCR_WORKER:-docling}"' in launch_script
    assert 'AUDIO_WORKER="${WENDAO_AUDIO_WORKER:-hosted}"' in launch_script
    assert (
        'export WENDAO_AUDIO_HOSTED_PROVIDER="${WENDAO_AUDIO_HOSTED_PROVIDER:-openrouter}"'
        in launch_script
    )
    assert (
        'export WENDAO_AUDIO_HOSTED_MODEL="${WENDAO_AUDIO_HOSTED_MODEL:-qwen/qwen3-asr-flash-2026-02-10}"'
        in launch_script
    )
    assert (
        'export WENDAO_AUDIO_HOSTED_ENDPOINT="${WENDAO_AUDIO_HOSTED_ENDPOINT:-audio-transcriptions}"'
        in launch_script
    )
    assert "wendao-analyzer.hosted-audio.jsonl" in launch_script
    assert "WENDAO_AUDIO_HOSTED_REQUEST_CONCURRENCY:-4" not in launch_script
    assert 'AUDIO_WORKER="docling"' not in launch_script
    assert "--extra documents-audio" in launch_script
    assert '--pdf-ocr-worker "$PDF_OCR_WORKER"' in launch_script
    assert '--audio-worker "$AUDIO_WORKER"' in launch_script
    assert 'AUDIO_LOCAL_BACKEND="${WENDAO_AUDIO_LOCAL_BACKEND:-auto}"' in launch_script
    assert 'AUDIO_LOCAL_BACKEND_RUNNER="${WENDAO_AUDIO_BACKEND_RUNNER:-qwen3-asr-mlx}"' in (
        launch_script
    )
    assert "start_audio_local_backend" in launch_script
    assert "--audio-start-backend" in launch_script
    assert "qwen3_asr_mlx_openai_adapter.py" in launch_script


def test_wendao_gateway_entrypoint_uses_production_flight_timeout_budget() -> None:
    entrypoint = (PROCESS_ROOT / "wendao-gateway" / "entrypoint.sh").read_text(encoding="utf-8")

    assert "XIUXIAN_WENDAO_GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS:-600" in entrypoint


def test_wendao_gateway_entrypoint_sets_attachment_route_defaults() -> None:
    entrypoint = (PROCESS_ROOT / "wendao-gateway" / "entrypoint.sh").read_text(encoding="utf-8")
    audio_provider_export = (
        'export WENDAO_AUDIO_TRANSCRIPT_ROUTE_PROVIDER="'
        "${WENDAO_AUDIO_TRANSCRIPT_ROUTE_PROVIDER:-"
        '${WENDAO_VLLM_SR_DEFAULT_PROVIDER:-openrouter}}"'
    )
    image_provider_export = (
        'export WENDAO_IMAGE_EXTRACT_ROUTE_PROVIDER="'
        "${WENDAO_IMAGE_EXTRACT_ROUTE_PROVIDER:-"
        '${WENDAO_VLLM_SR_DEFAULT_PROVIDER:-openrouter}}"'
    )

    assert audio_provider_export in entrypoint
    assert (
        'export WENDAO_AUDIO_TRANSCRIPT_ROUTE_MODEL="${WENDAO_AUDIO_TRANSCRIPT_ROUTE_MODEL:-qwen/qwen3-asr-flash-2026-02-10}"'
        in entrypoint
    )
    assert image_provider_export in entrypoint
    assert (
        'export WENDAO_IMAGE_EXTRACT_ROUTE_MODEL="${WENDAO_IMAGE_EXTRACT_ROUTE_MODEL:-qwen/qwen3-vl-8b-instruct}"'
        in entrypoint
    )


def test_wendao_gateway_entrypoint_uses_auto_build_mode() -> None:
    entrypoint = (PROCESS_ROOT / "wendao-gateway" / "entrypoint.sh").read_text(encoding="utf-8")

    assert 'BUILD_MODE="${WENDAO_GATEWAY_BUILD:-auto}"' in entrypoint
    assert 'auto|"")' in entrypoint
    assert "build_wendao_gateway" in entrypoint
    assert "if command -v cargo >/dev/null 2>&1; then\n      build_wendao_gateway" in entrypoint


def test_process_healthchecks_have_short_internal_retries() -> None:
    gateway_healthcheck = (
        PROJECT_ROOT / "scripts/runtime/wendao-gateway-healthcheck.sh"
    ).read_text(encoding="utf-8")
    analyzer_healthcheck = (
        PROJECT_ROOT / "scripts/runtime/wendao-analyzer-healthcheck.sh"
    ).read_text(encoding="utf-8")

    assert 'TIMEOUT_SECS="${WENDAO_GATEWAY_HEALTH_TIMEOUT_SECS:-0.5}"' in gateway_healthcheck
    assert 'ATTEMPTS="${WENDAO_GATEWAY_HEALTH_ATTEMPTS:-3}"' in gateway_healthcheck
    assert '--attempts "$ATTEMPTS"' in gateway_healthcheck
    assert '--retry-delay-secs "$RETRY_DELAY_SECS"' in gateway_healthcheck
    assert 'TIMEOUT_SECS="${WENDAO_ANALYZER_HEALTH_TIMEOUT_SECS:-0.5}"' in analyzer_healthcheck
    assert 'ATTEMPTS="${WENDAO_ANALYZER_HEALTH_ATTEMPTS:-3}"' in analyzer_healthcheck
    assert "after {attempts} attempt(s)" in analyzer_healthcheck


def test_qianji_server_healthcheck_rejects_missing_llm_worker_route() -> None:
    healthcheck = (PROJECT_ROOT / "scripts/runtime/qianji-server-healthcheck.sh").read_text(
        encoding="utf-8"
    )

    assert "/control/runs/__health__/workers/openai-compatible-llm/run-and-complete" in healthcheck
    assert "route_status == 404" in healthcheck
    assert "qianji-server LLM worker route is missing" in healthcheck


def test_wendao_gateway_entrypoint_builds_full_attachment_surface() -> None:
    gateway_entrypoint = (PROCESS_ROOT / "wendao-gateway" / "entrypoint.sh").read_text(
        encoding="utf-8"
    )
    sentinel_entrypoint = (PROCESS_ROOT / "wendao-sentinel" / "entrypoint.sh").read_text(
        encoding="utf-8"
    )
    required_features = (
        "cli-bin-support",
        "zhenfa-router",
        "document-extract-attachment-audit",
        "document-extract-pdf-render",
        "document-extract-audio-shards",
    )

    for feature in required_features:
        assert feature in gateway_entrypoint
        assert feature in sentinel_entrypoint
    assert 'GATEWAY_FEATURES="${WENDAO_GATEWAY_FEATURES:-' in gateway_entrypoint
    assert '--features "$GATEWAY_FEATURES"' in gateway_entrypoint
    assert 'SENTINEL_FEATURES="${WENDAO_SENTINEL_FEATURES:-${WENDAO_GATEWAY_FEATURES:-' in (
        sentinel_entrypoint
    )
    assert '--features "$SENTINEL_FEATURES"' in sentinel_entrypoint


def test_process_nix_does_not_expose_retired_julia_search_processes() -> None:
    process_nix = PROCESS_NIX.read_text(encoding="utf-8")

    assert "wendaocodeparser-parser-summary = {" not in process_nix
    assert "wendaosearch-solver-demo = {" not in process_nix
    assert "wendaosearch-parser-summary = {" not in process_nix


def test_process_nix_exposes_vllm_sr_model_routing_plane() -> None:
    process_nix = PROCESS_NIX.read_text(encoding="utf-8")

    assert "vllm-sr = {" in process_nix
    assert 'exec = processEntrypoint "vllm-sr";' in process_nix
    assert 'exec.command = processHealthcheck "vllm-sr";' in process_nix
    assert 'vllm-sr.condition = "process_healthy";' in process_nix
    assert 'restart = "no";' in process_nix


def test_wendao_ai_process_waits_for_qianji_server() -> None:
    process_nix = PROCESS_NIX.read_text(encoding="utf-8")
    wendao_ai_block = process_nix.split('"wendao-ai" = {', maxsplit=1)[1].split(
        "};\n    };",
        maxsplit=1,
    )[0]

    assert 'qianji-server.condition = "process_healthy";' in wendao_ai_block
    assert 'wendao-gateway.condition = "process_healthy";' in wendao_ai_block


def test_wendao_gateway_readiness_budget_allows_local_cold_builds() -> None:
    process_nix = PROCESS_NIX.read_text(encoding="utf-8")
    gateway_block = process_nix.split("wendao-gateway = {", maxsplit=1)[1].split(
        "};\n    };",
        maxsplit=1,
    )[0]

    assert "initial_delay_seconds = 30;" in gateway_block
    assert "period_seconds = 5;" in gateway_block
    assert "failure_threshold = 120;" in gateway_block


def test_vllm_sr_entrypoint_uses_required_mode_gate() -> None:
    entrypoint = (PROCESS_ROOT / "vllm-sr" / "entrypoint.sh").read_text(encoding="utf-8")
    healthcheck = (PROCESS_ROOT / "vllm-sr" / "healthcheck.sh").read_text(encoding="utf-8")

    assert 'MODE="${WENDAO_MODEL_ROUTING_MODE:-deterministic}"' in entrypoint
    assert "unsupported WENDAO_MODEL_ROUTING_MODE" in entrypoint
    assert 'SERVE_ARGS=(serve --config "$CONFIG_PATH"' in entrypoint
    assert (
        'CONFIG_PATH="${WENDAO_VLLM_SR_CONFIG_PATH:-$PROJECT_CONFIG_ROOT/vllm-sr/config.yaml}"'
        in entrypoint
    )
    assert "Wendao model routing mode is deterministic" in entrypoint
    assert 'MODE="${WENDAO_MODEL_ROUTING_MODE:-deterministic}"' in healthcheck
    assert "socket.create_connection" in healthcheck


def test_wendao_gateway_entrypoint_defaults_to_local_gateway_routing() -> None:
    entrypoint = (PROCESS_ROOT / "wendao-gateway" / "entrypoint.sh").read_text(encoding="utf-8")

    assert 'WENDAO_MODEL_ROUTING_MODE="${WENDAO_MODEL_ROUTING_MODE:-deterministic}"' in entrypoint
    assert (
        'WENDAO_CHAT_ROUTE_MODEL="${WENDAO_CHAT_ROUTE_MODEL:-${WENDAO_VLLM_SR_DEFAULT_MODEL:-deepseek/deepseek-v4-pro}}"'
        in entrypoint
    )
    assert (
        'WENDAO_CHAT_ROUTE_PROVIDER="${WENDAO_CHAT_ROUTE_PROVIDER:-${WENDAO_VLLM_SR_DEFAULT_PROVIDER:-openrouter}}"'
        in entrypoint
    )


def test_retired_julia_search_process_entrypoints_are_removed() -> None:
    for process_name in (
        "wendaocodeparser-parser-summary",
        "wendaosearch-solver-demo",
        "wendaosearch",
    ):
        assert not (PROCESS_ROOT / process_name / "entrypoint.sh").exists()
        assert not (PROCESS_ROOT / process_name / "healthcheck.sh").exists()
