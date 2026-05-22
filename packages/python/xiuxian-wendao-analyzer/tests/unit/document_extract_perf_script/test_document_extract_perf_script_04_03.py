"""document_extract_perf_script test slice 4."""

from __future__ import annotations

from .support import (
    _load_benchmark_module,
)


def test_hosted_vlm_ocr_process_env_maps_cli_args(monkeypatch) -> None:
    benchmark = _load_benchmark_module()
    monkeypatch.delenv("OPENROUTE_API_KEY", raising=False)
    args = benchmark.argparse.Namespace(
        hosted_vlm_ocr_provider="openrouter",
        hosted_vlm_ocr_base_url="http://127.0.0.1:8000/v1",
        hosted_vlm_ocr_model="community/hosted-vlm-awq",
        hosted_vlm_ocr_prompt="<image>\nmarkdown",
        hosted_vlm_ocr_max_tokens=4096,
        hosted_vlm_ocr_region_max_tokens=2048,
        hosted_vlm_ocr_region_prompt_mode="compact-region-markdown",
        hosted_vlm_ocr_region_composite_size=2,
        hosted_vlm_ocr_region_composite_mode="adaptive-small-region",
        hosted_vlm_ocr_region_composite_max_source_pixels=30000,
        hosted_vlm_ocr_region_composite_max_image_bytes=65536,
        hosted_vlm_ocr_region_atlas_mode="same-page-json",
        hosted_vlm_ocr_scaffold_mode="region-table-json",
        hosted_vlm_ocr_image_optimization="region-whitespace-trim",
        hosted_vlm_ocr_timeout_seconds=120.0,
        hosted_vlm_ocr_request_concurrency=4,
        hosted_vlm_ocr_speculative_retry_delay_seconds=1.5,
        hosted_vlm_ocr_speculative_retry_min_source_pixels=1_000_000,
        hosted_vlm_ocr_speculative_retry_min_image_bytes=200_000,
        hosted_vlm_ocr_page_window_size=3,
        hosted_vlm_ocr_openrouter_provider_json='{"sort":{"by":"latency"}}',
        pdf_ocr_prewarm_profile=["docling-fast-text-ocr"],
        pdf_ocr_prewarm_source_path="tests/fixtures/document.pdf",
        pdf_ocr_prewarm_page_index=5,
        pdf_ocr_prewarm_page_indices="5,11",
        pdf_ocr_backend_text_page_fallback="compatible-page",
        pdf_ocr_backend_text_empty_page="verified-empty",
        openrouter_model="openrouter/vision-ocr",
        openrouter_http_referer="https://wendao.local",
        openrouter_title="Wendao OCR Benchmark",
    )

    assert benchmark.hosted_vlm_ocr_process_env(args) == {
        "WENDAO_HOSTED_VLM_OCR_PROVIDER": "openrouter",
        "WENDAO_HOSTED_VLM_OCR_BASE_URL": "http://127.0.0.1:8000/v1",
        "WENDAO_HOSTED_VLM_OCR_MODEL": "community/hosted-vlm-awq",
        "WENDAO_HOSTED_VLM_OCR_PROMPT": "<image>\nmarkdown",
        "WENDAO_HOSTED_VLM_OCR_MAX_TOKENS": "4096",
        "WENDAO_HOSTED_VLM_OCR_REGION_MAX_TOKENS": "2048",
        "WENDAO_HOSTED_VLM_OCR_REGION_PROMPT_MODE": "compact-region-markdown",
        "WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE": "2",
        "WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_MODE": "adaptive-small-region",
        "WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_MAX_SOURCE_PIXELS": "30000",
        "WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_MAX_IMAGE_BYTES": "65536",
        "WENDAO_HOSTED_VLM_OCR_REGION_ATLAS_MODE": "same-page-json",
        "WENDAO_HOSTED_VLM_OCR_SCAFFOLD_MODE": "region-table-json",
        "WENDAO_HOSTED_VLM_OCR_IMAGE_OPTIMIZATION": "region-whitespace-trim",
        "WENDAO_HOSTED_VLM_OCR_TIMEOUT_SECONDS": "120.0",
        "WENDAO_HOSTED_VLM_OCR_REQUEST_CONCURRENCY": "4",
        "WENDAO_HOSTED_VLM_OCR_SPECULATIVE_RETRY_DELAY_SECONDS": "1.5",
        "WENDAO_HOSTED_VLM_OCR_SPECULATIVE_RETRY_MIN_SOURCE_PIXELS": "1000000",
        "WENDAO_HOSTED_VLM_OCR_SPECULATIVE_RETRY_MIN_IMAGE_BYTES": "200000",
        "WENDAO_HOSTED_VLM_OCR_PAGE_WINDOW_SIZE": "3",
        "WENDAO_HOSTED_VLM_OCR_OPENROUTER_PROVIDER_JSON": '{"sort":{"by":"latency"}}',
        "WENDAO_PDF_OCR_PREWARM_PROFILES": "docling-fast-text-ocr",
        "WENDAO_PDF_OCR_PREWARM_SOURCE_PATH": "tests/fixtures/document.pdf",
        "WENDAO_PDF_OCR_PREWARM_PAGE_INDICES": "5,11",
        "WENDAO_PDF_OCR_BACKEND_TEXT_PAGE_FALLBACK": "compatible-page",
        "WENDAO_PDF_OCR_BACKEND_TEXT_EMPTY_PAGE": "verified-empty",
        "WENDAO_OPENROUTER_MODEL": "openrouter/vision-ocr",
        "WENDAO_OPENROUTER_HTTP_REFERER": "https://wendao.local",
        "WENDAO_OPENROUTER_TITLE": "Wendao OCR Benchmark",
    }


