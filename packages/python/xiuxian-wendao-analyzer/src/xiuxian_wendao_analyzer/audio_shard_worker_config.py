"""Hosted audio worker configuration."""

from __future__ import annotations

import os
from dataclasses import dataclass, field

from .audio_language import PRIMARY_LANGUAGE_UNKNOWN, normalize_primary_language
from .audio_openai_protocol import AUDIO_OPENAI_DEFAULT_PROMPT
from .audio_shard_quality import AudioTranscriptQualityOptions

AUDIO_WORKER_ENV = "WENDAO_AUDIO_WORKER"
AUDIO_PRIMARY_LANGUAGE_ENV = "WENDAO_AUDIO_PRIMARY_LANGUAGE"

AUDIO_HOSTED_PROVIDER_ENV = "WENDAO_AUDIO_HOSTED_PROVIDER"
AUDIO_HOSTED_BASE_URL_ENV = "WENDAO_AUDIO_HOSTED_BASE_URL"
AUDIO_HOSTED_MODEL_ENV = "WENDAO_AUDIO_HOSTED_MODEL"
AUDIO_HOSTED_API_KEY_ENV = "WENDAO_AUDIO_HOSTED_API_KEY"
AUDIO_HOSTED_TIMEOUT_SECONDS_ENV = "WENDAO_AUDIO_HOSTED_TIMEOUT_SECONDS"
AUDIO_HOSTED_REQUEST_CONCURRENCY_ENV = "WENDAO_AUDIO_HOSTED_REQUEST_CONCURRENCY"
AUDIO_HOSTED_MAX_ATTEMPTS_ENV = "WENDAO_AUDIO_HOSTED_MAX_ATTEMPTS"
AUDIO_TRANSCRIPT_QUALITY_GATE_ENV = "WENDAO_AUDIO_TRANSCRIPT_QUALITY_GATE"
AUDIO_TRANSCRIPT_QUALITY_MAX_CHARS_PER_MINUTE_ENV = (
    "WENDAO_AUDIO_TRANSCRIPT_MAX_CHARS_PER_MINUTE"
)
AUDIO_TRANSCRIPT_QUALITY_MAX_REPEATED_NGRAM_RATIO_ENV = (
    "WENDAO_AUDIO_TRANSCRIPT_MAX_REPEATED_NGRAM_RATIO"
)
AUDIO_TRANSCRIPT_QUALITY_MAX_LATIN_RATIO_FOR_CHINESE_ENV = (
    "WENDAO_AUDIO_TRANSCRIPT_MAX_LATIN_RATIO_FOR_CHINESE"
)

AUDIO_HOSTED_OPENROUTER_PROVIDER = "openrouter"
AUDIO_HOSTED_OPENAI_COMPATIBLE_PROVIDER = "openai-compatible"
AUDIO_HOSTED_OPENROUTER_BASE_URL = "https://openrouter.ai/api/v1"
AUDIO_HOSTED_OPENAI_COMPATIBLE_BASE_URL = "https://api.openai.com/v1"
AUDIO_HOSTED_OPENROUTER_PUBLIC_API_KEY_ENV = "OPENROUTER_API_KEY"
AUDIO_HOSTED_DEFAULT_TIMEOUT_SECONDS = 3600.0
AUDIO_HOSTED_DEFAULT_MAX_ATTEMPTS = 2
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
    primary_language: str = PRIMARY_LANGUAGE_UNKNOWN
    max_attempts: int = AUDIO_HOSTED_DEFAULT_MAX_ATTEMPTS
    quality_options: AudioTranscriptQualityOptions = field(
        default_factory=AudioTranscriptQualityOptions
    )

    @property
    def completion_url(self) -> str:
        normalized = self.base_url.rstrip("/")
        if normalized.endswith("/chat/completions"):
            return normalized
        return f"{normalized}/chat/completions"


def hosted_audio_config_from_env() -> HostedAudioConfig:
    """Resolve hosted audio worker configuration from the environment."""

    provider = _env_value(
        AUDIO_HOSTED_PROVIDER_ENV,
        AUDIO_HOSTED_OPENAI_COMPATIBLE_PROVIDER,
    ).lower()
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
    model = _env_value(AUDIO_HOSTED_MODEL_ENV)
    if not model:
        raise ValueError(f"missing {AUDIO_HOSTED_MODEL_ENV}")
    api_key = _env_value(AUDIO_HOSTED_API_KEY_ENV)
    if provider == AUDIO_HOSTED_OPENROUTER_PROVIDER and not api_key:
        api_key = _env_value(AUDIO_HOSTED_OPENROUTER_PUBLIC_API_KEY_ENV)
    if not api_key:
        raise ValueError(f"missing {AUDIO_HOSTED_API_KEY_ENV}")
    return HostedAudioConfig(
        provider=provider,
        base_url=_env_value(AUDIO_HOSTED_BASE_URL_ENV, default_base_url),
        model=model,
        api_key=api_key,
        timeout_seconds=_positive_float_env(
            AUDIO_HOSTED_TIMEOUT_SECONDS_ENV,
            AUDIO_HOSTED_DEFAULT_TIMEOUT_SECONDS,
        ),
        request_concurrency=_optional_positive_int_env(
            AUDIO_HOSTED_REQUEST_CONCURRENCY_ENV
        ),
        primary_language=normalize_primary_language(
            os.environ.get(AUDIO_PRIMARY_LANGUAGE_ENV)
        ),
        max_attempts=_positive_int_env(
            AUDIO_HOSTED_MAX_ATTEMPTS_ENV,
            AUDIO_HOSTED_DEFAULT_MAX_ATTEMPTS,
        ),
        quality_options=AudioTranscriptQualityOptions(
            enabled=_quality_gate_enabled_env(AUDIO_TRANSCRIPT_QUALITY_GATE_ENV),
            max_chars_per_minute=_positive_float_env(
                AUDIO_TRANSCRIPT_QUALITY_MAX_CHARS_PER_MINUTE_ENV,
                AudioTranscriptQualityOptions().max_chars_per_minute,
            ),
            max_repeated_ngram_ratio=_positive_float_env(
                AUDIO_TRANSCRIPT_QUALITY_MAX_REPEATED_NGRAM_RATIO_ENV,
                AudioTranscriptQualityOptions().max_repeated_ngram_ratio,
            ),
            max_latin_ratio_for_chinese=_positive_float_env(
                AUDIO_TRANSCRIPT_QUALITY_MAX_LATIN_RATIO_FOR_CHINESE_ENV,
                AudioTranscriptQualityOptions().max_latin_ratio_for_chinese,
            ),
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


def _env_value(name: str, default: str = "") -> str:
    value = os.environ.get(name, default)
    if value is None:
        return ""
    return _strip_wrapping_quotes(value.strip())


def _strip_wrapping_quotes(value: str) -> str:
    if len(value) >= 2 and value[0] == value[-1] and value[0] in {"'", '"'}:
        return value[1:-1]
    return value


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


def _positive_int_env(name: str, default: int) -> int:
    value = os.environ.get(name, "").strip()
    if not value:
        return default
    try:
        parsed = int(value)
    except ValueError as exc:
        raise ValueError(f"{name} must be a positive integer") from exc
    if parsed <= 0:
        raise ValueError(f"{name} must be positive")
    return parsed


def _quality_gate_enabled_env(name: str) -> bool:
    value = os.environ.get(name, "enabled").strip().lower()
    if value in {"", "enabled", "true", "1", "yes"}:
        return True
    if value in {"disabled", "false", "0", "no"}:
        return False
    raise ValueError(f"{name} must be enabled or disabled")
