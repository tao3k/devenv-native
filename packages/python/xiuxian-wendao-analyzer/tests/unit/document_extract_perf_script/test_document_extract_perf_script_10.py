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
                        6 if report_path.name == "shard-cache-reuse.json" else 0
                    ),
                    "hybridPageOcrTimingOcr2RegionRenderCacheMissCount": (
                        0 if report_path.name == "shard-cache-reuse.json" else 6
                    ),
                    "hybridPageOcrTimingSchedulerTrace": [
                        {
                            "lane": "source-pdf-page-range",
                            "shardCount": 7,
                            "pageStart": 0,
                            "pageEnd": 6,
                            "queueWaitMs": 3.0,
                            "dispatchStartMs": 4.0,
                            "dispatchEndMs": 19_331.0,
                            "latencyMs": 19_327.0,
                            "textCharCount": 48_879,
                        },
                    ],
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
    assert result["shardCacheReuseMetricsRustSchedulerElapsedMs"] == 40.0
    assert (
        result["shardCacheReuseHybridPageOcrTimingPhaseElapsedMs"]["ocrScheduler"]
        == 4.0
    )
    assert result["forceHybridPageOcrTimingOcr2RegionRenderCacheMissCount"] == 6
    assert (
        result["shardCacheReuseHybridPageOcrTimingOcr2RegionRenderCacheHitCount"] == 6
    )
    assert (
        result["shardCacheReuseHybridPageOcrTimingOcr2RegionRenderCacheMissCount"] == 0
    )
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


def test_pdf_ocr_milestone_guard_passes_stored_precision_speed_shape() -> None:
    benchmark = _load_benchmark_module()

    summary = benchmark.summarize_results([_pdf_ocr_milestone_result()])

    guard = summary["precisionSpeedSummary"]["pdfOcrMilestoneGuard"]
    assert guard["checked"] is True
    assert guard["passed"] is True
    assert guard["regressions"] == []
    assert guard["observations"][0]["metricsResultChars"] == 103_984


def test_pdf_ocr_milestone_guard_flags_latency_regression() -> None:
    benchmark = _load_benchmark_module()
    result = _pdf_ocr_milestone_result(force_ms=46_000.0)

    guard = benchmark.summarize_results([result])["precisionSpeedSummary"][
        "pdfOcrMilestoneGuard"
    ]

    assert guard["checked"] is True
    assert guard["passed"] is False
    assert (
        "forceRefreshMs 46000.000 exceeded baseline 45941.076" in guard["regressions"]
    )


def test_pdf_ocr_milestone_guard_flags_char_count_regression() -> None:
    benchmark = _load_benchmark_module()
    result = _pdf_ocr_milestone_result(metrics_result_chars=98_157)

    guard = benchmark.summarize_results([result])["precisionSpeedSummary"][
        "pdfOcrMilestoneGuard"
    ]

    assert guard["checked"] is True
    assert guard["passed"] is False
    assert "metricsResultChars 98157 below baseline 103984" in guard["regressions"]
    assert guard["observations"][0]["metricsResultChars"] == 98_157


def test_pdf_ocr_milestone_guard_accepts_region_sidecars() -> None:
    benchmark = _load_benchmark_module()
    result = _pdf_ocr_milestone_result(
        force_ms=25_660.228292,
        resources_rows=27,
        structure_rows=27,
        ocr_page_blocks=21,
        ocr_region_blocks=6,
        bbox_blocks=27,
        metrics_rows=27,
        metrics_result_chars=113_088,
        shard_cache_reuse_ms=5600.0,
        shard_cache_reuse_scheduler_ms=3.6,
    )

    guard = benchmark.summarize_results([result])["precisionSpeedSummary"][
        "pdfOcrMilestoneGuard"
    ]

    assert guard["checked"] is True
    assert guard["passed"] is True
    assert guard["regressions"] == []
    assert guard["observations"][0]["ocrPageBlocks"] == 21
    assert guard["observations"][0]["ocrRegionBlocks"] == 6
    assert guard["observations"][0]["shardCacheReuseForceMs"] == 5600.0
    assert (
        guard["observations"][0]["shardCacheReuseMetricsRustSchedulerElapsedMs"] == 3.6
    )


def test_pdf_ocr_milestone_guard_flags_region_sidecar_scheduler_regression() -> None:
    benchmark = _load_benchmark_module()
    result = _pdf_ocr_milestone_result(
        resources_rows=27,
        structure_rows=27,
        ocr_page_blocks=21,
        ocr_region_blocks=6,
        bbox_blocks=27,
        metrics_rows=27,
        metrics_result_chars=113_088,
        shard_cache_reuse_ms=5600.0,
        shard_cache_reuse_scheduler_ms=300.0,
    )

    guard = benchmark.summarize_results([result])["precisionSpeedSummary"][
        "pdfOcrMilestoneGuard"
    ]

    assert guard["checked"] is True
    assert guard["passed"] is False
    assert (
        "shardCacheReuseMetricsRustSchedulerElapsedMs 300.000 exceeded baseline 213.161"
        in guard["regressions"]
    )


def test_pdf_ocr_milestone_guard_ignores_non_milestone_fixture() -> None:
    benchmark = _load_benchmark_module()
    result = _pdf_ocr_milestone_result(
        fixture="small-md",
        attachment_class="structured_text",
        resources_rows=1,
        structure_rows=1,
        ocr_page_blocks=0,
        bbox_blocks=0,
        metrics_rows=0,
        metrics_result_chars=0,
    )

    guard = benchmark.summarize_results([result])["precisionSpeedSummary"][
        "pdfOcrMilestoneGuard"
    ]

    assert guard["checked"] is False
    assert guard["passed"] is False
    assert guard["reason"] == "no OCR-positive 21-page PDF milestone fixture observed"


