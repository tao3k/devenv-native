"""document_extract_perf_script test slice 10."""

from __future__ import annotations

from .support import (
    _load_benchmark_module,
)


def _pdf_ocr_milestone_result(
    *,
    fixture: str = "ocr-positive-pdf",
    attachment_class: str = "pdf",
    force_ms: float = 43_917.25,
    shard_cache_reuse_ms: float = 144.232,
    shard_cache_reuse_scheduler_ms: float | None = 144.232,
    resources_rows: int = 21,
    structure_rows: int = 21,
    ocr_page_blocks: int = 21,
    ocr_region_blocks: int = 0,
    bbox_blocks: int = 21,
    metrics_rows: int = 21,
    metrics_result_chars: int = 103_984,
) -> dict[str, object]:
    return {
        "fixture": fixture,
        "source": "/fixtures/2604.17337.pdf",
        "attachmentClass": attachment_class,
        "forceRefreshMs": force_ms,
        "forceErrorRows": 0,
        "shardCacheReuseForceMs": shard_cache_reuse_ms,
        "shardCacheReuseErrorRows": 0,
        "shardCacheReuseMetricsRustSchedulerElapsedMs": shard_cache_reuse_scheduler_ms,
        "shardCacheReuseHybridPageOcrTimingPhaseElapsedMs": {
            "regionMaterialize": max(shard_cache_reuse_ms - 10.0, 0.0),
            "ocrScheduler": shard_cache_reuse_scheduler_ms,
            "total": shard_cache_reuse_ms,
        },
        "forceHybridPageOcrTimingSchedulerTraceSummary": {
            "sourceRangeChunkCount": 3,
            "sourceRangeShardCount": 21,
            "sourceRangeTextCharCount": metrics_result_chars,
            "sourceRangeLatencyMsMax": 19_327.0,
            "sourceRangeQueueWaitMsMax": 3.0,
            "sourceRangeDispatchStartMsMin": 4.0,
            "sourceRangeDispatchEndMsMax": 19_331.0,
            "sourceRangeLongestPageStart": 0,
            "sourceRangeLongestPageEnd": 6,
            "sourceRangeLongestShardCount": 7,
            "sourceRangeLongestQueueWaitMs": 3.0,
            "sourceRangeLongestDispatchStartMs": 4.0,
            "sourceRangeLongestDispatchEndMs": 19_331.0,
            "sourceRangeLongestTextCharCount": 48_879,
        },
        "artifactRegistryReuseErrorRows": 0,
        "cacheHitP95Ms": 11.921,
        "cacheErrorRows": 0,
        "cacheSpeedup": 10.0,
        "requestCount": 1,
        "arrowIpcBytes": 117_128,
        "totalRows": resources_rows,
        "duplicateMissConverterCalls": 0,
        "resourcesRows": resources_rows,
        "structureRows": structure_rows,
        "structureOcrPageBlocks": ocr_page_blocks,
        "structureOcrRegionBlocks": ocr_region_blocks,
        "structureBboxBlocks": bbox_blocks,
        "structureReadingOrderSorted": True,
        "structureOrderStable": True,
        "structureOrderMismatchCount": 0,
        "structureParityErrorCount": 0,
        "metricsRows": metrics_rows,
        "metricsResultChars": metrics_result_chars,
        "metricsBboxCount": bbox_blocks,
        "metricsRustSchedulerElapsedMs": 144.232,
        "documentTimingTotalElapsedMs": force_ms,
        "documentTimingOverheadMs": 0.0,
        "documentTimingPhaseElapsedMs": {"doclingConvert": force_ms - 100.0},
        "artifactErrorCount": 0,
        "rustJobsStatusSummary": {},
    }


