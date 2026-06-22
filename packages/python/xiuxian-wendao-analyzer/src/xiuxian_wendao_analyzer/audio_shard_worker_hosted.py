"""OpenAI-compatible audio shard worker."""

from __future__ import annotations

import json
import time
import urllib.request
from pathlib import Path
from typing import TYPE_CHECKING, Any

from .audio_hosted_trace import write_hosted_audio_trace_record
from .audio_language import (
    PRIMARY_LANGUAGE_UNKNOWN,
    normalize_primary_language,
    prompt_with_primary_language,
)
from .audio_openai_protocol import (
    build_audio_transcription_payload,
    build_chat_audio_payload,
    extract_audio_transcription_text,
    extract_openai_message_content,
)
from .audio_shard_quality import audio_transcript_quality_failure
from .audio_shard_results import failed_audio_shard_result, succeeded_audio_shard_result
from .audio_shard_worker_common import (
    is_transient_audio_request_error,
    map_audio_rows,
    resolve_hosted_audio_worker_count,
    short_error_message,
)
from .audio_shard_worker_config import (
    AUDIO_HOSTED_DEFAULT_PROMPT,
    AUDIO_HOSTED_ENDPOINT_AUDIO_TRANSCRIPTIONS,
    AUDIO_HOSTED_OPENROUTER_PROVIDER,
    HostedAudioConfig,
    hosted_audio_config_from_env,
)

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping, Sequence

_HOSTED_AUDIO_RETRY_BASE_SECONDS = 0.25
_HOSTED_AUDIO_RETRY_MAX_SECONDS = 4.0


class HostedAudioShardWorker:
    """OpenAI-compatible audio transcript worker."""

    def __init__(
        self,
        *,
        max_workers: int | str | None = "auto",
        config: HostedAudioConfig | None = None,
        request_sender: (Callable[[HostedAudioConfig, Mapping[str, Any]], Any] | None) = None,
    ) -> None:
        self._max_workers = max_workers
        self._config = config
        self._request_sender = request_sender or send_hosted_audio_request

    def process(
        self,
        inputs: Sequence[Mapping[str, Any]],
        *,
        max_workers: int | str | None = None,
    ) -> Sequence[Mapping[str, Any]]:
        try:
            config = self._config or hosted_audio_config_from_env()
        except ValueError as exc:
            return [
                failed_audio_shard_result(
                    input_row,
                    f"Audio model worker failed: {exc}",
                )
                for input_row in inputs
            ]
        worker_count = resolve_hosted_audio_worker_count(
            len(inputs),
            requested=max_workers or self._max_workers,
            request_concurrency=config.request_concurrency,
        )
        return map_audio_rows(inputs, worker_count, lambda row: self._process_one(row, config))

    def _process_one(
        self,
        input_row: Mapping[str, Any],
        config: HostedAudioConfig,
    ) -> Mapping[str, Any]:
        payload = hosted_audio_payload(
            input_row,
            config,
            primary_language=_effective_primary_language(input_row, config),
        )
        last_error = "Audio model worker returned empty text"
        max_attempts = max(1, config.max_attempts)
        for attempt in range(max_attempts):
            started = time.perf_counter()
            try:
                response = self._request_sender(config, payload)
                text = hosted_audio_text(response, config).strip()
            except Exception as exc:
                last_error = f"Audio model worker failed: {short_error_message(exc)}"
                _write_trace(
                    input_row,
                    config,
                    status="failed",
                    started=started,
                    error=exc,
                    http_attempt_count=attempt + 1,
                )
                if (
                    attempt + 1 < max_attempts
                    and is_transient_audio_request_error(exc)
                ):
                    time.sleep(hosted_audio_retry_delay_seconds(attempt))
                continue
            if text:
                quality_failure = audio_transcript_quality_failure(
                    input_row,
                    text,
                    primary_language=_effective_primary_language(input_row, config),
                    options=config.quality_options,
                )
                if quality_failure is not None:
                    last_error = quality_failure
                    _write_trace(
                        input_row,
                        config,
                        status="failed",
                        started=started,
                        text_chars=len(text),
                        error=RuntimeError(quality_failure),
                        http_attempt_count=attempt + 1,
                    )
                    continue
                _write_trace(
                    input_row,
                    config,
                    status="succeeded",
                    started=started,
                    text_chars=len(text),
                    http_attempt_count=attempt + 1,
                )
                return succeeded_audio_shard_result(input_row, text, 1.0)
            last_error = "Audio model worker returned empty text"
            _write_trace(
                input_row,
                config,
                status="failed",
                started=started,
                error=RuntimeError(last_error),
                http_attempt_count=attempt + 1,
            )
        return failed_audio_shard_result(
            input_row,
            last_error,
        )


