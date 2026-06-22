"""Audio shard worker registry."""

from __future__ import annotations

import os
from typing import TYPE_CHECKING, Any

from .audio_shard_results import failed_audio_shard_result, skipped_audio_shard_result
from .audio_shard_worker_config import (
    AUDIO_BACKEND_DOCLING,
    AUDIO_BACKEND_DOCLING_PROFILE,
    AUDIO_BACKEND_HOSTED,
    AUDIO_BACKEND_HOSTED_PROFILE,
    AUDIO_BACKEND_SKIP,
    AUDIO_WORKER_ENV,
    SUPPORTED_AUDIO_WORKERS,
)
from .audio_shard_worker_docling import DoclingAudioShardWorker
from .audio_shard_worker_hosted import HostedAudioShardWorker

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


class SkippingAudioShardWorker:
    """Default no-model audio worker used when no real backend is configured."""

    def process(
        self,
        inputs: Sequence[Mapping[str, Any]],
        *,
        max_workers: int | str | None = None,
    ) -> Sequence[Mapping[str, Any]]:
        _ = max_workers
        return [
            skipped_audio_shard_result(input_row, "audio shard worker is not configured")
            for input_row in inputs
        ]


class UnsupportedAudioShardWorker:
    """Worker that reports an unsupported backend as failed rows."""

    def __init__(self, worker_name: str) -> None:
        self._worker_name = worker_name

    def process(
        self,
        inputs: Sequence[Mapping[str, Any]],
        *,
        max_workers: int | str | None = None,
    ) -> Sequence[Mapping[str, Any]]:
        _ = max_workers
        return [
            failed_audio_shard_result(
                input_row,
                f"unsupported audio shard worker: {self._worker_name}",
            )
            for input_row in inputs
        ]


def build_audio_shard_worker(
    worker_name: str | None = None,
    max_workers: int | str | None = "auto",
    *,
    hosted_config: Any | None = None,
) -> Any:
    """Build an audio shard worker by registry name."""

    normalized = normalize_audio_worker_name(
        worker_name or os.environ.get(AUDIO_WORKER_ENV, AUDIO_BACKEND_SKIP)
    )
    if normalized == AUDIO_BACKEND_SKIP:
        return SkippingAudioShardWorker()
    if normalized == AUDIO_BACKEND_DOCLING_PROFILE:
        return DoclingAudioShardWorker(max_workers=max_workers)
    if normalized == AUDIO_BACKEND_HOSTED_PROFILE:
        return HostedAudioShardWorker(config=hosted_config, max_workers=max_workers)
    return UnsupportedAudioShardWorker(worker_name or "")


def normalize_audio_worker_name(worker_name: str | None) -> str:
    """Normalize CLI/env worker aliases to stable backend profiles."""

    normalized = (worker_name or AUDIO_BACKEND_SKIP).strip().lower()
    if normalized in {"", AUDIO_BACKEND_SKIP}:
        return AUDIO_BACKEND_SKIP
    if normalized in {AUDIO_BACKEND_DOCLING, AUDIO_BACKEND_DOCLING_PROFILE}:
        return AUDIO_BACKEND_DOCLING_PROFILE
    if normalized in {AUDIO_BACKEND_HOSTED, AUDIO_BACKEND_HOSTED_PROFILE}:
        return AUDIO_BACKEND_HOSTED_PROFILE
    if normalized in SUPPORTED_AUDIO_WORKERS:
        return normalized
    return normalized