def _hosted_vlm_promotion_payload(
    benchmark: object,
    *,
    rust_pdf_ocr_profile_planner: str = "hosted-vlm-risk-window",
    rust_pdf_hosted_vlm_region_planner: str = "profile-risk-window",
    force_ms: float = 10_000.0,
    shard_cache_reuse_ms: float = 144.232,
    shard_cache_reuse_scheduler_ms: float | None = 144.232,
    metrics_result_chars: int = 109_412,
    request_count: int = 3,
    success_count: int = 3,
    failure_count: int = 0,
    parse_error_count: int = 0,
    ocr_region_blocks: int = 3,
    region_atlas_mode: str = "disabled",
    scaffold_mode: str = "disabled",
    scaffold_applied_count: int = 0,
    scaffold_validation_failure_count: int = 0,
) -> dict[str, object]:
    result = _pdf_ocr_milestone_result(
        force_ms=force_ms,
        shard_cache_reuse_ms=shard_cache_reuse_ms,
        shard_cache_reuse_scheduler_ms=shard_cache_reuse_scheduler_ms,
        resources_rows=21 + ocr_region_blocks,
        structure_rows=21 + ocr_region_blocks,
        ocr_page_blocks=21,
        ocr_region_blocks=ocr_region_blocks,
        bbox_blocks=21 + ocr_region_blocks,
        metrics_rows=21 + ocr_region_blocks,
        metrics_result_chars=metrics_result_chars,
    )
    summary = benchmark.summarize_results([result])
    return {
        "rustPdfOcrProfilePlanner": rust_pdf_ocr_profile_planner,
        "rustPdfHostedVlmRegionPlanner": rust_pdf_hosted_vlm_region_planner,
        "summary": summary,
        "hostedVlmOcr": {
            "provider": "openrouter",
            "openRouterModel": "baidu/qianfan-ocr-fast",
            "openRouterApiKeyConfigured": True,
            "regionAtlasMode": region_atlas_mode,
            "scaffoldMode": scaffold_mode,
            "requestSummary": {
                "requestCount": request_count,
                "successCount": success_count,
                "failureCount": failure_count,
                "parseErrorCount": parse_error_count,
                "regionShardCount": ocr_region_blocks,
                "scaffoldAppliedCount": scaffold_applied_count,
                "scaffoldValidationFailureCount": scaffold_validation_failure_count,
                "scaffoldJsonCharCountTotal": 800,
                "canonicalMarkdownCharCountTotal": 640,
                "latencyMsP95": 10_000.0,
                "sourcePixelAreaTotal": 8_734_917,
            },
        },
    }


def test_hosted_vlm_promotion_gate_requires_scaffold_count_coverage() -> None:
    benchmark = _load_benchmark_module()
    payload = _hosted_vlm_promotion_payload(
        benchmark,
        scaffold_mode="region-table-json",
        scaffold_applied_count=2,
        scaffold_validation_failure_count=0,
    )

    gate = benchmark.hosted_vlm_promotion_gate(payload)

    assert gate["checked"] is True
    assert gate["passed"] is False
    assert (
        "Hosted VLM/OCR scaffold applied count 2 did not match region shard count 3"
        in gate["reasons"]
    )


def test_hosted_vlm_promotion_gate_requires_fixed_composite_activation() -> None:
    benchmark = _load_benchmark_module()
    payload = _hosted_vlm_promotion_payload(
        benchmark, request_count=3, ocr_region_blocks=3
    )
    payload["hostedVlmOcr"]["regionCompositeSize"] = 3
    payload["hostedVlmOcr"]["regionCompositeMode"] = "fixed"
    payload["hostedVlmOcr"]["requestSummary"]["requestKindCounts"] = {"region": 3}

    gate = benchmark.hosted_vlm_promotion_gate(payload)

    assert gate["checked"] is True
    assert gate["passed"] is False
    assert (
        "Hosted VLM/OCR fixed region composite was configured but no composite "
        "request kind was observed"
    ) in gate["reasons"]
    assert gate["observed"]["regionCompositeRequestCount"] == 0
    assert gate["observed"]["regionCompositeActivated"] is False


def test_hosted_vlm_promotion_gate_records_composite_activation() -> None:
    benchmark = _load_benchmark_module()
    payload = _hosted_vlm_promotion_payload(
        benchmark, request_count=2, ocr_region_blocks=3
    )
    payload["hostedVlmOcr"]["regionCompositeSize"] = 3
    payload["hostedVlmOcr"]["regionCompositeMode"] = "fixed"
    payload["hostedVlmOcr"]["requestSummary"]["requestKindCounts"] = {
        "region-composite": 1,
        "region": 1,
    }

    gate = benchmark.hosted_vlm_promotion_gate(payload)

    assert gate["observed"]["regionCompositeSize"] == 3
    assert gate["observed"]["regionCompositeMode"] == "fixed"
    assert gate["observed"]["regionCompositeRequestCount"] == 1
    assert gate["observed"]["regionCompositeActivated"] is True
