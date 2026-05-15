"""document_extract_perf_script test slice 9."""

from __future__ import annotations

from .support import (
    Path,
    _load_benchmark_module,
)


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
