"""Audio diagnostic quality summary helpers."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Sequence

    from xiuxian_wendao_analyzer.audio_diagnostic_quality import QualityRow


def summarize_quality(rows: Sequence[QualityRow]) -> dict[str, object]:
    """Summarize quality rows by backend and review status."""

    by_backend: dict[str, dict[str, object]] = {}
    for row in rows:
        item = by_backend.setdefault(
            row.backend,
            {
                "rows": 0,
                "failed": 0,
                "reviewNeeded": 0,
                "weakRows": 0,
                "shortUtteranceRows": 0,
                "referencePass": 0,
                "referenceFail": 0,
                "requiredTermMiss": 0,
                "avgCharsPerMinute": 0.0,
                "avgChineseRatio": 0.0,
                "avgInaudiblePerMinute": 0.0,
                "avgRepeatedNgramRatio": 0.0,
                "avgRequiredTermRecall": 0.0,
                "requiredTermRows": 0,
            },
        )
        item["rows"] = int(item["rows"]) + 1
        item["failed"] = int(item["failed"]) + (1 if row.review_status == "failed" else 0)
        item["reviewNeeded"] = int(item["reviewNeeded"]) + (
            1 if row.review_status == "review-needed" else 0
        )
        item["weakRows"] = int(item["weakRows"]) + (
            1 if row.review_status.startswith("weak-") else 0
        )
        item["shortUtteranceRows"] = int(item["shortUtteranceRows"]) + (
            1 if row.review_status == "short-utterance-review" else 0
        )
        item["referencePass"] = int(item["referencePass"]) + (
            1 if row.review_status == "reference-pass" else 0
        )
        item["referenceFail"] = int(item["referenceFail"]) + (
            1 if row.review_status == "reference-fail" else 0
        )
        item["requiredTermMiss"] = int(item["requiredTermMiss"]) + (
            1 if row.review_status == "required-term-miss" else 0
        )
        item["avgCharsPerMinute"] = float(item["avgCharsPerMinute"]) + row.chars_per_minute
        item["avgChineseRatio"] = float(item["avgChineseRatio"]) + (row.chinese_ratio or 0.0)
        item["avgInaudiblePerMinute"] = (
            float(item["avgInaudiblePerMinute"]) + row.inaudible_per_minute
        )
        item["avgRepeatedNgramRatio"] = (
            float(item["avgRepeatedNgramRatio"]) + row.repeated_ngram_ratio
        )
        if row.required_term_recall is not None:
            item["avgRequiredTermRecall"] = (
                float(item["avgRequiredTermRecall"]) + row.required_term_recall
            )
            item["requiredTermRows"] = int(item["requiredTermRows"]) + 1
    for item in by_backend.values():
        row_count = int(item["rows"])
        if row_count:
            item["avgCharsPerMinute"] = float(item["avgCharsPerMinute"]) / row_count
            item["avgChineseRatio"] = float(item["avgChineseRatio"]) / row_count
            item["avgInaudiblePerMinute"] = float(item["avgInaudiblePerMinute"]) / row_count
            item["avgRepeatedNgramRatio"] = float(item["avgRepeatedNgramRatio"]) / row_count
        term_row_count = int(item["requiredTermRows"])
        if term_row_count:
            item["avgRequiredTermRecall"] = float(item["avgRequiredTermRecall"]) / term_row_count
    return {"qualityByBackend": by_backend}


def summarize_timeline_structure(
    rows: Sequence[QualityRow], *, allow_planned_gaps: bool = False
) -> dict[str, object]:
    """Summarize timestamp coverage and order for audio transcript rows."""

    by_backend: dict[str, dict[str, object]] = {}
    for backend in sorted({row.backend for row in rows}):
        backend_rows = sorted(
            (row for row in rows if row.backend == backend),
            key=lambda row: (row.source, row.start_seconds, row.chunk_index),
        )
        item = _timeline_backend_summary(backend_rows, allow_planned_gaps=allow_planned_gaps)
        by_backend[backend] = item
    return {
        "timelineStructureByBackend": by_backend,
        "timelineGapPolicy": ("planned-gaps-allowed" if allow_planned_gaps else "contiguous"),
        "timelineStructurePassed": all(bool(item["passed"]) for item in by_backend.values()),
    }


def _timeline_backend_summary(
    rows: Sequence[QualityRow], *, allow_planned_gaps: bool
) -> dict[str, object]:
    source_summaries: dict[str, dict[str, object]] = {}
    for source in sorted({row.source for row in rows}):
        source_rows = sorted(
            (row for row in rows if row.source == source),
            key=lambda row: (row.start_seconds, row.chunk_index),
        )
        source_summaries[source] = _timeline_source_summary(source_rows)
    gap_seconds = sum(float(item["gapSeconds"]) for item in source_summaries.values())
    overlap_seconds = sum(float(item["overlapSeconds"]) for item in source_summaries.values())
    coverage_seconds = sum(float(item["coverageSeconds"]) for item in source_summaries.values())
    expected_span_seconds = sum(
        float(item["expectedSpanSeconds"]) for item in source_summaries.values()
    )
    row_count = sum(int(item["rows"]) for item in source_summaries.values())
    ordered = all(bool(item["ordered"]) for item in source_summaries.values())
    passed = (
        row_count > 0
        and ordered
        and overlap_seconds == 0.0
        and (allow_planned_gaps or gap_seconds == 0.0)
    )
    return {
        "rows": row_count,
        "sources": len(source_summaries),
        "ordered": ordered,
        "coverageSeconds": coverage_seconds,
        "expectedSpanSeconds": expected_span_seconds,
        "coverageRatio": (
            coverage_seconds / expected_span_seconds if expected_span_seconds else 0.0
        ),
        "gapSeconds": gap_seconds,
        "overlapSeconds": overlap_seconds,
        "passed": passed,
        "bySource": source_summaries,
    }


def _timeline_source_summary(rows: Sequence[QualityRow]) -> dict[str, object]:
    if not rows:
        return {
            "rows": 0,
            "ordered": False,
            "coverageSeconds": 0.0,
            "expectedSpanSeconds": 0.0,
            "coverageRatio": 0.0,
            "gapSeconds": 0.0,
            "overlapSeconds": 0.0,
        }
    ordered = all(
        rows[index].start_seconds <= rows[index + 1].start_seconds for index in range(len(rows) - 1)
    )
    coverage_seconds = sum(row.duration_seconds for row in rows)
    start = min(row.start_seconds for row in rows)
    end = max(row.start_seconds + row.duration_seconds for row in rows)
    expected_span_seconds = end - start
    gap_seconds = 0.0
    overlap_seconds = 0.0
    previous_end = rows[0].start_seconds + rows[0].duration_seconds
    for row in rows[1:]:
        if row.start_seconds > previous_end:
            gap_seconds += row.start_seconds - previous_end
        elif row.start_seconds < previous_end:
            overlap_seconds += previous_end - row.start_seconds
        previous_end = max(previous_end, row.start_seconds + row.duration_seconds)
    return {
        "rows": len(rows),
        "ordered": ordered,
        "coverageSeconds": coverage_seconds,
        "expectedSpanSeconds": expected_span_seconds,
        "coverageRatio": (
            coverage_seconds / expected_span_seconds if expected_span_seconds else 0.0
        ),
        "gapSeconds": gap_seconds,
        "overlapSeconds": overlap_seconds,
    }


def summarize_precision_gate(
    rows: Sequence[QualityRow],
    *,
    reference_configured: bool,
    reference_candidate_draft_rows: int = 0,
    max_reference_cer: float,
    required_terms_configured: bool,
) -> dict[str, object]:
    """Return promotion-gate status from reference and critical-term checks."""

    reference_rows = [row for row in rows if row.reference_cer is not None and row.status == "ok"]
    reference_missing_rows = [
        row for row in rows if row.status == "ok" and row.reference_cer is None
    ]
    failed_rows = [row for row in rows if row.review_status == "failed"]
    weak_quality_rows = [row for row in rows if row.review_status.startswith("weak-")]
    reference_fail_rows = [row for row in rows if row.review_status == "reference-fail"]
    required_term_miss_rows = [row for row in rows if row.review_status == "required-term-miss"]
    max_observed_cer = max(row.reference_cer for row in reference_rows) if reference_rows else None
    precision_gate_passed = (
        reference_configured
        and reference_candidate_draft_rows == 0
        and not reference_missing_rows
        and not failed_rows
        and not weak_quality_rows
        and not reference_fail_rows
        and not required_term_miss_rows
    )
    if failed_rows:
        reason = "backend-failed-rows"
    elif weak_quality_rows:
        reason = "quality-weak-rows"
    elif not reference_configured:
        reason = "reference-not-configured"
    elif reference_candidate_draft_rows:
        reason = "reference-candidate-draft"
    elif reference_missing_rows:
        reason = "reference-coverage-missing"
    elif reference_fail_rows:
        reason = "reference-cer-failed"
    elif required_term_miss_rows:
        reason = "required-term-missing"
    else:
        reason = "passed"
    return {
        "precisionGatePassed": precision_gate_passed,
        "precisionGateReason": reason,
        "maxReferenceCer": max_reference_cer,
        "maxObservedReferenceCer": max_observed_cer,
        "referenceCoverageRows": len(reference_rows),
        "referenceMissingRows": len(reference_missing_rows),
        "referenceCandidateDraftRows": reference_candidate_draft_rows,
        "referenceFailRows": len(reference_fail_rows),
        "failedRows": len(failed_rows),
        "weakQualityRows": len(weak_quality_rows),
        "requiredTermMissRows": len(required_term_miss_rows),
        "criticalTermsConfigured": required_terms_configured,
    }


def summarize_reference_subset(rows: Sequence[QualityRow]) -> dict[str, object]:
    """Summarize curated-reference rows without changing the promotion gate."""

    reference_rows = [row for row in rows if row.reference_cer is not None and row.status == "ok"]
    cer_values = [float(row.reference_cer) for row in reference_rows]
    return {
        "referenceSubsetConfigured": bool(reference_rows),
        "referenceSubsetRows": len(reference_rows),
        "referenceSubsetPassRows": sum(
            1 for row in reference_rows if row.review_status == "reference-pass"
        ),
        "referenceSubsetFailRows": sum(
            1 for row in reference_rows if row.review_status == "reference-fail"
        ),
        "referenceSubsetMaxObservedCer": max(cer_values) if cer_values else None,
        "referenceSubsetAvgObservedCer": (
            sum(cer_values) / len(cer_values) if cer_values else None
        ),
    }
