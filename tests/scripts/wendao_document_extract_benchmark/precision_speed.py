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

HOSTED_VLM_PROMOTION_BASELINE = {
    "id": "arxiv-2604.17337-fast-risk-window-r9",
    "forceRefreshMs": 12_856.546292,
    "maxShardCacheReuseForceMs": PDF_OCR_MILESTONE_BASELINE["maxShardCacheReuseForceMs"],
    "minMetricsResultChars": PDF_OCR_MILESTONE_BASELINE["minMetricsResultChars"],
    "expectedOcrPageBlocks": PDF_OCR_MILESTONE_BASELINE["ocrPageBlocks"],
    "minMetricsRows": PDF_OCR_MILESTONE_BASELINE["metricsRows"],
}

HOSTED_VLM_AUTOMATIC_REGION_PLANNERS = {
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
    docling_groundtruth_passed: bool | None,
    docling_groundtruth_failure_count: int,
) -> dict[str, Any]:
    pdf_milestone_guard = pdf_ocr_milestone_guard(results)
    pdf_milestone_precision_passed = (
        not pdf_milestone_guard["checked"] or pdf_milestone_guard["passed"]
    )
    precision_gate_passed = (
        total_error_rows == 0
        and artifact_error_count == 0
        and structure_parity_error_count == 0
        and docling_groundtruth_failure_count == 0
        and structure_reading_order_sorted is not False
        and structure_order_stable is not False
        and structure_order_mismatch_count == 0
        and structure_parity_passed is not False
        and docling_groundtruth_passed is not False
        and pdf_milestone_precision_passed
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
        "doclingGroundtruthPassed": docling_groundtruth_passed,
        "doclingGroundtruthFailures": docling_groundtruth_failure_count,
        "structureRows": sum(result.get("structureRows", 0) for result in results),
        "ocrPageBlocks": sum(result.get("structureOcrPageBlocks", 0) for result in results),
        "ocrRegionBlocks": sum(result.get("structureOcrRegionBlocks", 0) for result in results),
        "bboxBlocks": sum(result.get("structureBboxBlocks", 0) for result in results),
        "metricsRows": sum(result.get("metricsRows", 0) for result in results),
        "metricsResultChars": sum(result.get("metricsResultChars", 0) for result in results),
        "pdfOcrMilestoneGuard": pdf_milestone_guard,
        **speed_observation_summary(results, distinct_miss_report),
    }


def pdf_ocr_milestone_guard(results: list[dict[str, Any]]) -> dict[str, Any]:
    observations = [
        pdf_ocr_milestone_observation(result)
        for result in results
        if is_pdf_ocr_milestone_candidate(result)
    ]
    regressions = [
        regression for observation in observations for regression in observation["regressions"]
    ]
    return {
        "checked": bool(observations),
        "passed": bool(observations) and not regressions,
        "baseline": PDF_OCR_MILESTONE_BASELINE,
        "observations": observations,
        "regressions": regressions,
        "reason": (
            None if observations else "no OCR-positive 21-page PDF milestone fixture observed"
        ),
    }


