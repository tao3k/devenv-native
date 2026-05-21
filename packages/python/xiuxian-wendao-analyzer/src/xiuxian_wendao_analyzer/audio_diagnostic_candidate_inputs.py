"""Audio diagnostic candidate summary parsing."""

from __future__ import annotations

import json
from typing import TYPE_CHECKING

from xiuxian_wendao_analyzer.audio_diagnostic_candidate_types import (
    AudioCandidateSummary,
)

if TYPE_CHECKING:
    from pathlib import Path


def candidate_from_summary_path(path: Path) -> AudioCandidateSummary:
    """Read one diagnostic summary and return a comparable candidate."""

    return _candidate_from_summary(path, _read_summary(path))


def _read_summary(path: Path) -> dict[str, object]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        msg = f"summary must be a JSON object: {path}"
        raise ValueError(msg)
    return payload


def _candidate_from_summary(
    path: Path,
    summary: dict[str, object],
) -> AudioCandidateSummary:
    backend = _first_backend(summary.get("byBackend"))
    quality = _mapping_for_backend(summary.get("qualityByBackend"), backend)
    metrics = _mapping_for_backend(summary.get("byBackend"), backend)
    timeline = _mapping_for_backend(summary.get("timelineStructureByBackend"), backend)
    quality_passed, quality_reason = _quality_proxy(quality)
    return AudioCandidateSummary(
        summary_path=str(path),
        label=_candidate_label(path, backend, summary),
        backend=backend,
        model=_model_label(summary),
        precision_gate_passed=summary.get("precisionGatePassed") is True,
        precision_gate_reason=_string_value(summary.get("precisionGateReason")),
        timeline_structure_passed=_timeline_passed(summary, timeline),
        timeline_coverage_ratio=_float_or_none(timeline.get("coverageRatio")),
        timeline_gap_seconds=_float_or_none(timeline.get("gapSeconds")),
        timeline_overlap_seconds=_float_or_none(timeline.get("overlapSeconds")),
        quality_proxy_passed=quality_passed,
        quality_proxy_reason=quality_reason,
        weak_rows=_int_value(quality.get("weakRows")),
        short_utterance_rows=_int_value(quality.get("shortUtteranceRows")),
        avg_inaudible_per_minute=_float_or_none(quality.get("avgInaudiblePerMinute")),
        max_observed_reference_cer=_float_or_none(
            summary.get("maxObservedReferenceCer")
        ),
        reference_coverage_rows=_int_value(summary.get("referenceCoverageRows")),
        reference_fail_rows=_int_value(summary.get("referenceFailRows")),
        failed_rows=_int_value(summary.get("failedRows")),
        required_term_miss_rows=_int_value(summary.get("requiredTermMissRows")),
        wall_seconds=_candidate_wall_seconds(summary, metrics),
        request_wall_seconds=_float_or_none(metrics.get("wallSeconds")),
        latency_p50_seconds=_float_or_none(metrics.get("latencyP50Seconds")),
        latency_p95_seconds=_float_or_none(metrics.get("latencyP95Seconds")),
        transcript_chars=_int_value(metrics.get("transcriptChars")),
        avg_chinese_ratio=_float_or_none(quality.get("avgChineseRatio")),
        avg_repeated_ngram_ratio=_float_or_none(quality.get("avgRepeatedNgramRatio")),
    )


def _first_backend(raw: object) -> str:
    if isinstance(raw, dict) and raw:
        return sorted(str(key) for key in raw)[0]
    return ""


def _mapping_for_backend(raw: object, backend: str) -> dict[str, object]:
    if isinstance(raw, dict):
        value = raw.get(backend)
        if isinstance(value, dict):
            return value
    return {}


def _timeline_passed(
    summary: dict[str, object],
    timeline: dict[str, object],
) -> bool:
    if timeline:
        return timeline.get("passed") is True
    return summary.get("timelineStructurePassed") is not False


def _quality_proxy(quality: dict[str, object]) -> tuple[bool, str]:
    weak_rows = _int_value(quality.get("weakRows"))
    repeated_ratio = _float_or_none(quality.get("avgRepeatedNgramRatio"))
    inaudible_per_minute = _float_or_none(quality.get("avgInaudiblePerMinute"))
    if weak_rows:
        return False, "weak-quality-rows"
    if repeated_ratio is not None and repeated_ratio > 0.35:
        return False, "repetition-heavy"
    if inaudible_per_minute is not None and inaudible_per_minute > 30.0:
        return False, "inaudible-heavy"
    return True, "passed"


def _candidate_label(
    path: Path,
    backend: str,
    summary: dict[str, object],
) -> str:
    model = _model_label(summary)
    if backend and model:
        return f"{backend}:{model}"
    if backend:
        return backend
    return path.parent.name


def _model_label(summary: dict[str, object]) -> str:
    for key in ("hostedAudioModel", "openRouterModel", "localAsrModel"):
        value = summary.get(key)
        if isinstance(value, str) and value.strip():
            return value
    return ""


def _candidate_wall_seconds(
    summary: dict[str, object], metrics: dict[str, object]
) -> float | None:
    diagnostic_wall_seconds = _float_or_none(summary.get("diagnosticWallSeconds"))
    if diagnostic_wall_seconds is not None:
        return diagnostic_wall_seconds
    return _float_or_none(metrics.get("wallSeconds"))


def _int_value(value: object) -> int:
    return value if isinstance(value, int) else 0


def _float_or_none(value: object) -> float | None:
    if isinstance(value, bool):
        return None
    if isinstance(value, int | float):
        return float(value)
    return None


def _string_value(value: object) -> str:
    return value if isinstance(value, str) else ""
