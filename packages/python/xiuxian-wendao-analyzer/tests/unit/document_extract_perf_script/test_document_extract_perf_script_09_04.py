"""document_extract_perf_script test slice 9."""

from __future__ import annotations

from .support import (
    Path,
    _load_benchmark_module,
)


def test_start_rust_provider_failed_page_recovery_enables_pdf_render_feature(
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
        rust_pdf_backend_text_topup="profile",
        rust_pdf_failed_page_recovery="hosted-vlm-page",
        rust_pdf_ocr_profile_planner="fast-risk-window",
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

    command, kwargs = calls[0]
    assert "document-extract-pdf-render" in command[6].split(",")
    assert (
        kwargs["env"]["WENDAO_DOCUMENT_EXTRACT_PDF_FAILED_PAGE_RECOVERY"]
        == "hosted-vlm-page"
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
