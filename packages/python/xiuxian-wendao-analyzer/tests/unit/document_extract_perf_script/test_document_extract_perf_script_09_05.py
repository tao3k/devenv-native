"""document_extract_perf_script test slice 9."""

from __future__ import annotations

from .support import (
    _load_benchmark_module,
    pytest,
)


def test_precision_speed_summary_tracks_quality_and_latency() -> None:
    benchmark = _load_benchmark_module()

    summary = benchmark.summarize_results(
        [
            {
                "totalRows": 21,
                "forceErrorRows": 0,
                "cacheErrorRows": 0,
                "shardCacheReuseErrorRows": 0,
                "requestCount": 2,
                "arrowIpcBytes": 2048,
                "cacheSpeedup": 12.5,
                "duplicateMissConverterCalls": 1,
                "artifactErrorCount": 0,
                "structureRows": 21,
                "structureOcrPageBlocks": 21,
                "structureOcrRegionBlocks": 0,
                "structureBboxBlocks": 21,
                "structureReadingOrderSorted": True,
                "structureOrderStable": True,
                "structureOrderMismatchCount": 0,
                "structureParityChecked": True,
                "structureParityPassed": True,
                "structureParityErrorCount": 0,
                "metricsRows": 21,
                "metricsResultChars": 4096,
                "metricsBboxCount": 21,
                "metricsRustSchedulerElapsedMs": 45.5,
                "documentTimingTotalElapsedMs": 950.0,
                "documentTimingOverheadMs": 50.0,
                "documentTimingPhaseElapsedMs": {
                    "doclingConvert": 900.0,
                    "total": 950.0,
                },
                "forceRefreshMs": 1000.0,
                "cacheHitP95Ms": 4.0,
                "shardCacheReuseForceMs": 80.0,
                "artifactRegistryReuseForceMs": 12.0,
                "wallTimeMs": 1005.0,
            }
        ],
        {
            "errorRows": 0,
            "wallTimeMs": 25.0,
            "rustJobsStatusSummary": benchmark.summarize_rust_jobs_status_samples([]),
        },
    )

    precision_speed = summary["precisionSpeedSummary"]
    assert precision_speed["precisionGatePassed"] is True
    assert precision_speed["structureReadingOrderSorted"] is True
    assert precision_speed["structureOrderStable"] is True
    assert precision_speed["structureOrderMismatches"] == 0
    assert precision_speed["structureParityPassed"] is True
    assert precision_speed["ocrPageBlocks"] == 21
    assert precision_speed["bboxBlocks"] == 21
    assert precision_speed["maxForceRefreshMs"] == 1000.0
    assert precision_speed["maxCacheHitP95Ms"] == 4.0
    assert precision_speed["maxShardCacheReuseForceMs"] == 80.0
    assert precision_speed["maxArtifactRegistryReuseForceMs"] == 12.0
    assert precision_speed["totalRustSchedulerElapsedMs"] == 45.5
    assert precision_speed["totalDocumentTimingElapsedMs"] == 950.0
    assert precision_speed["totalDoclingConvertMs"] == 900.0
    assert precision_speed["maxDoclingConvertMs"] == 900.0
    assert precision_speed["maxDoclingConvertShare"] == pytest.approx(900.0 / 950.0)
    assert precision_speed["maxDocumentTimingOverheadMs"] == 50.0
    assert precision_speed["maxDocumentTimingOverheadShare"] == pytest.approx(0.05)
    assert precision_speed["distinctMissWallTimeMs"] == 25.0
