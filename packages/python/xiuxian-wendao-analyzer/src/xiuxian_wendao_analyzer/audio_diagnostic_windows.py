"""Audio diagnostic shard window planning helpers."""

from __future__ import annotations

import json
from collections.abc import Mapping, Sequence
from pathlib import Path

from xiuxian_wendao_analyzer.audio_diagnostic_identity import SpeechSegment


def media_window_for_chunk(
    *,
    start_seconds: float,
    duration_seconds: float,
    context_seconds: float,
    source_duration_seconds: float | None,
) -> tuple[float, float, float, float]:
    """Return actual media window and effective before/after context."""

    media_start = max(0.0, start_seconds - context_seconds)
    logical_end = start_seconds + duration_seconds
    requested_end = logical_end + context_seconds
    media_end = (
        min(requested_end, source_duration_seconds)
        if source_duration_seconds is not None
        else requested_end
    )
    media_duration = max(0.0, media_end - media_start)
    before = max(0.0, start_seconds - media_start)
    after = max(0.0, media_end - logical_end)
    return media_start, media_duration, before, after


def chunk_start_offsets(
    *,
    duration_seconds: float | None,
    chunk_seconds: int,
    limit_chunks: int,
    strategy: str,
    start_offset_seconds: float,
) -> list[float]:
    """Return deterministic start offsets for bounded audio sampling."""

    if limit_chunks <= 0:
        raise ValueError("limit_chunks must be positive")
    if chunk_seconds <= 0:
        raise ValueError("chunk_seconds must be positive")
    if strategy == "head":
        return [
            start_offset_seconds + index * chunk_seconds
            for index in range(limit_chunks)
        ]
    if strategy != "uniform":
        raise ValueError(f"unsupported sample strategy: {strategy}")
    if duration_seconds is None:
        raise ValueError("uniform sampling requires duration_seconds")
    max_start = max(start_offset_seconds, duration_seconds - chunk_seconds)
    if limit_chunks == 1:
        return [min(start_offset_seconds, max_start)]
    span = max(0.0, max_start - start_offset_seconds)
    return [
        min(max_start, start_offset_seconds + span * index / (limit_chunks - 1))
        for index in range(limit_chunks)
    ]


def _json_number(row: Mapping[str, object], *names: str) -> float | None:
    for name in names:
        value = row.get(name)
        if isinstance(value, int | float) and not isinstance(value, bool):
            return float(value)
    return None


def _json_string(row: Mapping[str, object], *names: str) -> str:
    for name in names:
        value = row.get(name)
        if isinstance(value, str):
            return value
    return ""


def _segment_source_matches(row_source: str, source: Path) -> bool:
    if not row_source:
        return True
    return row_source == str(source) or Path(row_source).name == source.name


def load_speech_segments(path: Path | None, *, source: Path) -> list[SpeechSegment]:
    """Load VAD/planner speech windows for one source from JSONL."""

    if path is None:
        return []
    segments: list[SpeechSegment] = []
    for line_number, raw_line in enumerate(
        path.read_text(encoding="utf-8").splitlines(), start=1
    ):
        if not raw_line.strip():
            continue
        row = json.loads(raw_line)
        if not isinstance(row, Mapping):
            raise ValueError(f"invalid speech segment row at line {line_number}")
        row_source = _json_string(row, "source", "sourceId", "sourcePath")
        if not _segment_source_matches(row_source, source):
            continue
        start_seconds = _json_number(row, "startSeconds", "start_seconds")
        start_ms = _json_number(row, "startMs", "start_ms")
        if start_seconds is None and start_ms is not None:
            start_seconds = start_ms / 1000.0
        end_seconds = _json_number(row, "endSeconds", "end_seconds")
        end_ms = _json_number(row, "endMs", "end_ms")
        if end_seconds is None and end_ms is not None:
            end_seconds = end_ms / 1000.0
        duration_seconds = _json_number(row, "durationSeconds", "duration_seconds")
        duration_ms = _json_number(row, "durationMs", "duration_ms")
        if duration_seconds is None and duration_ms is not None:
            duration_seconds = duration_ms / 1000.0
        if start_seconds is None:
            raise ValueError(
                f"speech segment row {line_number} is missing startSeconds/startMs"
            )
        if duration_seconds is None:
            if end_seconds is None:
                raise ValueError(
                    f"speech segment row {line_number} is missing duration or end"
                )
            duration_seconds = end_seconds - start_seconds
        if duration_seconds <= 0:
            raise ValueError(
                f"speech segment row {line_number} has non-positive duration"
            )
        confidence = _json_number(row, "confidence", "score", "probability")
        segments.append(
            SpeechSegment(
                source=row_source,
                index=int(_json_number(row, "index", "segmentIndex") or len(segments)),
                start_seconds=start_seconds,
                duration_seconds=duration_seconds,
                confidence=confidence,
                label=_json_string(row, "label", "kind"),
            )
        )
    return sorted(segments, key=lambda segment: (segment.start_seconds, segment.index))


