"""Startup log summaries for the document extraction Flight service."""

from __future__ import annotations

import json
import os
from typing import TYPE_CHECKING, Any

from .audio_shard_worker_config import (
    AUDIO_BACKEND_HOSTED_PROFILE,
    AUDIO_HOSTED_API_KEY_ENV,
    AUDIO_HOSTED_BASE_URL_ENV,
    AUDIO_HOSTED_ENDPOINT_ENV,
    AUDIO_HOSTED_MAX_ATTEMPTS_ENV,
    AUDIO_HOSTED_MODEL_ENV,
    AUDIO_HOSTED_OPENROUTER_PUBLIC_API_KEY_ENV,
    AUDIO_HOSTED_PROVIDER_ENV,
    AUDIO_HOSTED_REQUEST_CONCURRENCY_ENV,
    AUDIO_HOSTED_TIMEOUT_SECONDS_ENV,
    AUDIO_HOSTED_TRACE_PATH_ENV,
    AUDIO_PRIMARY_LANGUAGE_ENV,
    AUDIO_TRANSCRIPT_QUALITY_GATE_ENV,
    AUDIO_WORKER_ENV,
    hosted_audio_config_from_env,
)
from .audio_shard_worker_registry import normalize_audio_worker_name
from .document_service_prewarm import (
    DOCUMENT_EXTRACT_PREWARM_PAGE_RANGES_ENV,
    DOCUMENT_EXTRACT_PREWARM_PROFILE_ENV,
    DOCUMENT_EXTRACT_PREWARM_SOURCE_PATH_ENV,
)
from .pdf_ocr_contracts import (
    HOSTED_VLM_OCR_API_KEY_ENV,
    HOSTED_VLM_OCR_BASE_URL_ENV,
    HOSTED_VLM_OCR_DEFAULT_BASE_URL,
    HOSTED_VLM_OCR_DEFAULT_IMAGE_OPTIMIZATION,
    HOSTED_VLM_OCR_DEFAULT_MODEL,
    HOSTED_VLM_OCR_DEFAULT_REGION_ATLAS_MODE,
    HOSTED_VLM_OCR_DEFAULT_REGION_COMPOSITE_MODE,
    HOSTED_VLM_OCR_DEFAULT_REGION_COMPOSITE_SIZE,
    HOSTED_VLM_OCR_DEFAULT_REQUEST_CONCURRENCY,
    HOSTED_VLM_OCR_DEFAULT_SCAFFOLD_MODE,
    HOSTED_VLM_OCR_DEFAULT_TIMEOUT_SECONDS,
    HOSTED_VLM_OCR_IMAGE_OPTIMIZATION_ENV,
    HOSTED_VLM_OCR_MODEL_ENV,
    HOSTED_VLM_OCR_OPENROUTER_API_KEY_ENV,
    HOSTED_VLM_OCR_OPENROUTER_MODEL_ENV,
    HOSTED_VLM_OCR_OPENROUTER_PUBLIC_API_KEY_ENV,
    HOSTED_VLM_OCR_PROVIDER_ENV,
    HOSTED_VLM_OCR_REGION_ATLAS_MODE_ENV,
    HOSTED_VLM_OCR_REGION_COMPOSITE_MODE_ENV,
    HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE_ENV,
    HOSTED_VLM_OCR_REQUEST_CONCURRENCY_ENV,
    HOSTED_VLM_OCR_SCAFFOLD_MODE_ENV,
    HOSTED_VLM_OCR_TIMEOUT_SECONDS_ENV,
    HOSTED_VLM_OCR_TRACE_PATH_ENV,
)

if TYPE_CHECKING:
    import argparse


STARTUP_LOG_SCHEMA = "xiuxian_wendao.analyzer_document_extract_startup.v1"


def document_extract_startup_log_payload(
    args: argparse.Namespace,
    *,
    location: str,
    prewarmed_converter_ready: bool,
) -> dict[str, Any]:
    """Build a redacted startup log payload for service observability."""

    audio_worker = normalize_audio_worker_name(
        str(getattr(args, "audio_worker", None) or os.environ.get(AUDIO_WORKER_ENV) or "")
    )
    return {
        "schema": STARTUP_LOG_SCHEMA,
        "event": "document_extract_service_ready",
        "location": location,
        "routes": [
            "/analysis/document-extract",
            "/analysis/pdf-ocr-shards",
            "/analysis/audio-shards",
        ],
        "pdfOcr": {
            "worker": getattr(args, "pdf_ocr_worker", "skip"),
            "maxWorkers": str(getattr(args, "pdf_ocr_workers", "auto")),
            "hostedVlm": _hosted_vlm_summary(),
        },
        "audio": {
            "worker": audio_worker,
            "maxWorkers": str(getattr(args, "audio_workers", "auto")),
            "hosted": _hosted_audio_summary(audio_worker),
        },
        "prewarm": {
            "enabled": _env_configured(DOCUMENT_EXTRACT_PREWARM_SOURCE_PATH_ENV),
            "sourcePathConfigured": _env_configured(DOCUMENT_EXTRACT_PREWARM_SOURCE_PATH_ENV),
            "pageRanges": os.environ.get(DOCUMENT_EXTRACT_PREWARM_PAGE_RANGES_ENV) or "1:1",
            "profile": os.environ.get(DOCUMENT_EXTRACT_PREWARM_PROFILE_ENV) or "full",
            "converterReady": prewarmed_converter_ready,
        },
    }