def hosted_vlm_promotion_gate(payload: dict[str, Any]) -> dict[str, Any]:
    summary = payload.get("summary", {})
    precision_speed = summary.get("precisionSpeedSummary", {})
    hosted_vlm_ocr = payload.get("hostedVlmOcr") or {}
    request_summary = hosted_vlm_ocr.get("requestSummary") or {}
    reasons: list[str] = []
    checked = hosted_vlm_promotion_candidate(payload, precision_speed, request_summary)

    if not checked:
        return {
            "checked": False,
            "passed": False,
            "baseline": HOSTED_VLM_PROMOTION_BASELINE,
            "reasons": ["not a hosted VLM/OCR promotion candidate"],
            "observed": hosted_vlm_promotion_observed(payload, precision_speed, request_summary),
        }

    if precision_speed.get("precisionGatePassed") is not True:
        reasons.append("precision gate did not pass")
    if precision_speed.get("errorRows") != 0:
        reasons.append(f"expected zero error rows, observed {precision_speed.get('errorRows')}")
    if precision_speed.get("structureReadingOrderSorted") is not True:
        reasons.append("structure reading order is not sorted")
    if precision_speed.get("structureOrderStable") is not True:
        reasons.append("structure order is not stable")
    if precision_speed.get("structureOrderMismatches") != 0:
        reasons.append(
            "expected zero structure order mismatches, observed "
            f"{precision_speed.get('structureOrderMismatches')}"
        )
    if precision_speed.get("structureParityPassed") is False:
        reasons.append("structure parity did not pass")
    if precision_speed.get("structureParityErrors", 0) != 0:
        reasons.append(
            "expected zero structure parity errors, observed "
            f"{precision_speed.get('structureParityErrors')}"
        )
    if precision_speed.get("doclingGroundtruthPassed") is False:
        reasons.append("Docling groundtruth comparison did not pass")
    if precision_speed.get("doclingGroundtruthFailures", 0) != 0:
        reasons.append(
            "expected zero Docling groundtruth failures, observed "
            f"{precision_speed.get('doclingGroundtruthFailures')}"
        )
    if (
        precision_speed.get("ocrPageBlocks")
        != HOSTED_VLM_PROMOTION_BASELINE["expectedOcrPageBlocks"]
    ):
        reasons.append(
            "expected "
            f"{HOSTED_VLM_PROMOTION_BASELINE['expectedOcrPageBlocks']} OCR page blocks, "
            f"observed {precision_speed.get('ocrPageBlocks')}"
        )
    if precision_speed.get("metricsRows", 0) < HOSTED_VLM_PROMOTION_BASELINE["minMetricsRows"]:
        reasons.append(
            "metricsRows "
            f"{precision_speed.get('metricsRows')} below promotion floor "
            f"{HOSTED_VLM_PROMOTION_BASELINE['minMetricsRows']}"
        )
    if (
        precision_speed.get("metricsResultChars", 0)
        < HOSTED_VLM_PROMOTION_BASELINE["minMetricsResultChars"]
    ):
        reasons.append(
            "metricsResultChars "
            f"{precision_speed.get('metricsResultChars')} below promotion floor "
            f"{HOSTED_VLM_PROMOTION_BASELINE['minMetricsResultChars']}"
        )
    force_ms = numeric_or_none(precision_speed.get("maxForceRefreshMs"))
    if force_ms is None:
        reasons.append("missing maxForceRefreshMs")
    elif force_ms > HOSTED_VLM_PROMOTION_BASELINE["forceRefreshMs"]:
        reasons.append(
            "maxForceRefreshMs "
            f"{force_ms:.3f} exceeded promotion baseline "
            f"{HOSTED_VLM_PROMOTION_BASELINE['forceRefreshMs']:.3f}"
        )
    shard_cache_reuse_ms = numeric_or_none(precision_speed.get("maxShardCacheReuseForceMs"))
    shard_cache_reuse_scheduler_ms = numeric_or_none(
        precision_speed.get("maxShardCacheReuseSchedulerElapsedMs")
    )
    has_hosted_vlm_region_sidecars = precision_speed.get("ocrRegionBlocks", 0) > 0
    if shard_cache_reuse_ms is None:
        reasons.append("missing maxShardCacheReuseForceMs")
    elif has_hosted_vlm_region_sidecars:
        if shard_cache_reuse_scheduler_ms is None:
            reasons.append("missing maxShardCacheReuseSchedulerElapsedMs")
        elif (
            shard_cache_reuse_scheduler_ms
            > HOSTED_VLM_PROMOTION_BASELINE["maxShardCacheReuseForceMs"]
        ):
            reasons.append(
                "maxShardCacheReuseSchedulerElapsedMs "
                f"{shard_cache_reuse_scheduler_ms:.3f} exceeded promotion baseline "
                f"{HOSTED_VLM_PROMOTION_BASELINE['maxShardCacheReuseForceMs']:.3f}"
            )
    elif shard_cache_reuse_ms > HOSTED_VLM_PROMOTION_BASELINE["maxShardCacheReuseForceMs"]:
        reasons.append(
            "maxShardCacheReuseForceMs "
            f"{shard_cache_reuse_ms:.3f} exceeded promotion baseline "
            f"{HOSTED_VLM_PROMOTION_BASELINE['maxShardCacheReuseForceMs']:.3f}"
        )
    if request_summary.get("requestCount", 0) <= 0:
        reasons.append("no Hosted VLM/OCR requests observed")
    if request_summary.get("successCount", 0) != request_summary.get("requestCount", 0):
        reasons.append(
            "Hosted VLM/OCR success count "
            f"{request_summary.get('successCount')} did not match request count "
            f"{request_summary.get('requestCount')}"
        )
    if request_summary.get("failureCount", 0) != 0:
        reasons.append(f"Hosted VLM/OCR failure count was {request_summary.get('failureCount')}")
    if request_summary.get("parseErrorCount", 0) != 0:
        reasons.append(
            f"Hosted VLM/OCR parse error count was {request_summary.get('parseErrorCount')}"
        )
    composite_size = hosted_vlm_ocr.get("regionCompositeSize")
    composite_mode = hosted_vlm_ocr.get("regionCompositeMode") or "fixed"
    composite_request_count = hosted_vlm_region_composite_request_count(request_summary)
    if (
        isinstance(composite_size, int)
        and composite_size > 1
        and composite_mode == "fixed"
        and request_summary.get("regionShardCount", 0) >= composite_size
        and composite_request_count <= 0
    ):
        reasons.append(
            "Hosted VLM/OCR fixed region composite was configured but no composite "
            "request kind was observed"
        )
    scaffold_mode = hosted_vlm_ocr.get("scaffoldMode") or "disabled"
    if scaffold_mode != "disabled":
        scaffold_failures = request_summary.get("scaffoldValidationFailureCount", 0)
        if scaffold_failures != 0:
            reasons.append(
                f"Hosted VLM/OCR scaffold validation failure count was {scaffold_failures}"
            )
        region_shards = request_summary.get("regionShardCount", 0)
        scaffold_applied = request_summary.get("scaffoldAppliedCount", 0)
        if region_shards > 0 and scaffold_applied != region_shards:
            reasons.append(
                "Hosted VLM/OCR scaffold applied count "
                f"{scaffold_applied} did not match region shard count "
                f"{region_shards}"
            )
    atlas_mode = hosted_vlm_ocr.get("regionAtlasMode") or "disabled"
    if atlas_mode != "disabled":
        atlas_failures = request_summary.get("scaffoldValidationFailureCount", 0)
        if atlas_failures != 0:
            reasons.append(f"Hosted VLM/OCR atlas validation failure count was {atlas_failures}")
    if hosted_vlm_ocr.get("provider") == "openrouter" and not hosted_vlm_ocr.get(
        "openRouterApiKeyConfigured"
    ):
        reasons.append("OpenRouter key was not configured")
    if (
        payload.get("rustPdfHostedVlmRegionPlanner") in HOSTED_VLM_AUTOMATIC_REGION_PLANNERS
        and request_summary.get("regionShardCount", 0) <= 0
    ):
        reasons.append(
            "automatic hosted VLM/OCR region planner produced no hosted VLM/OCR region requests"
        )

    return {
        "checked": True,
        "passed": not reasons,
        "baseline": HOSTED_VLM_PROMOTION_BASELINE,
        "reasons": reasons,
        "observed": hosted_vlm_promotion_observed(payload, precision_speed, request_summary),
    }


