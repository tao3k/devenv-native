"""Gate short-window audio recovery patches against base transcript rows."""

from __future__ import annotations

import json
from dataclasses import dataclass
from typing import TYPE_CHECKING, Any

from xiuxian_wendao_analyzer.audio_diagnostic_results import write_json

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence
    from pathlib import Path

AUDIO_RECOVERY_PATCH_GATE_SCHEMA = "xiuxian_wendao.audio_recovery_patch_gate.v1"


@dataclass(frozen=True)
class AudioRecoveryPatchGateOptions:
    """Thresholds for deciding whether short-window recovery may patch a parent."""

    max_chinese_ratio_drop: float = 0.03
    min_char_ratio: float = 0.65
    max_char_ratio: float = 1.40
    max_part_repeated_ngram_ratio: float = 0.35


def build_recovery_patch_gate_report(
    *,
    base_quality_json: Path,
    base_results_json: Path | None,
    recovery_quality_json: Path,
    recovery_results_json: Path | None,
    recovery_plan_json: Path,
    output_json: Path | None,
    options: AudioRecoveryPatchGateOptions | None = None,
) -> dict[str, object]:
    """Build and optionally write parent-level recovery patch decisions."""

    effective_options = options or AudioRecoveryPatchGateOptions()
    base_quality = _quality_rows_by_chunk(base_quality_json)
    base_results = _result_rows_by_chunk(base_results_json)
    recovery_quality = _quality_rows_by_chunk(recovery_quality_json)
    recovery_results = _result_rows_by_chunk(recovery_results_json)
    plan = _read_json_object(recovery_plan_json)
    parent_map = _plan_parent_map(plan)
    grouped = _group_recovery_rows(
        recovery_quality,
        recovery_results=recovery_results,
        parent_map=parent_map,
    )
    rows = [
        _patch_decision_row(
            parent_index=parent_index,
            base_quality=base_quality.get(parent_index),
            base_result=base_results.get(parent_index),
            recovery_rows=grouped[parent_index],
            options=effective_options,
        )
        for parent_index in sorted(grouped)
    ]
    accepted = sum(1 for row in rows if row["decision"] == "accept-patch")
    report: dict[str, object] = {
        "schema": AUDIO_RECOVERY_PATCH_GATE_SCHEMA,
        "baseQualityJson": str(base_quality_json),
        "baseResultsJson": "" if base_results_json is None else str(base_results_json),
        "recoveryQualityJson": str(recovery_quality_json),
        "recoveryResultsJson": (
            "" if recovery_results_json is None else str(recovery_results_json)
        ),
        "recoveryPlanJson": str(recovery_plan_json),
        "outputJson": "" if output_json is None else str(output_json),
        "options": effective_options.__dict__,
        "parentRows": len(rows),
        "acceptedPatches": accepted,
        "rejectedPatches": len(rows) - accepted,
        "rows": rows,
    }
    if output_json is not None:
        write_json(output_json, report)
    return report


def _patch_decision_row(
    *,
    parent_index: int,
    base_quality: Mapping[str, object] | None,
    base_result: Mapping[str, object] | None,
    recovery_rows: Sequence[Mapping[str, object]],
    options: AudioRecoveryPatchGateOptions,
) -> dict[str, object]:
    base_metrics = _base_metrics(base_quality, base_result)
    recovery_metrics = _recovery_metrics(
        recovery_rows,
        base_transcript_chars=int(base_metrics["transcriptChars"]),
    )
    reasons = _patch_rejection_reasons(
        base_metrics=base_metrics,
        recovery_metrics=recovery_metrics,
        recovery_rows=recovery_rows,
        options=options,
    )
    return {
        "parentChunkIndex": parent_index,
        "decision": "accept-patch" if not reasons else "reject-patch",
        "rejectionReasons": reasons,
        "base": base_metrics,
        "recovery": recovery_metrics,
    }


def _patch_rejection_reasons(
    *,
    base_metrics: Mapping[str, object],
    recovery_metrics: Mapping[str, object],
    recovery_rows: Sequence[Mapping[str, object]],
    options: AudioRecoveryPatchGateOptions,
) -> list[str]:
    reasons: list[str] = []
    if not base_metrics["present"]:
        reasons.append("missing-base-parent")
    if not recovery_rows:
        reasons.append("missing-recovery-rows")
    if any(str(row.get("status", "")) != "ok" for row in recovery_rows):
        reasons.append("recovery-row-failed")
    base_repeat = float(base_metrics["repeatedNgramRatio"])
    recovery_repeat = float(recovery_metrics["weightedRepeatedNgramRatio"])
    if recovery_repeat >= base_repeat:
        reasons.append("repeat-not-improved")
    chinese_drop = float(base_metrics["chineseRatio"]) - float(
        recovery_metrics["weightedChineseRatio"]
    )
    if chinese_drop > options.max_chinese_ratio_drop:
        reasons.append("chinese-ratio-drop")
    char_ratio = float(recovery_metrics["charRatio"])
    if char_ratio < options.min_char_ratio:
        reasons.append("char-collapse")
    if char_ratio > options.max_char_ratio:
        reasons.append("char-expansion")
    if float(recovery_metrics["maxPartRepeatedNgramRatio"]) > options.max_part_repeated_ngram_ratio:
        reasons.append("part-repeat-too-high")
    return reasons


