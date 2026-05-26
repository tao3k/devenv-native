"""Cache reuse probe fixtures for document extract perf tests."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .support import Path


def install_cache_reuse_probe_fake(benchmark: object) -> list[dict[str, object]]:
    """Install a fake cargo perf runner and return captured calls."""

    calls: list[dict[str, object]] = []

    def fake_run_cargo_perf_test(
        args: object,
        source: Path,
        output_dir: Path,
        *,
        force: bool,
        iterations: int,
        concurrency: int,
        report_path: Path,
        **_kwargs: object,
    ) -> dict[str, object]:
        calls.append(
            {
                "source": source,
                "output_dir": output_dir,
                "force": force,
                "iterations": iterations,
                "concurrency": concurrency,
                "report_path": report_path,
            }
        )
        latency = _latency_by_report(report_path.name)
        return {
            "latenciesMs": [latency],
            "requestCount": 1,
            "rowCount": 21,
            "batchCount": 1,
            "arrowIpcBytes": 117128,
            "wallTimeMs": latency,
            "concurrency": concurrency,
            "errorRowCount": 0,
            "statusCounts": {"succeeded": 21},
            "maxRssKb": None,
            "artifactReports": [_artifact_report(report_path.name, latency)],
        }

    benchmark.run_cargo_perf_test = fake_run_cargo_perf_test
    return calls


def _latency_by_report(report_name: str) -> float:
    return {
        "force.json": 1000.0,
        "shard-cache-reuse.json": 42.0,
        "region-projection-reuse.json": 21.0,
        "artifact-registry-reuse.json": 9.0,
        "cache.json": 4.0,
    }[report_name]


def _artifact_report(report_name: str, latency: float) -> dict[str, object]:
    return {
        "resourcesArrowExists": True,
        "resourcesRowCount": 21,
        "audioMaterializationArtifactCacheConfigured": True,
        "audioMaterializationArtifactCacheBackend": "foyer",
        "audioMaterializationArtifactCacheMemoryBytes": 67_108_864,
        "audioMaterializationArtifactCacheStorageBytes": 536_870_912,
        "audioMaterializationShardCount": 3,
        "audioMaterializationByteCount": 900,
        "audioMaterializationArtifactCacheHitCount": (
            3 if report_name == "artifact-registry-reuse.json" else 0
        ),
        "audioMaterializationArtifactCacheHitBytes": (
            900 if report_name == "artifact-registry-reuse.json" else 0
        ),
        "audioMaterializationExistingOutputCount": 0,
        "audioMaterializationExistingOutputBytes": 0,
        "audioMaterializationMediaSplitterCount": (
            0 if report_name == "artifact-registry-reuse.json" else 3
        ),
        "audioMaterializationMediaSplitterBytes": (
            0 if report_name == "artifact-registry-reuse.json" else 900
        ),
        "audioMaterializationSourceCounts": (
            {"artifact-cache": 3}
            if report_name == "artifact-registry-reuse.json"
            else {"media-splitter": 3}
        ),
        "audioMaterializationSourceBytes": (
            {"artifact-cache": 900}
            if report_name == "artifact-registry-reuse.json"
            else {"media-splitter": 900}
        ),
        "audioMaterializationWorkflowId": "audio.recovery",
        "audioMaterializationWorkflowStageCount": 2,
        "audioMaterializationWorkflowTotalElapsedMs": (
            2.0 if report_name == "artifact-registry-reuse.json" else 12.0
        ),
        "audioMaterializationWorkflowStageElapsedMs": (
            {
                "audio.base.materialize_shards": 0.5,
                "audio.base.invoke_worker": 1.5,
            }
            if report_name == "artifact-registry-reuse.json"
            else {
                "audio.base.materialize_shards": 5.0,
                "audio.base.invoke_worker": 7.0,
            }
        ),
        "structureArrowExists": True,
        "structureRowCount": 21,
        "structureOcrPageBlocks": 21,
        "structureOcrRegionBlocks": 0,
        "structureBboxBlocks": 21,
        "structureReadingOrderSorted": True,
        "structureOrderSignature": "stable-order",
        "structureOrderFirstKey": "000000|000000.000000|000000|page-0",
        "structureOrderLastKey": "000020|000020.000000|000020|page-20",
        "metricsArrowExists": True,
        "metricsRowCount": 21,
        "metricsResultChars": 2048,
        "metricsBboxCount": 21,
        "metricsRustSchedulerElapsedMs": 40.0,
        "hybridPageOcrTimingReportBytes": 128,
        "hybridPageOcrTimingTotalElapsedMs": latency,
        "hybridPageOcrTimingPhaseElapsedMs": {
            "regionMaterialize": 38.0,
            "ocrScheduler": 4.0,
            "total": latency,
        },
        "hybridPageOcrTimingOcrShardCount": 21,
        "hybridPageOcrTimingOcr2RegionShardCount": 0,
        "hybridPageOcrTimingOcr2RegionRequestCount": 6,
        "hybridPageOcrTimingOcr2RegionRenderedShardCount": 6,
        "hybridPageOcrTimingOcr2RegionRenderCacheHitCount": (
            6 if report_name == "shard-cache-reuse.json" else 0
        ),
        "hybridPageOcrTimingOcr2RegionRenderCacheMissCount": (
            0 if report_name == "shard-cache-reuse.json" else 6
        ),
        "hybridPageOcrTimingOcr2RegionRenderArtifactCacheHitCount": (
            6
            if report_name in {"artifact-registry-reuse.json", "region-projection-reuse.json"}
            else 0
        ),
        "hybridPageOcrTimingOcr2RegionRenderArtifactCacheMissCount": (
            6 if report_name == "force.json" else 0
        ),
        "hybridPageOcrTimingOcr2RegionRenderArtifactCacheThrottledCount": 0,
        "hybridPageOcrTimingOcr2RegionRenderArtifactCacheByteCount": (
            1200
            if report_name
            in {"force.json", "artifact-registry-reuse.json", "region-projection-reuse.json"}
            else 0
        ),
        "hybridPageOcrTimingOcr2RegionRenderArtifactCachePageRasterHitCount": (
            2
            if report_name in {"artifact-registry-reuse.json", "region-projection-reuse.json"}
            else 0
        ),
        "hybridPageOcrTimingOcr2RegionRenderArtifactCachePageRasterMissCount": (
            2 if report_name == "force.json" else 0
        ),
        "hybridPageOcrTimingOcr2RegionRenderArtifactCachePageRasterThrottledCount": 0,
        "hybridPageOcrTimingOcr2RegionRenderArtifactCachePageRasterByteCount": (
            300
            if report_name
            in {"force.json", "artifact-registry-reuse.json", "region-projection-reuse.json"}
            else 0
        ),
        "hybridPageOcrTimingOcr2RegionRenderArtifactCacheRegionCropHitCount": (
            2
            if report_name in {"artifact-registry-reuse.json", "region-projection-reuse.json"}
            else 0
        ),
        "hybridPageOcrTimingOcr2RegionRenderArtifactCacheRegionCropMissCount": (
            2 if report_name == "force.json" else 0
        ),
        "hybridPageOcrTimingOcr2RegionRenderArtifactCacheRegionCropThrottledCount": 0,
        "hybridPageOcrTimingOcr2RegionRenderArtifactCacheRegionCropByteCount": (
            600
            if report_name
            in {"force.json", "artifact-registry-reuse.json", "region-projection-reuse.json"}
            else 0
        ),
        "hybridPageOcrTimingOcr2RegionRenderArtifactCacheRegionManifestProjectionHitCount": 0,
        "hybridPageOcrTimingOcr2RegionRenderArtifactCacheRegionManifestProjectionMissCount": 0,
        "hybridPageOcrTimingOcr2RegionRenderArtifactCacheRegionManifestProjectionThrottledCount": 0,
        "hybridPageOcrTimingOcr2RegionRenderArtifactCacheRegionManifestProjectionByteCount": 0,
        "hybridPageOcrTimingOcr2RegionRenderArtifactCacheRegionManifestProjectionRowHitCount": (
            2
            if report_name in {"artifact-registry-reuse.json", "region-projection-reuse.json"}
            else 0
        ),
        "hybridPageOcrTimingOcr2RegionRenderArtifactCacheRegionManifestProjectionRowMissCount": (
            2 if report_name == "force.json" else 0
        ),
        "hybridPageOcrTimingOcr2RegionRenderArtifactCacheRegionManifestProjectionRowThrottledCount": 0,
        "hybridPageOcrTimingOcr2RegionRenderArtifactCacheRegionManifestProjectionRowByteCount": (
            300
            if report_name
            in {"force.json", "artifact-registry-reuse.json", "region-projection-reuse.json"}
            else 0
        ),
        "hybridPageOcrTimingOcr2RegionRenderReportedElapsedMs": (
            0.0 if report_name == "shard-cache-reuse.json" else 125.0
        ),
        "hybridPageOcrTimingOcr2RegionPipelinePlannedRenderChunkCount": (
            2 if report_name != "cache.json" else 0
        ),
        "hybridPageOcrTimingOcr2RegionPipelineEndpointCount": (
            4 if report_name != "cache.json" else 0
        ),
        "hybridPageOcrTimingOcr2RegionPipelineRenderAheadLimit": (
            3 if report_name != "cache.json" else 0
        ),
        "hybridPageOcrTimingOcr2RegionPipelineRenderSpawnCount": (
            2 if report_name != "cache.json" else 0
        ),
        "hybridPageOcrTimingOcr2RegionPipelineRenderChunkCount": (
            2 if report_name != "cache.json" else 0
        ),
        "hybridPageOcrTimingOcr2RegionPipelineRegionDispatchCount": (
            2 if report_name != "cache.json" else 0
        ),
        "structureAuthorityPages": 2,
        "textShortcutPages": 4,
        "ocrPatchRegions": 3,
        "pageRangeDoclingFallbackPages": 1,
        "pageRangeDoclingFallbackChunkCount": 1,
        "pageRangeDoclingFallbackPlan": _page_range_plan(),
        "pageRangeDoclingFallbackChunks": [_page_range_chunk()],
        "fullDoclingFallbackCount": 0,
        "hybridPageOcrTimingSchedulerTrace": [_scheduler_trace()],
    }


def _page_range_plan() -> dict[str, object]:
    return {
        "strategy": "source-profile-weighted",
        "targetChunkCount": 4,
        "fallbackPageCount": 7,
        "rangeCount": 1,
        "chunkSize": None,
        "sourceProfileUsed": True,
        "ranges": [
            {"pageStart": 0, "pageEnd": 6, "oneBasedStart": 1, "oneBasedEnd": 7},
        ],
    }


def _page_range_chunk() -> dict[str, object]:
    return {
        "pageStart": 0,
        "pageEnd": 6,
        "oneBasedStart": 1,
        "oneBasedEnd": 7,
        "documentExtractProfile": "structure-text",
        "elapsedMs": 19_327.0,
        "resourceRows": 21,
        "documentTimingTotalElapsedMs": 19_100.0,
        "documentTimingPhaseElapsedMs": {
            "doclingConvert": 18_700.0,
            "doclingMarkdownExport": 250.0,
            "total": 19_100.0,
        },
        "sourceProfile": {
            "pageCount": 7,
            "estimatedWeightTotal": 420,
            "estimatedWeightMax": 90,
            "contentBytesTotal": 4096,
            "operationCountTotal": 1200,
            "textShowOpsTotal": 700,
            "pathOpsTotal": 180,
            "rectangleOpsTotal": 8,
            "drawObjectOpsTotal": 3,
            "structureAuthorityRequiredCount": 4,
            "fastProfileRiskCount": 2,
            "backendTextTopupCount": 1,
        },
    }


def _scheduler_trace() -> dict[str, object]:
    return {
        "lane": "source-pdf-page-range",
        "ocrProfile": "docling-fast-text-ocr",
        "shardType": "page",
        "shardCount": 7,
        "pageStart": 0,
        "pageEnd": 6,
        "queueWaitMs": 3.0,
        "dispatchStartMs": 4.0,
        "dispatchEndMs": 19_331.0,
        "latencyMs": 19_327.0,
        "textCharCount": 48_879,
    }