def chunk_windows(
    *,
    duration_seconds: float | None,
    chunk_seconds: int,
    limit_chunks: int,
    sample_strategy: str,
    start_offset_seconds: float,
    speech_segments: Sequence[SpeechSegment] | None,
    explicit_windows: Sequence[SpeechSegment] | None = None,
    speech_segment_merge_gap_seconds: float = 0.0,
    speech_segment_min_window_seconds: float = 0.0,
    speech_segment_short_merge_gap_seconds: float | None = None,
    speech_segment_max_window_seconds: float | None = None,
) -> list[tuple[float, float]]:
    """Return logical audio windows for fixed or speech-segment sampling."""

    if sample_strategy == "full-coverage":
        return full_coverage_chunk_windows(
            duration_seconds=duration_seconds,
            chunk_seconds=chunk_seconds,
            limit_chunks=limit_chunks,
            start_offset_seconds=start_offset_seconds,
        )
    if sample_strategy == "speech-segments":
        if not speech_segments:
            raise ValueError("speech-segments sampling requires speech segment rows")
        return pack_speech_segment_windows(
            speech_segments,
            limit_windows=limit_chunks,
            merge_gap_seconds=speech_segment_merge_gap_seconds,
            min_window_seconds=speech_segment_min_window_seconds,
            short_merge_gap_seconds=speech_segment_short_merge_gap_seconds,
            max_window_seconds=speech_segment_max_window_seconds,
        )
    if sample_strategy == "explicit-windows":
        if not explicit_windows:
            raise ValueError("explicit-windows sampling requires explicit window rows")
        return explicit_audio_windows(
            explicit_windows,
            limit_windows=limit_chunks,
        )
    return [
        (offset, float(chunk_seconds))
        for offset in chunk_start_offsets(
            duration_seconds=duration_seconds,
            chunk_seconds=chunk_seconds,
            limit_chunks=limit_chunks,
            strategy=sample_strategy,
            start_offset_seconds=start_offset_seconds,
        )
    ]


def full_coverage_chunk_windows(
    *,
    duration_seconds: float | None,
    chunk_seconds: int,
    limit_chunks: int,
    start_offset_seconds: float,
) -> list[tuple[float, float]]:
    """Return contiguous windows that cover the source without overlap."""

    if duration_seconds is None:
        raise ValueError("full-coverage sampling requires duration_seconds")
    if limit_chunks <= 0:
        raise ValueError("limit_chunks must be positive")
    if chunk_seconds <= 0:
        raise ValueError("chunk_seconds must be positive")
    if start_offset_seconds < 0:
        raise ValueError("start_offset_seconds cannot be negative")
    if start_offset_seconds >= duration_seconds:
        raise ValueError("start_offset_seconds must be before the source end")
    windows: list[tuple[float, float]] = []
    start = start_offset_seconds
    while start < duration_seconds:
        duration = min(float(chunk_seconds), duration_seconds - start)
        windows.append((start, duration))
        start += chunk_seconds
    if len(windows) > limit_chunks:
        raise ValueError(
            "full-coverage sampling needs "
            f"{len(windows)} chunks; increase limit_chunks from {limit_chunks}"
        )
    return windows