def test_hosted_vlm_promotion_gate_ignores_non_hosted_candidate() -> None:
    benchmark = _load_benchmark_module()
    payload = _hosted_vlm_promotion_payload(
        benchmark,
        rust_pdf_ocr_profile_planner="fast-risk-window",
        rust_pdf_hosted_vlm_region_planner="disabled",
        request_count=0,
        ocr_region_blocks=0,
    )

    gate = benchmark.hosted_vlm_promotion_gate(payload)

    assert gate["checked"] is False
    assert gate["passed"] is False
    assert gate["reasons"] == ["not a hosted VLM/OCR promotion candidate"]


def test_hosted_vlm_promotion_gate_passes_precise_fast_candidate() -> None:
    benchmark = _load_benchmark_module()
    payload = _hosted_vlm_promotion_payload(
        benchmark,
        force_ms=10_000.0,
        shard_cache_reuse_ms=144.232,
    )

    gate = benchmark.hosted_vlm_promotion_gate(payload)

    assert gate["checked"] is True
    assert gate["passed"] is True
    assert gate["reasons"] == []
    assert gate["observed"]["requestCount"] == 3
    assert gate["observed"]["ocrRegionBlocks"] == 3


def test_hosted_vlm_promotion_observed_reports_local_overhead() -> None:
    benchmark = _load_benchmark_module()
    payload = _hosted_vlm_promotion_payload(
        benchmark,
        force_ms=24_362.710,
        shard_cache_reuse_ms=144.232,
    )
    payload["rustPdfHostedVlmRegionPipeline"] = "render-dispatch"
    payload["rustPdfHostedVlmRegionRenderAhead"] = 3
    payload["rustPdfHostedVlmRegionRenderChunk"] = "region"
    payload["hostedVlmOcr"]["requestSummary"]["requestWallSpanMs"] = 12_261.0
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

    gate = benchmark.hosted_vlm_promotion_gate(payload)

    observed = gate["observed"]
    assert observed["rustPdfHostedVlmRegionPipeline"] == "render-dispatch"
    assert observed["rustPdfHostedVlmRegionRenderAhead"] == 3
    assert observed["rustPdfHostedVlmRegionRenderChunk"] == "region"
    assert observed["forceHostedVlmLocalOverheadMs"] == pytest.approx(12_101.71)
    assert observed["forceHostedVlmSchedulerNonRequestMs"] == pytest.approx(4_477.118)
    assert observed["forceHostedVlmRegionRenderMs"] == pytest.approx(7_255.609)
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


def test_hosted_vlm_promotion_gate_rejects_current_auto_region_latency() -> None:
    benchmark = _load_benchmark_module()
    payload = _hosted_vlm_promotion_payload(
        benchmark,
        force_ms=39_276.940291,
        shard_cache_reuse_ms=3268.908833,
        shard_cache_reuse_scheduler_ms=3.6,
    )

    gate = benchmark.hosted_vlm_promotion_gate(payload)

    assert gate["checked"] is True
    assert gate["passed"] is False
    assert any(
        "maxForceRefreshMs 39276.940 exceeded" in reason for reason in gate["reasons"]
    )
    assert not any(
        "maxShardCacheReuseForceMs 3268.909 exceeded" in reason
        for reason in gate["reasons"]
    )


def test_hosted_vlm_promotion_gate_treats_adaptive_region_planner_as_candidate() -> (
    None
):
    benchmark = _load_benchmark_module()
    payload = _hosted_vlm_promotion_payload(
        benchmark,
        rust_pdf_ocr_profile_planner="fast-risk-window",
        rust_pdf_hosted_vlm_region_planner="profile-risk-window-adaptive",
        request_count=0,
        ocr_region_blocks=0,
    )

    gate = benchmark.hosted_vlm_promotion_gate(payload)

    assert gate["checked"] is True
    assert gate["passed"] is False
    assert "no Hosted VLM/OCR requests observed" in gate["reasons"]
    assert (
        "automatic hosted VLM/OCR region planner produced no hosted VLM/OCR region requests"
        in gate["reasons"]
    )


def test_hosted_vlm_promotion_gate_requires_clean_scaffold_validation() -> None:
    benchmark = _load_benchmark_module()
    payload = _hosted_vlm_promotion_payload(
        benchmark,
        scaffold_mode="region-table-json",
        scaffold_applied_count=3,
        scaffold_validation_failure_count=1,
    )

    gate = benchmark.hosted_vlm_promotion_gate(payload)

    assert gate["checked"] is True
    assert gate["passed"] is False
    assert "Hosted VLM/OCR scaffold validation failure count was 1" in gate["reasons"]
    assert gate["observed"]["scaffoldMode"] == "region-table-json"
    assert gate["observed"]["scaffoldValidationFailureCount"] == 1


def test_hosted_vlm_promotion_gate_requires_clean_atlas_validation() -> None:
    benchmark = _load_benchmark_module()
    payload = _hosted_vlm_promotion_payload(
        benchmark,
        region_atlas_mode="same-page-json",
        scaffold_validation_failure_count=1,
    )

    gate = benchmark.hosted_vlm_promotion_gate(payload)

    assert gate["checked"] is True
    assert gate["passed"] is False
    assert "Hosted VLM/OCR atlas validation failure count was 1" in gate["reasons"]
    assert gate["observed"]["regionAtlasMode"] == "same-page-json"
    assert gate["observed"]["scaffoldValidationFailureCount"] == 1


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
            "openRouterModel": "baidu/qianfan-ocr-fast:free",
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
    assert "Rust OCR source-range trace" in markdown
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
