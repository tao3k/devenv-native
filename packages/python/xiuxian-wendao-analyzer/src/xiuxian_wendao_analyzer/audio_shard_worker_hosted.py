"""Hosted OpenAI-compatible audio shard worker."""

from __future__ import annotations

import json
import urllib.request
from pathlib import Path
from typing import TYPE_CHECKING, Any

from .audio_openai_protocol import (
    build_chat_audio_payload,
    extract_openai_message_content,
)
from .audio_shard_results import failed_audio_shard_result, succeeded_audio_shard_result
from .audio_shard_worker_common import (
    map_audio_rows,
    resolve_hosted_audio_worker_count,
    short_error_message,
)
from .audio_shard_worker_config import (
    AUDIO_HOSTED_DEFAULT_PROMPT,
    HostedAudioConfig,
    hosted_audio_config_from_env,
)

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping, Sequence


class HostedAudioShardWorker:
    """OpenAI-compatible hosted audio transcript worker."""

    def __init__(
        self,
        *,
        max_workers: int | str | None = "auto",
        config: HostedAudioConfig | None = None,
        request_sender: (
            Callable[[HostedAudioConfig, Mapping[str, Any]], Any] | None
        ) = None,
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
                    f"Hosted audio worker failed: {exc}",
                )
                for input_row in inputs
            ]
        worker_count = resolve_hosted_audio_worker_count(
            len(inputs),
            requested=max_workers or self._max_workers,
            request_concurrency=config.request_concurrency,
        )
        return map_audio_rows(
            inputs, worker_count, lambda row: self._process_one(row, config)
        )

    def _process_one(
        self,
        input_row: Mapping[str, Any],
        config: HostedAudioConfig,
    ) -> Mapping[str, Any]:
        try:
            payload = hosted_audio_payload(input_row, config.model)
            response = self._request_sender(config, payload)
            text = extract_openai_message_content(response).strip()
        except Exception as exc:
            return failed_audio_shard_result(
                input_row,
                f"Hosted audio worker failed: {short_error_message(exc)}",
            )
        if not text:
            return failed_audio_shard_result(
                input_row,
                "Hosted audio worker returned empty text",
            )
        return succeeded_audio_shard_result(input_row, text, 1.0)


def hosted_audio_payload(
    input_row: Mapping[str, Any],
    model: str,
) -> dict[str, Any]:
    """Build an OpenAI-compatible audio input chat completion payload."""

    audio_path = Path(str(input_row["shardPath"]))
    audio_format = str(input_row.get("audioFormat") or "wav").strip().lower()
    return build_chat_audio_payload(
        model=model,
        audio_path=audio_path,
        audio_format=audio_format,
        prompt=AUDIO_HOSTED_DEFAULT_PROMPT,
    )


def send_hosted_audio_request(
    config: HostedAudioConfig,
    payload: Mapping[str, Any],
) -> Mapping[str, Any]:
    """Send one hosted audio request."""

    request = urllib.request.Request(
        config.completion_url,
        data=json.dumps(payload).encode("utf-8"),
        headers={
            "Authorization": f"Bearer {config.api_key}",
            "Content-Type": "application/json",
        },
        method="POST",
    )
    with urllib.request.urlopen(request, timeout=config.timeout_seconds) as response:
        return json.loads(response.read().decode("utf-8"))
