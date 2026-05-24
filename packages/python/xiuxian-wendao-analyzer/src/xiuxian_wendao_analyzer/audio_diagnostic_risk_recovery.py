"""Build timestamp-authoritative audio risk recovery plans."""

from __future__ import annotations

import json
import math
from dataclasses import dataclass
from typing import TYPE_CHECKING, Any

from xiuxian_wendao_analyzer.audio_diagnostic_results import write_json

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence
    from pathlib import Path

RISK_RECOVERY_PLAN_SCHEMA = "xiuxian_wendao.audio_risk_recovery_plan.v1"


@dataclass(frozen=True)
class AudioRiskRecoveryOptions:
    """Thresholds for selecting audio windows that need short reprocessing."""

    split_seconds: float = 30.0
    limit_parents: int = 20
    min_repeated_ngram_ratio: float = 0.14
    max_chinese_ratio: float = 0.85
    max_chars_per_minute: float = 180.0
    min_latency_seconds: float = 55.0
    include_boundaries: bool = True


def build_risk_recovery_plan_report(
    *,
    quality_json: Path,
    results_json: Path | None,
    output_json: Path | None,
    options: AudioRiskRecoveryOptions | None = None,
) -> dict[str, object]:
    """Build and optionally write an explicit-window short recovery plan."""

    effective_options = options or AudioRiskRecoveryOptions()
    quality_rows = _read_quality_rows(quality_json)
    result_rows = _read_result_rows(results_json) if results_json is not None else {}
    selected = select_audio_risk_parent_rows(
        quality_rows,
        result_rows=result_rows,
        options=effective_options,
    )
    recovery_rows = build_short_window_rows(
        selected,
        split_seconds=effective_options.split_seconds,
    )
    report: dict[str, object] = {
        "schema": RISK_RECOVERY_PLAN_SCHEMA,
        "qualityJson": str(quality_json),
        "resultsJson": "" if results_json is None else str(results_json),
        "outputJson": "" if output_json is None else str(output_json),
        "selectionPolicy": "timestamp-risk-short-window-recovery",
        "selectedParentRows": len(selected),
        "recoveryRows": len(recovery_rows),
        "options": effective_options.__dict__,
        "selectedParents": selected,
        "rows": recovery_rows,
    }
    if output_json is not None:
        write_json(output_json, report)
    return report


def select_audio_risk_parent_rows(
    quality_rows: Sequence[Mapping[str, object]],
    *,
    result_rows: Mapping[int, Mapping[str, object]],
    options: AudioRiskRecoveryOptions,
) -> list[dict[str, object]]:
    """Select base timeline rows that should be reprocessed in short windows."""

    if options.limit_parents <= 0:
        raise ValueError("risk recovery limit_parents must be positive")
    sorted_rows = sorted(
        quality_rows,
        key=lambda row: (_float(row, "start_seconds"), _int(row, "chunk_index")),
    )
    candidates: list[dict[str, object]] = []
    last_index = len(sorted_rows) - 1
    for offset, row in enumerate(sorted_rows):
        chunk_index = _int(row, "chunk_index")
        reasons = _risk_reasons(
            row,
            result_rows.get(chunk_index),
            is_boundary=offset == 0 or offset == last_index,
            options=options,
        )
        if not reasons:
            continue
        item = _parent_row(row, result_rows.get(chunk_index), reasons)
        item["_score"] = _risk_score(row, result_rows.get(chunk_index), reasons, options)
        candidates.append(item)
    selected = _select_with_boundary_reservation(candidates, options.limit_parents)
    for item in selected:
        item.pop("_score", None)
    return selected


def _select_with_boundary_reservation(
    candidates: Sequence[dict[str, object]],
    limit: int,
) -> list[dict[str, object]]:
    boundary = [item for item in candidates if "timeline-boundary" in set(item.get("reasons", []))]
    boundary_by_parent = {
        int(item["parentChunkIndex"]): item
        for item in sorted(
            boundary,
            key=lambda item: (
                float(item["startSeconds"]),
                int(item["parentChunkIndex"]),
            ),
        )
    }
    selected_by_parent = dict(boundary_by_parent)
    remaining_slots = max(0, limit - len(selected_by_parent))
    ranked = sorted(
        (item for item in candidates if int(item["parentChunkIndex"]) not in selected_by_parent),
        key=lambda item: item["_score"],
        reverse=True,
    )
    for item in ranked[:remaining_slots]:
        selected_by_parent[int(item["parentChunkIndex"])] = item
    return sorted(
        selected_by_parent.values(),
        key=lambda item: (float(item["startSeconds"]), int(item["parentChunkIndex"])),
    )