def test_hosted_vlm_ocr_process_env_defaults_openrouter_smoke_model(
    monkeypatch,
) -> None:
    benchmark = _load_benchmark_module()
    monkeypatch.delenv("OPENROUTE_API_KEY", raising=False)
    args = benchmark.argparse.Namespace(
        hosted_vlm_ocr_provider="openrouter",
        hosted_vlm_ocr_base_url=None,
        hosted_vlm_ocr_model=None,
        hosted_vlm_ocr_prompt=None,
        hosted_vlm_ocr_max_tokens=None,
        hosted_vlm_ocr_region_max_tokens=None,
        hosted_vlm_ocr_region_composite_size=None,
        hosted_vlm_ocr_region_atlas_mode=None,
        hosted_vlm_ocr_scaffold_mode=None,
        hosted_vlm_ocr_image_optimization=None,
        hosted_vlm_ocr_timeout_seconds=None,
        hosted_vlm_ocr_request_concurrency=None,
        hosted_vlm_ocr_speculative_retry_delay_seconds=None,
        hosted_vlm_ocr_page_window_size=None,
        pdf_ocr_backend_text_empty_page="disabled",
        openrouter_model=None,
        openrouter_http_referer=None,
        openrouter_title=None,
    )

    assert benchmark.hosted_vlm_ocr_process_env(args) == {
        "WENDAO_HOSTED_VLM_OCR_PROVIDER": "openrouter",
        "WENDAO_OPENROUTER_MODEL": "baidu/qianfan-ocr-fast",
    }


def test_hosted_vlm_ocr_process_env_forwards_legacy_openroute_key(monkeypatch) -> None:
    benchmark = _load_benchmark_module()
    for key in (
        "WENDAO_OPENROUTER_API_KEY",
        "OPENROUTER_API_KEY",
        "WENDAO_HOSTED_VLM_OCR_API_KEY",
        "OPENROUTE_API_KEY",
    ):
        monkeypatch.delenv(key, raising=False)
    monkeypatch.setenv("OPENROUTE_API_KEY", '"or-legacy-key"')
    args = benchmark.argparse.Namespace(
        hosted_vlm_ocr_provider="openrouter",
        hosted_vlm_ocr_base_url=None,
        hosted_vlm_ocr_model=None,
        hosted_vlm_ocr_prompt=None,
        hosted_vlm_ocr_max_tokens=None,
        hosted_vlm_ocr_region_max_tokens=None,
        hosted_vlm_ocr_region_composite_size=None,
        hosted_vlm_ocr_region_atlas_mode=None,
        hosted_vlm_ocr_scaffold_mode=None,
        hosted_vlm_ocr_image_optimization=None,
        hosted_vlm_ocr_timeout_seconds=None,
        hosted_vlm_ocr_request_concurrency=None,
        hosted_vlm_ocr_speculative_retry_delay_seconds=None,
        hosted_vlm_ocr_page_window_size=None,
        pdf_ocr_backend_text_empty_page="disabled",
        openrouter_model=None,
        openrouter_http_referer=None,
        openrouter_title=None,
    )

    env = benchmark.hosted_vlm_ocr_process_env(args)

    assert env["OPENROUTER_API_KEY"] == "or-legacy-key"
