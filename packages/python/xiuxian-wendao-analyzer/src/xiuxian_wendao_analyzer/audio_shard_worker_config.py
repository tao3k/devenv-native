"""OpenAI-compatible audio model worker configuration."""

from __future__ import annotations

import os
from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING

from .audio_language import PRIMARY_LANGUAGE_UNKNOWN, normalize_primary_language
from .audio_openai_protocol import AUDIO_OPENAI_DEFAULT_PROMPT
from .audio_shard_quality import AudioTranscriptQualityOptions

if TYPE_CHECKING:
    from collections.abc import Mapping

AUDIO_WORKER_ENV = "WENDAO_AUDIO_WORKER"
AUDIO_PRIMARY_LANGUAGE_ENV = "WENDAO_AUDIO_PRIMARY_LANGUAGE"

AUDIO_HOSTED_PROVIDER_ENV = "WENDAO_AUDIO_HOSTED_PROVIDER"
AUDIO_HOSTED_BASE_URL_ENV = "WENDAO_AUDIO_HOSTED_BASE_URL"
AUDIO_HOSTED_ENDPOINT_ENV = "WENDAO_AUDIO_HOSTED_ENDPOINT"
AUDIO_HOSTED_MODEL_ENV = "WENDAO_AUDIO_HOSTED_MODEL"
AUDIO_HOSTED_API_KEY_ENV = "WENDAO_AUDIO_HOSTED_API_KEY"
AUDIO_HOSTED_TIMEOUT_SECONDS_ENV = "WENDAO_AUDIO_HOSTED_TIMEOUT_SECONDS"
AUDIO_HOSTED_REQUEST_CONCURRENCY_ENV = "WENDAO_AUDIO_HOSTED_REQUEST_CONCURRENCY"
AUDIO_HOSTED_MAX_ATTEMPTS_ENV = "WENDAO_AUDIO_HOSTED_MAX_ATTEMPTS"
AUDIO_HOSTED_TRACE_PATH_ENV = "WENDAO_AUDIO_HOSTED_TRACE_PATH"
AUDIO_TRANSCRIPT_QUALITY_GATE_ENV = "WENDAO_AUDIO_TRANSCRIPT_QUALITY_GATE"
AUDIO_TRANSCRIPT_QUALITY_MAX_CHARS_PER_MINUTE_ENV = "WENDAO_AUDIO_TRANSCRIPT_MAX_CHARS_PER_MINUTE"
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
AUDIO_HOSTED_ENDPOINT_AUTO = "auto"
AUDIO_HOSTED_ENDPOINT_CHAT_COMPLETIONS = "chat-completions"
AUDIO_HOSTED_ENDPOINT_AUDIO_TRANSCRIPTIONS = "audio-transcriptions"

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
    """Configuration for OpenAI-compatible audio model workers."""

    provider: str
    base_url: str
    model: str
    api_key: str
    timeout_seconds: float
    request_concurrency: int | None
    endpoint: str = AUDIO_HOSTED_ENDPOINT_CHAT_COMPLETIONS
    primary_language: str = PRIMARY_LANGUAGE_UNKNOWN
    max_attempts: int = AUDIO_HOSTED_DEFAULT_MAX_ATTEMPTS
    trace_path: Path | None = None
    quality_options: AudioTranscriptQualityOptions = field(
        default_factory=AudioTranscriptQualityOptions
    )

    @property
    def completion_url(self) -> str:
        return f"{_api_root_url(self.base_url)}/chat/completions"

    @property
    def transcription_url(self) -> str:
        return f"{_api_root_url(self.base_url)}/audio/transcriptions"

    @property
    def request_url(self) -> str:
        if self.endpoint == AUDIO_HOSTED_ENDPOINT_AUDIO_TRANSCRIPTIONS:
            return self.transcription_url
        return self.completion_url


