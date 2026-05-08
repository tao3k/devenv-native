"""document_extract_perf_script test slice 6."""

from __future__ import annotations

from .support import (
    Path,
    _load_benchmark_module,
    pytest,
)


@pytest.mark.parametrize(
    ("flight_mode", "artifact_registry_reuse_probe", "expected"),
    [
        ("sync", False, False),
        ("sync", True, True),
        ("async", False, True),
        ("hybrid-page-ocr", False, True),
    ],
)
def test_artifact_registry_reuse_probe_routes_through_rust_provider(
    flight_mode: str,
    artifact_registry_reuse_probe: bool,
    expected: bool,
) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        flight_mode=flight_mode,
        artifact_registry_reuse_probe=artifact_registry_reuse_probe,
    )

    assert benchmark.should_start_local_rust_provider(args) is expected


def test_report_payload_exposes_top_level_precision_speed_summary(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        real_docling=False,
        benchmark_host="127.0.0.1",
        benchmark_port=50052,
        rust_rest_endpoint=None,
        iterations=1,
        concurrency=1,
        flight_mode="sync",
        wait_ms=0,
        pdf_ocr_worker="skip",
        pdf_ocr_workers="auto",
        pdf_ocr_prewarm_profile=["docling-fast-text-ocr"],
        local_python_ocr_endpoint_count=1,
        rust_pdf_ocr_workers=None,
        rust_pdf_ocr_source_range_workers=None,
        rust_pdf_local_backend_text="rust-lopdf",
        rust_pdf_local_backend_text_empty="fail-fast",
        rust_pdf_local_fast_text="rust-lopdf",
        rust_pdf_fast_text_source_range_split="single-page",
        rust_pdf_backend_text_topup="disabled",
        rust_document_extract_endpoint=[],
        rust_pdf_ocr_endpoint=[],
        structure_baseline_root=None,
        shard_cache_reuse_probe=False,
        artifact_registry_reuse_probe=True,
        ocr_shard_cache_root=tmp_path / "ocr-shards",
        hosted_vlm_ocr_image_optimization="region-whitespace-trim",
    )
    result = {
        "fixture": "markdown",
        "attachmentClass": "structured_text",
        "forceRefreshMs": 10.0,
        "artifactRegistryReuseForceMs": 4.0,
        "cacheHitP95Ms": 2.0,
        "wallTimeMs": 3.0,
        "cacheSpeedup": 5.0,
        "forceErrorRows": 0,
        "artifactRegistryReuseErrorRows": 0,
        "cacheErrorRows": 0,
        "totalRows": 2,
        "requestCount": 1,
        "arrowIpcBytes": 64,
        "duplicateMissConverterCalls": None,
        "structureRows": 1,
        "structureReadingOrderSorted": True,
        "structureOrderStable": True,
        "structureOrderMismatchCount": 0,
    }

    payload = benchmark.build_report_payload(
        args,
        real_fixture_root=None,
        results=[result],
        distinct_miss_report=None,
        structure_baseline_report=None,
        ocr_shard_cache_summary={
            "root": str(tmp_path),
            "fileCount": 0,
            "totalBytes": 0,
        },
    )

    assert (
        payload["precisionSpeedSummary"] == payload["summary"]["precisionSpeedSummary"]
    )
    assert payload["precisionSpeedSummary"]["maxArtifactRegistryReuseForceMs"] == 4.0
    assert payload["hostedVlmOcr"]["imageOptimizationMode"] == "region-whitespace-trim"
    assert payload["pdfOcrPrewarmProfiles"] == ["docling-fast-text-ocr"]
    assert payload["pdfOcrPrewarmSourcePath"] is None
    assert payload["pdfOcrPrewarmPageIndex"] is None
    assert payload["pdfOcrPrewarmPageIndices"] is None
    assert payload["pdfOcrPrewarmEndpointCount"] is None
    assert payload["rustPdfLocalBackendText"] == "rust-lopdf"
    assert payload["rustPdfLocalBackendTextEmpty"] == "fail-fast"
    assert payload["rustPdfLocalFastText"] == "rust-lopdf"
    assert payload["rustPdfFastTextSourceRangeSplit"] == "single-page"
    assert payload["rustPdfFastTextEndpointAffinity"] == "disabled"
    assert payload["rustPdfBackendTextTopup"] == "disabled"


