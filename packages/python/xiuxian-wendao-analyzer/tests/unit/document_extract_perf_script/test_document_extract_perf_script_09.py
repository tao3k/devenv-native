"""document_extract_perf_script test slice 9."""

from __future__ import annotations

from .support import (
    Path,
    _load_benchmark_module,
    pytest,
)


def test_start_rust_provider_forwards_hybrid_region_env(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    calls = []

    class FakePopen:
        def __init__(self, command, **kwargs):
            calls.append((command, kwargs))

    monkeypatch.setattr(benchmark.subprocess, "Popen", FakePopen)
    monkeypatch.setenv("SDKROOT", "/tmp/macos-sdk")
    monkeypatch.setenv("LIBRARY_PATH", "/tmp/macos-sdk/usr/lib")
    monkeypatch.setenv("PRJ_ROOT", str(tmp_path / "repo"))
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        rust_provider_bin=None,
        rust_provider_features="cli-bin-support,zhenfa-router,duckdb",
        flight_mode="hybrid-page-ocr",
        hybrid_pdf_render_selection="region-shards",
        pdf_render_region=["pdf=0,1,10,20,110,220"],
        benchmark_fixtures={"pdf": tmp_path / "sample.pdf"},
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        rust_pdf_ocr_workers="6",
        rust_pdf_ocr_source_range_workers="2",
        rust_pdf_local_backend_text="rust-lopdf",
        rust_pdf_local_fast_text="rust-lopdf",
        rust_pdf_fast_text_source_range_split="single-page",
        rust_pdf_fast_text_endpoint_affinity="single-page-first",
        rust_pdf_backend_text_topup="hosted-vlm",
        rust_pdf_ocr_profile_planner="fast-risk-window",
        rust_pdf_hosted_vlm_render_dpi=360,
        rust_pdf_ocr_region_context_ratio=0.2,
        rust_pdf_hosted_vlm_region_planner="profile-risk-window",
        rust_pdf_hosted_vlm_region_pipeline="render-dispatch",
        rust_pdf_hosted_vlm_region_render_ahead=3,
        rust_pdf_hosted_vlm_region_render_chunk="all",
        hosted_vlm_ocr_region_composite_size=3,
        hosted_vlm_ocr_scaffold_mode="region-table-json",
        rust_pdf_ocr_endpoint=["http://127.0.0.1:52051"],
        rust_document_extract_endpoint=["http://127.0.0.1:53051"],
    )

    benchmark.start_rust_provider_server(
        args,
        rust_host="127.0.0.1",
        rust_port=51052,
        python_host="127.0.0.1",
        python_port=51051,
        temp_root=tmp_path,
    )

    command, kwargs = calls[0]
    assert command[:6] == [
        "cargo",
        "run",
        "-p",
        "xiuxian-wendao-studio",
        "--no-default-features",
        "--features",
    ]
    assert command[6] == (
        "performance,cli-bin-support,zhenfa-router,duckdb,document-extract-pdf-render"
    )
    env = kwargs["env"]
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_WORKERS"] == "6"
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS"] == "2"
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_LOCAL_BACKEND_TEXT"] == "rust-lopdf"
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_LOCAL_FAST_TEXT"] == "rust-lopdf"
    assert (
        env["WENDAO_DOCUMENT_EXTRACT_PDF_FAST_TEXT_SOURCE_RANGE_SPLIT"] == "single-page"
    )
    assert (
        env["WENDAO_DOCUMENT_EXTRACT_PDF_FAST_TEXT_ENDPOINT_AFFINITY"]
        == "single-page-first"
    )
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_TOPUP"] == "hosted-vlm"
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER"] == "fast-risk-window"
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_RENDER_DPI"] == "360"
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO"] == "0.2"
    assert (
        env["WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER"]
        == "profile-risk-window"
    )
    assert (
        env["WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PIPELINE"]
        == "render-dispatch"
    )
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_AHEAD"] == "3"
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_CHUNK"] == "all"
    assert env["WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE"] == "3"
    assert (
        env["WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_SCAFFOLD_MODE"]
        == "region-table-json"
    )
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_ENDPOINTS"] == (
        "http://127.0.0.1:52051"
    )
    assert env["WENDAO_DOCUMENT_EXTRACT_ENDPOINTS"] == "http://127.0.0.1:53051"
    assert env["WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT"] == str(
        (tmp_path / "ocr-shard-cache").resolve()
    )
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_SELECTION"] == "region_shards"
    regions = benchmark.json.loads(
        env["WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_JSON"]
    )
    assert regions[0]["source"] == str(tmp_path / "sample.pdf")
    assert regions[0]["regions"][0]["regionIndex"] == 1


