"""Support helpers for audio diagnostic backend dispatch."""

from __future__ import annotations

from typing import TYPE_CHECKING

from xiuxian_wendao_analyzer.audio_diagnostic_results import AsrResult

if TYPE_CHECKING:
    from pathlib import Path

    from xiuxian_wendao_analyzer.audio_diagnostic_materialization import AudioChunk


def coerce_transcription(value: object) -> tuple[str, list[dict[str, object]]]:
    """Normalize backend return values to transcript plus optional segments."""

    if isinstance(value, tuple) and len(value) == 2:
        text, segments = value
        return str(text), (
            [segment for segment in segments if isinstance(segment, dict)]
            if isinstance(segments, list)
            else []
        )
    return str(value), []


def absolute_segments(
    chunk: AudioChunk, segments: list[dict[str, object]]
) -> list[dict[str, object]]:
    """Convert backend-relative segment timestamps to source timeline seconds."""

    absolute: list[dict[str, object]] = []
    base_seconds = chunk.media_start_seconds
    for index, segment in enumerate(segments):
        start = segment.get("startSeconds")
        end = segment.get("endSeconds")
        text = segment.get("text")
        if not isinstance(start, int | float):
            continue
        if not isinstance(end, int | float):
            continue
        if not isinstance(text, str) or not text.strip():
            continue
        absolute.append(
            {
                "backendRelativeIndex": index,
                "startSeconds": base_seconds + float(start),
                "endSeconds": base_seconds + float(end),
                "text": text.strip(),
            }
        )
    return absolute


def build_asr_result(
    *,
    backend: str,
    chunk: AudioChunk,
    model: str,
    status: str,
    wall_seconds: float,
    transcript: str,
    transcript_path: Path,
    error: str,
    task_admission_key: str,
    segments_path: Path,
    segments: list[dict[str, object]],
) -> AsrResult:
    """Build the stable diagnostic result row for one backend run."""

    return AsrResult(
        backend=backend,
        source=str(chunk.source),
        chunk=str(chunk.path),
        chunk_index=chunk.index,
        start_seconds=chunk.start_seconds,
        duration_seconds=chunk.duration_seconds,
        model=model,
        status=status,
        wall_seconds=wall_seconds,
        transcript_chars=len(transcript),
        transcript_path=str(transcript_path) if transcript else "",
        error=error,
        shard_id=chunk.shard_id,
        shard_cache_key=chunk.cache_key,
        task_admission_key=task_admission_key,
        segments_path=str(segments_path) if segments else "",
        segment_count=len(segments),
    )
