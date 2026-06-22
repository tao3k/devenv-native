"""Hosted audio request trace writer."""

from __future__ import annotations

import json
import threading
import time
from typing import TYPE_CHECKING, Any

from .audio_shard_worker_common import short_error_message

if TYPE_CHECKING:
    from collections.abc import Mapping
    from pathlib import Path

_HOSTED_AUDIO_TRACE_LOCK = threading.Lock()


def write_hosted_audio_trace_record(
    *,
    trace_path: Path | None,
    input_row: Mapping[str, Any],
    provider: str,
    model: str,
    endpoint: str,
    request_url: str,
    status: str,
    started: float,
    http_status: int | None = None,
    text_chars: int = 0,
    error: BaseException | None = None,
    http_attempt_count: int = 1,
) -> None:
    """Write one hosted audio request trace row when tracing is enabled."""

    if trace_path is None:
        return
    ended_unix_ms = int(time.time() * 1000)
    latency_ms = round((time.perf_counter() - started) * 1000.0, 3)
    started_unix_ms = max(0, ended_unix_ms - round(latency_ms))
    record = {
        "schema": "xiuxian_wendao.hosted_audio_request_trace.v1",
        "timestampUnixMs": ended_unix_ms,
        "startedUnixMs": started_unix_ms,
        "endedUnixMs": ended_unix_ms,
        "status": status,
        "httpStatus": http_status,
        "latencyMs": latency_ms,
        "provider": provider,
        "model": model,
        "endpoint": request_url,
        "endpointKind": endpoint,
        "requestKind": "audio-shard",
        "httpAttemptCount": max(1, http_attempt_count),
        "sourceContentHash": input_row.get("sourceContentHash"),
        "shardElementId": input_row.get("shardElementId"),
        "shardSha256": input_row.get("shardSha256"),
        "shardProfile": input_row.get("shardProfile"),
        "readingOrderKey": input_row.get("readingOrderKey"),
        "backendProfile": input_row.get("backendProfile"),
        "taskProfile": input_row.get("taskProfile"),
        "sampleRateHz": input_row.get("sampleRateHz"),
        "channels": input_row.get("channels"),
        "audioFormat": input_row.get("audioFormat"),
        "startMs": input_row.get("startMs"),
        "durationMs": input_row.get("durationMs"),
        "mediaStartMs": input_row.get("mediaStartMs"),
        "mediaDurationMs": input_row.get("mediaDurationMs"),
        "contextBeforeMs": input_row.get("contextBeforeMs"),
        "contextAfterMs": input_row.get("contextAfterMs"),
        "textChars": text_chars,
        "errorType": type(error).__name__ if error is not None else None,
        "errorMessage": short_error_message(error) if error is not None else None,
    }
    try:
        with _HOSTED_AUDIO_TRACE_LOCK:
            trace_path.parent.mkdir(parents=True, exist_ok=True)
            with trace_path.open("a", encoding="utf-8") as handle:
                handle.write(json.dumps(record, sort_keys=True))
                handle.write("\n")
    except OSError:
        return