def build_short_window_rows(
    parent_rows: Sequence[Mapping[str, object]],
    *,
    split_seconds: float,
) -> list[dict[str, object]]:
    """Split selected parent rows into explicit short-window rows."""

    if split_seconds <= 0:
        raise ValueError("risk recovery split_seconds must be positive")
    rows: list[dict[str, object]] = []
    for parent in parent_rows:
        parent_chunk_index = _int(parent, "parentChunkIndex")
        start_seconds = _float(parent, "startSeconds")
        duration_seconds = _float(parent, "durationSeconds")
        if duration_seconds <= 0:
            raise ValueError(f"parent chunk {parent_chunk_index} has non-positive duration")
        part_count = max(1, math.ceil(duration_seconds / split_seconds))
        parent_reasons = [
            str(reason)
            for reason in parent.get("reasons", [])
            if isinstance(reason, str) and reason
        ]
        for part in range(part_count):
            part_start = start_seconds + part * split_seconds
            remaining = start_seconds + duration_seconds - part_start
            part_duration = min(split_seconds, remaining)
            if part_duration <= 0:
                continue
            rows.append(
                {
                    "chunkIndex": parent_chunk_index * 10 + part,
                    "parentChunkIndex": parent_chunk_index,
                    "startSeconds": part_start,
                    "durationSeconds": part_duration,
                    "reasons": ["short-window-reprocess", *parent_reasons],
                }
            )
    return rows


def _risk_reasons(
    quality_row: Mapping[str, object],
    result_row: Mapping[str, object] | None,
    *,
    is_boundary: bool,
    options: AudioRiskRecoveryOptions,
) -> list[str]:
    reasons: list[str] = []
    if _float(quality_row, "repeated_ngram_ratio") >= options.min_repeated_ngram_ratio:
        reasons.append("high-repetition")
    if _float(quality_row, "chinese_ratio") <= options.max_chinese_ratio:
        reasons.append("low-chinese-ratio")
    if _float(quality_row, "chars_per_minute") <= options.max_chars_per_minute:
        reasons.append("low-text-density")
    if result_row is not None and _float(result_row, "wall_seconds") >= options.min_latency_seconds:
        reasons.append("high-latency")
    if is_boundary and options.include_boundaries:
        reasons.append("timeline-boundary")
    return reasons


def _risk_score(
    quality_row: Mapping[str, object],
    result_row: Mapping[str, object] | None,
    reasons: Sequence[str],
    options: AudioRiskRecoveryOptions,
) -> float:
    score = float(len(reasons))
    score += max(
        0.0,
        (_float(quality_row, "repeated_ngram_ratio") - options.min_repeated_ngram_ratio) * 10.0,
    )
    score += max(0.0, options.max_chinese_ratio - _float(quality_row, "chinese_ratio"))
    score += max(
        0.0,
        (options.max_chars_per_minute - _float(quality_row, "chars_per_minute"))
        / max(options.max_chars_per_minute, 1.0),
    )
    if result_row is not None:
        score += max(
            0.0,
            (_float(result_row, "wall_seconds") - options.min_latency_seconds)
            / max(options.min_latency_seconds, 1.0),
        )
    return score


def _parent_row(
    quality_row: Mapping[str, object],
    result_row: Mapping[str, object] | None,
    reasons: Sequence[str],
) -> dict[str, object]:
    chunk_index = _int(quality_row, "chunk_index")
    return {
        "parentChunkIndex": chunk_index,
        "source": str(quality_row.get("source", "")),
        "backend": str(quality_row.get("backend", "")),
        "model": str(quality_row.get("model", "")),
        "startSeconds": _float(quality_row, "start_seconds"),
        "durationSeconds": _float(quality_row, "duration_seconds"),
        "transcriptChars": _int(quality_row, "transcript_chars"),
        "charsPerMinute": _float(quality_row, "chars_per_minute"),
        "chineseRatio": _float(quality_row, "chinese_ratio"),
        "repeatedNgramRatio": _float(quality_row, "repeated_ngram_ratio"),
        "requestSeconds": (0.0 if result_row is None else _float(result_row, "wall_seconds")),
        "reasons": list(reasons),
    }


def _read_quality_rows(path: Path) -> list[dict[str, object]]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    rows = payload.get("qualityRows", []) if isinstance(payload, dict) else payload
    if not isinstance(rows, list):
        raise ValueError(f"quality JSON must be an array or contain qualityRows: {path}")
    return _object_rows(rows, path)


def _read_result_rows(path: Path) -> dict[int, dict[str, object]]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, list):
        raise ValueError(f"results JSON must be an array: {path}")
    return {
        _int(row, "chunk_index"): row for row in _object_rows(payload, path) if "chunk_index" in row
    }


def _object_rows(rows: Sequence[object], path: Path) -> list[dict[str, object]]:
    typed: list[dict[str, object]] = []
    for index, row in enumerate(rows):
        if not isinstance(row, dict):
            raise ValueError(f"row {index} must be a JSON object: {path}")
        typed.append(row)
    return typed


def _int(row: Mapping[str, object], key: str) -> int:
    value: Any = row.get(key)
    if isinstance(value, bool):
        return 0
    if isinstance(value, int):
        return value
    if isinstance(value, float):
        return int(value)
    return 0


def _float(row: Mapping[str, object], key: str) -> float:
    value: Any = row.get(key)
    if isinstance(value, bool):
        return 0.0
    if isinstance(value, int | float):
        return float(value)
    return 0.0
