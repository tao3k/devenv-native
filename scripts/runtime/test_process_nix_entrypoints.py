from __future__ import annotations

import os
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[2]
PROCESS_NIX = PROJECT_ROOT / "nix/modules/process.nix"
PROCESS_ROOT = PROJECT_ROOT / "scripts/runtime/processes"

ENTRYPOINT_PROCESSES = (
    "carfox",
    "valkey",
    "wendao-analyzer",
    "wendao-frontend",
    "wendao-gateway",
    "wendao-sentinel",
    "wendaocodeparser-parser-summary",
    "wendaosearch-solver-demo",
)

HEALTHCHECK_PROCESSES = (
    "valkey",
    "wendao-analyzer",
    "wendao-frontend",
    "wendao-gateway",
    "wendao-sentinel",
    "wendaocodeparser-parser-summary",
    "wendaosearch-solver-demo",
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


def test_wendao_gateway_entrypoint_uses_auto_build_mode() -> None:
    entrypoint = (PROCESS_ROOT / "wendao-gateway" / "entrypoint.sh").read_text(encoding="utf-8")

    assert 'BUILD_MODE="${WENDAO_GATEWAY_BUILD:-auto}"' in entrypoint
    assert 'auto|"")' in entrypoint
    assert "build_wendao_gateway" in entrypoint
    assert "if command -v cargo >/dev/null 2>&1; then\n      build_wendao_gateway" in entrypoint


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


def test_process_nix_exposes_code_parser_summary_service() -> None:
    process_nix = PROCESS_NIX.read_text(encoding="utf-8")

    assert "wendaocodeparser-parser-summary = {" in process_nix
    assert 'exec = processEntrypoint "wendaocodeparser-parser-summary";' in process_nix
    assert 'exec.command = processHealthcheck "wendaocodeparser-parser-summary";' in process_nix
    assert "wendaosearch-parser-summary = {" not in process_nix


def test_code_parser_summary_entrypoint_targets_wendaocodeparser_runtime() -> None:
    entrypoint = (PROCESS_ROOT / "wendaocodeparser-parser-summary" / "entrypoint.sh").read_text(
        encoding="utf-8"
    )
    wendaosearch_entrypoint = (PROCESS_ROOT / "wendaosearch" / "entrypoint.sh").read_text(
        encoding="utf-8"
    )

    assert "WENDAO_CODE_PARSER_PACKAGE_DIR" in entrypoint
    assert "WendaoCodeParser.jl" in entrypoint
    assert "run_service.jl" in entrypoint
    assert "WENDAOSEARCH_JULIA_PROJECT" in entrypoint
    assert "WENDAO_SEARCH_USE_ACTIVE_PROJECT" in entrypoint
    assert "run_parser_summary_service.jl" not in entrypoint
    assert "WendaoSearch.jl/config/live/parser_summary.toml" not in entrypoint
    assert 'JULIA_PROJECT="${WENDAOSEARCH_JULIA_PROJECT:-$PACKAGE_DIR}"' in (
        wendaosearch_entrypoint
    )
    assert 'SCRIPT_PATH="$(process_abs_path "$PACKAGE_DIR/scripts" "$SCRIPT_NAME")"' in (
        wendaosearch_entrypoint
    )