def hosted_audio_retry_delay_seconds(attempt: int) -> float:
    """Return bounded retry backoff for hosted audio transport failures."""

    return min(
        _HOSTED_AUDIO_RETRY_BASE_SECONDS * (2**max(0, attempt)),
        _HOSTED_AUDIO_RETRY_MAX_SECONDS,
    )


def hosted_audio_payload(
    input_row: Mapping[str, Any],
    config: HostedAudioConfig,
    *,
    primary_language: str = PRIMARY_LANGUAGE_UNKNOWN,
) -> dict[str, Any]:
    """Build an OpenAI-compatible audio input chat completion payload."""

    audio_path = Path(str(input_row["shardPath"]))
    audio_format = str(input_row.get("audioFormat") or "wav").strip().lower()
    if config.endpoint == AUDIO_HOSTED_ENDPOINT_AUDIO_TRANSCRIPTIONS:
        return build_audio_transcription_payload(
            model=config.model,
            audio_path=audio_path,
            audio_format=audio_format,
        )
    return build_chat_audio_payload(
        model=config.model,
        audio_path=audio_path,
        audio_format=audio_format,
        prompt=prompt_with_primary_language(
            AUDIO_HOSTED_DEFAULT_PROMPT,
            primary_language,
        ),
        disable_reasoning=config.provider == AUDIO_HOSTED_OPENROUTER_PROVIDER,
    )


def hosted_audio_text(
    response: Mapping[str, Any],
    config: HostedAudioConfig,
) -> str:
    """Extract text from the configured hosted audio response shape."""

    if config.endpoint == AUDIO_HOSTED_ENDPOINT_AUDIO_TRANSCRIPTIONS:
        return extract_audio_transcription_text(response)
    return extract_openai_message_content(response)


def send_hosted_audio_request(
    config: HostedAudioConfig,
    payload: Mapping[str, Any],
) -> Mapping[str, Any]:
    """Send one hosted audio request."""

    request = urllib.request.Request(
        config.request_url,
        data=json.dumps(payload).encode("utf-8"),
        headers={
            "Authorization": f"Bearer {config.api_key}",
            "Content-Type": "application/json",
        },
        method="POST",
    )
    with urllib.request.urlopen(request, timeout=config.timeout_seconds) as response:
        return json.loads(response.read().decode("utf-8"))


def _effective_primary_language(
    input_row: Mapping[str, Any],
    config: HostedAudioConfig,
) -> str:
    configured = normalize_primary_language(config.primary_language)
    if configured != PRIMARY_LANGUAGE_UNKNOWN:
        return configured
    return normalize_primary_language(str(input_row.get("preferredLanguages") or ""))


def _write_trace(
    input_row: Mapping[str, Any],
    config: HostedAudioConfig,
    *,
    status: str,
    started: float,
    text_chars: int = 0,
    error: BaseException | None = None,
    http_attempt_count: int = 1,
) -> None:
    write_hosted_audio_trace_record(
        trace_path=config.trace_path,
        input_row=input_row,
        provider=config.provider,
        model=config.model,
        endpoint=config.endpoint,
        request_url=config.request_url,
        status=status,
        started=started,
        text_chars=text_chars,
        error=error,
        http_attempt_count=http_attempt_count,
    )
