"""Audio shard Arrow contracts and worker protocol."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any, Protocol

import pyarrow as pa

from .arrow_schema_contracts import ArrowSchemaColumn, build_arrow_schema

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


AUDIO_SHARD_INPUT_SCHEMA_VERSION = "xiuxian_wendao.audio_shard_input.v1"

AUDIO_SHARD_RESULT_SCHEMA_VERSION = "xiuxian_wendao.audio_shard_result.v1"

AUDIO_SHARD_INPUT_TABLE = "audio_shard_input"

AUDIO_SHARD_RESULT_TABLE = "audio_shard_result"

AUDIO_SHARD_DEFAULT_PROFILE = "audio-shards-v1"

AUDIO_SHARD_DEFAULT_TASK_PROFILE = "transcription"

AUDIO_SHARD_INPUT_SCHEMA = build_arrow_schema(
    AUDIO_SHARD_INPUT_TABLE,
    (
        ArrowSchemaColumn("contractVersion", pa.string(), nullable=False),
        ArrowSchemaColumn("sourcePath", pa.string(), nullable=False),
        ArrowSchemaColumn("sourceContentHash", pa.string(), nullable=False),
        ArrowSchemaColumn("shardPath", pa.string(), nullable=False),
        ArrowSchemaColumn("shardSha256", pa.string(), nullable=False),
        ArrowSchemaColumn("shardProfile", pa.string(), nullable=False),
        ArrowSchemaColumn("taskProfile", pa.string(), nullable=False),
        ArrowSchemaColumn("backendProfile", pa.string(), nullable=False),
        ArrowSchemaColumn("preferredLanguages", pa.string(), nullable=False),
        ArrowSchemaColumn("sampleRateHz", pa.int32(), nullable=False),
        ArrowSchemaColumn("channels", pa.int32(), nullable=False),
        ArrowSchemaColumn("audioFormat", pa.string(), nullable=False),
        ArrowSchemaColumn("startMs", pa.int64(), nullable=False),
        ArrowSchemaColumn("durationMs", pa.int64(), nullable=False),
        ArrowSchemaColumn("mediaStartMs", pa.int64(), nullable=False),
        ArrowSchemaColumn("mediaDurationMs", pa.int64(), nullable=False),
        ArrowSchemaColumn("contextBeforeMs", pa.int64(), nullable=False),
        ArrowSchemaColumn("contextAfterMs", pa.int64(), nullable=False),
        ArrowSchemaColumn("shardElementId", pa.string(), nullable=False),
        ArrowSchemaColumn("readingOrderKey", pa.string(), nullable=False),
    ),
)

AUDIO_SHARD_RESULT_SCHEMA = build_arrow_schema(
    AUDIO_SHARD_RESULT_TABLE,
    (
        ArrowSchemaColumn("contractVersion", pa.string(), nullable=False),
        ArrowSchemaColumn("sourcePath", pa.string(), nullable=False),
        ArrowSchemaColumn("sourceContentHash", pa.string(), nullable=False),
        ArrowSchemaColumn("shardPath", pa.string(), nullable=False),
        ArrowSchemaColumn("shardSha256", pa.string(), nullable=False),
        ArrowSchemaColumn("shardProfile", pa.string(), nullable=False),
        ArrowSchemaColumn("taskProfile", pa.string(), nullable=False),
        ArrowSchemaColumn("backendProfile", pa.string(), nullable=False),
        ArrowSchemaColumn("status", pa.string(), nullable=False),
        ArrowSchemaColumn("text", pa.string(), nullable=True),
        ArrowSchemaColumn("textMimeType", pa.string(), nullable=False),
        ArrowSchemaColumn("confidence", pa.float64(), nullable=True),
        ArrowSchemaColumn("errorMessage", pa.string(), nullable=True),
        ArrowSchemaColumn("shardElementId", pa.string(), nullable=False),
        ArrowSchemaColumn("elementId", pa.string(), nullable=False),
    ),
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