def candidate_taxonomy(payload: dict[str, Any]) -> dict[str, Any]:
    summary = payload.get("summary", {})
    precision_speed = summary.get("precisionSpeedSummary", {})
    promotion_gate = payload.get("hostedVlmPromotionGate") or {}
    max_force_ms = numeric_or_none(precision_speed.get("maxForceRefreshMs"))
    precision_candidate = _precision_candidate(precision_speed)
    rejected_structure_loss = _rejected_structure_loss(precision_speed, summary)
    promotion_candidate = promotion_gate.get("passed") is True
    opt_in_controls = hosted_vlm_opt_in_promotion_controls(payload)
    speed_candidate = (
        precision_candidate
        and not rejected_structure_loss
        and max_force_ms is not None
        and max_force_ms < HOSTED_VLM_PROMOTION_BASELINE["forceRefreshMs"]
    )
    return {
        "precisionCandidate": precision_candidate,
        "speedCandidate": speed_candidate,
        "promotionCandidate": promotion_candidate,
        "defaultPromotionCandidate": promotion_candidate and not opt_in_controls,
        "optInPromotionControls": opt_in_controls,
        "rejectedStructureLoss": rejected_structure_loss,
        "structureAuthorityPages": summary.get("structureAuthorityPages", 0),
        "textShortcutPages": summary.get("textShortcutPages", 0),
        "ocrPatchRegions": summary.get("ocrPatchRegions", 0),
        "pageRangeDoclingFallbackPages": summary.get(
            "pageRangeDoclingFallbackPages",
            0,
        ),
        "fullDoclingFallbackCount": summary.get("fullDoclingFallbackCount", 0),
        "maxForceRefreshMs": precision_speed.get("maxForceRefreshMs"),
        "promotionBaselineForceRefreshMs": HOSTED_VLM_PROMOTION_BASELINE["forceRefreshMs"],
    }


def hosted_vlm_opt_in_promotion_controls(payload: dict[str, Any]) -> list[str]:
    hosted_vlm_ocr = payload.get("hostedVlmOcr") or {}
    controls: list[str] = []
    if (hosted_vlm_ocr.get("regionPromptMode") or "default") != "default":
        controls.append("hosted_vlm_region_prompt_mode")
    if hosted_vlm_ocr.get("openRouterProvider"):
        controls.append("openrouter_provider_routing")
    region_max_tokens = hosted_vlm_ocr.get("regionMaxTokens")
    if region_max_tokens not in (None, 2048):
        controls.append("hosted_vlm_region_max_tokens")
    retry_delay = hosted_vlm_ocr.get("speculativeRetryDelaySeconds")
    if retry_delay not in (None, 5.0):
        controls.append("hosted_vlm_speculative_retry_delay")
    if (hosted_vlm_ocr.get("scaffoldMode") or "disabled") != "disabled":
        controls.append("hosted_vlm_scaffold_mode")
    if (hosted_vlm_ocr.get("regionAtlasMode") or "disabled") != "disabled":
        controls.append("hosted_vlm_region_atlas_mode")
    if hosted_vlm_ocr.get("regionCompositeSize") is not None:
        controls.append("hosted_vlm_region_composite_size")
    if hosted_vlm_ocr.get("pageWindowSize") is not None:
        controls.append("hosted_vlm_page_window_size")
    if hosted_vlm_ocr.get("prompt"):
        controls.append("hosted_vlm_prompt")
    return controls


def _precision_candidate(precision_speed: dict[str, Any]) -> bool:
    return (
        precision_speed.get("precisionGatePassed") is True
        and precision_speed.get("errorRows") == 0
        and precision_speed.get("structureReadingOrderSorted") is True
        and precision_speed.get("structureOrderStable") is True
        and precision_speed.get("structureOrderMismatches") == 0
        and precision_speed.get("structureParityPassed") is not False
        and precision_speed.get("structureParityErrors", 0) == 0
        and precision_speed.get("doclingGroundtruthPassed") is not False
        and precision_speed.get("doclingGroundtruthFailures", 0) == 0
        and precision_speed.get("metricsRows", 0) >= HOSTED_VLM_PROMOTION_BASELINE["minMetricsRows"]
        and precision_speed.get("metricsResultChars", 0)
        >= HOSTED_VLM_PROMOTION_BASELINE["minMetricsResultChars"]
    )