def explicit_audio_windows(
    windows: Sequence[SpeechSegment],
    *,
    limit_windows: int,
) -> list[tuple[float, float]]:
    """Return caller-selected windows without VAD packing or resampling."""

    if limit_windows <= 0:
        raise ValueError("limit_windows must be positive")
    if len(windows) > limit_windows:
        raise ValueError(
            "explicit-windows sampling needs "
            f"{len(windows)} chunks; increase limit_chunks from {limit_windows}"
        )
    return [
        (float(window.start_seconds), float(window.duration_seconds))
        for window in windows
    ]


def pack_speech_segment_windows(
    speech_segments: Sequence[SpeechSegment],
    *,
    limit_windows: int,
    merge_gap_seconds: float,
    min_window_seconds: float,
    short_merge_gap_seconds: float | None,
    max_window_seconds: float | None,
) -> list[tuple[float, float]]:
    """Pack VAD rows into bounded ASR windows without long-context expansion."""

    if limit_windows <= 0:
        raise ValueError("limit_windows must be positive")
    if merge_gap_seconds < 0:
        raise ValueError("speech segment merge gap cannot be negative")
    if min_window_seconds < 0:
        raise ValueError("speech segment min window cannot be negative")
    if short_merge_gap_seconds is not None and short_merge_gap_seconds < 0:
        raise ValueError("speech segment short merge gap cannot be negative")
    if max_window_seconds is not None and max_window_seconds <= 0:
        raise ValueError("speech segment max window must be positive")
    if (
        max_window_seconds is not None
        and min_window_seconds
        and min_window_seconds > max_window_seconds
    ):
        raise ValueError("speech segment min window cannot exceed max window")
    pieces = _split_long_speech_segments(speech_segments, max_window_seconds)
    if not pieces:
        return []
    windows: list[tuple[float, float]] = []
    effective_short_merge_gap = (
        min_window_seconds
        if short_merge_gap_seconds is None
        else short_merge_gap_seconds
    )
    current_start, current_duration = pieces[0]
    current_end = current_start + current_duration
    for start, duration in pieces[1:]:
        end = start + duration
        gap = start - current_end
        merged_duration = end - current_start
        short_window_context = (
            min_window_seconds > 0
            and (
                current_end - current_start < min_window_seconds
                or duration < min_window_seconds
            )
            and gap <= effective_short_merge_gap
        )
        can_merge = (gap <= merge_gap_seconds or short_window_context) and (
            max_window_seconds is None or merged_duration <= max_window_seconds
        )
        if can_merge:
            current_end = max(current_end, end)
            continue
        windows.append((current_start, current_end - current_start))
        if len(windows) >= limit_windows:
            return windows
        current_start, current_end = start, end
    windows.append((current_start, current_end - current_start))
    return windows[:limit_windows]


def _split_long_speech_segments(
    speech_segments: Sequence[SpeechSegment],
    max_window_seconds: float | None,
) -> list[tuple[float, float]]:
    pieces: list[tuple[float, float]] = []
    for segment in sorted(
        speech_segments, key=lambda row: (row.start_seconds, row.index)
    ):
        if max_window_seconds is None or segment.duration_seconds <= max_window_seconds:
            pieces.append((segment.start_seconds, segment.duration_seconds))
            continue
        remaining = segment.duration_seconds
        start = segment.start_seconds
        while remaining > 0:
            duration = min(max_window_seconds, remaining)
            pieces.append((start, duration))
            start += duration
            remaining -= duration
    return pieces