def test_start_rust_provider_defaults_document_extract_pool_to_local_worker(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    calls = []

    class FakePopen:
        def __init__(self, command, **kwargs):
            calls.append((command, kwargs))

    monkeypatch.setattr(benchmark.subprocess, "Popen", FakePopen)
    monkeypatch.setenv("SDKROOT", "/tmp/macos-sdk")
    monkeypatch.setenv("LIBRARY_PATH", "/tmp/macos-sdk/usr/lib")
    monkeypatch.setenv("PRJ_ROOT", str(tmp_path / "repo"))
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        rust_provider_bin=None,
        rust_provider_features="cli-bin-support,zhenfa-router,duckdb",
        flight_mode="async",
        hybrid_pdf_render_selection="shard-fallback-pages",
        pdf_render_region=[],
        benchmark_fixtures={},
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        rust_pdf_ocr_workers=None,
        rust_pdf_ocr_source_range_workers=None,
        rust_pdf_ocr_profile_planner="disabled",
        rust_pdf_hosted_vlm_render_dpi=None,
        rust_pdf_ocr_region_context_ratio=None,
        rust_pdf_hosted_vlm_region_planner=None,
        rust_pdf_ocr_endpoint=[],
        rust_document_extract_endpoint=[],
    )

    benchmark.start_rust_provider_server(
        args,
        rust_host="127.0.0.1",
        rust_port=51052,
        python_host="127.0.0.1",
        python_port=51051,
        temp_root=tmp_path,
    )

    env = calls[0][1]["env"]
    assert env["WENDAO_DOCUMENT_EXTRACT_ENDPOINTS"] == "http://127.0.0.1:51051"


def test_start_rust_provider_does_not_forward_hosted_vlm_dpi_downgrade(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    calls = []

    class FakePopen:
        def __init__(self, command, **kwargs):
            calls.append((command, kwargs))

    monkeypatch.setattr(benchmark.subprocess, "Popen", FakePopen)
    monkeypatch.setenv("SDKROOT", "/tmp/macos-sdk")
    monkeypatch.setenv("LIBRARY_PATH", "/tmp/macos-sdk/usr/lib")
    monkeypatch.setenv("PRJ_ROOT", str(tmp_path / "repo"))
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        rust_provider_bin=None,
        rust_provider_features="cli-bin-support,zhenfa-router,duckdb",
        flight_mode="hybrid-page-ocr",
        hybrid_pdf_render_selection="shard-fallback-pages",
        pdf_render_region=[],
        benchmark_fixtures={},
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        rust_pdf_ocr_workers=None,
        rust_pdf_ocr_source_range_workers=None,
        rust_pdf_ocr_profile_planner="hosted-vlm-risk-window",
        rust_pdf_hosted_vlm_render_dpi=180,
        rust_pdf_ocr_region_context_ratio=None,
        rust_pdf_hosted_vlm_region_planner=None,
        rust_pdf_ocr_endpoint=[],
        rust_document_extract_endpoint=[],
    )

    benchmark.start_rust_provider_server(
        args,
        rust_host="127.0.0.1",
        rust_port=51052,
        python_host="127.0.0.1",
        python_port=51051,
        temp_root=tmp_path,
    )

    env = calls[0][1]["env"]
    assert "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_RENDER_DPI" not in env


def test_start_rust_provider_hosted_vlm_planner_enables_pdf_render_feature(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    calls = []

    class FakePopen:
        def __init__(self, command, **kwargs):
            calls.append((command, kwargs))

    monkeypatch.setattr(benchmark.subprocess, "Popen", FakePopen)
    monkeypatch.setenv("SDKROOT", "/tmp/macos-sdk")
    monkeypatch.setenv("LIBRARY_PATH", "/tmp/macos-sdk/usr/lib")
    monkeypatch.setenv("PRJ_ROOT", str(tmp_path / "repo"))
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        rust_provider_bin=None,
        rust_provider_features="cli-bin-support,zhenfa-router,duckdb",
        flight_mode="hybrid-page-ocr",
        hybrid_pdf_render_selection="shard-fallback-pages",
        pdf_render_region=[],
        benchmark_fixtures={},
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        rust_pdf_ocr_workers=None,
        rust_pdf_ocr_source_range_workers=None,
        rust_pdf_ocr_profile_planner="hosted-vlm-risk-window",
        rust_pdf_hosted_vlm_render_dpi=None,
        rust_pdf_ocr_region_context_ratio=None,
        rust_pdf_hosted_vlm_region_planner=None,
        rust_pdf_ocr_endpoint=[],
        rust_document_extract_endpoint=[],
    )

    benchmark.start_rust_provider_server(
        args,
        rust_host="127.0.0.1",
        rust_port=51052,
        python_host="127.0.0.1",
        python_port=51051,
        temp_root=tmp_path,
    )

    command, _kwargs = calls[0]
    assert "document-extract-pdf-render" in command[6].split(",")
    assert (
        calls[0][1]["env"]["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER"]
        == "hosted-vlm-risk-window"
    )


