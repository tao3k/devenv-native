"""Audio diagnostic reference validation helpers."""

from __future__ import annotations

import json
from pathlib import Path
from typing import TYPE_CHECKING

from xiuxian_wendao_analyzer.audio_diagnostic_reference_inputs import (
    reference_candidate_draft_row_count,
)

if TYPE_CHECKING:
    from collections.abc import Mapping


def validate_reference_jsonl(
    reference_path: Path,
    *,
    audio_shards_path: Path | None = None,
) -> dict[str, object]:
    """Validate curated reference readiness without running ASR."""

    rows = _reference_rows(reference_path)
    issues: list[str] = []
    keys: list[tuple[str, int]] = []
    empty_text_rows = 0
    for index, row in enumerate(rows, start=1):
        key = _reference_row_key(row, index)
        keys.append(key)
        text = row.get("text")
        if not isinstance(text, str) or not text.strip():
            empty_text_rows += 1
    duplicate_keys = len(keys) - len(set(keys))
    candidate_draft_rows = reference_candidate_draft_row_count(reference_path)
    if empty_text_rows:
        issues.append("empty-reference-text")
    if duplicate_keys:
        issues.append("duplicate-reference-key")
    if candidate_draft_rows:
        issues.append("candidate-draft-reference")

    timeline_report = audio_shard_manifest_report(audio_shards_path)
    expected_keys = timeline_report["keys"]
    missing_shards = sorted(expected_keys - set(keys))
    extra_references = sorted(set(keys) - expected_keys) if expected_keys else []
    if missing_shards:
        issues.append("reference-coverage-missing")
    if extra_references:
        issues.append("reference-extra-rows")
    if timeline_report["issues"]:
        issues.append("audio-shard-timeline-invalid")
    return {
        "ready": not issues,
        "referenceRows": len(rows),
        "candidateDraftRows": candidate_draft_rows,
        "emptyTextRows": empty_text_rows,
        "duplicateKeys": duplicate_keys,
        "expectedShardRows": len(expected_keys) if expected_keys else None,
        "missingShardRows": len(missing_shards),
        "extraReferenceRows": len(extra_references),
        "timelineAuthorityConfigured": audio_shards_path is not None,
        "timelineAuthorityPassed": (
            None if audio_shards_path is None else not timeline_report["issues"]
        ),
        "timelineAuthorityIssueRows": len(timeline_report["issues"]),
        "timelineAuthorityIssues": timeline_report["issues"],
        "issues": issues,
    }


def audio_shard_manifest_keys(path: Path | None) -> set[tuple[str, int]]:
    """Return source/chunk keys from an audio shard manifest."""

    return audio_shard_manifest_report(path)["keys"]


def audio_shard_manifest_report(path: Path | None) -> dict[str, object]:
    """Return timestamp-authority facts from an audio shard manifest."""

    if path is None:
        return {"keys": set(), "issues": []}
    manifest = json.loads(path.read_text(encoding="utf-8"))
    items = manifest.get("items") if isinstance(manifest, dict) else None
    if not isinstance(items, list):
        raise ValueError("invalid audio shard manifest")
    keys: set[tuple[str, int]] = set()
    shard_ids: set[str] = set()
    reading_order_keys: set[str] = set()
    issues: list[dict[str, object]] = []
    previous_start_ms: int | None = None
    for index, item in enumerate(items, start=1):
        source, chunk_index = _manifest_source_key(item, index)
        row_issues = _manifest_row_issues(
            item,
            shard_ids=shard_ids,
            reading_order_keys=reading_order_keys,
            previous_start_ms=previous_start_ms,
        )
        start_ms = item.get("startMs")
        if isinstance(start_ms, int):
            previous_start_ms = start_ms
        if row_issues:
            issues.append(
                {
                    "row": index,
                    "source": Path(source).name,
                    "chunkIndex": chunk_index,
                    "issues": row_issues,
                }
            )
        keys.add((Path(source).name, chunk_index))
    return {"keys": keys, "issues": issues}


def _reference_rows(path: Path) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    for line_number, raw_line in enumerate(
        path.read_text(encoding="utf-8").splitlines(), start=1
    ):
        if not raw_line.strip():
            continue
        row = json.loads(raw_line)
        if not isinstance(row, dict):
            raise ValueError(f"invalid reference row at line {line_number}")
        rows.append(row)
    return rows


def _reference_row_key(row: Mapping[str, object], line_number: int) -> tuple[str, int]:
    source = row.get("source")
    chunk_index = row.get("chunkIndex", row.get("chunk_index"))
    if not isinstance(source, str) or not isinstance(chunk_index, int):
        raise ValueError(f"invalid reference row at line {line_number}")
    return (Path(source).name, chunk_index)


def _manifest_source_key(item: object, index: int) -> tuple[str, int]:
    if not isinstance(item, dict):
        raise ValueError(f"invalid audio shard manifest item at line {index}")
    source = item.get("sourceId")
    chunk_index = item.get("chunkIndex")
    if not isinstance(source, str) or not isinstance(chunk_index, int):
        raise ValueError(f"invalid audio shard manifest item at line {index}")
    return source, chunk_index


def _manifest_row_issues(
    item: Mapping[str, object],
    *,
    shard_ids: set[str],
    reading_order_keys: set[str],
    previous_start_ms: int | None,
) -> list[str]:
    row_issues: list[str] = []
    _check_unique_text(item.get("shardId"), shard_ids, "duplicate-shard-id", row_issues)
    _check_unique_text(
        item.get("readingOrderKey"),
        reading_order_keys,
        "duplicate-reading-order-key",
        row_issues,
    )
    start_ms = item.get("startMs")
    duration_ms = item.get("durationMs")
    media_start_ms = item.get("mediaStartMs")
    media_duration_ms = item.get("mediaDurationMs")
    if not isinstance(start_ms, int):
        row_issues.append("missing-start-ms")
    elif previous_start_ms is not None and start_ms < previous_start_ms:
        row_issues.append("non-monotonic-start-ms")
    if not isinstance(duration_ms, int) or duration_ms <= 0:
        row_issues.append("invalid-duration-ms")
    if not isinstance(media_start_ms, int):
        row_issues.append("missing-media-start-ms")
    if not isinstance(media_duration_ms, int) or media_duration_ms <= 0:
        row_issues.append("invalid-media-duration-ms")
    if _media_window_misses_logical_chunk(
        start_ms, duration_ms, media_start_ms, media_duration_ms
    ):
        row_issues.append("media-window-does-not-cover-logical-chunk")
    return row_issues


def _check_unique_text(
    value: object,
    seen: set[str],
    issue: str,
    row_issues: list[str],
) -> None:
    if isinstance(value, str) and value:
        if value in seen:
            row_issues.append(issue)
        seen.add(value)


def _media_window_misses_logical_chunk(
    start_ms: object,
    duration_ms: object,
    media_start_ms: object,
    media_duration_ms: object,
) -> bool:
    return (
        isinstance(start_ms, int)
        and isinstance(duration_ms, int)
        and isinstance(media_start_ms, int)
        and isinstance(media_duration_ms, int)
        and (
            media_start_ms > start_ms
            or media_start_ms + media_duration_ms < start_ms + duration_ms
        )
    )
