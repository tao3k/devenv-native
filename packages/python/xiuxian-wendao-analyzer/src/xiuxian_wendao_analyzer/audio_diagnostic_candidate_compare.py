"""Audio diagnostic candidate comparison helpers."""

from __future__ import annotations

from typing import TYPE_CHECKING

from xiuxian_wendao_analyzer.audio_diagnostic_candidate_inputs import (
    candidate_from_summary_path,
)

if TYPE_CHECKING:
    from collections.abc import Sequence
    from pathlib import Path

    from xiuxian_wendao_analyzer.audio_diagnostic_candidate_types import (
        AudioCandidateSummary,
    )


def compare_audio_candidate_summaries(
    summary_paths: Sequence[Path],
) -> dict[str, object]:
    """Compare diagnostic summaries with precision as the hard gate."""

    candidates = [candidate_from_summary_path(path) for path in summary_paths]
    eligible = [
        candidate
        for candidate in candidates
        if (
            candidate.precision_gate_passed
            and candidate.timeline_structure_passed
            and candidate.quality_proxy_passed
        )
    ]
    ranked = sorted(eligible, key=_candidate_rank_key)
    winner = ranked[0] if ranked else None
    return {
        "candidateCount": len(candidates),
        "eligiblePrecisionCandidateCount": sum(
            1 for candidate in candidates if candidate.precision_gate_passed
        ),
        "eligibleTimelineCandidateCount": sum(
            1 for candidate in candidates if candidate.timeline_structure_passed
        ),
        "eligibleQualityCandidateCount": sum(
            1 for candidate in candidates if candidate.quality_proxy_passed
        ),
        "eligiblePromotionCandidateCount": len(eligible),
        "promotionCandidate": winner.label if winner is not None else "",
        "promotionReason": _promotion_reason(winner),
        "precisionFirst": True,
        "timelineStructureRequired": True,
        "qualityProxyRequired": True,
        "candidates": [candidate.as_json() for candidate in candidates],
        "rankedCandidates": [candidate.label for candidate in ranked],
    }


def _candidate_rank_key(candidate: AudioCandidateSummary) -> tuple[object, ...]:
    return (
        _cer_sort_key(candidate.max_observed_reference_cer),
        candidate.reference_fail_rows,
        candidate.failed_rows,
        candidate.required_term_miss_rows,
        _wall_sort_key(candidate.wall_seconds),
        candidate.label,
    )


def _promotion_reason(candidate: AudioCandidateSummary | None) -> str:
    if candidate is None:
        return "no-precision-timeline-quality-candidate"
    return "lowest-cer-then-wall-time"


def _cer_sort_key(value: float | None) -> float:
    return float("inf") if value is None else value


def _wall_sort_key(value: float | None) -> float:
    return float("inf") if value is None else value
