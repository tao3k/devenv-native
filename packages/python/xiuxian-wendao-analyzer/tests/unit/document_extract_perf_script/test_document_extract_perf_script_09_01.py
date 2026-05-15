"""document_extract_perf_script test slice 9."""

from __future__ import annotations

from .support import (
    Path,
    _load_benchmark_module,
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
        rust_pdf_docling_page_range_chunk_plan="1:3,4:4,5:6,7:9",
        rust_pdf_docling_page_range_profile="structure-text",
        rust_pdf_docling_page_range_hedge_delay_ms=7000,
        rust_pdf_docling_page_range_structure_cost_budget=2400,
        rust_pdf_docling_text_shortcut_promotion="disabled",
        pdf_ocr_backend_text_empty_page="verified-empty",
        rust_pdf_local_backend_text="rust-lopdf",
        rust_pdf_local_backend_text_empty="fail-fast",
        rust_pdf_local_fast_text="rust-lopdf",
        rust_pdf_fast_text_source_range_split="single-page",
        rust_pdf_fast_text_endpoint_affinity="single-page-first",
        rust_pdf_backend_text_topup="hosted-vlm",
        rust_pdf_failed_page_recovery="hosted-vlm-page",
        rust_pdf_ocr_profile_planner="fast-risk-window",
        rust_pdf_hosted_vlm_render_dpi=360,
        rust_pdf_ocr_region_context_ratio=0.2,
        rust_pdf_hosted_vlm_region_planner="profile-risk-window",
        rust_pdf_hosted_vlm_region_pipeline="render-dispatch",
        rust_pdf_hosted_vlm_region_render_ahead=3,
        rust_pdf_hosted_vlm_region_render_chunk="region-seed-page",
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
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_PLAN"] == (
        "1:3,4:4,5:6,7:9"
    )
    assert (
        env["WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_PROFILE"]
        == "structure-text"
    )
    assert (
        env["WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_HEDGE_DELAY_MS"] == "7000"
    )
    assert (
        env["WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_STRUCTURE_COST_BUDGET"]
        == "2400"
    )
    assert (
        env["WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_TEXT_SHORTCUT_PROMOTION"] == "disabled"
    )
    assert (
        env["WENDAO_DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_EMPTY_PAGE"] == "verified-empty"
    )
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_LOCAL_BACKEND_TEXT"] == "rust-lopdf"
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_LOCAL_BACKEND_TEXT_EMPTY"] == "fail-fast"
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_LOCAL_FAST_TEXT"] == "rust-lopdf"
    assert (
        env["WENDAO_DOCUMENT_EXTRACT_PDF_FAST_TEXT_SOURCE_RANGE_SPLIT"] == "single-page"
    )
    assert (
        env["WENDAO_DOCUMENT_EXTRACT_PDF_FAST_TEXT_ENDPOINT_AFFINITY"]
        == "single-page-first"
    )
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_TOPUP"] == "hosted-vlm"
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_FAILED_PAGE_RECOVERY"] == "hosted-vlm-page"
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
    assert (
        env["WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_CHUNK"]
        == "region-seed-page"
    )
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
