"""Audio transcript Org export helpers for benchmark evidence."""

from __future__ import annotations

import csv
import hashlib
import json
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path

AUDIO_TRANSCRIPT_RESOURCE_TYPE = "audio-transcript"
DOCUMENT_RESOURCES_ARROW_CACHE_NAME = "_resources.arrow"
REFERENCE_STATUS_CANDIDATE_DRAFT = "candidate-draft"


def export_audio_transcript_org(
    resources_path: Path,
    org_path: Path,
) -> dict[str, Any]:
    """Export audio transcript resource rows to an Org review ledger."""

    if not resources_path.exists():
        return _empty_export()
    rows = _audio_transcript_rows(resources_path)
    if not rows:
        return _empty_export()

    org_path.parent.mkdir(parents=True, exist_ok=True)
    text = _render_audio_transcript_org(rows)
    org_path.write_text(text, encoding="utf-8")
    return {
        "path": str(org_path),
        "rows": len(rows),
        "chars": sum(len(_row_string(row, "content")) for row in rows),
        "timelineMarkerCount": sum(
            _timeline_marker_count(_row_string(row, "content")) for row in rows
        ),
    }


def export_audio_transcript_reference_drafts(
    resources_path: Path,
    jsonl_path: Path,
    tsv_path: Path,
) -> dict[str, Any]:
    """Export audio transcript timeline segments as editable reference drafts."""

    if not resources_path.exists():
        return _empty_reference_draft_export()
    resource_rows = _audio_transcript_rows(resources_path)
    draft_rows = _reference_draft_rows_from_resources(resource_rows)
    if not draft_rows:
        return _empty_reference_draft_export()

    jsonl_path.parent.mkdir(parents=True, exist_ok=True)
    tsv_path.parent.mkdir(parents=True, exist_ok=True)
    _write_reference_draft_jsonl(jsonl_path, draft_rows)
    _write_reference_draft_tsv(tsv_path, draft_rows)
    return {
        "jsonlPath": str(jsonl_path),
        "tsvPath": str(tsv_path),
        "rows": len(draft_rows),
        "chars": sum(len(str(row["text"])) for row in draft_rows),
        **_reference_draft_text_stats(draft_rows),
    }


def _empty_export() -> dict[str, Any]:
    return {
        "path": None,
        "rows": 0,
        "chars": 0,
        "timelineMarkerCount": 0,
    }


def _empty_reference_draft_export() -> dict[str, Any]:
    return {
        "jsonlPath": None,
        "tsvPath": None,
        "rows": 0,
        "chars": 0,
        "emptyRows": 0,
        "minChars": 0,
        "maxChars": 0,
        "duplicateTextHashCount": 0,
        "uniqueTextHashCount": 0,
    }


def _audio_transcript_rows(resources_path: Path) -> list[dict[str, Any]]:
    import pyarrow.ipc as arrow_ipc

    with resources_path.open("rb") as handle, arrow_ipc.open_file(handle) as reader:
        table = reader.read_all()
    rows = []
    for row in table.to_pylist():
        if _row_string(row, "resourceType") == AUDIO_TRANSCRIPT_RESOURCE_TYPE:
            rows.append(row)
    return rows


def _reference_draft_rows_from_resources(
    rows: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    draft_rows: list[dict[str, Any]] = []
    for row in rows:
        source_path = _row_string(row, "sourcePath")
        source_name = _source_basename(source_path)
        for segment in _timeline_segments(_row_string(row, "content")):
            draft_rows.append(
                {
                    "source": source_name,
                    "sourceId": source_path,
                    "chunkIndex": len(draft_rows),
                    "startSeconds": segment["startSeconds"],
                    "durationSeconds": (segment["endSeconds"] - segment["startSeconds"]),
                    "referenceStatus": REFERENCE_STATUS_CANDIDATE_DRAFT,
                    "text": segment["text"],
                }
            )
    return draft_rows


def _timeline_segments(content: str) -> list[dict[str, Any]]:
    segments: list[dict[str, Any]] = []
    current: dict[str, Any] | None = None
    text_parts: list[str] = []
    for raw_line in content.splitlines():
        marker = _parse_timeline_marker(raw_line.lstrip())
        if marker is not None:
            _push_segment(segments, current, text_parts)
            current = {
                "startSeconds": marker["startSeconds"],
                "endSeconds": marker["endSeconds"],
            }
            text_parts = [marker["text"]]
            continue
        if current is not None:
            text_parts.append(raw_line)
    _push_segment(segments, current, text_parts)
    return segments


def _push_segment(
    segments: list[dict[str, Any]],
    current: dict[str, Any] | None,
    text_parts: list[str],
) -> None:
    if current is None:
        return
    text = "\n".join(text_parts).strip()
    if not text:
        return
    segments.append({**current, "text": text})


def _parse_timeline_marker(line: str) -> dict[str, Any] | None:
    marker_end = line.find("]")
    if marker_end < 0 or not line.startswith("["):
        return None
    marker_body = line[1:marker_end]
    if "-" not in marker_body:
        return None
    start_text, end_text = marker_body.split("-", 1)
    start_seconds = _parse_time_offset(start_text)
    end_seconds = _parse_time_offset(end_text)
    if start_seconds is None or end_seconds is None or end_seconds <= start_seconds:
        return None
    return {
        "startSeconds": start_seconds,
        "endSeconds": end_seconds,
        "text": line[marker_end + 1 :].strip(),
    }


def _parse_time_offset(value: str) -> float | None:
    parts = value.split(":")
    if len(parts) == 2:
        minutes_text, seconds_text = parts
        hours = 0
    elif len(parts) == 3:
        hours_text, minutes_text, seconds_text = parts
        try:
            hours = int(hours_text)
        except ValueError:
            return None
    else:
        return None
    try:
        minutes = int(minutes_text)
        seconds = float(seconds_text)
    except ValueError:
        return None
    return float((hours * 3600) + (minutes * 60)) + seconds


def _write_reference_draft_jsonl(path: Path, rows: list[dict[str, Any]]) -> None:
    lines = [json.dumps(row, ensure_ascii=False, sort_keys=True) for row in rows]
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def _reference_draft_text_stats(rows: list[dict[str, Any]]) -> dict[str, Any]:
    texts = [str(row["text"]) for row in rows]
    lengths = [len(text) for text in texts]
    hashes = {hashlib.sha256(text.encode("utf-8")).hexdigest() for text in texts}
    return {
        "emptyRows": sum(1 for length in lengths if length == 0),
        "minChars": min(lengths, default=0),
        "maxChars": max(lengths, default=0),
        "duplicateTextHashCount": len(texts) - len(hashes),
        "uniqueTextHashCount": len(hashes),
    }


def _write_reference_draft_tsv(path: Path, rows: list[dict[str, Any]]) -> None:
    header = [
        "source",
        "sourceId",
        "chunkIndex",
        "startSeconds",
        "durationSeconds",
        "referenceStatus",
        "text",
    ]
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=header,
            dialect="excel-tab",
            extrasaction="ignore",
        )
        writer.writeheader()
        writer.writerows(rows)


