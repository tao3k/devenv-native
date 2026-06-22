"""document_extract_perf_script test slice 10."""

from __future__ import annotations

import subprocess

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
        region_projection_reuse_probe=True,
        artifact_registry_reuse_probe=True,
        ocr_shard_cache_root=tmp_path / "ocr-shard-cache",
    )
    region_render_cache = tmp_path / "hosted-vlm-region-renders"
    region_render_cache.mkdir()
    (region_render_cache / "stale.marker").write_text("route-local cache", encoding="utf-8")

    result = benchmark.run_fixture_probe(
        args,
        "arxiv",
        tmp_path / "source.pdf",
        tmp_path / "out",
    )

    assert [call["report_path"].name for call in calls] == [
        "force.json",
        "shard-cache-reuse.json",
        "region-projection-reuse.json",
        "artifact-registry-reuse.json",
        "cache.json",
    ]
    assert calls[1]["output_dir"] == tmp_path / "out" / "shard-cache-reuse"
    assert calls[1]["force"] is True
    assert calls[2]["output_dir"] == tmp_path / "out" / "region-projection-reuse"
    assert calls[2]["force"] is True
    assert calls[3]["output_dir"] == tmp_path / "out" / "artifact-registry-reuse"
    assert calls[3]["force"] is False
    assert not region_render_cache.exists()
    assert result["shardCacheReuseEnabled"] is True
    assert result["shardCacheReuseForceMs"] == 42.0
    assert result["shardCacheReuseErrorRows"] == 0
    assert result["shardCacheReuseMetricsRustSchedulerElapsedMs"] == 40.0
    assert result["shardCacheReuseHybridPageOcrTimingPhaseElapsedMs"]["ocrScheduler"] == 4.0
    assert result["forceHybridPageOcrTimingOcr2RegionRenderCacheMissCount"] == 6
    assert result["forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheHitCount"] == 0
    assert result["forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheMissCount"] == 6
    assert result["forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheThrottledCount"] == 0
    assert result["forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheByteCount"] == 1200
    assert result["forceHybridPageOcrTimingOcr2RegionRenderReportedElapsedMs"] == 125.0
    assert result["forceHybridPageOcrTimingOcr2RegionPipelinePlannedRenderChunkCount"] == 2
    assert result["forceHybridPageOcrTimingOcr2RegionPipelineEndpointCount"] == 4
    assert result["forceHybridPageOcrTimingOcr2RegionPipelineRenderAheadLimit"] == 3
    assert result["forceHybridPageOcrTimingOcr2RegionPipelineRenderSpawnCount"] == 2
    assert result["forceHybridPageOcrTimingOcr2RegionPipelineRenderChunkCount"] == 2
    assert result["forceHybridPageOcrTimingOcr2RegionPipelineRegionDispatchCount"] == 2
    assert result["shardCacheReuseHybridPageOcrTimingOcr2RegionRenderCacheHitCount"] == 6
    assert result["shardCacheReuseHybridPageOcrTimingOcr2RegionRenderCacheMissCount"] == 0
    assert result["shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheHitCount"] == 0
    assert result["shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheMissCount"] == 0
    assert (
        result["shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheThrottledCount"] == 0
    )
    assert result["shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheByteCount"] == 0
    assert result["shardCacheReuseHybridPageOcrTimingOcr2RegionRenderReportedElapsedMs"] == 0.0
    assert (
        result["shardCacheReuseHybridPageOcrTimingOcr2RegionPipelinePlannedRenderChunkCount"] == 2
    )
    assert result["shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineEndpointCount"] == 4
    assert result["shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineRenderAheadLimit"] == 3
    assert result["shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineRenderSpawnCount"] == 2
    assert result["shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineRenderChunkCount"] == 2
    assert result["shardCacheReuseHybridPageOcrTimingOcr2RegionPipelineRegionDispatchCount"] == 2
    assert result["regionProjectionReuseEnabled"] is True
    assert result["regionProjectionReuseForceMs"] == 21.0
    assert result["regionProjectionReuseErrorRows"] == 0
    assert result["regionProjectionReusePurgePath"] == str(region_render_cache)
    assert result["regionProjectionReusePurgeExisted"] is True
    assert result["regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderCacheHitCount"] == 0
    assert result["regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderCacheMissCount"] == 6
    assert (
        result["regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheHitCount"] == 6
    )
    assert (
        result["regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheMissCount"]
        == 0
    )
    assert (
        result["regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheByteCount"]
        == 1200
    )
    assert (
        result[
            "regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCachePageRasterHitCount"
        ]
        == 2
    )
    assert (
        result[
            "regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheRegionCropHitCount"
        ]
        == 2
    )
    assert (
        result[
            "regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheRegionManifestProjectionRowHitCount"
        ]
        == 2
    )
    assert (
        result[
            "regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheRegionCropMissCount"
        ]
        == 0
    )
    assert result["regionProjectionReuseMetricsRustSchedulerElapsedMs"] == 40.0
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
    assert result["structureOrderComparedRuns"] == 5
    assert result["structureOrderMismatchCount"] == 0


def test_region_projection_reuse_probe_restarts_provider_before_retry(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    install_cache_reuse_probe_fake(benchmark)
    base_run_cargo_perf_test = benchmark.run_cargo_perf_test
    attempts: list[tuple[str, Path]] = []
    failed_once = False

    def flaky_run_cargo_perf_test(
        args: object,
        source: Path,
        output_dir: Path,
        *,
        force: bool,
        iterations: int,
        concurrency: int,
        report_path: Path,
        **kwargs: object,
    ) -> dict[str, object]:
        nonlocal failed_once
        attempts.append((report_path.name, output_dir))
        if report_path.name == "region-projection-reuse.json" and not failed_once:
            failed_once = True
            raise subprocess.CalledProcessError(returncode=1, cmd="cargo test")
        return base_run_cargo_perf_test(
            args,
            source,
            output_dir,
            force=force,
            iterations=iterations,
            concurrency=concurrency,
            report_path=report_path,
            **kwargs,
        )

    benchmark.run_cargo_perf_test = flaky_run_cargo_perf_test
    restart_reasons: list[str] = []
    args = benchmark.argparse.Namespace(
        duplicate_miss_concurrency=0,
        fail_on_error_rows=True,
        fail_on_duplicate_conversions=False,
        fail_on_structure_order_mismatch=True,
        iterations=1,
        concurrency=1,
        shard_cache_reuse_probe=False,
        region_projection_reuse_probe=True,
        artifact_registry_reuse_probe=False,
        ocr_shard_cache_root=tmp_path / "ocr-shard-cache",
    )

    result = benchmark.run_fixture_probe(
        args,
        "arxiv",
        tmp_path / "source.pdf",
        tmp_path / "out",
        restart_provider=restart_reasons.append,
    )

    assert result["regionProjectionReuseErrorRows"] == 0
    assert restart_reasons == ["region projection reuse probe retry"]
    assert (
        "region-projection-reuse.json",
        tmp_path / "out" / "region-projection-reuse",
    ) in attempts
    assert (
        "region-projection-reuse.json",
        tmp_path / "out" / "region-projection-reuse-retry-2",
    ) in attempts
