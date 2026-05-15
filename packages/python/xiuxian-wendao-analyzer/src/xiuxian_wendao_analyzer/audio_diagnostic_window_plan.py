"""Speech-window planning reports for audio diagnostics."""

from __future__ import annotations

from typing import TYPE_CHECKING

from xiuxian_wendao_analyzer.audio_diagnostic_windows import chunk_windows

if TYPE_CHECKING:
    from collections.abc import Sequence

    from xiuxian_wendao_analyzer.audio_diagnostic_identity import SpeechSegment


def parse_window_min_candidates(raw_value: str) -> list[float]:
    """Parse comma-separated minimum window candidates."""

    candidates: list[float] = []
    for item in raw_value.split(","):
        value = item.strip()
        if not value:
            continue
        seconds = float(value)
        if seconds < 0:
            raise ValueError("speech-window minimum candidates cannot be negative")
        candidates.append(seconds)
    if not candidates:
        raise ValueError("at least one speech-window minimum candidate is required")
    return candidates


def build_speech_window_plan_report(
    *,
    speech_segments: Sequence[SpeechSegment],
    duration_seconds: float | None,
    chunk_seconds: int,
    limit_chunks: int,
    merge_gap_seconds: float,
    max_window_seconds: float | None,
    min_window_candidates: Sequence[float],
    short_merge_gap_seconds: float | None = None,
) -> dict[str, object]:
    """Build an offline report for candidate VAD speech-window strategies."""

    raw_duration_seconds = sum(segment.duration_seconds for segment in speech_segments)
    candidates = []
    for min_window_seconds in min_window_candidates:
        windows = chunk_windows(
            duration_seconds=duration_seconds,
            chunk_seconds=chunk_seconds,
            limit_chunks=limit_chunks,
            sample_strategy="speech-segments",
            start_offset_seconds=0.0,
            speech_segments=speech_segments,
            speech_segment_merge_gap_seconds=merge_gap_seconds,
            speech_segment_min_window_seconds=min_window_seconds,
            speech_segment_short_merge_gap_seconds=short_merge_gap_seconds,
            speech_segment_max_window_seconds=max_window_seconds,
        )
        durations = [duration for _start, duration in windows]
        coverage_seconds = sum(durations)
        candidates.append(
            {
                "minWindowSeconds": min_window_seconds,
                "chunks": len(windows),
                "coverageSeconds": coverage_seconds,
                "coverageExpansionSeconds": coverage_seconds - raw_duration_seconds,
                "averageWindowSeconds": (
                    coverage_seconds / len(windows) if windows else 0.0
                ),
                "maxWindowSeconds": max(durations) if durations else 0.0,
                "shortWindowsUnder3Seconds": sum(
                    1 for duration in durations if duration < 3.0
                ),
                "shortWindowsUnder5Seconds": sum(
                    1 for duration in durations if duration < 5.0
                ),
                "shortWindowsUnder8Seconds": sum(
                    1 for duration in durations if duration < 8.0
                ),
            }
        )
    return {
        "schema": "xiuxian_wendao.audio_speech_window_plan.v1",
        "rawSpeechSegmentCount": len(speech_segments),
        "rawSpeechDurationSeconds": raw_duration_seconds,
        "mergeGapSeconds": merge_gap_seconds,
        "shortMergeGapSeconds": short_merge_gap_seconds,
        "maxWindowSeconds": max_window_seconds,
        "limitChunks": limit_chunks,
        "candidates": candidates,
    }