def _render_audio_transcript_org(rows: list[dict[str, Any]]) -> str:
    lines = [
        "#+TITLE: Audio Transcript Timeline",
        "#+OPTIONS: toc:nil",
        "",
    ]
    chunk_index = 0
    for index, row in enumerate(rows, start=1):
        element_id = _row_string(row, "elementId") or f"audio-transcript-{index}"
        source_path = _row_string(row, "sourcePath")
        source_name = _source_basename(source_path)
        status = _row_string(row, "status")
        mime_type = _row_string(row, "mimeType")
        content = _row_string(row, "content").strip()
        segments = _timeline_segments(content)
        if segments:
            for segment in segments:
                start_seconds = float(segment["startSeconds"])
                end_seconds = float(segment["endSeconds"])
                text = str(segment["text"]).strip()
                lines.extend(
                    [
                        (
                            f"* {_format_org_timestamp(start_seconds)} -- "
                            f"{_format_org_timestamp(end_seconds)} "
                            f"{source_name} chunk {chunk_index:04d}"
                        ),
                        ":PROPERTIES:",
                        f":ELEMENT_ID: {_org_property_value(element_id)}",
                        f":RESOURCE_TYPE: {AUDIO_TRANSCRIPT_RESOURCE_TYPE}",
                        f":SOURCE: {_org_property_value(source_name)}",
                        f":SOURCE_PATH: {_org_property_value(source_path)}",
                        f":CHUNK_INDEX: {chunk_index}",
                        f":START_SECONDS: {start_seconds:.3f}",
                        f":END_SECONDS: {end_seconds:.3f}",
                        f":STATUS: {_org_property_value(status)}",
                        f":MIME_TYPE: {_org_property_value(mime_type)}",
                        ":END:",
                        "",
                        text,
                        "",
                    ]
                )
                chunk_index += 1
            continue
        lines.extend(
            [
                f"* Transcript {index}",
                ":PROPERTIES:",
                f":ELEMENT_ID: {_org_property_value(element_id)}",
                f":RESOURCE_TYPE: {AUDIO_TRANSCRIPT_RESOURCE_TYPE}",
                f":SOURCE_PATH: {_org_property_value(source_path)}",
                f":STATUS: {_org_property_value(status)}",
                f":MIME_TYPE: {_org_property_value(mime_type)}",
                ":END:",
                "",
                content,
                "",
            ]
        )
    return "\n".join(lines)


def _row_string(row: dict[str, Any], key: str) -> str:
    value = row.get(key)
    if value is None:
        return ""
    return str(value)


def _org_property_value(value: str) -> str:
    return value.replace("\r", " ").replace("\n", " ").strip()


def _source_basename(source_path: str) -> str:
    return source_path.rsplit("/", 1)[-1] if source_path else ""


def _format_org_timestamp(seconds: float) -> str:
    milliseconds = round(seconds * 1000)
    hours = milliseconds // 3_600_000
    minutes = (milliseconds % 3_600_000) // 60_000
    seconds_part = (milliseconds % 60_000) // 1000
    millis = milliseconds % 1000
    return f"{hours:02}:{minutes:02}:{seconds_part:02}.{millis:03}"


def _timeline_marker_count(content: str) -> int:
    return sum(1 for line in content.splitlines() if _is_timeline_marker(line.lstrip()))


def _is_timeline_marker(line: str) -> bool:
    marker_end = line.find("]")
    if marker_end < 0:
        return False
    marker = line[: marker_end + 1]
    return (
        marker.startswith("[")
        and "-" in marker
        and all(":" in part for part in marker[:marker_end].split("-"))
    )