def _rejected_structure_loss(
    precision_speed: dict[str, Any],
    summary: dict[str, Any],
) -> bool:
    return (
        precision_speed.get("structureReadingOrderSorted") is False
        or precision_speed.get("structureOrderStable") is False
        or precision_speed.get("structureOrderMismatches", 0) != 0
        or precision_speed.get("structureParityPassed") is False
        or precision_speed.get("structureParityErrors", 0) != 0
        or precision_speed.get("doclingGroundtruthPassed") is False
        or precision_speed.get("doclingGroundtruthFailures", 0) != 0
        or summary.get("totalStructureRows", 0) == 0
    )


def hosted_vlm_promotion_candidate(
    payload: dict[str, Any],
    precision_speed: dict[str, Any],
    request_summary: dict[str, Any],
) -> bool:
    planner = payload.get("rustPdfOcrProfilePlanner")
    if planner in {
        "hosted-vlm-all",
        "hosted-vlm-risk-window",
        "hosted-vlm-risk-window-backend-text",
        "docling-structure-recovery",
    }:
        return True
    if payload.get("rustPdfHostedVlmRegionPlanner") in HOSTED_VLM_AUTOMATIC_REGION_PLANNERS:
        return True
    if precision_speed.get("ocrRegionBlocks", 0) > 0:
        return True
    return request_summary.get("requestCount", 0) > 0


def hosted_vlm_region_composite_request_count(request_summary: dict[str, Any]) -> int:
    request_kind_counts = request_summary.get("requestKindCounts")
    if not isinstance(request_kind_counts, dict):
        return 0
    return sum(
        int(count)
        for kind, count in request_kind_counts.items()
        if isinstance(kind, str) and kind.startswith("region-composite") and isinstance(count, int)
    )