def _base_metrics(
    quality_row: Mapping[str, object] | None,
    result_row: Mapping[str, object] | None,
) -> dict[str, object]:
    if quality_row is None:
        return {
            "present": False,
            "startSeconds": 0.0,
            "durationSeconds": 0.0,
            "transcriptChars": 0,
            "chineseRatio": 0.0,
            "repeatedNgramRatio": 0.0,
            "requestSeconds": 0.0,
        }
    return {
        "present": True,
        "startSeconds": _float(quality_row, "start_seconds"),
        "durationSeconds": _float(quality_row, "duration_seconds"),
        "transcriptChars": _int(quality_row, "transcript_chars"),
        "chineseRatio": _float(quality_row, "chinese_ratio"),
        "repeatedNgramRatio": _float(quality_row, "repeated_ngram_ratio"),
        "requestSeconds": (0.0 if result_row is None else _float(result_row, "wall_seconds")),
    }


def _recovery_metrics(
    rows: Sequence[Mapping[str, object]], *, base_transcript_chars: int
) -> dict[str, object]:
    transcript_chars = sum(_int(row, "transcript_chars") for row in rows)
    chunk_indexes = [_int(row, "chunk_index") for row in rows]
    wall_seconds = sum(_float(row, "_wallSeconds") for row in rows)
    return {
        "chunkIndexes": chunk_indexes,
        "rows": len(rows),
        "transcriptChars": transcript_chars,
        "charRatio": (
            0.0 if base_transcript_chars <= 0 else transcript_chars / base_transcript_chars
        ),
        "weightedRepeatedNgramRatio": _weighted_average(
            rows, "repeated_ngram_ratio", "transcript_chars"
        ),
        "maxPartRepeatedNgramRatio": max(
            (_float(row, "repeated_ngram_ratio") for row in rows),
            default=0.0,
        ),
        "weightedChineseRatio": _weighted_average(rows, "chinese_ratio", "transcript_chars"),
        "requestCumulativeSeconds": wall_seconds,
    }


def _group_recovery_rows(
    recovery_quality: Mapping[int, Mapping[str, object]],
    *,
    recovery_results: Mapping[int, Mapping[str, object]],
    parent_map: Mapping[int, int],
) -> dict[int, list[dict[str, object]]]:
    grouped: dict[int, list[dict[str, object]]] = {}
    for chunk_index, row in recovery_quality.items():
        parent_index = parent_map.get(chunk_index, chunk_index // 10)
        result_row = recovery_results.get(chunk_index, {})
        item = dict(row)
        item["status"] = str(result_row.get("status", "ok"))
        item["_wallSeconds"] = _float(result_row, "wall_seconds")
        grouped.setdefault(parent_index, []).append(item)
    for rows in grouped.values():
        rows.sort(key=lambda row: (_float(row, "start_seconds"), _int(row, "chunk_index")))
    return grouped


def _plan_parent_map(plan: Mapping[str, object]) -> dict[int, int]:
    rows = plan.get("rows", [])
    if not isinstance(rows, list):
        raise ValueError("recovery plan rows must be an array")
    mapping: dict[int, int] = {}
    for row in rows:
        if not isinstance(row, dict):
            continue
        chunk_index = _int(row, "chunkIndex")
        parent_index = _int(row, "parentChunkIndex")
        mapping[chunk_index] = parent_index
    return mapping


def _quality_rows_by_chunk(path: Path) -> dict[int, dict[str, object]]:
    rows = _read_rows(path)
    by_chunk: dict[int, dict[str, object]] = {}
    for row in rows:
        chunk_index = _int(row, "chunk_index")
        by_chunk[chunk_index] = dict(row)
    return by_chunk


def _result_rows_by_chunk(path: Path | None) -> dict[int, dict[str, object]]:
    if path is None:
        return {}
    return {_int(row, "chunk_index"): row for row in _read_rows(path)}


def _read_rows(path: Path) -> list[dict[str, object]]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    rows = payload.get("qualityRows", []) if isinstance(payload, dict) else payload
    if not isinstance(rows, list):
        raise ValueError(f"expected JSON array or qualityRows object: {path}")
    typed: list[dict[str, object]] = []
    for index, row in enumerate(rows):
        if not isinstance(row, dict):
            raise ValueError(f"row {index} must be a JSON object: {path}")
        typed.append(row)
    return typed


def _read_json_object(path: Path) -> dict[str, object]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError(f"expected JSON object: {path}")
    return payload


def _weighted_average(
    rows: Sequence[Mapping[str, object]], value_key: str, weight_key: str
) -> float:
    total_weight = sum(max(0, _int(row, weight_key)) for row in rows)
    if total_weight <= 0:
        return sum(_float(row, value_key) for row in rows) / len(rows) if rows else 0.0
    return (
        sum(_float(row, value_key) * max(0, _int(row, weight_key)) for row in rows) / total_weight
    )


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