def test_run_structure_baseline_probe_generates_sync_fixture_baselines(
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
        flight_mode: str,
        include_structure_baseline_root: Path | None,
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
                "flight_mode": flight_mode,
                "include_structure_baseline_root": include_structure_baseline_root,
            }
        )
        return {
            "errorRowCount": 0,
            "artifactReports": [
                {
                    "resourcesArrowExists": True,
                    "resourcesRowCount": 2,
                    "structureArrowExists": True,
                    "structureRowCount": 2,
                    "structureReadingOrderSorted": True,
                }
            ],
        }

    monkeypatch.setattr(benchmark, "run_cargo_perf_test", fake_run_cargo_perf_test)
    args = benchmark.argparse.Namespace(
        generate_structure_baselines=True,
        fail_on_error_rows=True,
    )
    baseline_root = tmp_path / "baselines"

    report = benchmark.run_structure_baseline_probe(
        args,
        {
            "pdf": tmp_path / "sample.pdf",
            "image": tmp_path / "sample.png",
        },
        baseline_root,
    )

    assert report["enabled"] is True
    assert report["root"] == str(baseline_root)
    assert report["fixtureCount"] == 2
    assert report["totalStructureRows"] == 4
    assert report["allStructureReadingOrderSorted"] is True
    assert [call["output_dir"].name for call in calls] == ["pdf", "image"]
    assert all(call["flight_mode"] == "sync" for call in calls)
    assert all(call["force"] is True for call in calls)
    assert all(call["include_structure_baseline_root"] is False for call in calls)


def test_structure_baseline_root_defaults_to_report_dir_when_generating(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        generate_structure_baselines=True,
        structure_baseline_root=None,
    )

    assert (
        benchmark.resolve_structure_baseline_root(args, tmp_path)
        == (tmp_path / "structure-baselines").resolve()
    )


def test_pdf_render_shard_audit_command_adds_feature_and_fixture_manifest(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        cargo_features=(
            "performance,studio,zhenfa-router,duckdb,document-extract-attachment-audit"
        ),
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        require_pdfium=False,
        pdf_render_selection="all-pages",
        pdf_render_region=[],
    )

    command, env = benchmark.build_pdf_render_shard_audit_command(
        args,
        {"pdf": tmp_path / "sample.pdf"},
        tmp_path / "reports",
    )

    assert command[:4] == ["cargo", "test", "-p", "xiuxian-wendao-studio"]
    assert command[command.index("--test") + 1] == "performance_test"
    assert command[command.index("--features") + 1] == (
        "performance,studio,zhenfa-router,duckdb,document-extract-pdf-render"
    )
    assert command[-4:] == [
        "pdf_render_page_render_shard_manifest",
        "--",
        "--ignored",
        "--nocapture",
    ]
    inputs = benchmark.json.loads(env["WENDAO_PDF_RENDER_SHARD_INPUTS_JSON"])
    assert inputs == [{"name": "pdf", "source": str(tmp_path / "sample.pdf")}]
    assert env["WENDAO_PDF_RENDER_SHARD_REPORT_DIR"] == str(tmp_path / "reports")
    assert env["WENDAO_PDF_RENDER_SELECTION"] == "all_pages"


def test_pdf_render_shard_audit_can_pin_pdfium_runtime_path(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    pdfium_library = tmp_path / "libpdfium.dylib"
    pdfium_library.write_bytes(b"pdfium")
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        cargo_features="performance",
        pdfium_library_path=pdfium_library,
        prepare_pdfium_runtime=False,
        require_pdfium=True,
        pdf_render_selection="shard-fallback-pages",
        pdf_render_region=[],
    )

    _command, env = benchmark.build_pdf_render_shard_audit_command(
        args,
        {"pdf": tmp_path / "sample.pdf"},
        tmp_path / "reports",
    )

    assert env["WENDAO_PDFIUM_LIBRARY_PATH"] == str(pdfium_library.resolve())
    assert env["WENDAO_PDF_RENDER_REQUIRE_PDFIUM"] == "1"
    assert env["WENDAO_PDF_RENDER_SELECTION"] == "shard_fallback_pages"