def hosted_vlm_promotion_observed(
    payload: dict[str, Any],
    precision_speed: dict[str, Any],
    request_summary: dict[str, Any],
) -> dict[str, Any]:
    hosted_vlm_ocr = payload.get("hostedVlmOcr") or {}
    summary = payload.get("summary", {})
    first_result = _first_result(payload)
    force_ms = numeric_or_none(precision_speed.get("maxForceRefreshMs"))
    request_wall_ms = numeric_or_none(request_summary.get("requestWallSpanMs"))
    force_phases = summary.get("forceHybridPageOcrTimingPhaseElapsedMs") or first_result.get(
        "forceHybridPageOcrTimingPhaseElapsedMs", {}
    )
    force_region_materialize_ms = nested_mapping_numeric(
        force_phases,
        "regionMaterialize",
    )
    force_region_pipeline_ms = nested_mapping_numeric(force_phases, "regionPipeline")
    force_region_pipeline_first_ready_ms = nested_mapping_numeric(
        force_phases,
        "regionPipelineFirstRegionReady",
    )
    force_region_pipeline_last_ready_ms = nested_mapping_numeric(
        force_phases,
        "regionPipelineLastRegionReady",
    )
    force_region_pipeline_first_dispatch_ms = nested_mapping_numeric(
        force_phases,
        "regionPipelineFirstRegionDispatch",
    )
    force_region_pipeline_last_dispatch_ms = nested_mapping_numeric(
        force_phases,
        "regionPipelineLastRegionDispatch",
    )
    force_region_pipeline_first_base_result_ms = nested_mapping_numeric(
        force_phases,
        "regionPipelineFirstBaseResult",
    )
    force_region_pipeline_last_base_result_ms = nested_mapping_numeric(
        force_phases,
        "regionPipelineLastBaseResult",
    )
    force_region_pipeline_first_region_result_ms = nested_mapping_numeric(
        force_phases,
        "regionPipelineFirstRegionResult",
    )
    force_region_pipeline_last_region_result_ms = nested_mapping_numeric(
        force_phases,
        "regionPipelineLastRegionResult",
    )
    force_region_render_ms = nested_mapping_numeric(
        force_phases,
        "regionMaterializeRender",
    )
    force_scheduler_ms = nested_mapping_numeric(force_phases, "ocrScheduler")
    return {
        "rustPdfOcrProfilePlanner": payload.get("rustPdfOcrProfilePlanner"),
        "rustPdfHostedVlmRegionPlanner": payload.get("rustPdfHostedVlmRegionPlanner"),
        "rustPdfHostedVlmRegionTargetPixels": payload.get("rustPdfHostedVlmRegionTargetPixels"),
        "rustPdfHostedVlmRegionMaxSlices": payload.get("rustPdfHostedVlmRegionMaxSlices"),
        "rustPdfHostedVlmRegionPipeline": payload.get("rustPdfHostedVlmRegionPipeline"),
        "rustPdfHostedVlmRegionRenderAhead": payload.get("rustPdfHostedVlmRegionRenderAhead"),
        "rustPdfHostedVlmRegionRenderChunk": payload.get("rustPdfHostedVlmRegionRenderChunk"),
        "rustPdfRegionRenderMode": payload.get("rustPdfRegionRenderMode"),
        "rustPdfHostedVlmRegionDispatchChunkSize": payload.get(
            "rustPdfHostedVlmRegionDispatchChunkSize"
        ),
        "rustPdfFastTextEndpointAffinity": payload.get("rustPdfFastTextEndpointAffinity"),
        "rustPdfOcrSchedulerLaneFairness": payload.get("rustPdfOcrSchedulerLaneFairness"),
        "pdfOcrFastTextSourceConverter": payload.get("pdfOcrFastTextSourceConverter"),
        "provider": hosted_vlm_ocr.get("provider"),
        "openRouterModel": hosted_vlm_ocr.get("openRouterModel"),
        "openRouterApiKeyConfigured": hosted_vlm_ocr.get("openRouterApiKeyConfigured"),
        "regionCompositeSize": hosted_vlm_ocr.get("regionCompositeSize"),
        "regionCompositeMode": hosted_vlm_ocr.get("regionCompositeMode"),
        "regionCompositeRequestCount": hosted_vlm_region_composite_request_count(request_summary),
        "regionCompositeActivated": hosted_vlm_region_composite_request_count(request_summary) > 0,
        "regionAtlasMode": hosted_vlm_ocr.get("regionAtlasMode"),
        "scaffoldMode": hosted_vlm_ocr.get("scaffoldMode"),
        "speculativeRetryMinSourcePixels": hosted_vlm_ocr.get("speculativeRetryMinSourcePixels"),
        "speculativeRetryMinImageBytes": hosted_vlm_ocr.get("speculativeRetryMinImageBytes"),
        "structureAuthorityPages": payload.get("summary", {}).get("structureAuthorityPages"),
        "textShortcutPages": payload.get("summary", {}).get("textShortcutPages"),
        "ocrPatchRegions": payload.get("summary", {}).get("ocrPatchRegions"),
        "pageRangeDoclingFallbackPages": payload.get("summary", {}).get(
            "pageRangeDoclingFallbackPages"
        ),
        "fullDoclingFallbackCount": payload.get("summary", {}).get("fullDoclingFallbackCount"),
        "precisionGatePassed": precision_speed.get("precisionGatePassed"),
        "errorRows": precision_speed.get("errorRows"),
        "structureReadingOrderSorted": precision_speed.get("structureReadingOrderSorted"),
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
        "forceHostedVlmRegionRenderReportedElapsedMs": _summary_or_result(
            summary,
            first_result,
            "forceHybridPageOcrTimingOcr2RegionRenderReportedElapsedMs",
        ),
        "forceHostedVlmRegionRenderArtifactCacheHitCount": _summary_or_result(
            summary,
            first_result,
            "forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheHitCount",
        ),
        "forceHostedVlmRegionRenderArtifactCacheMissCount": _summary_or_result(
            summary,
            first_result,
            "forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheMissCount",
        ),
        "forceHostedVlmRegionRenderArtifactCacheThrottledCount": _summary_or_result(
            summary,
            first_result,
            "forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheThrottledCount",
        ),
        "forceHostedVlmRegionRenderArtifactCacheByteCount": _summary_or_result(
            summary,
            first_result,
            "forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheByteCount",
        ),
        "forceHostedVlmRegionPipelinePlannedRenderChunkCount": _summary_or_result(
            summary,
            first_result,
            "forceHybridPageOcrTimingOcr2RegionPipelinePlannedRenderChunkCount",
        ),
        "forceHostedVlmRegionPipelineEndpointCount": _summary_or_result(
            summary,
            first_result,
            "forceHybridPageOcrTimingOcr2RegionPipelineEndpointCount",
        ),
        "forceHostedVlmRegionPipelineRenderAheadLimit": _summary_or_result(
            summary,
            first_result,
            "forceHybridPageOcrTimingOcr2RegionPipelineRenderAheadLimit",
        ),
        "forceHostedVlmRegionPipelineRenderSpawnCount": _summary_or_result(
            summary,
            first_result,
            "forceHybridPageOcrTimingOcr2RegionPipelineRenderSpawnCount",
        ),
        "forceHostedVlmRegionPipelineRenderChunkCount": _summary_or_result(
            summary,
            first_result,
            "forceHybridPageOcrTimingOcr2RegionPipelineRenderChunkCount",
        ),
        "forceHostedVlmRegionPipelineRegionDispatchCount": _summary_or_result(
            summary,
            first_result,
            "forceHybridPageOcrTimingOcr2RegionPipelineRegionDispatchCount",
        ),
        "shardCacheReuseHostedVlmRegionRenderReportedElapsedMs": _summary_or_result(
            summary,
            first_result,
            "shardCacheReuseHybridPageOcrTimingOcr2RegionRenderReportedElapsedMs",
        ),
        "shardCacheReuseHostedVlmRegionRenderArtifactCacheHitCount": _summary_or_result(
            summary,
            first_result,
            "shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheHitCount",
        ),
        "shardCacheReuseHostedVlmRegionRenderArtifactCacheMissCount": _summary_or_result(
            summary,
            first_result,
            "shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheMissCount",
        ),
        "shardCacheReuseHostedVlmRegionRenderArtifactCacheThrottledCount": _summary_or_result(
            summary,
            first_result,
            "shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheThrottledCount",
        ),
        "shardCacheReuseHostedVlmRegionRenderArtifactCacheByteCount": _summary_or_result(
            summary,
            first_result,
            "shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheByteCount",
        ),
        "shardCacheReuseHostedVlmRegionPipelinePlannedRenderChunkCount": _summary_or_result(
            summary,
            first_result,
            "shardCacheReuseHybridPageOcrTimingOcr2RegionPipelinePlannedRenderChunkCount",
        ),
        "shardCacheReuseHostedVlmRegionPipelineEndpointCount": _summary_or_result(
            summary,
            first_result,
            "shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineEndpointCount",
        ),
        "shardCacheReuseHostedVlmRegionPipelineRenderAheadLimit": _summary_or_result(
            summary,
            first_result,
            "shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineRenderAheadLimit",
        ),
        "shardCacheReuseHostedVlmRegionPipelineRenderSpawnCount": _summary_or_result(
            summary,
            first_result,
            "shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineRenderSpawnCount",
        ),
        "shardCacheReuseHostedVlmRegionPipelineRenderChunkCount": _summary_or_result(
            summary,
            first_result,
            "shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineRenderChunkCount",
        ),
        "shardCacheReuseHostedVlmRegionPipelineRegionDispatchCount": _summary_or_result(
            summary,
            first_result,
            "shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineRegionDispatchCount",
        ),
        "maxCacheHitP95Ms": precision_speed.get("maxCacheHitP95Ms"),
        "requestCount": request_summary.get("requestCount"),
        "successCount": request_summary.get("successCount"),
        "failureCount": request_summary.get("failureCount"),
        "parseErrorCount": request_summary.get("parseErrorCount"),
        "regionShardCount": request_summary.get("regionShardCount"),
        "scaffoldAppliedCount": request_summary.get("scaffoldAppliedCount"),
        "scaffoldValidationFailureCount": request_summary.get("scaffoldValidationFailureCount"),
        "scaffoldJsonCharCountTotal": request_summary.get("scaffoldJsonCharCountTotal"),
        "canonicalMarkdownCharCountTotal": request_summary.get("canonicalMarkdownCharCountTotal"),
        "requestLatencyMsP95": request_summary.get("latencyMsP95"),
        "requestWallSpanMs": request_summary.get("requestWallSpanMs"),
        "requestLatencyOverlapRatio": request_summary.get("requestLatencyOverlapRatio"),
        "sourcePixelAreaTotal": request_summary.get("sourcePixelAreaTotal"),
        "sourcePixelAreaMax": request_summary.get("sourcePixelAreaMax"),
        "sourcePixelAreaPerRequestAvg": request_summary.get("sourcePixelAreaPerRequestAvg"),
        "imageBytesTotal": request_summary.get("imageBytesTotal"),
        "imageBytesMax": request_summary.get("imageBytesMax"),
        "imageBytesPerRequestAvg": request_summary.get("imageBytesPerRequestAvg"),
        "slowestRequests": request_summary.get("slowestRequests"),
        "forceHostedVlmLocalOverheadMs": subtract_numeric(force_ms, request_wall_ms),
        "forceHostedVlmRegionMaterializeMs": force_region_materialize_ms,
        "forceHostedVlmRegionPipelineMs": force_region_pipeline_ms,
        "forceHostedVlmRegionPipelineFirstReadyMs": (force_region_pipeline_first_ready_ms),
        "forceHostedVlmRegionPipelineLastReadyMs": (force_region_pipeline_last_ready_ms),
        "forceHostedVlmRegionPipelineFirstDispatchMs": (force_region_pipeline_first_dispatch_ms),
        "forceHostedVlmRegionPipelineLastDispatchMs": (force_region_pipeline_last_dispatch_ms),
        "forceHostedVlmRegionPipelineFirstBaseResultMs": (
            force_region_pipeline_first_base_result_ms
        ),
        "forceHostedVlmRegionPipelineLastBaseResultMs": (force_region_pipeline_last_base_result_ms),
        "forceHostedVlmRegionPipelineFirstRegionResultMs": (
            force_region_pipeline_first_region_result_ms
        ),
        "forceHostedVlmRegionPipelineLastRegionResultMs": (
            force_region_pipeline_last_region_result_ms
        ),
        "forceHostedVlmRegionRenderMs": force_region_render_ms,
        "forceHostedVlmSchedulerMs": force_scheduler_ms,
        "forceHostedVlmSourceRangeChunkMaxMs": precision_speed.get(
            "maxForceHybridPageOcrSourceRangeChunkMs"
        ),
        "forceHostedVlmSourceRangeChunkPageStart": precision_speed.get(
            "maxForceHybridPageOcrSourceRangeChunkPageStart"
        ),
        "forceHostedVlmSourceRangeChunkPageEnd": precision_speed.get(
            "maxForceHybridPageOcrSourceRangeChunkPageEnd"
        ),
        "forceHostedVlmSourceRangeChunkCount": precision_speed.get(
            "totalForceHybridPageOcrSourceRangeChunkCount"
        ),
        "forceHostedVlmSourceRangeTraceChars": precision_speed.get(
            "totalForceHybridPageOcrSourceRangeTraceChars"
        ),
        "forceHostedVlmSchedulerNonRequestMs": subtract_numeric(
            force_scheduler_ms,
            request_wall_ms,
        ),
    }


