"""Audio shard Arrow contracts and worker protocol."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any, Protocol

import pyarrow as pa

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


AUDIO_SHARD_INPUT_SCHEMA_VERSION = "xiuxian_wendao.audio_shard_input.v1"

AUDIO_SHARD_RESULT_SCHEMA_VERSION = "xiuxian_wendao.audio_shard_result.v1"

AUDIO_SHARD_DEFAULT_PROFILE = "audio-shards-v1"

AUDIO_SHARD_DEFAULT_TASK_PROFILE = "transcription"

AUDIO_SHARD_INPUT_SCHEMA = pa.schema(
    [
        pa.field("contractVersion", pa.string(), nullable=False),
        pa.field("sourcePath", pa.string(), nullable=False),
        pa.field("sourceContentHash", pa.string(), nullable=False),
        pa.field("shardPath", pa.string(), nullable=False),
        pa.field("shardSha256", pa.string(), nullable=False),
        pa.field("shardProfile", pa.string(), nullable=False),
        pa.field("taskProfile", pa.string(), nullable=False),
        pa.field("backendProfile", pa.string(), nullable=False),
        pa.field("preferredLanguages", pa.string(), nullable=False),
        pa.field("sampleRateHz", pa.int32(), nullable=False),
        pa.field("channels", pa.int32(), nullable=False),
        pa.field("audioFormat", pa.string(), nullable=False),
        pa.field("startMs", pa.int64(), nullable=False),
        pa.field("durationMs", pa.int64(), nullable=False),
        pa.field("mediaStartMs", pa.int64(), nullable=False),
        pa.field("mediaDurationMs", pa.int64(), nullable=False),
        pa.field("contextBeforeMs", pa.int64(), nullable=False),
        pa.field("contextAfterMs", pa.int64(), nullable=False),
        pa.field("shardElementId", pa.string(), nullable=False),
        pa.field("readingOrderKey", pa.string(), nullable=False),
    ],
)

AUDIO_SHARD_RESULT_SCHEMA = pa.schema(
    [
        pa.field("contractVersion", pa.string(), nullable=False),
        pa.field("sourcePath", pa.string(), nullable=False),
        pa.field("sourceContentHash", pa.string(), nullable=False),
        pa.field("shardPath", pa.string(), nullable=False),
        pa.field("shardSha256", pa.string(), nullable=False),
        pa.field("shardProfile", pa.string(), nullable=False),
        pa.field("taskProfile", pa.string(), nullable=False),
        pa.field("backendProfile", pa.string(), nullable=False),
        pa.field("status", pa.string(), nullable=False),
        pa.field("text", pa.string(), nullable=True),
        pa.field("textMimeType", pa.string(), nullable=False),
        pa.field("confidence", pa.float64(), nullable=True),
        pa.field("errorMessage", pa.string(), nullable=True),
        pa.field("shardElementId", pa.string(), nullable=False),
        pa.field("elementId", pa.string(), nullable=False),
    ],
)


class AudioShardWorkerProtocol(Protocol):
    """Protocol implemented by injected audio shard workers."""

    def process(
        self,
        inputs: Sequence[Mapping[str, Any]],
        *,
        max_workers: int | str | None = None,
    ) -> Sequence[Mapping[str, Any]]:
        """Return text/result rows for audio shard rows."""
