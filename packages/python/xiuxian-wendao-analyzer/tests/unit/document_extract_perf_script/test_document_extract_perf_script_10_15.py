"""document_extract_perf_script test slice 10."""

from __future__ import annotations

from .support import (
    _load_benchmark_module,
    pytest,
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


def test_hosted_vlm_promotion_observed_reports_local_overhead() -> None:
    benchmark = _load_benchmark_module()
    payload = _hosted_vlm_promotion_payload(
        benchmark,
        force_ms=24_362.710,
        shard_cache_reuse_ms=144.232,
    )
    payload["rustPdfHostedVlmRegionPipeline"] = "render-dispatch"
    payload["rustPdfHostedVlmRegionTargetPixels"] = 750_000.0
    payload["rustPdfHostedVlmRegionMaxSlices"] = 4
    payload["rustPdfHostedVlmRegionRenderAhead"] = 3
    payload["rustPdfHostedVlmRegionRenderChunk"] = "region"
    payload["rustPdfRegionRenderMode"] = "direct-crop"
    payload["rustPdfHostedVlmRegionDispatchChunkSize"] = 1
    payload["rustPdfOcrSchedulerLaneFairness"] = "source-first"
    payload["hostedVlmOcr"]["speculativeRetryMinSourcePixels"] = 1_000_000
    payload["hostedVlmOcr"]["speculativeRetryMinImageBytes"] = 200_000
    payload["hostedVlmOcr"]["requestSummary"]["requestWallSpanMs"] = 12_261.0
    payload["hostedVlmOcr"]["requestSummary"]["slowestRequests"] = [
        {
            "latencyMs": 6924.51,
            "requestKind": "region-hedged",
            "pageIndex": 12,
            "regionIndex": 1,
            "readingOrderKey": "000012.000010",
            "httpAttemptCount": 2,
            "imageBytes": 242_150,
            "sourcePixelArea": 1_795_980,
            "markdownChars": 2086,
            "rasterWidthPx": 930,
            "rasterHeightPx": 1931,
        }
    ]
    payload["summary"]["forceHybridPageOcrTimingPhaseElapsedMs"] = {
        "ocrScheduler": 16_738.118,
        "regionMaterialize": 7_256.222,
        "regionMaterializeRender": 7_255.609,
        "regionPipelineFirstRegionReady": 5_500.0,
        "regionPipelineLastRegionReady": 7_200.0,
        "regionPipelineFirstRegionDispatch": 5_520.0,
        "regionPipelineLastRegionDispatch": 7_220.0,
        "regionPipelineFirstBaseResult": 11_000.0,
        "regionPipelineLastBaseResult": 11_000.0,
        "regionPipelineFirstRegionResult": 12_000.0,
        "regionPipelineLastRegionResult": 16_700.0,
        "total": 24_350.906,
    }
    payload["summary"]["forceHybridPageOcrTimingOcr2RegionRenderReportedElapsedMs"] = 7_255.609
    payload["summary"]["forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheHitCount"] = 4
    payload["summary"]["forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheMissCount"] = 2
    payload["summary"]["forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheThrottledCount"] = 1
    payload["summary"]["forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheByteCount"] = 8192
    payload["summary"]["forceHybridPageOcrTimingOcr2RegionPipelinePlannedRenderChunkCount"] = 6
    payload["summary"]["forceHybridPageOcrTimingOcr2RegionPipelineEndpointCount"] = 4
    payload["summary"]["forceHybridPageOcrTimingOcr2RegionPipelineRenderAheadLimit"] = 3
    payload["summary"]["forceHybridPageOcrTimingOcr2RegionPipelineRenderSpawnCount"] = 6
    payload["summary"]["forceHybridPageOcrTimingOcr2RegionPipelineRenderChunkCount"] = 6
    payload["summary"]["forceHybridPageOcrTimingOcr2RegionPipelineRegionDispatchCount"] = 6
    payload["summary"]["shardCacheReuseHybridPageOcrTimingOcr2RegionRenderReportedElapsedMs"] = 0.0
    payload["summary"][
        "shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheHitCount"
    ] = 0
    payload["summary"][
        "shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheMissCount"
    ] = 0
    payload["summary"][
        "shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheThrottledCount"
    ] = 0
    payload["summary"][
        "shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheByteCount"
    ] = 0
    payload["summary"][
        "shardCacheReuseHybridPageOcrTimingOcr2RegionPipelinePlannedRenderChunkCount"
    ] = 6
    payload["summary"]["shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineEndpointCount"] = 4
    payload["summary"]["shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineRenderAheadLimit"] = 3
    payload["summary"]["shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineRenderSpawnCount"] = 6
    payload["summary"]["shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineRenderChunkCount"] = 6
    payload["summary"][
        "shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineRegionDispatchCount"
    ] = 6

    gate = benchmark.hosted_vlm_promotion_gate(payload)

    observed = gate["observed"]
    assert observed["rustPdfHostedVlmRegionPipeline"] == "render-dispatch"
    assert observed["rustPdfHostedVlmRegionTargetPixels"] == 750_000.0
    assert observed["rustPdfHostedVlmRegionMaxSlices"] == 4
    assert observed["rustPdfHostedVlmRegionRenderAhead"] == 3
    assert observed["rustPdfHostedVlmRegionRenderChunk"] == "region"
    assert observed["rustPdfRegionRenderMode"] == "direct-crop"
    assert observed["rustPdfHostedVlmRegionDispatchChunkSize"] == 1
    assert observed["rustPdfOcrSchedulerLaneFairness"] == "source-first"
    assert observed["speculativeRetryMinSourcePixels"] == 1_000_000
    assert observed["speculativeRetryMinImageBytes"] == 200_000
    assert observed["forceHostedVlmLocalOverheadMs"] == pytest.approx(12_101.71)
    assert observed["forceHostedVlmSchedulerNonRequestMs"] == pytest.approx(4_477.118)
    assert observed["forceHostedVlmRegionRenderMs"] == pytest.approx(7_255.609)
    assert observed["forceHostedVlmRegionRenderReportedElapsedMs"] == pytest.approx(7_255.609)
    assert observed["forceHostedVlmRegionRenderArtifactCacheHitCount"] == 4
    assert observed["forceHostedVlmRegionRenderArtifactCacheMissCount"] == 2
    assert observed["forceHostedVlmRegionRenderArtifactCacheThrottledCount"] == 1
    assert observed["forceHostedVlmRegionRenderArtifactCacheByteCount"] == 8192
    assert observed["forceHostedVlmRegionPipelinePlannedRenderChunkCount"] == 6
    assert observed["forceHostedVlmRegionPipelineEndpointCount"] == 4
    assert observed["forceHostedVlmRegionPipelineRenderAheadLimit"] == 3
    assert observed["forceHostedVlmRegionPipelineRenderSpawnCount"] == 6
    assert observed["forceHostedVlmRegionPipelineRenderChunkCount"] == 6
    assert observed["forceHostedVlmRegionPipelineRegionDispatchCount"] == 6
    assert observed["shardCacheReuseHostedVlmRegionRenderReportedElapsedMs"] == 0.0
    assert observed["shardCacheReuseHostedVlmRegionRenderArtifactCacheHitCount"] == 0
    assert observed["shardCacheReuseHostedVlmRegionRenderArtifactCacheMissCount"] == 0
    assert observed["shardCacheReuseHostedVlmRegionRenderArtifactCacheThrottledCount"] == 0
    assert observed["shardCacheReuseHostedVlmRegionRenderArtifactCacheByteCount"] == 0
    assert observed["shardCacheReuseHostedVlmRegionPipelinePlannedRenderChunkCount"] == 6
    assert observed["shardCacheReuseHostedVlmRegionPipelineEndpointCount"] == 4
    assert observed["shardCacheReuseHostedVlmRegionPipelineRenderAheadLimit"] == 3
    assert observed["shardCacheReuseHostedVlmRegionPipelineRenderSpawnCount"] == 6
    assert observed["shardCacheReuseHostedVlmRegionPipelineRenderChunkCount"] == 6
    assert observed["shardCacheReuseHostedVlmRegionPipelineRegionDispatchCount"] == 6
    assert observed["forceHostedVlmRegionPipelineFirstReadyMs"] == 5_500.0
    assert observed["forceHostedVlmRegionPipelineLastDispatchMs"] == 7_220.0
    assert observed["forceHostedVlmRegionPipelineLastBaseResultMs"] == 11_000.0
    assert observed["forceHostedVlmRegionPipelineFirstRegionResultMs"] == 12_000.0
    assert observed["forceHostedVlmRegionPipelineLastRegionResultMs"] == 16_700.0
    assert observed["forceHostedVlmSourceRangeChunkMaxMs"] == 19_327.0
    assert observed["forceHostedVlmSourceRangeChunkPageStart"] == 0
    assert observed["forceHostedVlmSourceRangeChunkPageEnd"] == 6
    assert observed["forceHostedVlmSourceRangeChunkCount"] == 3
    assert observed["forceHostedVlmSourceRangeTraceChars"] == 109_412
    assert observed["slowestRequests"] == [
        {
            "latencyMs": 6924.51,
            "requestKind": "region-hedged",
            "pageIndex": 12,
            "regionIndex": 1,
            "readingOrderKey": "000012.000010",
            "httpAttemptCount": 2,
            "imageBytes": 242_150,
            "sourcePixelArea": 1_795_980,
            "markdownChars": 2086,
            "rasterWidthPx": 930,
            "rasterHeightPx": 1931,
        }
    ]


def test_hosted_vlm_promotion_observed_reads_fixture_result_timing_fallback() -> None:
    benchmark = _load_benchmark_module()
    payload = _hosted_vlm_promotion_payload(
        benchmark,
        force_ms=11_791.672,
        shard_cache_reuse_ms=163.949,
    )
    payload["rustPdfHostedVlmRegionPipeline"] = "render-dispatch"
    payload["rustPdfHostedVlmRegionRenderAhead"] = None
    payload["hostedVlmOcr"]["requestSummary"]["requestWallSpanMs"] = 8_930.0
    payload["results"] = [
        {
            "forceHybridPageOcrTimingPhaseElapsedMs": {
                "ocrScheduler": 11_591.297,
                "regionPipelineFirstRenderSpawn": 0.1,
                "regionPipelineFirstRegionReady": 2_642.936,
                "regionPipelineLastRegionDispatch": 3_570.775,
            },
            "forceHybridPageOcrTimingOcr2RegionRenderReportedElapsedMs": 9_152.285,
            "forceHybridPageOcrTimingOcr2RegionPipelinePlannedRenderChunkCount": 3,
            "forceHybridPageOcrTimingOcr2RegionPipelineEndpointCount": 4,
            "forceHybridPageOcrTimingOcr2RegionPipelineRenderAheadLimit": 3,
            "forceHybridPageOcrTimingOcr2RegionPipelineRenderSpawnCount": 3,
            "forceHybridPageOcrTimingOcr2RegionPipelineRenderChunkCount": 3,
            "forceHybridPageOcrTimingOcr2RegionPipelineRegionDispatchCount": 3,
            "shardCacheReuseHybridPageOcrTimingOcr2RegionRenderReportedElapsedMs": 0.0,
            "shardCacheReuseHybridPageOcrTimingOcr2RegionPipelinePlannedRenderChunkCount": 3,
            "shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineEndpointCount": 4,
            "shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineRenderAheadLimit": 3,
            "shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineRenderSpawnCount": 3,
            "shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineRenderChunkCount": 3,
            "shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineRegionDispatchCount": 3,
        }
    ]

    observed = benchmark.hosted_vlm_promotion_gate(payload)["observed"]

    assert observed["rustPdfHostedVlmRegionRenderAhead"] is None
    assert observed["forceHostedVlmRegionPipelineFirstReadyMs"] == 2_642.936
    assert observed["forceHostedVlmRegionPipelineLastDispatchMs"] == 3_570.775
    assert observed["forceHostedVlmRegionRenderReportedElapsedMs"] == 9_152.285
    assert observed["forceHostedVlmRegionPipelinePlannedRenderChunkCount"] == 3
    assert observed["forceHostedVlmRegionPipelineEndpointCount"] == 4
    assert observed["forceHostedVlmRegionPipelineRenderAheadLimit"] == 3
    assert observed["forceHostedVlmRegionPipelineRenderSpawnCount"] == 3
    assert observed["forceHostedVlmRegionPipelineRenderChunkCount"] == 3
    assert observed["forceHostedVlmRegionPipelineRegionDispatchCount"] == 3
    assert observed["shardCacheReuseHostedVlmRegionRenderReportedElapsedMs"] == 0.0
    assert observed["shardCacheReuseHostedVlmRegionPipelinePlannedRenderChunkCount"] == 3
    assert observed["shardCacheReuseHostedVlmRegionPipelineEndpointCount"] == 4
    assert observed["shardCacheReuseHostedVlmRegionPipelineRenderAheadLimit"] == 3
    assert observed["shardCacheReuseHostedVlmRegionPipelineRenderSpawnCount"] == 3
    assert observed["shardCacheReuseHostedVlmRegionPipelineRenderChunkCount"] == 3
    assert observed["shardCacheReuseHostedVlmRegionPipelineRegionDispatchCount"] == 3
