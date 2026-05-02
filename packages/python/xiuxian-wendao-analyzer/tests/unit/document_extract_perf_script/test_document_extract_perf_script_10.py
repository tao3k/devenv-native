"""document_extract_perf_script test slice 10."""

from __future__ import annotations

from .support import (
    Path,
    _load_benchmark_module,
    pytest,
)


def test_benchmark_ocr_shard_cache_root_defaults_to_temp_for_local_runs(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    monkeypatch.delenv("WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT", raising=False)
    args = benchmark.argparse.Namespace(
        ocr_shard_cache_root=None,
        external_endpoint=False,
    )

    assert (
        benchmark.benchmark_ocr_shard_cache_root(args, tmp_path)
        == (tmp_path / "ocr-shard-cache").resolve()
    )


def test_benchmark_ocr_shard_cache_root_honors_explicit_root(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    explicit_root = tmp_path / "explicit-ocr-shards"
    args = benchmark.argparse.Namespace(
        ocr_shard_cache_root=explicit_root,
        external_endpoint=False,
    )

    assert (
        benchmark.benchmark_ocr_shard_cache_root(args, tmp_path)
        == explicit_root.resolve()
    )


def test_run_fixture_probe_can_measure_cache_reuse_probes(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
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
        latency_by_report = {
            "force.json": 1000.0,
            "shard-cache-reuse.json": 42.0,
            "artifact-registry-reuse.json": 9.0,
            "cache.json": 4.0,
        }
        latency = latency_by_report[report_path.name]
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
            "artifactReports": [
                {
                    "resourcesArrowExists": True,
                    "resourcesRowCount": 21,
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
                }
            ],
        }

    monkeypatch.setattr(benchmark, "run_cargo_perf_test", fake_run_cargo_perf_test)
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
    assert result["artifactRegistryReuseEnabled"] is True
    assert result["artifactRegistryReuseForceMs"] == 9.0
    assert result["artifactRegistryReuseErrorRows"] == 0
    assert result["cacheHitP50Ms"] == 4.0
    assert result["metricsRows"] == 21
    assert result["metricsResultChars"] == 2048
    assert result["metricsBboxCount"] == 21
    assert result["structureOrderStable"] is True
    assert result["structureOrderComparedRuns"] == 4
    assert result["structureOrderMismatchCount"] == 0


def test_run_fixture_probe_can_fail_on_structure_order_mismatch(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()

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
        _ = args, source, output_dir, force, iterations
        signature_by_report = {
            "force.json": "force-order",
            "shard-cache-reuse.json": "force-order",
            "cache.json": "cache-order",
        }
        return {
            "latenciesMs": [1.0],
            "requestCount": 1,
            "rowCount": 1,
            "batchCount": 1,
            "arrowIpcBytes": 1,
            "wallTimeMs": 1.0,
            "concurrency": concurrency,
            "errorRowCount": 0,
            "statusCounts": {"succeeded": 1},
            "artifactReports": [
                {
                    "structureArrowExists": True,
                    "structureRowCount": 1,
                    "structureReadingOrderSorted": True,
                    "structureOrderSignature": signature_by_report[report_path.name],
                    "structureOrderFirstKey": "000000|000000.000000|000000|a",
                    "structureOrderLastKey": "000000|000000.000000|000000|a",
                }
            ],
        }

    monkeypatch.setattr(benchmark, "run_cargo_perf_test", fake_run_cargo_perf_test)
    args = benchmark.argparse.Namespace(
        duplicate_miss_concurrency=0,
        fail_on_error_rows=True,
        fail_on_duplicate_conversions=False,
        fail_on_structure_order_mismatch=True,
        iterations=1,
        concurrency=1,
        shard_cache_reuse_probe=True,
        artifact_registry_reuse_probe=False,
    )

    with pytest.raises(SystemExit, match="unstable structure order"):
        benchmark.run_fixture_probe(
            args,
            "arxiv",
            tmp_path / "source.pdf",
            tmp_path / "out",
        )


def test_summary_and_markdown_report_distinct_miss_burst() -> None:
    benchmark = _load_benchmark_module()
    result = {
        "fixture": "small-md",
        "totalRows": 10,
        "forceErrorRows": 0,
        "cacheErrorRows": 0,
        "shardCacheReuseEnabled": True,
        "shardCacheReuseForceMs": 42.0,
        "shardCacheReuseErrorRows": 0,
        "artifactRegistryReuseEnabled": True,
        "artifactRegistryReuseForceMs": 9.0,
        "artifactRegistryReuseErrorRows": 0,
        "requestCount": 2,
        "arrowIpcBytes": 1024,
        "cacheSpeedup": 2.0,
        "duplicateMissConverterCalls": None,
        "rustJobsStatusSummary": benchmark.summarize_rust_jobs_status_samples([]),
        "rows": 5,
        "forceRefreshMs": 10.0,
        "cacheHitP50Ms": 1.0,
        "cacheHitP95Ms": 2.0,
        "wallTimeMs": 3.0,
        "cacheMaxRssKb": None,
        "rustJobsMaxQueuedJobs": None,
        "rustJobsMaxRunningJobs": None,
        "rustJobsMinAvailableConversionPermits": None,
        "metricsRows": 2,
        "metricsResultChars": 80,
        "metricsBboxCount": 2,
        "metricsRustSchedulerElapsedMs": 12.0,
        "documentTimingRows": 3,
        "documentTimingTotalElapsedMs": 30.0,
        "documentTimingOverheadMs": 8.0,
        "documentTimingPhaseElapsedMs": {
            "doclingConvert": 20.0,
            "total": 30.0,
        },
        "structureParityChecked": True,
        "structureParityPassed": True,
        "structureParityErrorCount": 0,
        "structureOrderStable": True,
        "structureOrderMismatchCount": 0,
        "artifactReports": [
            {
                "resourceTypeCounts": {"document": 1, "table": 1},
                "resourceStatusCounts": {"ok": 2},
                "structureBlockTypeCounts": {"document": 1, "table": 1},
                "metricsStatusCounts": {"succeeded": 2},
                "documentTimingStatusCounts": {"ok": 3},
                "documentTimingPhaseElapsedMs": {
                    "doclingConvert": 20.0,
                    "total": 30.0,
                },
                "imageAttachmentAudit": {
                    "format": "png",
                    "widthPx": 640,
                    "heightPx": 480,
                    "pixelCount": 307200,
                    "dimensionSource": "png_ihdr",
                    "rustAccelerationCandidate": "image_ocr_cache_candidate",
                },
                "archiveAttachmentAudit": {
                    "archiveFormat": "tar.gz",
                    "memberCount": 10,
                    "regularFileCount": 10,
                    "xmlMemberCount": 1,
                    "imageMemberCount": 3,
                    "totalMemberSizeBytes": 267702,
                    "extensionCounts": {
                        "html": 3,
                        "tif": 3,
                        "txt": 3,
                        "xml": 1,
                    },
                    "largestMemberSizeBytes": 59518,
                    "rustAccelerationCandidate": "mets_gbs_member_manifest_candidate",
                },
            }
        ],
    }
    distinct_report = {
        "enabled": True,
        "fixtures": ["distinct-01", "distinct-02"],
        "fixtureCount": 2,
        "requestCount": 2,
        "converterCalls": 2,
        "errorRows": 0,
        "wallTimeMs": 25.0,
        "rustJobsStatusSummary": {
            "sampleCount": 3,
            "maxQueuedJobs": 2,
            "maxRunningJobs": 2,
            "maxInProcessRunningConversions": 2,
            "maxInProcessScheduledJobs": 2,
            "minAvailableConversionPermits": 2,
            "maxRunningConversions": 4,
            "lastConversionDurationMs": 20,
            "maxConversionDurationMs": 21,
        },
    }

    summary = benchmark.summarize_results([result], distinct_report)

    assert summary["distinctMissFixtureCount"] == 2
    assert summary["distinctMissConverterCalls"] == 2
    assert summary["totalErrorRows"] == 0
    assert summary["rustJobsStatusSummary"]["maxRunningJobs"] == 2
    assert summary["totalDocumentTimingRows"] == 3
    assert summary["totalDocumentTimingElapsedMs"] == 30.0
    assert summary["totalDocumentTimingOverheadMs"] == 8.0
    assert summary["imageAttachmentAuditCount"] == 1
    assert summary["imageKnownDimensionCount"] == 1
    assert summary["imageFormatCounts"] == {"png": 1}
    assert summary["imageDimensionSourceCounts"] == {"png_ihdr": 1}
    assert summary["imageAccelerationCandidates"] == {"image_ocr_cache_candidate": 1}
    assert summary["maxImageWidthPx"] == 640
    assert summary["maxImageHeightPx"] == 480
    assert summary["maxImagePixelCount"] == 307200
    assert summary["archiveAttachmentAuditCount"] == 1
    assert summary["archiveMemberCount"] == 10
    assert summary["archiveXmlMemberCount"] == 1
    assert summary["archiveImageMemberCount"] == 3
    assert summary["archiveFormatCounts"] == {"tar.gz": 1}
    assert summary["archiveExtensionCounts"] == {
        "html": 3,
        "tif": 3,
        "txt": 3,
        "xml": 1,
    }
    assert summary["archiveAccelerationCandidates"] == {
        "mets_gbs_member_manifest_candidate": 1,
    }
    assert summary["maxArchiveLargestMemberSizeBytes"] == 59518
    assert summary["documentTimingPhaseElapsedMs"] == {
        "doclingConvert": 20.0,
        "total": 30.0,
    }
    assert summary["precisionSpeedSummary"]["maxForceRefreshMs"] == 10.0
    assert summary["precisionSpeedSummary"]["maxCacheHitP95Ms"] == 2.0
    assert summary["precisionSpeedSummary"]["totalDoclingConvertMs"] == 20.0
    assert summary["precisionSpeedSummary"]["maxDoclingConvertMs"] == 20.0
    assert summary["precisionSpeedSummary"]["maxDoclingConvertShare"] == pytest.approx(
        20.0 / 30.0
    )
    assert summary["precisionSpeedSummary"][
        "maxDocumentTimingOverheadShare"
    ] == pytest.approx(0.8)
    assert summary["precisionSpeedSummary"]["precisionGatePassed"] is True
    assert summary["precisionSpeedSummary"]["structureOrderStable"] is True
    assert summary["attachmentClassSummary"][0]["attachmentClass"] == "unknown"
    assert summary["attachmentClassSummary"][0]["archiveAttachmentAuditCount"] == 1
    assert summary["attachmentClassSummary"][0]["archiveMemberCount"] == 10
    assert summary["attachmentClassSummary"][0]["archiveFormatCounts"] == {"tar.gz": 1}
    assert summary["attachmentClassSummary"][0]["archiveAccelerationCandidates"] == {
        "mets_gbs_member_manifest_candidate": 1,
    }

    markdown = benchmark.render_markdown(
        {
            "schema": benchmark.REPORT_SCHEMA,
            "mode": "fixture",
            "endpoint": "http://127.0.0.1:50052",
            "rustRestEndpoint": None,
            "iterations": 1,
            "concurrency": 1,
            "flightMode": "async",
            "waitMs": 0,
            "pdfOcrWorker": "skip",
            "pdfOcrWorkers": "auto",
            "rustPdfOcrWorkers": None,
            "rustPdfOcrSourceRangeWorkers": "2",
            "structureBaselineRoot": "/tmp/baselines",
            "pdfOcrProfile": "skip",
            "shardCacheReuseProbe": True,
            "artifactRegistryReuseProbe": True,
            "ocrShardCache": {
                "root": "/tmp/ocr-shards",
                "fileCount": 2,
                "totalBytes": 7,
                "maxBytes": 100,
            },
            "summary": summary,
            "results": [result],
            "distinctMiss": distinct_report,
            "structureBaseline": {
                "enabled": True,
                "fixtureCount": 1,
                "totalErrorRows": 0,
            },
        }
    )
    assert "## Distinct Cold Miss Burst" in markdown
    assert "## Attachment Class Summary" in markdown
    assert "document=1, table=1" in markdown
    assert "image_ocr_cache_candidate=1" in markdown
    assert "small-md:10.000" in markdown
    assert "distinct-01" in markdown
    assert "Shard reuse force ms" in markdown
    assert "Artifact-registry reuse probe" in markdown
    assert "Artifact reuse ms" in markdown
    assert "9.000" in markdown
    assert "42.000" in markdown
    assert "OCR shard cache" in markdown
    assert "files=2" in markdown
    assert "Metrics sidecar" in markdown
    assert "chars=80" in markdown
    assert "Document timing sidecar" in markdown
    assert "Image audit summary" in markdown
    assert "knownDims=1" in markdown
    assert "dimensionSources=png_ihdr=1" in markdown
    assert "Archive audit summary" in markdown
    assert "members=10" in markdown
    assert "suffixes=html=3, tif=3, txt=3, xml=1" in markdown
    assert "mets_gbs_member_manifest_candidate=1" in markdown
    assert "doclingConvert=20.000" in markdown
    assert "overheadMs=8.000" in markdown
    assert "maxDoclingConvertMs=20.000" in markdown
    assert "maxDoclingShare=66.7%" in markdown
    assert "maxTimingOverheadMs=8.000" in markdown
    assert "maxBoundaryOverheadShare=80.0%" in markdown
    assert "Rust PDF OCR source-range workers" in markdown
    assert "Structure parity" in markdown
    assert "Structure order stable across runs" in markdown
    assert "Structure baseline generation" in markdown
    assert "Precision-speed summary" in markdown
    assert "orderStable=True" in markdown
    assert "maxForceMs=10.000" in markdown