def _first_result(payload: dict[str, Any]) -> dict[str, Any]:
    results = payload.get("results")
    if isinstance(results, list) and results and isinstance(results[0], dict):
        return results[0]
    return {}


def _summary_or_result(
    summary: dict[str, Any],
    result: dict[str, Any],
    key: str,
) -> Any:
    value = summary.get(key)
    if value is not None:
        return value
    return result.get(key)


def is_pdf_ocr_milestone_candidate(result: dict[str, Any]) -> bool:
    ocr_region_blocks = int(result.get("structureOcrRegionBlocks") or 0)
    expected_rows = PDF_OCR_MILESTONE_BASELINE["resourcesRows"] + ocr_region_blocks
    expected_bbox_blocks = PDF_OCR_MILESTONE_BASELINE["bboxBlocks"] + ocr_region_blocks
    return (
        result.get("attachmentClass") == "pdf"
        and result.get("resourcesRows") == expected_rows
        and result.get("structureRows") == expected_rows
        and result.get("structureOcrPageBlocks") == PDF_OCR_MILESTONE_BASELINE["ocrPageBlocks"]
        and result.get("structureBboxBlocks") == expected_bbox_blocks
        and result.get("metricsRows") == expected_rows
    )


def pdf_ocr_milestone_observation(result: dict[str, Any]) -> dict[str, Any]:
    regressions: list[str] = []
    force_ms = numeric_or_none(result.get("forceRefreshMs"))
    cache_p95_ms = numeric_or_none(result.get("cacheHitP95Ms"))
    shard_cache_reuse_ms = numeric_or_none(result.get("shardCacheReuseForceMs"))
    region_projection_reuse_ms = numeric_or_none(result.get("regionProjectionReuseForceMs"))
    shard_cache_reuse_scheduler_ms = numeric_or_none(
        result.get("shardCacheReuseMetricsRustSchedulerElapsedMs")
    )
    region_projection_reuse_scheduler_ms = numeric_or_none(
        result.get("regionProjectionReuseMetricsRustSchedulerElapsedMs")
    )
    ocr_region_blocks = int(result.get("structureOcrRegionBlocks") or 0)
    structure_stable = result.get("structureOrderStable")
    structure_mismatches = result.get("structureOrderMismatchCount", 0)
    error_rows = (
        int(result.get("forceErrorRows", 0))
        + int(result.get("shardCacheReuseErrorRows", 0))
        + int(result.get("regionProjectionReuseErrorRows", 0))
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
    if cache_p95_ms is not None and cache_p95_ms > PDF_OCR_MILESTONE_BASELINE["maxCacheHitP95Ms"]:
        regressions.append(
            "cacheHitP95Ms "
            f"{cache_p95_ms:.3f} exceeded baseline "
            f"{PDF_OCR_MILESTONE_BASELINE['maxCacheHitP95Ms']:.3f}"
        )
    if (
        shard_cache_reuse_ms is not None
        and shard_cache_reuse_ms > PDF_OCR_MILESTONE_BASELINE["maxShardCacheReuseForceMs"]
    ):
        if ocr_region_blocks > 0:
            if shard_cache_reuse_scheduler_ms is None:
                regressions.append(
                    "missing shardCacheReuseMetricsRustSchedulerElapsedMs for Hosted VLM/OCR region sidecars"
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
        "regionProjectionReuseForceMs": region_projection_reuse_ms,
        "regionProjectionReuseMetricsRustSchedulerElapsedMs": (
            region_projection_reuse_scheduler_ms
        ),
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
        "maxRegionProjectionReuseForceMs": max_numeric(
            results,
            "regionProjectionReuseForceMs",
        ),
        "maxShardCacheReuseSchedulerElapsedMs": max_numeric(
            results,
            "shardCacheReuseMetricsRustSchedulerElapsedMs",
        ),
        "maxRegionProjectionReuseSchedulerElapsedMs": max_numeric(
            results,
            "regionProjectionReuseMetricsRustSchedulerElapsedMs",
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
        "maxForceHybridPageOcrSourceRangeChunkMs": max_nested_numeric(
            results,
            "forceHybridPageOcrTimingSchedulerTraceSummary",
            "sourceRangeLatencyMsMax",
        ),
        "maxForceHybridPageOcrSourceRangeChunkPageStart": max_nested_numeric(
            results,
            "forceHybridPageOcrTimingSchedulerTraceSummary",
            "sourceRangeLongestPageStart",
        ),
        "maxForceHybridPageOcrSourceRangeChunkPageEnd": max_nested_numeric(
            results,
            "forceHybridPageOcrTimingSchedulerTraceSummary",
            "sourceRangeLongestPageEnd",
        ),
        "maxForceHybridPageOcrSourceRangeChunkProfile": max_nested_mapping_string(
            results,
            "forceHybridPageOcrTimingSchedulerTraceSummary",
            "sourceRangeLatencyMsMax",
            "sourceRangeLongestOcrProfile",
        ),
        "maxForceHybridPageOcrSourceRangeChunkShardType": max_nested_mapping_string(
            results,
            "forceHybridPageOcrTimingSchedulerTraceSummary",
            "sourceRangeLatencyMsMax",
            "sourceRangeLongestShardType",
        ),
        "maxForceHybridPageOcrSourceRangeChunkTextChars": max_nested_mapping_numeric(
            results,
            "forceHybridPageOcrTimingSchedulerTraceSummary",
            "sourceRangeLatencyMsMax",
            "sourceRangeLongestTextCharCount",
        ),
        "totalForceHybridPageOcrSourceRangeChunkCount": sum_nested_numeric(
            results,
            "forceHybridPageOcrTimingSchedulerTraceSummary",
            "sourceRangeChunkCount",
        ),
        "totalForceHybridPageOcrSourceRangeTraceChars": sum_nested_numeric(
            results,
            "forceHybridPageOcrTimingSchedulerTraceSummary",
            "sourceRangeTextCharCount",
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
        "totalDoclingConvertMs": total_docling_convert_ms(results),
        "maxDoclingConvertMs": max_docling_convert_ms(results),
        "maxDoclingConvertShare": max_docling_convert_share(results),
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


def total_docling_convert_ms(results: list[dict[str, Any]]) -> float:
    return sum(docling_convert_total_value(result) for result in results)


def docling_convert_total_value(result: dict[str, Any]) -> float:
    top_level = (
        nested_mapping_numeric(
            result.get("documentTimingPhaseElapsedMs"),
            "doclingConvert",
        )
        or 0.0
    )
    chunk_summary = page_range_docling_chunk_summary(result)
    chunk_total = (
        nested_mapping_numeric(
            chunk_summary.get("documentTimingPhaseElapsedMs"),
            "doclingConvert",
        )
        or 0.0
    )
    return top_level + chunk_total


def max_docling_convert_ms(results: list[dict[str, Any]]) -> float | None:
    values = [
        value
        for result in results
        for value in docling_convert_max_values(result)
        if value is not None
    ]
    return max(values) if values else None


def docling_convert_max_values(result: dict[str, Any]) -> list[float | None]:
    chunk_summary = page_range_docling_chunk_summary(result)
    return [
        nested_mapping_numeric(
            result.get("documentTimingPhaseElapsedMs"),
            "doclingConvert",
        ),
        nested_mapping_numeric(
            chunk_summary.get("longestDocumentTimingPhaseElapsedMs"),
            "doclingConvert",
        )
        or nested_mapping_numeric(
            chunk_summary.get("documentTimingPhaseElapsedMs"),
            "doclingConvert",
        ),
    ]


def max_docling_convert_share(results: list[dict[str, Any]]) -> float | None:
    values = [
        value
        for result in results
        for value in docling_convert_share_values(result)
        if value is not None
    ]
    return max(values) if values else None


def docling_convert_share_values(result: dict[str, Any]) -> list[float | None]:
    chunk_summary = page_range_docling_chunk_summary(result)
    return [
        ratio_or_none(
            nested_mapping_numeric(
                result.get("documentTimingPhaseElapsedMs"),
                "doclingConvert",
            ),
            numeric_or_none(result.get("documentTimingTotalElapsedMs")),
        ),
        ratio_or_none(
            nested_mapping_numeric(
                chunk_summary.get("longestDocumentTimingPhaseElapsedMs"),
                "doclingConvert",
            )
            or nested_mapping_numeric(
                chunk_summary.get("documentTimingPhaseElapsedMs"),
                "doclingConvert",
            ),
            numeric_or_none(chunk_summary.get("longestDocumentTimingTotalElapsedMs"))
            or numeric_or_none(chunk_summary.get("documentTimingTotalElapsedMs")),
        ),
    ]


def page_range_docling_chunk_summary(result: dict[str, Any]) -> dict[str, Any]:
    summary = result.get("forceHybridPageOcrTimingPageRangeDoclingFallbackChunkSummary")
    return summary if isinstance(summary, dict) else {}


def sum_numeric(results: list[dict[str, Any]], key: str) -> float:
    return sum(numeric_values(results, key))


def max_nested_numeric(
    results: list[dict[str, Any]],
    mapping_key: str,
    value_key: str,
) -> float | None:
    values = nested_numeric_values(results, mapping_key, value_key)
    return max(values) if values else None


def max_nested_mapping_string(
    results: list[dict[str, Any]],
    mapping_key: str,
    value_key: str,
    string_key: str,
) -> str | None:
    best_value: float | None = None
    best_string: str | None = None
    for result in results:
        mapping = result.get(mapping_key)
        if not isinstance(mapping, dict):
            continue
        value = numeric_or_none(mapping.get(value_key))
        string_value = mapping.get(string_key)
        if value is None or not isinstance(string_value, str) or not string_value:
            continue
        if best_value is None or value > best_value:
            best_value = value
            best_string = string_value
    return best_string


def max_nested_mapping_numeric(
    results: list[dict[str, Any]],
    mapping_key: str,
    value_key: str,
    numeric_key: str,
) -> float | None:
    best_value: float | None = None
    best_numeric: float | None = None
    for result in results:
        mapping = result.get(mapping_key)
        if not isinstance(mapping, dict):
            continue
        value = numeric_or_none(mapping.get(value_key))
        numeric_value = numeric_or_none(mapping.get(numeric_key))
        if value is None or numeric_value is None:
            continue
        if best_value is None or value > best_value:
            best_value = value
            best_numeric = numeric_value
    return best_numeric


def sum_nested_numeric(
    results: list[dict[str, Any]],
    mapping_key: str,
    value_key: str,
) -> float:
    return sum(nested_numeric_values(results, mapping_key, value_key))


def numeric_values(results: list[dict[str, Any]], key: str) -> list[float]:
    return [
        float(value) for result in results if isinstance((value := result.get(key)), int | float)
    ]


def numeric_or_none(value: Any) -> float | None:
    return float(value) if isinstance(value, int | float) else None


def nested_mapping_numeric(mapping: Any, value_key: str) -> float | None:
    if not isinstance(mapping, dict):
        return None
    return numeric_or_none(mapping.get(value_key))


def subtract_numeric(left: float | None, right: float | None) -> float | None:
    if left is None or right is None:
        return None
    return max(left - right, 0.0)


def ratio_or_none(numerator: float | None, denominator: float | None) -> float | None:
    if numerator is None or denominator is None or denominator <= 0.0:
        return None
    return numerator / denominator


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
