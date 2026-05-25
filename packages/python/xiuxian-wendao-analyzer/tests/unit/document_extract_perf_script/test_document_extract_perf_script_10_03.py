"""document_extract_perf_script test slice 10."""

from __future__ import annotations

from .cache_reuse_probe import install_cache_reuse_probe_fake
from .support import (
    Path,
    _load_benchmark_module,
    pytest,
)


def test_run_fixture_probe_can_measure_cache_reuse_probes(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    calls = install_cache_reuse_probe_fake(benchmark)
    args = benchmark.argparse.Namespace(
        duplicate_miss_concurrency=0,
        fail_on_error_rows=True,
        fail_on_duplicate_conversions=False,
        fail_on_structure_order_mismatch=True,
        iterations=1,
        concurrency=1,
        shard_cache_reuse_probe=True,
        artifact_registry_reuse_probe=True,
    )

    result = benchmark.run_fixture_probe(
        args,
        "arxiv",
        tmp_path / "source.pdf",
        tmp_path / "out",
    )

    assert [call["report_path"].name for call in calls] == [
        "force.json",
        "shard-cache-reuse.json",
        "artifact-registry-reuse.json",
        "cache.json",
    ]
    assert calls[1]["output_dir"] == tmp_path / "out" / "shard-cache-reuse"
    assert calls[1]["force"] is True
    assert calls[2]["output_dir"] == tmp_path / "out" / "artifact-registry-reuse"
    assert calls[2]["force"] is False
    assert result["shardCacheReuseEnabled"] is True
    assert result["shardCacheReuseForceMs"] == 42.0
    assert result["shardCacheReuseErrorRows"] == 0
    assert result["shardCacheReuseMetricsRustSchedulerElapsedMs"] == 40.0
    assert result["shardCacheReuseHybridPageOcrTimingPhaseElapsedMs"]["ocrScheduler"] == 4.0
    assert result["forceHybridPageOcrTimingOcr2RegionRenderCacheMissCount"] == 6
    assert result["forceHybridPageOcrTimingOcr2RegionRenderReportedElapsedMs"] == 125.0
    assert result["forceHybridPageOcrTimingOcr2RegionPipelinePlannedRenderChunkCount"] == 2
    assert result["forceHybridPageOcrTimingOcr2RegionPipelineEndpointCount"] == 4
    assert result["forceHybridPageOcrTimingOcr2RegionPipelineRenderAheadLimit"] == 3
    assert result["forceHybridPageOcrTimingOcr2RegionPipelineRenderSpawnCount"] == 2
    assert result["forceHybridPageOcrTimingOcr2RegionPipelineRenderChunkCount"] == 2
    assert result["forceHybridPageOcrTimingOcr2RegionPipelineRegionDispatchCount"] == 2
    assert result["shardCacheReuseHybridPageOcrTimingOcr2RegionRenderCacheHitCount"] == 6
    assert result["shardCacheReuseHybridPageOcrTimingOcr2RegionRenderCacheMissCount"] == 0
    assert result["shardCacheReuseHybridPageOcrTimingOcr2RegionRenderReportedElapsedMs"] == 0.0
    assert (
        result["shardCacheReuseHybridPageOcrTimingOcr2RegionPipelinePlannedRenderChunkCount"] == 2
    )
    assert result["shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineEndpointCount"] == 4
    assert result["shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineRenderAheadLimit"] == 3
    assert result["shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineRenderSpawnCount"] == 2
    assert result["shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineRenderChunkCount"] == 2
    assert result["shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineRegionDispatchCount"] == 2
    assert result["artifactRegistryReuseEnabled"] is True
    assert result["artifactRegistryReuseForceMs"] == 9.0
    assert result["artifactRegistryReuseErrorRows"] == 0
    assert result["forceAudioMaterializationByteCount"] == 900
    assert result["forceAudioMaterializationArtifactCacheBackendCounts"] == {
        "foyer": 1,
    }
    assert result["forceAudioMaterializationArtifactCacheHitBytes"] == 0
    assert result["forceAudioMaterializationMediaSplitterBytes"] == 900
    assert result["forceAudioMaterializationWorkflowTotalElapsedMs"] == 12.0
    assert result["forceAudioMaterializationWorkflowStageElapsedMs"] == {
        "audio.base.invoke_worker": 7.0,
        "audio.base.materialize_shards": 5.0,
    }
    assert result["artifactRegistryReuseAudioMaterializationByteCount"] == 900
    assert result["artifactRegistryReuseAudioMaterializationArtifactCacheBackendCounts"] == {
        "foyer": 1
    }
    assert result["artifactRegistryReuseAudioMaterializationArtifactCacheHitBytes"] == 900
    assert result["artifactRegistryReuseAudioMaterializationMediaSplitterBytes"] == 0
    assert result["artifactRegistryReuseAudioMaterializationWorkflowTotalElapsedMs"] == 2.0
    assert result["artifactRegistryReuseAudioMaterializationWorkflowStageElapsedMs"] == {
        "audio.base.invoke_worker": 1.5,
        "audio.base.materialize_shards": 0.5,
    }
    assert result["cacheHitP50Ms"] == 4.0
    assert result["metricsRows"] == 21
    assert result["metricsResultChars"] == 2048
    assert result["metricsBboxCount"] == 21
    assert (
        result["forceHybridPageOcrTimingSchedulerTraceSummary"]["sourceRangeLongestOcrProfile"]
        == "docling-fast-text-ocr"
    )
    assert (
        result["forceHybridPageOcrTimingSchedulerTraceSummary"]["sourceRangeLongestShardType"]
        == "page"
    )
    assert result["structureAuthorityPages"] == 2
    assert result["textShortcutPages"] == 4
    assert result["ocrPatchRegions"] == 3
    assert result["pageRangeDoclingFallbackPages"] == 1
    assert result["pageRangeDoclingFallbackChunkCount"] == 1
    assert (
        result["forceHybridPageOcrTimingPageRangeDoclingFallbackPlan"]["strategy"]
        == "source-profile-weighted"
    )
    assert result["forceHybridPageOcrTimingPageRangeDoclingFallbackPlan"]["targetChunkCount"] == 4
    assert (
        result["forceHybridPageOcrTimingPageRangeDoclingFallbackChunkSummary"]["elapsedMsMax"]
        == 19_327.0
    )
    assert (
        result["forceHybridPageOcrTimingPageRangeDoclingFallbackChunkSummary"]["elapsedMsMin"]
        == 19_327.0
    )
    assert (
        result["forceHybridPageOcrTimingPageRangeDoclingFallbackChunkSummary"]["elapsedMsMean"]
        == 19_327.0
    )
    assert (
        result["forceHybridPageOcrTimingPageRangeDoclingFallbackChunkSummary"]["elapsedMsSpread"]
        == 0.0
    )
    assert (
        result["forceHybridPageOcrTimingPageRangeDoclingFallbackChunkSummary"][
            "elapsedMsMaxToMeanRatio"
        ]
        == 1.0
    )
    assert (
        result["forceHybridPageOcrTimingPageRangeDoclingFallbackChunkSummary"]["longestPageEnd"]
        == 6
    )
    assert (
        result["forceHybridPageOcrTimingPageRangeDoclingFallbackChunkSummary"][
            "sourceProfileEstimatedWeightTotal"
        ]
        == 420
    )
    assert (
        result["forceHybridPageOcrTimingPageRangeDoclingFallbackChunkSummary"][
            "sourceProfileStructureAuthorityRequiredCount"
        ]
        == 4
    )
    assert (
        result["forceHybridPageOcrTimingPageRangeDoclingFallbackChunkSummary"][
            "sourceProfileFastProfileRiskCount"
        ]
        == 2
    )
    assert (
        result["forceHybridPageOcrTimingPageRangeDoclingFallbackChunkSummary"][
            "sourceProfileBackendTextTopupCount"
        ]
        == 1
    )
    assert (
        result["forceHybridPageOcrTimingPageRangeDoclingFallbackChunkSummary"][
            "documentTimingTotalElapsedMs"
        ]
        == 19_100.0
    )
    assert (
        result["forceHybridPageOcrTimingPageRangeDoclingFallbackChunkSummary"][
            "documentTimingPhaseElapsedMs"
        ]["doclingConvert"]
        == 18_700.0
    )
    assert result["forceHybridPageOcrTimingPageRangeDoclingFallbackChunkSummary"][
        "documentExtractProfileCounts"
    ] == {"structure-text": 1}
    assert (
        result["forceHybridPageOcrTimingPageRangeDoclingFallbackChunkSummary"][
            "longestDocumentTimingTotalElapsedMs"
        ]
        == 19_100.0
    )
    assert (
        result["forceHybridPageOcrTimingPageRangeDoclingFallbackChunkSummary"][
            "longestSourceProfile"
        ]["fastProfileRiskCount"]
        == 2
    )
    assert result["fullDoclingFallbackCount"] == 0
    assert result["structureOrderStable"] is True
    assert result["structureOrderComparedRuns"] == 4
    assert result["structureOrderMismatchCount"] == 0
