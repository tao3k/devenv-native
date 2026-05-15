"""Audio shard Arrow table entrypoints."""

from __future__ import annotations

import os

import pyarrow as pa

from .audio_shard_contracts import (
    AUDIO_SHARD_INPUT_SCHEMA,
    AUDIO_SHARD_INPUT_SCHEMA_VERSION,
    AUDIO_SHARD_RESULT_SCHEMA,
    AudioShardWorkerProtocol,
)
from .audio_shard_results import normalize_audio_shard_result

AUDIO_SHARD_WORKERS_ENV = "WENDAO_AUDIO_SHARD_WORKERS"

AUDIO_SHARD_MAX_WORKERS_ENV = "WENDAO_AUDIO_SHARD_MAX_WORKERS"


def build_audio_shard_result_table(
    input_table: pa.Table,
    *,
    worker: AudioShardWorkerProtocol | None = None,
    max_workers: int | str | None = None,
) -> pa.Table:
    """Build an audio result table from audio shard input rows.

    # Errors

    Raises `ValueError` when the input schema, contract version, or worker
    result count is invalid.
    """

    validate_audio_shard_input_table(input_table)
    input_rows = input_table.to_pylist()
    if worker is None:
        from .audio_shard_workers import SkippingAudioShardWorker

        effective_worker = SkippingAudioShardWorker()
    else:
        effective_worker = worker
    result_rows = list(effective_worker.process(input_rows, max_workers=max_workers))
    if len(result_rows) != len(input_rows):
        raise ValueError(
            f"audio worker returned {len(result_rows)} rows for {len(input_rows)} input rows"
        )
    normalized_rows = [
        normalize_audio_shard_result(input_row, result_row)
        for input_row, result_row in zip(input_rows, result_rows, strict=True)
    ]
    return pa.Table.from_pylist(normalized_rows, schema=AUDIO_SHARD_RESULT_SCHEMA)


def validate_audio_shard_input_table(input_table: pa.Table) -> None:
    """Validate the audio shard input Arrow table.

    # Errors

    Raises `ValueError` when required columns or contract versions are invalid.
    """

    _validate_schema_compatible(input_table.schema, AUDIO_SHARD_INPUT_SCHEMA)
    versions = set(input_table.column("contractVersion").to_pylist())
    if versions - {AUDIO_SHARD_INPUT_SCHEMA_VERSION}:
        raise ValueError(
            f"Unexpected audio shard input contract versions: {sorted(versions)}"
        )
    for column in ("sampleRateHz", "channels", "durationMs", "mediaDurationMs"):
        values = set(input_table.column(column).to_pylist())
        if any(int(value) <= 0 for value in values):
            raise ValueError(f"audio shard input column `{column}` must be positive")


def resolve_audio_shard_worker_count(
    input_count: int,
    requested: int | str | None = None,
) -> int:
    """Resolve the bounded audio worker count for a shard request."""

    if input_count <= 0:
        return 1
    requested_value = requested
    if requested_value is None:
        requested_value = os.environ.get(AUDIO_SHARD_WORKERS_ENV, "auto")
    if isinstance(requested_value, str):
        normalized = requested_value.strip().lower()
        if normalized and normalized != "auto":
            parsed = _parse_positive_int(normalized)
            if parsed is not None:
                return _cap_audio_worker_count(input_count, parsed)
        cpu_count = os.cpu_count() or 1
        return _cap_audio_worker_count(input_count, cpu_count)
    return _cap_audio_worker_count(input_count, int(requested_value))


def _cap_audio_worker_count(input_count: int, worker_count: int) -> int:
    capped = max(1, min(input_count, worker_count))
    max_worker_count = _parse_positive_int(
        os.environ.get(AUDIO_SHARD_MAX_WORKERS_ENV, "")
    )
    if max_worker_count is not None:
        capped = min(capped, max_worker_count)
    return max(1, capped)


def _parse_positive_int(value: str | None) -> int | None:
    if not value:
        return None
    try:
        parsed = int(value)
    except ValueError:
        return None
    return parsed if parsed > 0 else None


def _validate_schema_compatible(actual: pa.Schema, expected: pa.Schema) -> None:
    if actual.names != expected.names:
        raise ValueError(f"Unexpected audio shard input columns: {actual.names}")
    for index, field in enumerate(expected):
        actual_field = actual.field(index)
        if actual_field.type != field.type:
            raise ValueError(
                f"Unexpected audio shard input type for `{field.name}`: {actual_field.type}"
            )
