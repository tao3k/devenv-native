"""document_extract_perf_script test slice 4."""

from __future__ import annotations

import json

from .support import (
    Path,
    _load_benchmark_module,
)


def test_fixture_server_code_can_record_converter_count(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()

    code = benchmark.fixture_server_code(
        "127.0.0.1",
        50051,
        tmp_path / "count.txt",
        "fixture",
    )

    assert "CONVERTER_COUNT_PATH" in code
    assert "self.calls += 1" in code
    assert "write_text(str(self.calls)" in code
    assert "class FixtureOcrWorker" in code


def test_real_docling_server_code_can_record_converter_count(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()

    code = benchmark.real_docling_server_code(
        "127.0.0.1",
        50051,
        tmp_path / "docling-fixtures",
        False,
        tmp_path / "count.txt",
        "docling",
    )

    assert "class CountingConverter" in code
    assert "return CountingConverter(converter)" in code
    assert "def make_converter(ocr_profile=None)" in code
    assert "PDF_OCR_FAST_TEXT_PROFILE" in code
    assert "AcceleratorOptions" in code
    assert "WENDAO_PDF_OCR_FAST_TEXT_THREADS" in code
    assert "TableFormerMode.FAST" in code
    assert "PDF_OCR_DOCLING_VLM_DEEPSEEK_OCR_PROFILE" in code
    assert 'VlmConvertOptions.from_preset("deepseek_ocr")' in code
    assert "DoclingPdfOcrShardWorker(" in code
    assert "converter_factory=make_converter" in code
    assert "max_workers='auto'" in code
    assert "write_text(str(self.calls)" in code


def test_python_worker_command_adds_workspace_package_and_extras() -> None:
    benchmark = _load_benchmark_module()

    command = benchmark.python_worker_command(
        "print('worker')",
        uv_package="xiuxian-wendao-analyzer",
        uv_extras=["documents", "documents-audio"],
    )

    assert command == [
        "uv",
        "run",
        "--package",
        "xiuxian-wendao-analyzer",
        "--extra",
        "documents",
        "--extra",
        "documents-audio",
        "python",
        "-c",
        "print('worker')",
    ]


def test_start_server_pool_starts_counted_local_ocr_endpoints(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    calls = []
    extra_ports = iter([52052, 52053])

    class FakeProcess:
        pass

    def fake_pick_free_port(host: str) -> int:
        assert host == "127.0.0.1"
        return next(extra_ports)

    def fake_start_server(host: str, port: int, **kwargs):
        calls.append((host, port, kwargs))
        return FakeProcess()

    monkeypatch.setattr(benchmark._workers, "pick_free_port", fake_pick_free_port)
    monkeypatch.setattr(benchmark._workers, "start_server", fake_start_server)

    workers = benchmark.start_server_pool(
        "127.0.0.1",
        52051,
        endpoint_count=3,
        real_docling=False,
        real_fixture_root=None,
        include_audio=False,
        converter_count_path=tmp_path / "counts",
        pdf_ocr_worker="fixture",
        pdf_ocr_workers="auto",
        python_uv_package="xiuxian-wendao-analyzer",
        python_uv_extras=[],
        hosted_vlm_ocr_env={
            "WENDAO_HOSTED_VLM_OCR_MODEL": "community/hosted-vlm-awq",
            "WENDAO_PDF_OCR_PREWARM_PROFILES": "docling-fast-text-ocr",
            "WENDAO_PDF_OCR_PREWARM_SOURCE_PATH": "tests/fixtures/document.pdf",
            "WENDAO_PDF_OCR_PREWARM_PAGE_INDICES": "5,11",
            "WENDAO_PDF_OCR_PREWARM_PAGE_INDEX": "5",
        },
        pdf_ocr_prewarm_endpoint_count=1,
        log_dir=tmp_path / "logs",
    )

    assert [worker.port for worker in workers] == [52051, 52052, 52053]
    assert [worker.endpoint_url for worker in workers] == [
        "http://127.0.0.1:52051",
        "http://127.0.0.1:52052",
        "http://127.0.0.1:52053",
    ]
    assert [call[2]["process_name"] for call in calls] == [
        "python-worker-0",
        "python-worker-1",
        "python-worker-2",
    ]
    assert [call[2]["converter_count_path"].name for call in calls] == [
        "python-worker-0.txt",
        "python-worker-1.txt",
        "python-worker-2.txt",
    ]
    assert [
        call[2]["hosted_vlm_ocr_env"]["WENDAO_HOSTED_VLM_OCR_MODEL"] for call in calls
    ] == [
        "community/hosted-vlm-awq",
        "community/hosted-vlm-awq",
        "community/hosted-vlm-awq",
    ]
    assert [
        call[2]["hosted_vlm_ocr_env"].get("WENDAO_PDF_OCR_PREWARM_PROFILES")
        for call in calls
    ] == ["docling-fast-text-ocr", None, None]
    assert [
        call[2]["hosted_vlm_ocr_env"].get("WENDAO_PDF_OCR_PREWARM_SOURCE_PATH")
        for call in calls
    ] == ["tests/fixtures/document.pdf", None, None]
    assert [
        call[2]["hosted_vlm_ocr_env"].get("WENDAO_PDF_OCR_PREWARM_PAGE_INDEX")
        for call in calls
    ] == ["5", None, None]
    assert [
        call[2]["hosted_vlm_ocr_env"].get("WENDAO_PDF_OCR_PREWARM_PAGE_INDICES")
        for call in calls
    ] == ["5,11", None, None]
    assert [
        Path(call[2]["hosted_vlm_ocr_env"]["WENDAO_HOSTED_VLM_OCR_TRACE_PATH"]).name
        for call in calls
    ] == [
        "python-worker-0.hosted-vlm-ocr.jsonl",
        "python-worker-1.hosted-vlm-ocr.jsonl",
        "python-worker-2.hosted-vlm-ocr.jsonl",
    ]


def test_hosted_vlm_ocr_process_env_maps_cli_args() -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        hosted_vlm_ocr_provider="openrouter",
        hosted_vlm_ocr_base_url="http://127.0.0.1:8000/v1",
        hosted_vlm_ocr_model="community/hosted-vlm-awq",
        hosted_vlm_ocr_prompt="<image>\nmarkdown",
        hosted_vlm_ocr_max_tokens=4096,
        hosted_vlm_ocr_region_max_tokens=2048,
        hosted_vlm_ocr_region_composite_size=2,
        hosted_vlm_ocr_region_atlas_mode="same-page-json",
        hosted_vlm_ocr_scaffold_mode="region-table-json",
        hosted_vlm_ocr_image_optimization="region-whitespace-trim",
        hosted_vlm_ocr_timeout_seconds=120.0,
        hosted_vlm_ocr_request_concurrency=4,
        hosted_vlm_ocr_speculative_retry_delay_seconds=1.5,
        hosted_vlm_ocr_page_window_size=3,
        pdf_ocr_prewarm_profile=["docling-fast-text-ocr"],
        pdf_ocr_prewarm_source_path="tests/fixtures/document.pdf",
        pdf_ocr_prewarm_page_index=5,
        pdf_ocr_prewarm_page_indices="5,11",
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
        "WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE": "2",
        "WENDAO_HOSTED_VLM_OCR_REGION_ATLAS_MODE": "same-page-json",
        "WENDAO_HOSTED_VLM_OCR_SCAFFOLD_MODE": "region-table-json",
        "WENDAO_HOSTED_VLM_OCR_IMAGE_OPTIMIZATION": "region-whitespace-trim",
        "WENDAO_HOSTED_VLM_OCR_TIMEOUT_SECONDS": "120.0",
        "WENDAO_HOSTED_VLM_OCR_REQUEST_CONCURRENCY": "4",
        "WENDAO_HOSTED_VLM_OCR_SPECULATIVE_RETRY_DELAY_SECONDS": "1.5",
        "WENDAO_HOSTED_VLM_OCR_PAGE_WINDOW_SIZE": "3",
        "WENDAO_PDF_OCR_PREWARM_PROFILES": "docling-fast-text-ocr",
        "WENDAO_PDF_OCR_PREWARM_SOURCE_PATH": "tests/fixtures/document.pdf",
        "WENDAO_PDF_OCR_PREWARM_PAGE_INDICES": "5,11",
        "WENDAO_OPENROUTER_MODEL": "openrouter/vision-ocr",
        "WENDAO_OPENROUTER_HTTP_REFERER": "https://wendao.local",
        "WENDAO_OPENROUTER_TITLE": "Wendao OCR Benchmark",
    }


def test_hosted_vlm_ocr_process_env_defaults_openrouter_smoke_model() -> None:
    benchmark = _load_benchmark_module()
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
        openrouter_model=None,
        openrouter_http_referer=None,
        openrouter_title=None,
    )

    assert benchmark.hosted_vlm_ocr_process_env(args) == {
        "WENDAO_HOSTED_VLM_OCR_PROVIDER": "openrouter",
        "WENDAO_OPENROUTER_MODEL": "baidu/qianfan-ocr-fast:free",
    }


def test_summarize_hosted_vlm_ocr_request_traces(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    log_dir = tmp_path / "logs"
    log_dir.mkdir()
    (log_dir / "python-worker.hosted-vlm-ocr.jsonl").write_text(
        "\n".join(
            [
                json.dumps(
                    {
                        "status": "succeeded",
                        "httpStatus": 200,
                        "startedUnixMs": 1_000,
                        "endedUnixMs": 1_010,
                        "latencyMs": 10.0,
                        "model": "baidu/qianfan-ocr-fast:free",
                        "markdownChars": 100,
                        "imageBytes": 2048,
                        "pageCount": 2,
                        "requestKind": "page-window-canary",
                        "httpAttemptCount": 1,
                        "shardCount": 2,
                        "shardTypeCounts": {"page": 2},
                        "sourcePixelArea": 2000,
                        "renderDpi": 300,
                        "scaffoldMode": "disabled",
                        "imageOptimizationMode": "disabled",
                        "scaffoldAppliedCount": 0,
                        "scaffoldValidationFailureCount": 0,
                        "scaffoldJsonChars": 0,
                        "canonicalMarkdownChars": 0,
                    }
                ),
                json.dumps(
                    {
                        "status": "failed",
                        "httpStatus": 429,
                        "startedUnixMs": 1_005,
                        "endedUnixMs": 1_035,
                        "latencyMs": 30.0,
                        "model": "baidu/qianfan-ocr-fast:free",
                        "markdownChars": 0,
                        "imageBytes": 1024,
                        "requestKind": "region",
                        "httpAttemptCount": 2,
                        "shardCount": 1,
                        "shardTypeCounts": {"region": 1},
                        "sourcePixelArea": 400,
                        "renderDpi": 300,
                        "scaffoldMode": "region-table-json",
                        "imageOptimizationMode": "region-whitespace-trim",
                        "scaffoldAppliedCount": 1,
                        "scaffoldValidationFailureCount": 1,
                        "scaffoldJsonChars": 17,
                        "canonicalMarkdownChars": 0,
                    }
                ),
                "{bad-json",
            ]
        ),
        encoding="utf-8",
    )

    summary = benchmark.summarize_hosted_vlm_ocr_request_traces(log_dir)

    assert summary["traceFileCount"] == 1
    assert summary["requestCount"] == 2
    assert summary["httpAttemptCountTotal"] == 3
    assert summary["successCount"] == 1
    assert summary["failureCount"] == 1
    assert summary["parseErrorCount"] == 1
    assert summary["statusCounts"] == {"failed": 1, "succeeded": 1}
    assert summary["httpStatusCounts"] == {"200": 1, "429": 1}
    assert summary["modelCounts"] == {"baidu/qianfan-ocr-fast:free": 2}
    assert summary["requestKindCounts"] == {
        "page-window-canary": 1,
        "region": 1,
    }
    assert summary["scaffoldModeCounts"] == {
        "disabled": 1,
        "region-table-json": 1,
    }
    assert summary["imageOptimizationModeCounts"] == {
        "disabled": 1,
        "region-whitespace-trim": 1,
    }
    assert summary["shardTypeCounts"] == {"page": 2, "region": 1}
    assert summary["renderDpiCounts"] == {"300": 2}
    assert summary["pageCountTotal"] == 3
    assert summary["shardCountTotal"] == 3
    assert summary["pageShardCount"] == 2
    assert summary["regionShardCount"] == 1
    assert summary["charCountTotal"] == 100
    assert summary["scaffoldAppliedCount"] == 1
    assert summary["scaffoldValidationFailureCount"] == 1
    assert summary["scaffoldJsonCharCountTotal"] == 17
    assert summary["canonicalMarkdownCharCountTotal"] == 0
    assert summary["imageBytesTotal"] == 3072
    assert summary["sourcePixelAreaTotal"] == 2400
    assert summary["latencyMsP50"] == 10.0
    assert summary["latencyMsP95"] == 30.0
    assert summary["latencyMsMax"] == 30.0
    assert summary["requestLatencyMsTotal"] == 40.0
    assert summary["requestWallStartUnixMs"] == 1_000
    assert summary["requestWallEndUnixMs"] == 1_035
    assert summary["requestWallSpanMs"] == 35
    assert summary["requestLatencyOverlapRatio"] == 1.143


def test_openrouter_key_configured_reads_environment(monkeypatch) -> None:
    benchmark = _load_benchmark_module()
    for key in (
        "WENDAO_OPENROUTER_API_KEY",
        "OPENROUTER_API_KEY",
        "WENDAO_HOSTED_VLM_OCR_API_KEY",
    ):
        monkeypatch.delenv(key, raising=False)

    assert benchmark._openrouter_key_configured() is False

    monkeypatch.setenv("OPENROUTER_API_KEY", "or-key")

    assert benchmark._openrouter_key_configured() is True


def test_converter_count_path_reads_external_fake_counter(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    count_path = tmp_path / "count.txt"
    count_path.write_text("9", encoding="utf-8")
    args = benchmark.argparse.Namespace(converter_count_path=count_path)

    assert benchmark.read_converter_count(args) == 9


def test_converter_count_path_sums_local_worker_counter_dir(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    count_dir = tmp_path / "counts"
    count_dir.mkdir()
    (count_dir / "python-worker-0.txt").write_text("3", encoding="utf-8")
    (count_dir / "python-worker-1.txt").write_text("4", encoding="utf-8")
    args = benchmark.argparse.Namespace(converter_count_path=count_dir)

    assert benchmark.read_converter_count(args) == 7