def test_start_rust_provider_can_use_prebuilt_binary(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    calls = []

    class FakePopen:
        def __init__(self, command, **kwargs):
            calls.append((command, kwargs))

    monkeypatch.setattr(benchmark.subprocess, "Popen", FakePopen)
    monkeypatch.setenv("SDKROOT", "/tmp/macos-sdk")
    monkeypatch.setenv("LIBRARY_PATH", "/tmp/macos-sdk/usr/lib")
    monkeypatch.setenv("PRJ_ROOT", str(tmp_path / "repo"))
    provider_bin = tmp_path / "wendao_search_flight_server"
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        rust_provider_bin=provider_bin,
        rust_provider_features="cli-bin-support,zhenfa-router,duckdb",
        flight_mode="hybrid-page-ocr",
        hybrid_pdf_render_selection="shard-fallback-pages",
        pdf_render_region=[],
        benchmark_fixtures={},
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        rust_pdf_ocr_workers=None,
        rust_pdf_ocr_source_range_workers=None,
        rust_pdf_ocr_profile_planner="disabled",
        rust_pdf_hosted_vlm_render_dpi=None,
        rust_pdf_ocr_region_context_ratio=None,
        rust_pdf_hosted_vlm_region_planner=None,
        rust_pdf_ocr_endpoint=[],
        rust_document_extract_endpoint=[],
    )

    benchmark.start_rust_provider_server(
        args,
        rust_host="127.0.0.1",
        rust_port=51052,
        python_host="127.0.0.1",
        python_port=51051,
        temp_root=tmp_path,
    )

    command, _kwargs = calls[0]
    assert command == [
        str(provider_bin),
        "127.0.0.1:51052",
        "alpha/repo",
        str(tmp_path / "repo"),
        "--schema-version=v2",
    ]


def test_start_valkey_server_uses_temp_runtime_flags(
    monkeypatch, tmp_path: Path
) -> None:
    benchmark = _load_benchmark_module()
    calls = []

    class FakePopen:
        def __init__(self, command, **kwargs):
            calls.append((command, kwargs))

    monkeypatch.setattr(benchmark.subprocess, "Popen", FakePopen)

    benchmark.start_valkey_server(host="127.0.0.1", port=51079, temp_root=tmp_path)

    command, kwargs = calls[0]
    assert command[:5] == ["valkey-server", "--bind", "127.0.0.1", "--port", "51079"]
    assert "--appendonly" in command
    assert "no" in command
    assert kwargs["start_new_session"] is True


def test_summary_reports_duplicate_miss_converter_calls() -> None:
    benchmark = _load_benchmark_module()

    summary = benchmark.summarize_results(
        [
            {
                "totalRows": 10,
                "forceErrorRows": 0,
                "cacheErrorRows": 0,
                "requestCount": 2,
                "arrowIpcBytes": 1024,
                "cacheSpeedup": 2.0,
                "duplicateMissConverterCalls": 1,
            }
        ]
    )

    assert summary["totalDuplicateMissConverterCalls"] == 1
    assert summary["maxDuplicateMissConverterCalls"] == 1
    assert summary["rustJobsStatusSummary"]["sampleCount"] == 0
    assert summary["precisionSpeedSummary"]["precisionGatePassed"] is True
    assert summary["precisionSpeedSummary"]["errorRows"] == 0


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


