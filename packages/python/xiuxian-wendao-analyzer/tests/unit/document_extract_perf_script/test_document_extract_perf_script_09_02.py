"""document_extract_perf_script test slice 9."""

from __future__ import annotations

from .support import (
    Path,
    _load_benchmark_module,
    pytest,
)


def test_python_worker_env_forwards_document_converter_cache() -> None:
    benchmark = _load_benchmark_module()

    disabled_env = benchmark.hosted_vlm_ocr_process_env(
        benchmark.argparse.Namespace(document_extract_converter_cache="disabled")
    )
    profile_env = benchmark.hosted_vlm_ocr_process_env(
        benchmark.argparse.Namespace(document_extract_converter_cache="profile")
    )

    assert "WENDAO_DOCUMENT_EXTRACT_CONVERTER_CACHE" not in disabled_env
    assert profile_env["WENDAO_DOCUMENT_EXTRACT_CONVERTER_CACHE"] == "profile"


def test_python_worker_env_sets_auto_docling_structure_threads() -> None:
    benchmark = _load_benchmark_module()

    env = benchmark.hosted_vlm_ocr_process_env(
        benchmark.argparse.Namespace(
            document_extract_full_threads="auto",
            real_docling=True,
            flight_mode="hybrid-page-ocr",
            rust_pdf_ocr_profile_planner="docling-structure-recovery",
        )
    )

    assert env["WENDAO_DOCUMENT_EXTRACT_FULL_THREADS"] == "1"


def test_python_worker_env_forwards_explicit_docling_threads() -> None:
    benchmark = _load_benchmark_module()

    env = benchmark.hosted_vlm_ocr_process_env(
        benchmark.argparse.Namespace(document_extract_full_threads="3")
    )

    assert env["WENDAO_DOCUMENT_EXTRACT_FULL_THREADS"] == "3"


def test_python_worker_env_forwards_fast_text_source_converter() -> None:
    benchmark = _load_benchmark_module()

    default_env = benchmark.hosted_vlm_ocr_process_env(
        benchmark.argparse.Namespace(pdf_ocr_fast_text_source_converter="default")
    )
    backend_table_env = benchmark.hosted_vlm_ocr_process_env(
        benchmark.argparse.Namespace(pdf_ocr_fast_text_source_converter="backend-table")
    )

    assert "WENDAO_PDF_OCR_FAST_TEXT_SOURCE_CONVERTER" not in default_env
    assert (
        backend_table_env["WENDAO_PDF_OCR_FAST_TEXT_SOURCE_CONVERTER"]
        == "backend-table"
    )


def test_python_worker_env_forwards_document_extract_prewarm() -> None:
    benchmark = _load_benchmark_module()

    env = benchmark.hosted_vlm_ocr_process_env(
        benchmark.argparse.Namespace(
            document_extract_prewarm_source_path="tests/fixtures/document.pdf",
            document_extract_prewarm_page_ranges="1:1,4:6",
        )
    )

    assert (
        env["WENDAO_DOCUMENT_EXTRACT_PREWARM_SOURCE_PATH"]
        == "tests/fixtures/document.pdf"
    )
    assert env["WENDAO_DOCUMENT_EXTRACT_PREWARM_PAGE_RANGES"] == "1:1,4:6"


def test_python_worker_env_can_reuse_rust_page_range_chunk_plan() -> None:
    benchmark = _load_benchmark_module()

    env = benchmark.hosted_vlm_ocr_process_env(
        benchmark.argparse.Namespace(
            document_extract_prewarm_source_path="tests/fixtures/document.pdf",
            document_extract_prewarm_page_ranges="rust-page-range-chunk-plan",
            rust_pdf_docling_page_range_chunk_plan="1:3,4:4,5:6,7:9",
        )
    )

    assert (
        env["WENDAO_DOCUMENT_EXTRACT_PREWARM_SOURCE_PATH"]
        == "tests/fixtures/document.pdf"
    )
    assert env["WENDAO_DOCUMENT_EXTRACT_PREWARM_PAGE_RANGES"] == "1:3,4:4,5:6,7:9"


def test_python_worker_env_requires_rust_page_range_chunk_plan_for_reuse() -> None:
    benchmark = _load_benchmark_module()

    with pytest.raises(SystemExit, match="rust-page-range-chunk-plan requires"):
        benchmark.hosted_vlm_ocr_process_env(
            benchmark.argparse.Namespace(
                document_extract_prewarm_page_ranges="rust-page-range-chunk-plan",
                rust_pdf_docling_page_range_chunk_plan=None,
            )
        )


def test_start_gateway_uses_prebuilt_wendao_binary(
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
    binary = tmp_path / "bin" / "wendao"
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        rust_provider_bin=binary,
        gateway_features="cli-bin-support,zhenfa-router,duckdb",
        hybrid_pdf_render_selection="shard-fallback-pages",
        pdf_render_region=[],
        benchmark_fixtures={},
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        rust_pdf_ocr_workers=None,
        rust_pdf_ocr_source_range_workers=None,
        pdf_ocr_backend_text_empty_page="disabled",
        rust_pdf_local_backend_text="disabled",
        rust_pdf_local_backend_text_empty="dispatch-python",
        rust_pdf_local_fast_text="disabled",
        rust_pdf_fast_text_source_range_split="disabled",
        rust_pdf_fast_text_endpoint_affinity="disabled",
        rust_pdf_backend_text_topup="profile",
        rust_pdf_failed_page_recovery="disabled",
        rust_pdf_ocr_profile_planner="disabled",
        rust_pdf_hosted_vlm_render_dpi=None,
        rust_pdf_ocr_region_context_ratio=None,
        rust_pdf_hosted_vlm_region_planner="disabled",
        rust_pdf_hosted_vlm_region_pipeline="disabled",
        rust_pdf_hosted_vlm_region_render_ahead=None,
        rust_pdf_hosted_vlm_region_render_chunk="page",
        hosted_vlm_ocr_region_composite_size=None,
        hosted_vlm_ocr_scaffold_mode="disabled",
        rust_pdf_ocr_endpoint=[],
        rust_document_extract_endpoint=[],
    )

    benchmark.start_gateway_server(
        args,
        gateway_port=55001,
        python_host="127.0.0.1",
        python_port=50051,
        valkey_url="redis://127.0.0.1:6379",
        temp_root=tmp_path,
    )

    command, kwargs = calls[0]
    assert command[:2] == [str(binary), "--conf"]
    assert "cargo" not in command
    assert command[-3:] == ["start", "--port", "55001"]
    assert kwargs["env"]["WENDAO_DOCUMENT_EXTRACT_ENDPOINT"] == "http://127.0.0.1:50051"
