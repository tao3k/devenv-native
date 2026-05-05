"""Precision and speed observation helpers for document extraction reports."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable

    from .common import Any

    NumericGetter = Callable[[dict[str, Any]], float | None]


PDF_OCR_MILESTONE_BASELINE = {
    "id": "arxiv-2604.17337-source-range-ocr",
    "source": "packages/rust/crates/xiuxian-wendao/docs/05_research/308_document_extract_pr_closing_report.md",
    "referenceForceRefreshMs": 45941.076,
    "referenceSourceRangeOverride4Ms": 43917.250,
    "bestHistoricalSourceRangeOverride4Ms": 19442.132,
    "maxCacheHitP95Ms": 23.209,
    "maxShardCacheReuseForceMs": 213.161,
    "resourcesRows": 21,
    "structureRows": 21,
    "ocrPageBlocks": 21,
    "bboxBlocks": 21,
    "metricsRows": 21,
    "minMetricsResultChars": 103_984,
}


def precision_speed_summary(
    results: list[dict[str, Any]],
    distinct_miss_report: dict[str, Any] | None,
    *,
    total_error_rows: int,
    artifact_error_count: int,
    structure_parity_error_count: int,
    structure_reading_order_sorted: bool | None,
    structure_order_stable: bool | None,
    structure_order_mismatch_count: int,
    structure_parity_passed: bool | None,
) -> dict[str, Any]:
    precision_gate_passed = (
        total_error_rows == 0
        and artifact_error_count == 0
        and structure_parity_error_count == 0
        and structure_reading_order_sorted is not False
        and structure_order_stable is not False
        and structure_order_mismatch_count == 0
        and structure_parity_passed is not False
    )
    return {
        "precisionGatePassed": precision_gate_passed,
        "errorRows": total_error_rows,
        "artifactErrors": artifact_error_count,
        "structureReadingOrderSorted": structure_reading_order_sorted,
        "structureOrderStable": structure_order_stable,
        "structureOrderMismatches": structure_order_mismatch_count,
        "structureParityPassed": structure_parity_passed,
        "structureParityErrors": structure_parity_error_count,
        "structureRows": sum(result.get("structureRows", 0) for result in results),
        "ocrPageBlocks": sum(
            result.get("structureOcrPageBlocks", 0) for result in results
        ),
        "ocrRegionBlocks": sum(
            result.get("structureOcrRegionBlocks", 0) for result in results
        ),
        "bboxBlocks": sum(result.get("structureBboxBlocks", 0) for result in results),
        "metricsRows": sum(result.get("metricsRows", 0) for result in results),
        "metricsResultChars": sum(
            result.get("metricsResultChars", 0) for result in results
        ),
        "pdfOcrMilestoneGuard": pdf_ocr_milestone_guard(results),
        **speed_observation_summary(results, distinct_miss_report),
    }


def pdf_ocr_milestone_guard(results: list[dict[str, Any]]) -> dict[str, Any]:
    observations = [
        pdf_ocr_milestone_observation(result)
        for result in results
        if is_pdf_ocr_milestone_candidate(result)
    ]
    regressions = [
        regression
        for observation in observations
        for regression in observation["regressions"]
    ]
    return {
        "checked": bool(observations),
        "passed": bool(observations) and not regressions,
        "baseline": PDF_OCR_MILESTONE_BASELINE,
        "observations": observations,
        "regressions": regressions,
        "reason": (
            None
            if observations
            else "no OCR-positive 21-page PDF milestone fixture observed"
        ),
    }


def is_pdf_ocr_milestone_candidate(result: dict[str, Any]) -> bool:
    return (
        result.get("attachmentClass") == "pdf"
        and result.get("resourcesRows") == PDF_OCR_MILESTONE_BASELINE["resourcesRows"]
        and result.get("structureRows") == PDF_OCR_MILESTONE_BASELINE["structureRows"]
        and result.get("structureOcrPageBlocks")
        == PDF_OCR_MILESTONE_BASELINE["ocrPageBlocks"]
        and result.get("structureBboxBlocks")
        == PDF_OCR_MILESTONE_BASELINE["bboxBlocks"]
        and result.get("metricsRows") == PDF_OCR_MILESTONE_BASELINE["metricsRows"]
        and result.get("metricsResultChars", 0)
        >= PDF_OCR_MILESTONE_BASELINE["minMetricsResultChars"]
    )


def pdf_ocr_milestone_observation(result: dict[str, Any]) -> dict[str, Any]:
    regressions: list[str] = []
    force_ms = numeric_or_none(result.get("forceRefreshMs"))
    cache_p95_ms = numeric_or_none(result.get("cacheHitP95Ms"))
    shard_cache_reuse_ms = numeric_or_none(result.get("shardCacheReuseForceMs"))
    structure_stable = result.get("structureOrderStable")
    structure_mismatches = result.get("structureOrderMismatchCount", 0)
    error_rows = (
        int(result.get("forceErrorRows", 0))
        + int(result.get("shardCacheReuseErrorRows", 0))
        + int(result.get("artifactRegistryReuseErrorRows", 0))
        + int(result.get("cacheErrorRows", 0))
    )

    if error_rows != 0:
        regressions.append(f"expected zero error rows, observed {error_rows}")
    if result.get("structureReadingOrderSorted") is False:
        regressions.append("structure reading order is not sorted")
    if structure_stable is False:
        regressions.append("structure order is not stable")
    if isinstance(structure_mismatches, int) and structure_mismatches != 0:
        regressions.append(
            f"expected zero structure order mismatches, observed {structure_mismatches}"
        )
    if force_ms is None:
        regressions.append("missing forceRefreshMs")
    elif force_ms > PDF_OCR_MILESTONE_BASELINE["referenceForceRefreshMs"]:
        regressions.append(
            "forceRefreshMs "
            f"{force_ms:.3f} exceeded baseline "
            f"{PDF_OCR_MILESTONE_BASELINE['referenceForceRefreshMs']:.3f}"
        )
    if (
        cache_p95_ms is not None
        and cache_p95_ms > PDF_OCR_MILESTONE_BASELINE["maxCacheHitP95Ms"]
    ):
        regressions.append(
            "cacheHitP95Ms "
            f"{cache_p95_ms:.3f} exceeded baseline "
            f"{PDF_OCR_MILESTONE_BASELINE['maxCacheHitP95Ms']:.3f}"
        )
    if (
        shard_cache_reuse_ms is not None
        and shard_cache_reuse_ms
        > PDF_OCR_MILESTONE_BASELINE["maxShardCacheReuseForceMs"]
    ):
        regressions.append(
            "shardCacheReuseForceMs "
            f"{shard_cache_reuse_ms:.3f} exceeded baseline "
            f"{PDF_OCR_MILESTONE_BASELINE['maxShardCacheReuseForceMs']:.3f}"
        )

    return {
        "fixture": result.get("fixture"),
        "source": result.get("source"),
        "forceRefreshMs": force_ms,
        "cacheHitP95Ms": cache_p95_ms,
        "shardCacheReuseForceMs": shard_cache_reuse_ms,
        "resourcesRows": result.get("resourcesRows"),
        "structureRows": result.get("structureRows"),
        "ocrPageBlocks": result.get("structureOcrPageBlocks"),
        "bboxBlocks": result.get("structureBboxBlocks"),
        "metricsRows": result.get("metricsRows"),
        "metricsResultChars": result.get("metricsResultChars"),
        "structureOrderStable": structure_stable,
        "structureOrderMismatchCount": structure_mismatches,
        "errorRows": error_rows,
        "passed": not regressions,
        "regressions": regressions,
    }


def all_structure_reading_order_sorted(results: list[dict[str, Any]]) -> bool | None:
    values = [
        result.get("structureReadingOrderSorted")
        for result in results
        if result.get("structureReadingOrderSorted") is not None
    ]
    return all(bool(value) for value in values) if values else None


def all_structure_parity_passed(results: list[dict[str, Any]]) -> bool | None:
    values = [
        result.get("structureParityPassed")
        for result in results
        if result.get("structureParityPassed") is not None
    ]
    return all(bool(value) for value in values) if values else None


def all_structure_order_stable(results: list[dict[str, Any]]) -> bool | None:
    values = [
        result.get("structureOrderStable")
        for result in results
        if result.get("structureOrderStable") is not None
    ]
    return all(bool(value) for value in values) if values else None


def structure_order_mismatch_count(results: list[dict[str, Any]]) -> int:
    return sum(
        int(value)
        for result in results
        if isinstance((value := result.get("structureOrderMismatchCount")), int)
    )


def speed_observation_summary(
    results: list[dict[str, Any]],
    distinct_miss_report: dict[str, Any] | None = None,
) -> dict[str, Any]:
    return {
        "maxForceRefreshMs": max_numeric(results, "forceRefreshMs"),
        "maxCacheHitP95Ms": max_numeric(results, "cacheHitP95Ms"),
        "maxShardCacheReuseForceMs": max_numeric(results, "shardCacheReuseForceMs"),
        "maxArtifactRegistryReuseForceMs": max_numeric(
            results,
            "artifactRegistryReuseForceMs",
        ),
        "maxWallTimeMs": max_numeric(results, "wallTimeMs"),
        "minCacheSpeedup": min_numeric(results, "cacheSpeedup"),
        "totalRustSchedulerElapsedMs": sum_numeric(
            results,
            "metricsRustSchedulerElapsedMs",
        ),
        "totalDocumentTimingElapsedMs": sum_numeric(
            results,
            "documentTimingTotalElapsedMs",
        ),
        "totalDoclingConvertMs": sum_nested_numeric(
            results,
            "documentTimingPhaseElapsedMs",
            "doclingConvert",
        ),
        "maxDoclingConvertMs": max_nested_numeric(
            results,
            "documentTimingPhaseElapsedMs",
            "doclingConvert",
        ),
        "maxDoclingConvertShare": max_ratio(
            results,
            nested_numeric_value("documentTimingPhaseElapsedMs", "doclingConvert"),
            numeric_value("documentTimingTotalElapsedMs"),
        ),
        "maxDocumentTimingOverheadMs": max_numeric(
            results,
            "documentTimingOverheadMs",
        ),
        "totalDocumentTimingOverheadMs": sum_numeric(
            results,
            "documentTimingOverheadMs",
        ),
        "maxDocumentTimingOverheadShare": max_ratio(
            results,
            numeric_value("documentTimingOverheadMs"),
            numeric_value("forceRefreshMs"),
        ),
        "distinctMissWallTimeMs": (
            distinct_miss_report.get("wallTimeMs") if distinct_miss_report else None
        ),
    }


def max_numeric(results: list[dict[str, Any]], key: str) -> float | None:
    values = numeric_values(results, key)
    return max(values) if values else None


def min_numeric(results: list[dict[str, Any]], key: str) -> float | None:
    values = numeric_values(results, key)
    return min(values) if values else None


def sum_numeric(results: list[dict[str, Any]], key: str) -> float:
    return sum(numeric_values(results, key))


def max_nested_numeric(
    results: list[dict[str, Any]],
    mapping_key: str,
    value_key: str,
) -> float | None:
    values = nested_numeric_values(results, mapping_key, value_key)
    return max(values) if values else None


def sum_nested_numeric(
    results: list[dict[str, Any]],
    mapping_key: str,
    value_key: str,
) -> float:
    return sum(nested_numeric_values(results, mapping_key, value_key))


def numeric_values(results: list[dict[str, Any]], key: str) -> list[float]:
    return [
        float(value)
        for result in results
        if isinstance((value := result.get(key)), int | float)
    ]


def numeric_or_none(value: Any) -> float | None:
    return float(value) if isinstance(value, int | float) else None


def nested_numeric_values(
    results: list[dict[str, Any]],
    mapping_key: str,
    value_key: str,
) -> list[float]:
    return [
        float(value)
        for result in results
        if isinstance((mapping := result.get(mapping_key)), dict)
        and isinstance((value := mapping.get(value_key)), int | float)
    ]


def max_ratio(
    results: list[dict[str, Any]],
    numerator_fn: NumericGetter,
    denominator_fn: NumericGetter,
) -> float | None:
    ratios: list[float] = []
    for result in results:
        numerator = numerator_fn(result)
        denominator = denominator_fn(result)
        if numerator is None or denominator is None or denominator <= 0.0:
            continue
        ratios.append(numerator / denominator)
    return max(ratios) if ratios else None


def numeric_value(key: str) -> NumericGetter:
    def getter(result: dict[str, Any]) -> float | None:
        value = result.get(key)
        return float(value) if isinstance(value, int | float) else None

    return getter


def nested_numeric_value(mapping_key: str, value_key: str) -> NumericGetter:
    def getter(result: dict[str, Any]) -> float | None:
        mapping = result.get(mapping_key)
        if not isinstance(mapping, dict):
            return None
        value = mapping.get(value_key)
        return float(value) if isinstance(value, int | float) else None

    return getter
