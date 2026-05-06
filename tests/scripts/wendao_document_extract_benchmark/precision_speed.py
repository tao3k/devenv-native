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
    "referenceAutoEndpointFanoutMs": 18969.021,
    "bestObservedAutoEndpointFanoutMs": 15811.373,
    "maxCacheHitP95Ms": 23.209,
    "maxShardCacheReuseForceMs": 213.161,
    "resourcesRows": 21,
    "structureRows": 21,
    "ocrPageBlocks": 21,
    "bboxBlocks": 21,
    "metricsRows": 21,
    "minMetricsResultChars": 103_984,
}

OCR2_PROMOTION_BASELINE = {
    "id": "arxiv-2604.17337-fast-risk-window-r9",
    "forceRefreshMs": 12_856.546292,
    "maxShardCacheReuseForceMs": PDF_OCR_MILESTONE_BASELINE[
        "maxShardCacheReuseForceMs"
    ],
    "minMetricsResultChars": PDF_OCR_MILESTONE_BASELINE["minMetricsResultChars"],
    "expectedOcrPageBlocks": PDF_OCR_MILESTONE_BASELINE["ocrPageBlocks"],
    "minMetricsRows": PDF_OCR_MILESTONE_BASELINE["metricsRows"],
}

OCR2_AUTOMATIC_REGION_PLANNERS = {
    "profile-risk-window",
    "profile-risk-window-slices",
    "profile-risk-window-adaptive",
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


def ocr2_promotion_gate(payload: dict[str, Any]) -> dict[str, Any]:
    summary = payload.get("summary", {})
    precision_speed = summary.get("precisionSpeedSummary", {})
    deepseek_ocr2 = payload.get("deepseekOcr2") or {}
    request_summary = deepseek_ocr2.get("requestSummary") or {}
    reasons: list[str] = []
    checked = ocr2_promotion_candidate(payload, precision_speed, request_summary)

    if not checked:
        return {
            "checked": False,
            "passed": False,
            "baseline": OCR2_PROMOTION_BASELINE,
            "reasons": ["not an OCR2 promotion candidate"],
            "observed": ocr2_promotion_observed(
                payload, precision_speed, request_summary
            ),
        }

    if precision_speed.get("precisionGatePassed") is not True:
        reasons.append("precision gate did not pass")
    if precision_speed.get("errorRows") != 0:
        reasons.append(
            f"expected zero error rows, observed {precision_speed.get('errorRows')}"
        )
    if precision_speed.get("structureReadingOrderSorted") is not True:
        reasons.append("structure reading order is not sorted")
    if precision_speed.get("structureOrderStable") is not True:
        reasons.append("structure order is not stable")
    if precision_speed.get("structureOrderMismatches") != 0:
        reasons.append(
            "expected zero structure order mismatches, observed "
            f"{precision_speed.get('structureOrderMismatches')}"
        )
    if (
        precision_speed.get("ocrPageBlocks")
        != OCR2_PROMOTION_BASELINE["expectedOcrPageBlocks"]
    ):
        reasons.append(
            "expected "
            f"{OCR2_PROMOTION_BASELINE['expectedOcrPageBlocks']} OCR page blocks, "
            f"observed {precision_speed.get('ocrPageBlocks')}"
        )
    if (
        precision_speed.get("metricsRows", 0)
        < OCR2_PROMOTION_BASELINE["minMetricsRows"]
    ):
        reasons.append(
            "metricsRows "
            f"{precision_speed.get('metricsRows')} below promotion floor "
            f"{OCR2_PROMOTION_BASELINE['minMetricsRows']}"
        )
    if (
        precision_speed.get("metricsResultChars", 0)
        < OCR2_PROMOTION_BASELINE["minMetricsResultChars"]
    ):
        reasons.append(
            "metricsResultChars "
            f"{precision_speed.get('metricsResultChars')} below promotion floor "
            f"{OCR2_PROMOTION_BASELINE['minMetricsResultChars']}"
        )
    force_ms = numeric_or_none(precision_speed.get("maxForceRefreshMs"))
    if force_ms is None:
        reasons.append("missing maxForceRefreshMs")
    elif force_ms > OCR2_PROMOTION_BASELINE["forceRefreshMs"]:
        reasons.append(
            "maxForceRefreshMs "
            f"{force_ms:.3f} exceeded promotion baseline "
            f"{OCR2_PROMOTION_BASELINE['forceRefreshMs']:.3f}"
        )
    shard_cache_reuse_ms = numeric_or_none(
        precision_speed.get("maxShardCacheReuseForceMs")
    )
    shard_cache_reuse_scheduler_ms = numeric_or_none(
        precision_speed.get("maxShardCacheReuseSchedulerElapsedMs")
    )
    has_ocr2_region_sidecars = precision_speed.get("ocrRegionBlocks", 0) > 0
    if shard_cache_reuse_ms is None:
        reasons.append("missing maxShardCacheReuseForceMs")
    elif has_ocr2_region_sidecars:
        if shard_cache_reuse_scheduler_ms is None:
            reasons.append("missing maxShardCacheReuseSchedulerElapsedMs")
        elif (
            shard_cache_reuse_scheduler_ms
            > OCR2_PROMOTION_BASELINE["maxShardCacheReuseForceMs"]
        ):
            reasons.append(
                "maxShardCacheReuseSchedulerElapsedMs "
                f"{shard_cache_reuse_scheduler_ms:.3f} exceeded promotion baseline "
                f"{OCR2_PROMOTION_BASELINE['maxShardCacheReuseForceMs']:.3f}"
            )
    elif shard_cache_reuse_ms > OCR2_PROMOTION_BASELINE["maxShardCacheReuseForceMs"]:
        reasons.append(
            "maxShardCacheReuseForceMs "
            f"{shard_cache_reuse_ms:.3f} exceeded promotion baseline "
            f"{OCR2_PROMOTION_BASELINE['maxShardCacheReuseForceMs']:.3f}"
        )
    if request_summary.get("requestCount", 0) <= 0:
        reasons.append("no OCR2 requests observed")
    if request_summary.get("successCount", 0) != request_summary.get("requestCount", 0):
        reasons.append(
            "OCR2 success count "
            f"{request_summary.get('successCount')} did not match request count "
            f"{request_summary.get('requestCount')}"
        )
    if request_summary.get("failureCount", 0) != 0:
        reasons.append(f"OCR2 failure count was {request_summary.get('failureCount')}")
    if request_summary.get("parseErrorCount", 0) != 0:
        reasons.append(
            f"OCR2 parse error count was {request_summary.get('parseErrorCount')}"
        )
    scaffold_mode = deepseek_ocr2.get("scaffoldMode") or "disabled"
    if scaffold_mode != "disabled":
        scaffold_failures = request_summary.get("scaffoldValidationFailureCount", 0)
        if scaffold_failures != 0:
            reasons.append(
                f"OCR2 scaffold validation failure count was {scaffold_failures}"
            )
        region_shards = request_summary.get("regionShardCount", 0)
        scaffold_applied = request_summary.get("scaffoldAppliedCount", 0)
        if region_shards > 0 and scaffold_applied != region_shards:
            reasons.append(
                "OCR2 scaffold applied count "
                f"{scaffold_applied} did not match region shard count "
                f"{region_shards}"
            )
    if deepseek_ocr2.get("provider") == "openrouter" and not deepseek_ocr2.get(
        "openRouterApiKeyConfigured"
    ):
        reasons.append("OpenRouter key was not configured")
    if (
        payload.get("rustPdfOcr2RegionPlanner") in OCR2_AUTOMATIC_REGION_PLANNERS
        and request_summary.get("regionShardCount", 0) <= 0
    ):
        reasons.append("automatic OCR2 region planner produced no OCR2 region requests")

    return {
        "checked": True,
        "passed": not reasons,
        "baseline": OCR2_PROMOTION_BASELINE,
        "reasons": reasons,
        "observed": ocr2_promotion_observed(payload, precision_speed, request_summary),
    }


def ocr2_promotion_candidate(
    payload: dict[str, Any],
    precision_speed: dict[str, Any],
    request_summary: dict[str, Any],
) -> bool:
    planner = payload.get("rustPdfOcrProfilePlanner")
    if planner in {"ocr2-all", "ocr2-risk-window"}:
        return True
    if payload.get("rustPdfOcr2RegionPlanner") in OCR2_AUTOMATIC_REGION_PLANNERS:
        return True
    if precision_speed.get("ocrRegionBlocks", 0) > 0:
        return True
    return request_summary.get("requestCount", 0) > 0


def ocr2_promotion_observed(
    payload: dict[str, Any],
    precision_speed: dict[str, Any],
    request_summary: dict[str, Any],
) -> dict[str, Any]:
    deepseek_ocr2 = payload.get("deepseekOcr2") or {}
    return {
        "rustPdfOcrProfilePlanner": payload.get("rustPdfOcrProfilePlanner"),
        "rustPdfOcr2RegionPlanner": payload.get("rustPdfOcr2RegionPlanner"),
        "provider": deepseek_ocr2.get("provider"),
        "openRouterModel": deepseek_ocr2.get("openRouterModel"),
        "openRouterApiKeyConfigured": deepseek_ocr2.get("openRouterApiKeyConfigured"),
        "scaffoldMode": deepseek_ocr2.get("scaffoldMode"),
        "precisionGatePassed": precision_speed.get("precisionGatePassed"),
        "errorRows": precision_speed.get("errorRows"),
        "structureReadingOrderSorted": precision_speed.get(
            "structureReadingOrderSorted"
        ),
        "structureOrderStable": precision_speed.get("structureOrderStable"),
        "structureOrderMismatches": precision_speed.get("structureOrderMismatches"),
        "metricsRows": precision_speed.get("metricsRows"),
        "metricsResultChars": precision_speed.get("metricsResultChars"),
        "ocrPageBlocks": precision_speed.get("ocrPageBlocks"),
        "ocrRegionBlocks": precision_speed.get("ocrRegionBlocks"),
        "bboxBlocks": precision_speed.get("bboxBlocks"),
        "maxForceRefreshMs": precision_speed.get("maxForceRefreshMs"),
        "maxShardCacheReuseForceMs": precision_speed.get("maxShardCacheReuseForceMs"),
        "maxShardCacheReuseSchedulerElapsedMs": precision_speed.get(
            "maxShardCacheReuseSchedulerElapsedMs"
        ),
        "maxShardCacheReuseRegionMaterializeMs": precision_speed.get(
            "maxShardCacheReuseRegionMaterializeMs"
        ),
        "maxCacheHitP95Ms": precision_speed.get("maxCacheHitP95Ms"),
        "requestCount": request_summary.get("requestCount"),
        "successCount": request_summary.get("successCount"),
        "failureCount": request_summary.get("failureCount"),
        "parseErrorCount": request_summary.get("parseErrorCount"),
        "regionShardCount": request_summary.get("regionShardCount"),
        "scaffoldAppliedCount": request_summary.get("scaffoldAppliedCount"),
        "scaffoldValidationFailureCount": request_summary.get(
            "scaffoldValidationFailureCount"
        ),
        "scaffoldJsonCharCountTotal": request_summary.get("scaffoldJsonCharCountTotal"),
        "canonicalMarkdownCharCountTotal": request_summary.get(
            "canonicalMarkdownCharCountTotal"
        ),
        "requestLatencyMsP95": request_summary.get("latencyMsP95"),
        "requestWallSpanMs": request_summary.get("requestWallSpanMs"),
        "requestLatencyOverlapRatio": request_summary.get("requestLatencyOverlapRatio"),
        "sourcePixelAreaTotal": request_summary.get("sourcePixelAreaTotal"),
    }


def is_pdf_ocr_milestone_candidate(result: dict[str, Any]) -> bool:
    ocr_region_blocks = int(result.get("structureOcrRegionBlocks") or 0)
    expected_rows = PDF_OCR_MILESTONE_BASELINE["resourcesRows"] + ocr_region_blocks
    expected_bbox_blocks = PDF_OCR_MILESTONE_BASELINE["bboxBlocks"] + ocr_region_blocks
    return (
        result.get("attachmentClass") == "pdf"
        and result.get("resourcesRows") == expected_rows
        and result.get("structureRows") == expected_rows
        and result.get("structureOcrPageBlocks")
        == PDF_OCR_MILESTONE_BASELINE["ocrPageBlocks"]
        and result.get("structureBboxBlocks") == expected_bbox_blocks
        and result.get("metricsRows") == expected_rows
    )


def pdf_ocr_milestone_observation(result: dict[str, Any]) -> dict[str, Any]:
    regressions: list[str] = []
    force_ms = numeric_or_none(result.get("forceRefreshMs"))
    cache_p95_ms = numeric_or_none(result.get("cacheHitP95Ms"))
    shard_cache_reuse_ms = numeric_or_none(result.get("shardCacheReuseForceMs"))
    shard_cache_reuse_scheduler_ms = numeric_or_none(
        result.get("shardCacheReuseMetricsRustSchedulerElapsedMs")
    )
    ocr_region_blocks = int(result.get("structureOcrRegionBlocks") or 0)
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
        if ocr_region_blocks > 0:
            if shard_cache_reuse_scheduler_ms is None:
                regressions.append(
                    "missing shardCacheReuseMetricsRustSchedulerElapsedMs for OCR2 region sidecars"
                )
            elif (
                shard_cache_reuse_scheduler_ms
                > PDF_OCR_MILESTONE_BASELINE["maxShardCacheReuseForceMs"]
            ):
                regressions.append(
                    "shardCacheReuseMetricsRustSchedulerElapsedMs "
                    f"{shard_cache_reuse_scheduler_ms:.3f} exceeded baseline "
                    f"{PDF_OCR_MILESTONE_BASELINE['maxShardCacheReuseForceMs']:.3f}"
                )
        else:
            regressions.append(
                "shardCacheReuseForceMs "
                f"{shard_cache_reuse_ms:.3f} exceeded baseline "
                f"{PDF_OCR_MILESTONE_BASELINE['maxShardCacheReuseForceMs']:.3f}"
            )
    metrics_result_chars = result.get("metricsResultChars", 0)
    if metrics_result_chars < PDF_OCR_MILESTONE_BASELINE["minMetricsResultChars"]:
        regressions.append(
            "metricsResultChars "
            f"{metrics_result_chars} below baseline "
            f"{PDF_OCR_MILESTONE_BASELINE['minMetricsResultChars']}"
        )

    return {
        "fixture": result.get("fixture"),
        "source": result.get("source"),
        "forceRefreshMs": force_ms,
        "cacheHitP95Ms": cache_p95_ms,
        "shardCacheReuseForceMs": shard_cache_reuse_ms,
        "shardCacheReuseMetricsRustSchedulerElapsedMs": shard_cache_reuse_scheduler_ms,
        "resourcesRows": result.get("resourcesRows"),
        "structureRows": result.get("structureRows"),
        "ocrPageBlocks": result.get("structureOcrPageBlocks"),
        "ocrRegionBlocks": result.get("structureOcrRegionBlocks"),
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
        "maxShardCacheReuseSchedulerElapsedMs": max_numeric(
            results,
            "shardCacheReuseMetricsRustSchedulerElapsedMs",
        ),
        "maxShardCacheReuseRegionMaterializeMs": max_nested_numeric(
            results,
            "shardCacheReuseHybridPageOcrTimingPhaseElapsedMs",
            "regionMaterialize",
        ),
        "maxForceHybridPageOcrRegionMaterializeMs": max_nested_numeric(
            results,
            "forceHybridPageOcrTimingPhaseElapsedMs",
            "regionMaterialize",
        ),
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
