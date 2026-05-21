"""Common helpers for audio shard workers."""

from __future__ import annotations

import urllib.error
from concurrent.futures import ThreadPoolExecutor
from typing import TYPE_CHECKING, Any

from .audio_shard_tables import resolve_audio_shard_worker_count

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping, Sequence


def map_audio_rows(
    inputs: Sequence[Mapping[str, Any]],
    max_workers: int | str | None,
    process_one: Callable[[Mapping[str, Any]], Mapping[str, Any]],
) -> list[Mapping[str, Any]]:
    """Map audio shard rows with bounded worker concurrency."""

    rows = list(inputs)
    worker_count = resolve_audio_shard_worker_count(len(rows), max_workers)
    if worker_count <= 1 or len(rows) <= 1:
        return [process_one(row) for row in rows]
    with ThreadPoolExecutor(max_workers=worker_count) as executor:
        return list(executor.map(process_one, rows))


def resolve_hosted_audio_worker_count(
    input_count: int,
    *,
    requested: int | str | None,
    request_concurrency: int | None,
) -> int:
    """Resolve hosted audio concurrency with provider-level caps."""

    worker_count = resolve_audio_shard_worker_count(input_count, requested)
    if request_concurrency is not None:
        return max(1, min(worker_count, request_concurrency))
    return worker_count


def short_error_message(error: BaseException) -> str:
    """Render model/backend errors without flooding result rows."""

    if isinstance(error, urllib.error.HTTPError):
        message = f"HTTP Error {error.code}: {error.reason}"
    else:
        message = str(error)
    if len(message) <= 240:
        return message
    return f"{message[:237]}..."
