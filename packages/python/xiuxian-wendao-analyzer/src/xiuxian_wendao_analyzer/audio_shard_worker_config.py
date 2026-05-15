"""Hosted audio worker configuration."""

from __future__ import annotations

import os
from dataclasses import dataclass

from .audio_openai_protocol import AUDIO_OPENAI_DEFAULT_PROMPT

AUDIO_WORKER_ENV = "WENDAO_AUDIO_WORKER"

AUDIO_HOSTED_PROVIDER_ENV = "WENDAO_AUDIO_HOSTED_PROVIDER"
AUDIO_HOSTED_BASE_URL_ENV = "WENDAO_AUDIO_HOSTED_BASE_URL"
AUDIO_HOSTED_MODEL_ENV = "WENDAO_AUDIO_HOSTED_MODEL"
AUDIO_HOSTED_API_KEY_ENV = "WENDAO_AUDIO_HOSTED_API_KEY"
AUDIO_HOSTED_TIMEOUT_SECONDS_ENV = "WENDAO_AUDIO_HOSTED_TIMEOUT_SECONDS"
AUDIO_HOSTED_REQUEST_CONCURRENCY_ENV = "WENDAO_AUDIO_HOSTED_REQUEST_CONCURRENCY"

AUDIO_HOSTED_OPENROUTER_PROVIDER = "openrouter"
AUDIO_HOSTED_OPENAI_COMPATIBLE_PROVIDER = "openai-compatible"
AUDIO_HOSTED_OPENROUTER_BASE_URL = "https://openrouter.ai/api/v1"
AUDIO_HOSTED_OPENAI_COMPATIBLE_BASE_URL = "https://api.openai.com/v1"
AUDIO_HOSTED_OPENROUTER_PUBLIC_API_KEY_ENV = "OPENROUTER_API_KEY"
AUDIO_HOSTED_DEFAULT_TIMEOUT_SECONDS = 3600.0
AUDIO_HOSTED_DEFAULT_PROMPT = AUDIO_OPENAI_DEFAULT_PROMPT

AUDIO_BACKEND_SKIP = "skip"
AUDIO_BACKEND_DOCLING = "docling"
AUDIO_BACKEND_HOSTED = "hosted"
AUDIO_BACKEND_DOCLING_PROFILE = "docling-audio-transcript-v1"
AUDIO_BACKEND_HOSTED_PROFILE = "hosted-audio-transcript-v1"

SUPPORTED_AUDIO_WORKERS = {
    AUDIO_BACKEND_SKIP,
    AUDIO_BACKEND_DOCLING,
    AUDIO_BACKEND_HOSTED,
    AUDIO_BACKEND_DOCLING_PROFILE,
    AUDIO_BACKEND_HOSTED_PROFILE,
}


@dataclass(frozen=True)
class HostedAudioConfig:
    """Configuration for OpenAI-compatible hosted audio workers."""

    provider: str
    base_url: str
    model: str
    api_key: str
    timeout_seconds: float
    request_concurrency: int | None

    @property
    def completion_url(self) -> str:
        normalized = self.base_url.rstrip("/")
        if normalized.endswith("/chat/completions"):
            return normalized
        return f"{normalized}/chat/completions"


def hosted_audio_config_from_env() -> HostedAudioConfig:
    """Resolve hosted audio worker configuration from the environment."""

    provider = (
        os.environ.get(
            AUDIO_HOSTED_PROVIDER_ENV,
            AUDIO_HOSTED_OPENAI_COMPATIBLE_PROVIDER,
        )
        .strip()
        .lower()
    )
    if provider not in {
        AUDIO_HOSTED_OPENAI_COMPATIBLE_PROVIDER,
        AUDIO_HOSTED_OPENROUTER_PROVIDER,
    }:
        raise ValueError(
            f"unsupported {AUDIO_HOSTED_PROVIDER_ENV}={provider}; "
            "supported values: openai-compatible, openrouter"
        )
    default_base_url = (
        AUDIO_HOSTED_OPENROUTER_BASE_URL
        if provider == AUDIO_HOSTED_OPENROUTER_PROVIDER
        else AUDIO_HOSTED_OPENAI_COMPATIBLE_BASE_URL
    )
    model = os.environ.get(AUDIO_HOSTED_MODEL_ENV, "").strip()
    if not model:
        raise ValueError(f"missing {AUDIO_HOSTED_MODEL_ENV}")
    api_key = os.environ.get(AUDIO_HOSTED_API_KEY_ENV, "").strip()
    if provider == AUDIO_HOSTED_OPENROUTER_PROVIDER and not api_key:
        api_key = os.environ.get(AUDIO_HOSTED_OPENROUTER_PUBLIC_API_KEY_ENV, "").strip()
    if not api_key:
        raise ValueError(f"missing {AUDIO_HOSTED_API_KEY_ENV}")
    return HostedAudioConfig(
        provider=provider,
        base_url=os.environ.get(AUDIO_HOSTED_BASE_URL_ENV, default_base_url).strip(),
        model=model,
        api_key=api_key,
        timeout_seconds=_positive_float_env(
            AUDIO_HOSTED_TIMEOUT_SECONDS_ENV,
            AUDIO_HOSTED_DEFAULT_TIMEOUT_SECONDS,
        ),
        request_concurrency=_optional_positive_int_env(
            AUDIO_HOSTED_REQUEST_CONCURRENCY_ENV
        ),
    )


def _positive_float_env(name: str, default: float) -> float:
    value = os.environ.get(name, "").strip()
    if not value:
        return default
    try:
        parsed = float(value)
    except ValueError as exc:
        raise ValueError(f"{name} must be a positive number") from exc
    if parsed <= 0:
        raise ValueError(f"{name} must be positive")
    return parsed


def _optional_positive_int_env(name: str) -> int | None:
    value = os.environ.get(name, "").strip()
    if not value or value == "auto":
        return None
    try:
        parsed = int(value)
    except ValueError as exc:
        raise ValueError(f"{name} must be a positive integer") from exc
    if parsed <= 0:
        raise ValueError(f"{name} must be positive")
    return parsed
