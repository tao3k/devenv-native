from __future__ import annotations

from xiuxian_wendao_analyzer.audio_shard_worker_config import (
    AUDIO_HOSTED_MODEL_ENV,
    AUDIO_HOSTED_PROVIDER_ENV,
)
from xiuxian_wendao_analyzer.audio_shard_worker_registry import UnsupportedAudioShardWorker
from xiuxian_wendao_analyzer.document_service_headers import route_decision_headers
from xiuxian_wendao_analyzer.document_service_routes import (
    WENDAO_ROUTE_ID_HEADER,
    WENDAO_ROUTE_MODALITY_HEADER,
    WENDAO_ROUTE_SELECTED_BACKEND_PROFILE_HEADER,
    WENDAO_ROUTE_SELECTED_MODEL_HEADER,
    WENDAO_ROUTE_SELECTED_PROVIDER_HEADER,
    WENDAO_ROUTE_TASK_KIND_HEADER,
)
from xiuxian_wendao_analyzer.document_service_server import (
    DocumentExtractFlightServer,
    hosted_audio_overrides_from_headers,
    hosted_vlm_image_config_from_headers,
)
from xiuxian_wendao_analyzer.pdf_ocr_contracts import (
    HOSTED_VLM_OCR_OPENROUTER_BASE_URL,
)


def test_route_decision_headers_are_read_from_gateway_metadata() -> None:
    headers = {
        WENDAO_ROUTE_ID_HEADER: "route-audio-1",
        WENDAO_ROUTE_TASK_KIND_HEADER: "attachment-extract",
        WENDAO_ROUTE_MODALITY_HEADER: "audio",
        WENDAO_ROUTE_SELECTED_PROVIDER_HEADER: "openrouter",
        WENDAO_ROUTE_SELECTED_MODEL_HEADER: "qwen/qwen3-asr-flash-2026-02-10",
        WENDAO_ROUTE_SELECTED_BACKEND_PROFILE_HEADER: "hosted-audio-transcript-v1",
    }

    decision = route_decision_headers(headers)

    assert decision[WENDAO_ROUTE_ID_HEADER] == "route-audio-1"
    assert decision[WENDAO_ROUTE_SELECTED_PROVIDER_HEADER] == "openrouter"
    assert decision[WENDAO_ROUTE_SELECTED_MODEL_HEADER] == "qwen/qwen3-asr-flash-2026-02-10"
    assert decision[WENDAO_ROUTE_SELECTED_BACKEND_PROFILE_HEADER] == "hosted-audio-transcript-v1"


def test_hosted_audio_overrides_prefer_gateway_route_decision() -> None:
    headers = {
        WENDAO_ROUTE_SELECTED_BACKEND_PROFILE_HEADER: "hosted-audio-transcript-v1",
        WENDAO_ROUTE_SELECTED_PROVIDER_HEADER: "openrouter",
        WENDAO_ROUTE_SELECTED_MODEL_HEADER: "qwen/qwen3-asr-flash-2026-02-10",
    }

    overrides = hosted_audio_overrides_from_headers(headers)

    assert overrides == {
        AUDIO_HOSTED_PROVIDER_ENV: "openrouter",
        AUDIO_HOSTED_MODEL_ENV: "qwen/qwen3-asr-flash-2026-02-10",
    }


def test_missing_selected_backend_returns_failed_worker() -> None:
    server = DocumentExtractFlightServer(location="grpc://0.0.0.0:0")

    worker = server._audio_worker_for_headers(  # noqa: SLF001
        {
            WENDAO_ROUTE_ID_HEADER: "route-audio-1",
            WENDAO_ROUTE_SELECTED_PROVIDER_HEADER: "openrouter",
            WENDAO_ROUTE_SELECTED_MODEL_HEADER: "qwen/qwen3-asr-flash-2026-02-10",
        }
    )

    assert isinstance(worker, UnsupportedAudioShardWorker)


def test_hosted_vlm_image_config_uses_gateway_route_decision(monkeypatch) -> None:
    monkeypatch.setenv("OPENROUTER_API_KEY", "route-key")
    headers = {
        WENDAO_ROUTE_SELECTED_BACKEND_PROFILE_HEADER: "hosted-vlm-image-extract-v1",
        WENDAO_ROUTE_SELECTED_PROVIDER_HEADER: "openrouter",
        WENDAO_ROUTE_SELECTED_MODEL_HEADER: "qwen/qwen3-vl-8b-instruct",
    }

    config = hosted_vlm_image_config_from_headers(headers)

    assert config is not None
    assert config.base_url == HOSTED_VLM_OCR_OPENROUTER_BASE_URL
    assert config.model == "qwen/qwen3-vl-8b-instruct"
    assert config.api_key == "route-key"


def test_hosted_vlm_image_config_ignores_non_image_backend() -> None:
    assert (
        hosted_vlm_image_config_from_headers(
            {
                WENDAO_ROUTE_SELECTED_BACKEND_PROFILE_HEADER: "hosted-audio-transcript-v1",
                WENDAO_ROUTE_SELECTED_PROVIDER_HEADER: "openrouter",
                WENDAO_ROUTE_SELECTED_MODEL_HEADER: "qwen/qwen3-asr-flash-2026-02-10",
            }
        )
        is None
    )
