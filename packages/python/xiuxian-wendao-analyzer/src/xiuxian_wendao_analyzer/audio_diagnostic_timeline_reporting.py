"""Audio diagnostic timeline report writers."""

from __future__ import annotations

import json
from collections.abc import Mapping, Sequence
from pathlib import Path

from xiuxian_wendao_analyzer.audio_diagnostic_quality import QualityRow, read_transcript
from xiuxian_wendao_analyzer.audio_diagnostic_report_writers import (
    write_jsonl,
    write_text,
)


def format_vtt_timestamp(seconds: float) -> str:
    """Format seconds as a WebVTT timestamp."""

    total_ms = max(0, round(seconds * 1000))
    hours, remainder = divmod(total_ms, 3_600_000)
    minutes, remainder = divmod(remainder, 60_000)
    secs, millis = divmod(remainder, 1000)
    return f"{hours:02}:{minutes:02}:{secs:02}.{millis:03}"


def format_srt_timestamp(seconds: float) -> str:
    """Format seconds as an SRT timestamp."""

    return format_vtt_timestamp(seconds).replace(".", ",")


def timeline_review_rows(rows: Sequence[QualityRow]) -> list[dict[str, object]]:
    """Build timestamped transcript rows for evidence review."""

    timeline: list[dict[str, object]] = []
    for row in rows:
        if row.segments_path:
            segment_rows = _read_segment_rows(row)
            if segment_rows:
                timeline.extend(_timeline_rows_from_segments(row, segment_rows))
                continue
        timeline.append(_timeline_row_from_shard(row))
    return timeline


def write_transcript_timeline_jsonl(path: Path, rows: Sequence[QualityRow]) -> None:
    """Write timestamped transcript rows as JSONL."""

    write_jsonl(path, timeline_review_rows(rows))


def write_transcript_timeline_vtt(path: Path, rows: Sequence[QualityRow]) -> None:
    """Write a YouTube-style WebVTT transcript timeline."""

    lines = ["WEBVTT", ""]
    for index, segment in enumerate(timeline_review_rows(rows), start=1):
        start = format_vtt_timestamp(float(segment["startSeconds"]))
        end = format_vtt_timestamp(float(segment["endSeconds"]))
        lines.extend(
            [
                f"{segment['source']}#{int(segment['chunkIndex']):04d}.{index:04d}",
                f"{start} --> {end}",
                str(segment["text"]).replace("\r", " ").replace("\n", " ").strip(),
                "",
            ]
        )
    write_text(path, "\n".join(lines))


def write_transcript_timeline_srt(path: Path, rows: Sequence[QualityRow]) -> None:
    """Write a SubRip transcript timeline."""

    lines: list[str] = []
    for index, segment in enumerate(timeline_review_rows(rows), start=1):
        start = format_srt_timestamp(float(segment["startSeconds"]))
        end = format_srt_timestamp(float(segment["endSeconds"]))
        lines.extend(
            [
                str(index),
                f"{start} --> {end}",
                str(segment["text"]).replace("\r", " ").replace("\n", " ").strip(),
                "",
            ]
        )
    write_text(path, "\n".join(lines))


def write_transcript_timeline_org(path: Path, rows: Sequence[QualityRow]) -> None:
    """Write an Org-mode transcript timeline."""

    lines = [
        "#+TITLE: Audio Transcript Timeline",
        "#+OPTIONS: toc:nil",
        "",
    ]
    for segment in timeline_review_rows(rows):
        start_seconds = float(segment["startSeconds"])
        end_seconds = float(segment["endSeconds"])
        start = format_vtt_timestamp(start_seconds)
        end = format_vtt_timestamp(end_seconds)
        source = str(segment["source"])
        chunk_index = int(segment["chunkIndex"])
        text = str(segment["text"]).strip()
        lines.extend(
            [
                f"* {start} -- {end} {source} chunk {chunk_index:04d}",
                ":PROPERTIES:",
                f":BACKEND: {segment['backend']}",
                f":SOURCE: {source}",
                f":CHUNK_INDEX: {chunk_index}",
                f":START_SECONDS: {start_seconds:.3f}",
                f":END_SECONDS: {end_seconds:.3f}",
                f":STATUS: {segment['status']}",
                f":REVIEW_STATUS: {segment['reviewStatus']}",
                ":END:",
                "",
                text,
                "",
            ]
        )
    write_text(path, "\n".join(lines))


def _read_segment_rows(row: QualityRow) -> list[dict[str, object]]:
    path = Path(row.segments_path)
    if not path.exists():
        return []
    segments: list[dict[str, object]] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        segment = _parse_segment_line(line)
        if segment is not None:
            segments.append(segment)
    return segments


def _parse_segment_line(line: str) -> dict[str, object] | None:
    if not line.strip():
        return None
    try:
        value = json.loads(line)
    except json.JSONDecodeError:
        return None
    if not isinstance(value, Mapping):
        return None
    start = value.get("startSeconds")
    end = value.get("endSeconds")
    text = value.get("text")
    if not isinstance(start, int | float) or not isinstance(end, int | float):
        return None
    if not isinstance(text, str) or not text.strip():
        return None
    return {
        "startSeconds": float(start),
        "endSeconds": float(end),
        "text": text.strip(),
    }


def _timeline_rows_from_segments(
    row: QualityRow,
    segment_rows: Sequence[Mapping[str, object]],
) -> list[dict[str, object]]:
    return [
        {
            "backend": row.backend,
            "source": Path(row.source).name,
            "chunkIndex": row.chunk_index,
            "startSeconds": segment["startSeconds"],
            "endSeconds": segment["endSeconds"],
            "status": row.status,
            "reviewStatus": row.review_status,
            "text": segment["text"],
        }
        for segment in segment_rows
    ]


def _timeline_row_from_shard(row: QualityRow) -> dict[str, object]:
    return {
        "backend": row.backend,
        "source": Path(row.source).name,
        "chunkIndex": row.chunk_index,
        "startSeconds": row.start_seconds,
        "endSeconds": row.start_seconds + row.duration_seconds,
        "status": row.status,
        "reviewStatus": row.review_status,
        "text": read_transcript(row.transcript_path),
    }