def hosted_audio_config_from_env(
    overrides: Mapping[str, str] | None = None,
) -> HostedAudioConfig:
    """Resolve OpenAI-compatible audio model worker configuration from the environment."""

    provider = _env_value(
        AUDIO_HOSTED_PROVIDER_ENV,
        AUDIO_HOSTED_OPENAI_COMPATIBLE_PROVIDER,
        overrides,
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
    base_url = _env_value(AUDIO_HOSTED_BASE_URL_ENV, default_base_url, overrides)
    model = _env_value(AUDIO_HOSTED_MODEL_ENV, overrides=overrides)
    if not model:
        raise ValueError(f"missing {AUDIO_HOSTED_MODEL_ENV}")
    endpoint = _audio_endpoint_env(
        provider=provider,
        model=model,
        overrides=overrides,
    )
    api_key = _env_value(AUDIO_HOSTED_API_KEY_ENV, overrides=overrides)
    if provider == AUDIO_HOSTED_OPENROUTER_PROVIDER and not api_key:
        api_key = _env_value(AUDIO_HOSTED_OPENROUTER_PUBLIC_API_KEY_ENV)
    if (
        provider == AUDIO_HOSTED_OPENAI_COMPATIBLE_PROVIDER
        and not api_key
        and _is_local_audio_base_url(base_url)
    ):
        api_key = "EMPTY"
    if not api_key:
        raise ValueError(f"missing {AUDIO_HOSTED_API_KEY_ENV}")
    return HostedAudioConfig(
        provider=provider,
        base_url=base_url,
        model=model,
        api_key=api_key,
        timeout_seconds=_positive_float_env(
            AUDIO_HOSTED_TIMEOUT_SECONDS_ENV,
            AUDIO_HOSTED_DEFAULT_TIMEOUT_SECONDS,
            overrides,
        ),
        request_concurrency=_optional_positive_int_env(
            AUDIO_HOSTED_REQUEST_CONCURRENCY_ENV,
            overrides,
        ),
        endpoint=endpoint,
        primary_language=normalize_primary_language(
            _env_value(AUDIO_PRIMARY_LANGUAGE_ENV, overrides=overrides)
        ),
        max_attempts=_positive_int_env(
            AUDIO_HOSTED_MAX_ATTEMPTS_ENV,
            AUDIO_HOSTED_DEFAULT_MAX_ATTEMPTS,
            overrides,
        ),
        trace_path=_optional_path_env(AUDIO_HOSTED_TRACE_PATH_ENV, overrides),
        quality_options=AudioTranscriptQualityOptions(
            enabled=_quality_gate_enabled_env(
                AUDIO_TRANSCRIPT_QUALITY_GATE_ENV,
                overrides,
            ),
            max_chars_per_minute=_positive_float_env(
                AUDIO_TRANSCRIPT_QUALITY_MAX_CHARS_PER_MINUTE_ENV,
                AudioTranscriptQualityOptions().max_chars_per_minute,
                overrides,
            ),
            max_repeated_ngram_ratio=_positive_float_env(
                AUDIO_TRANSCRIPT_QUALITY_MAX_REPEATED_NGRAM_RATIO_ENV,
                AudioTranscriptQualityOptions().max_repeated_ngram_ratio,
                overrides,
            ),
            max_latin_ratio_for_chinese=_positive_float_env(
                AUDIO_TRANSCRIPT_QUALITY_MAX_LATIN_RATIO_FOR_CHINESE_ENV,
                AudioTranscriptQualityOptions().max_latin_ratio_for_chinese,
                overrides,
            ),
        ),
    )


def _api_root_url(base_url: str) -> str:
    normalized = base_url.rstrip("/")
    for suffix in ("/chat/completions", "/audio/transcriptions"):
        if normalized.endswith(suffix):
            return normalized[: -len(suffix)]
    return normalized


def _optional_path_env(
    name: str,
    overrides: Mapping[str, str] | None = None,
) -> Path | None:
    value = _env_value(name, overrides=overrides)
    if not value:
        return None
    return Path(value)


def _audio_endpoint_env(
    *,
    provider: str,
    model: str,
    overrides: Mapping[str, str] | None = None,
) -> str:
    configured = _env_value(
        AUDIO_HOSTED_ENDPOINT_ENV,
        AUDIO_HOSTED_ENDPOINT_AUTO,
        overrides,
    ).lower()
    supported = {
        AUDIO_HOSTED_ENDPOINT_AUTO,
        AUDIO_HOSTED_ENDPOINT_CHAT_COMPLETIONS,
        AUDIO_HOSTED_ENDPOINT_AUDIO_TRANSCRIPTIONS,
    }
    if configured not in supported:
        raise ValueError(
            f"unsupported {AUDIO_HOSTED_ENDPOINT_ENV}={configured}; "
            "supported values: auto, chat-completions, audio-transcriptions"
        )
    if configured != AUDIO_HOSTED_ENDPOINT_AUTO:
        return configured
    if provider == AUDIO_HOSTED_OPENROUTER_PROVIDER and _looks_like_stt_model(model):
        return AUDIO_HOSTED_ENDPOINT_AUDIO_TRANSCRIPTIONS
    return AUDIO_HOSTED_ENDPOINT_CHAT_COMPLETIONS


def _looks_like_stt_model(model: str) -> bool:
    normalized = model.lower()
    return "asr" in normalized or "whisper" in normalized


def _positive_float_env(
    name: str,
    default: float,
    overrides: Mapping[str, str] | None = None,
) -> float:
    value = _env_value(name, overrides=overrides)
    if not value:
        return default
    try:
        parsed = float(value)
    except ValueError as exc:
        raise ValueError(f"{name} must be a positive number") from exc
    if parsed <= 0:
        raise ValueError(f"{name} must be positive")
    return parsed


def _env_value(
    name: str,
    default: str = "",
    overrides: Mapping[str, str] | None = None,
) -> str:
    value = (
        overrides.get(name) if overrides and name in overrides else os.environ.get(name, default)
    )
    if value is None:
        return ""
    return _strip_wrapping_quotes(value.strip())


def _strip_wrapping_quotes(value: str) -> str:
    if len(value) >= 2 and value[0] == value[-1] and value[0] in {"'", '"'}:
        return value[1:-1]
    return value


def _is_local_audio_base_url(value: str) -> bool:
    normalized = value.lower()
    return normalized.startswith(("http://127.0.0.1", "http://localhost"))


def _optional_positive_int_env(
    name: str,
    overrides: Mapping[str, str] | None = None,
) -> int | None:
    value = _env_value(name, overrides=overrides)
    if not value or value == "auto":
        return None
    try:
        parsed = int(value)
    except ValueError as exc:
        raise ValueError(f"{name} must be a positive integer") from exc
    if parsed <= 0:
        raise ValueError(f"{name} must be positive")
    return parsed


def _positive_int_env(
    name: str,
    default: int,
    overrides: Mapping[str, str] | None = None,
) -> int:
    value = _env_value(name, overrides=overrides)
    if not value:
        return default
    try:
        parsed = int(value)
    except ValueError as exc:
        raise ValueError(f"{name} must be a positive integer") from exc
    if parsed <= 0:
        raise ValueError(f"{name} must be positive")
    return parsed


def _quality_gate_enabled_env(
    name: str,
    overrides: Mapping[str, str] | None = None,
) -> bool:
    value = _env_value(name, "enabled", overrides).lower()
    if value in {"", "enabled", "true", "1", "yes"}:
        return True
    if value in {"disabled", "false", "0", "no"}:
        return False
    raise ValueError(f"{name} must be enabled or disabled")