def write_document_extract_startup_log(
    stream: Any,
    args: argparse.Namespace,
    *,
    location: str,
    prewarmed_converter_ready: bool,
) -> None:
    """Write a single parseable startup event for process logs."""

    payload = document_extract_startup_log_payload(
        args,
        location=location,
        prewarmed_converter_ready=prewarmed_converter_ready,
    )
    stream.write(f"WENDAO_ANALYZER_STARTUP {json.dumps(payload, sort_keys=True)}\n")
    stream.flush()


def _hosted_audio_summary(audio_worker: str) -> dict[str, Any]:
    configured = _audio_hosted_env_summary()
    if audio_worker != AUDIO_BACKEND_HOSTED_PROFILE:
        configured["active"] = False
        return configured
    configured["active"] = True
    try:
        config = hosted_audio_config_from_env()
    except ValueError as exc:
        configured["configError"] = str(exc)
        return configured
    configured.update(
        {
            "provider": config.provider,
            "baseUrl": config.base_url,
            "endpoint": config.endpoint,
            "model": config.model,
            "timeoutSeconds": config.timeout_seconds,
            "requestConcurrency": config.request_concurrency,
            "maxAttempts": config.max_attempts,
            "primaryLanguage": config.primary_language,
            "tracePathConfigured": config.trace_path is not None,
            "qualityGateEnabled": config.quality_options.enabled,
        }
    )
    return configured


def _audio_hosted_env_summary() -> dict[str, Any]:
    return {
        "provider": os.environ.get(AUDIO_HOSTED_PROVIDER_ENV),
        "baseUrlConfigured": _env_configured(AUDIO_HOSTED_BASE_URL_ENV),
        "endpoint": os.environ.get(AUDIO_HOSTED_ENDPOINT_ENV),
        "model": os.environ.get(AUDIO_HOSTED_MODEL_ENV),
        "apiKeyConfigured": _any_env_configured(
            AUDIO_HOSTED_API_KEY_ENV,
            AUDIO_HOSTED_OPENROUTER_PUBLIC_API_KEY_ENV,
        ),
        "timeoutSeconds": os.environ.get(AUDIO_HOSTED_TIMEOUT_SECONDS_ENV),
        "requestConcurrency": os.environ.get(AUDIO_HOSTED_REQUEST_CONCURRENCY_ENV),
        "maxAttempts": os.environ.get(AUDIO_HOSTED_MAX_ATTEMPTS_ENV),
        "primaryLanguage": os.environ.get(AUDIO_PRIMARY_LANGUAGE_ENV),
        "tracePathConfigured": _env_configured(AUDIO_HOSTED_TRACE_PATH_ENV),
        "qualityGate": os.environ.get(AUDIO_TRANSCRIPT_QUALITY_GATE_ENV),
    }


def _hosted_vlm_summary() -> dict[str, Any]:
    return {
        "provider": os.environ.get(HOSTED_VLM_OCR_PROVIDER_ENV),
        "baseUrl": os.environ.get(HOSTED_VLM_OCR_BASE_URL_ENV) or HOSTED_VLM_OCR_DEFAULT_BASE_URL,
        "model": os.environ.get(HOSTED_VLM_OCR_MODEL_ENV) or HOSTED_VLM_OCR_DEFAULT_MODEL,
        "openRouterModel": os.environ.get(HOSTED_VLM_OCR_OPENROUTER_MODEL_ENV),
        "apiKeyConfigured": _any_env_configured(
            HOSTED_VLM_OCR_API_KEY_ENV,
            HOSTED_VLM_OCR_OPENROUTER_API_KEY_ENV,
            HOSTED_VLM_OCR_OPENROUTER_PUBLIC_API_KEY_ENV,
        ),
        "timeoutSeconds": os.environ.get(HOSTED_VLM_OCR_TIMEOUT_SECONDS_ENV)
        or HOSTED_VLM_OCR_DEFAULT_TIMEOUT_SECONDS,
        "requestConcurrency": os.environ.get(HOSTED_VLM_OCR_REQUEST_CONCURRENCY_ENV)
        or HOSTED_VLM_OCR_DEFAULT_REQUEST_CONCURRENCY,
        "scaffoldMode": os.environ.get(HOSTED_VLM_OCR_SCAFFOLD_MODE_ENV)
        or HOSTED_VLM_OCR_DEFAULT_SCAFFOLD_MODE,
        "imageOptimization": os.environ.get(HOSTED_VLM_OCR_IMAGE_OPTIMIZATION_ENV)
        or HOSTED_VLM_OCR_DEFAULT_IMAGE_OPTIMIZATION,
        "regionCompositeSize": os.environ.get(HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE_ENV)
        or HOSTED_VLM_OCR_DEFAULT_REGION_COMPOSITE_SIZE,
        "regionCompositeMode": os.environ.get(HOSTED_VLM_OCR_REGION_COMPOSITE_MODE_ENV)
        or HOSTED_VLM_OCR_DEFAULT_REGION_COMPOSITE_MODE,
        "regionAtlasMode": os.environ.get(HOSTED_VLM_OCR_REGION_ATLAS_MODE_ENV)
        or HOSTED_VLM_OCR_DEFAULT_REGION_ATLAS_MODE,
        "tracePathConfigured": _env_configured(HOSTED_VLM_OCR_TRACE_PATH_ENV),
    }


def _any_env_configured(*names: str) -> bool:
    return any(_env_configured(name) for name in names)


def _env_configured(name: str) -> bool:
    return bool((os.environ.get(name) or "").strip())


__all__ = [
    "STARTUP_LOG_SCHEMA",
    "document_extract_startup_log_payload",
    "write_document_extract_startup_log",
]