def test_attachment_class_summary_groups_precision_and_speed() -> None:
    benchmark = _load_benchmark_module()

    summary = benchmark.summarize_results(
        [
            {
                "fixture": "docx",
                "attachmentClass": "office",
                "totalRows": 10,
                "forceErrorRows": 0,
                "cacheErrorRows": 0,
                "shardCacheReuseErrorRows": 0,
                "requestCount": 2,
                "arrowIpcBytes": 100,
                "cacheSpeedup": 4.0,
                "duplicateMissConverterCalls": 1,
                "artifactErrorCount": 0,
                "structureRows": 4,
                "structureOcrPageBlocks": 0,
                "structureOcrRegionBlocks": 0,
                "structureBboxBlocks": 0,
                "structureReadingOrderSorted": True,
                "structureOrderStable": True,
                "structureOrderMismatchCount": 0,
                "structureParityPassed": None,
                "structureParityErrorCount": 0,
                "metricsRows": 0,
                "metricsResultChars": 0,
                "metricsBboxCount": 0,
                "metricsRustSchedulerElapsedMs": 0.0,
                "documentTimingRows": 3,
                "documentTimingTotalElapsedMs": 18.0,
                "documentTimingOverheadMs": 2.0,
                "documentTimingPhaseElapsedMs": {
                    "doclingConvert": 12.0,
                    "total": 18.0,
                },
                "forceRefreshMs": 20.0,
                "cacheHitP95Ms": 2.0,
                "wallTimeMs": 3.0,
                "resourcesRows": 4,
                "artifactReports": [
                    {
                        "resourceTypeCounts": {
                            "document": 1,
                            "docling_json": 1,
                            "image": 1,
                            "table": 1,
                        },
                        "resourceStatusCounts": {"ok": 4},
                        "structureBlockTypeCounts": {
                            "document": 1,
                            "image": 1,
                            "table": 1,
                        },
                        "metricsStatusCounts": {},
                        "documentTimingStatusCounts": {"ok": 3},
                        "documentTimingPhaseElapsedMs": {
                            "doclingConvert": 12.0,
                            "total": 18.0,
                        },
                    }
                ],
            },
            {
                "fixture": "image-png",
                "attachmentClass": "image",
                "totalRows": 5,
                "forceErrorRows": 0,
                "cacheErrorRows": 0,
                "shardCacheReuseErrorRows": 0,
                "requestCount": 1,
                "arrowIpcBytes": 80,
                "cacheSpeedup": 2.0,
                "duplicateMissConverterCalls": 1,
                "artifactErrorCount": 0,
                "structureRows": 1,
                "structureOcrPageBlocks": 0,
                "structureOcrRegionBlocks": 0,
                "structureBboxBlocks": 0,
                "structureReadingOrderSorted": True,
                "structureOrderStable": True,
                "structureOrderMismatchCount": 0,
                "structureParityPassed": None,
                "structureParityErrorCount": 0,
                "metricsRows": 0,
                "metricsResultChars": 0,
                "metricsBboxCount": 0,
                "metricsRustSchedulerElapsedMs": 0.0,
                "documentTimingRows": 3,
                "documentTimingTotalElapsedMs": 45.0,
                "documentTimingOverheadMs": 5.0,
                "documentTimingPhaseElapsedMs": {
                    "doclingConvert": 40.0,
                    "total": 45.0,
                },
                "forceRefreshMs": 50.0,
                "cacheHitP95Ms": 5.0,
                "wallTimeMs": 6.0,
                "resourcesRows": 3,
                "artifactReports": [
                    {
                        "resourceTypeCounts": {
                            "document": 1,
                            "docling_json": 1,
                            "table": 1,
                        },
                        "resourceStatusCounts": {"ok": 3},
                        "structureBlockTypeCounts": {"document": 1, "table": 1},
                        "metricsStatusCounts": {},
                        "documentTimingStatusCounts": {"ok": 3},
                        "documentTimingPhaseElapsedMs": {
                            "doclingConvert": 40.0,
                            "total": 45.0,
                        },
                        "imageAttachmentAudit": {
                            "format": "png",
                            "widthPx": 640,
                            "heightPx": 480,
                            "pixelCount": 307200,
                            "dimensionSource": "png_ihdr",
                            "rustAccelerationCandidate": ("image_ocr_cache_candidate"),
                        },
                    }
                ],
            },
        ],
    )

    class_summary = {
        item["attachmentClass"]: item for item in summary["attachmentClassSummary"]
    }
    assert set(class_summary) == {"image", "office"}
    assert summary["imageAttachmentAuditCount"] == 1
    assert summary["imageKnownDimensionCount"] == 1
    assert summary["imageFormatCounts"] == {"png": 1}
    assert summary["imageDimensionSourceCounts"] == {"png_ihdr": 1}
    assert summary["imageAccelerationCandidates"] == {"image_ocr_cache_candidate": 1}
    assert summary["maxImageWidthPx"] == 640
    assert summary["maxImageHeightPx"] == 480
    assert summary["maxImagePixelCount"] == 307200
    assert class_summary["office"]["fixtureCount"] == 1
    assert class_summary["office"]["fixtures"] == ["docx"]
    assert (
        class_summary["office"]["precisionSpeedSummary"]["precisionGatePassed"] is True
    )
    assert class_summary["office"]["precisionSpeedSummary"]["maxForceRefreshMs"] == 20.0
    assert class_summary["office"]["resourcesRows"] == 4
    assert class_summary["office"]["resourceTypeCounts"] == {
        "docling_json": 1,
        "document": 1,
        "image": 1,
        "table": 1,
    }
    assert class_summary["office"]["structureBlockTypeCounts"] == {
        "document": 1,
        "image": 1,
        "table": 1,
    }
    assert class_summary["office"]["slowestForceFixture"] == {
        "fixture": "docx",
        "latencyMs": 20.0,
    }
    assert class_summary["office"]["documentTimingTotalElapsedMs"] == 18.0
    assert class_summary["office"]["documentTimingOverheadMs"] == 2.0
    assert class_summary["office"]["documentTimingStatusCounts"] == {"ok": 3}
    assert class_summary["office"]["precisionSpeedSummary"][
        "maxDoclingConvertShare"
    ] == pytest.approx(12.0 / 18.0)
    assert class_summary["office"]["precisionSpeedSummary"][
        "maxDocumentTimingOverheadShare"
    ] == pytest.approx(0.1)
    assert class_summary["image"]["structureRows"] == 1
    assert class_summary["image"]["resourceTypeCounts"]["table"] == 1
    assert class_summary["image"]["imageAttachmentAuditCount"] == 1
    assert class_summary["image"]["imageKnownDimensionCount"] == 1
    assert class_summary["image"]["imageFormatCounts"] == {"png": 1}
    assert class_summary["image"]["imageDimensionSourceCounts"] == {"png_ihdr": 1}
    assert class_summary["image"]["imageAccelerationCandidates"] == {
        "image_ocr_cache_candidate": 1
    }
    assert class_summary["image"]["maxImageWidthPx"] == 640
    assert class_summary["image"]["maxImageHeightPx"] == 480
    assert class_summary["image"]["maxImagePixelCount"] == 307200
    assert class_summary["image"]["slowestCacheP95Fixture"] == {
        "fixture": "image-png",
        "latencyMs": 5.0,
    }
    assert class_summary["image"]["slowestTimingOverheadFixture"] == {
        "fixture": "image-png",
        "latencyMs": 5.0,
    }
    assert class_summary["image"]["precisionSpeedSummary"]["maxCacheHitP95Ms"] == 5.0
    assert (
        class_summary["image"]["precisionSpeedSummary"]["maxDocumentTimingOverheadMs"]
        == 5.0
    )
    assert class_summary["image"]["documentTimingPhaseElapsedMs"] == {
        "doclingConvert": 40.0,
        "total": 45.0,
    }
    assert class_summary["image"]["precisionSpeedSummary"]["maxDoclingConvertMs"] == (
        40.0
    )
    assert class_summary["image"]["precisionSpeedSummary"][
        "maxDoclingConvertShare"
    ] == pytest.approx(40.0 / 45.0)


def test_summarize_ocr_shard_cache_reports_root_files_and_limits(
    monkeypatch, tmp_path
) -> None:
    benchmark = _load_benchmark_module()
    cache_root = tmp_path / "ocr-shards"
    (cache_root / "aa").mkdir(parents=True)
    (cache_root / "aa" / "one.arrow").write_bytes(b"123")
    (cache_root / "bb").mkdir()
    (cache_root / "bb" / "two.arrow").write_bytes(b"4567")
    monkeypatch.setenv("WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT", str(cache_root))
    monkeypatch.setenv("WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_BYTES", "100")
    monkeypatch.setenv("WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_ENTRIES", "10")

    summary = benchmark.summarize_ocr_shard_cache()

    assert summary["root"] == str(cache_root.resolve())
    assert summary["fileCount"] == 2
    assert summary["totalBytes"] == 7
    assert summary["maxBytes"] == 100
    assert summary["maxEntries"] == 10
