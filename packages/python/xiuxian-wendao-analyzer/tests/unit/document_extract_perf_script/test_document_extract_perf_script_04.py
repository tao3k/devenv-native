"""document_extract_perf_script test slice 4."""

from __future__ import annotations

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
        deepseek_ocr2_env={"WENDAO_DEEPSEEK_OCR2_MODEL": "community/ocr2-awq"},
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
    assert [call[2]["deepseek_ocr2_env"] for call in calls] == [
        {"WENDAO_DEEPSEEK_OCR2_MODEL": "community/ocr2-awq"},
        {"WENDAO_DEEPSEEK_OCR2_MODEL": "community/ocr2-awq"},
        {"WENDAO_DEEPSEEK_OCR2_MODEL": "community/ocr2-awq"},
    ]


def test_deepseek_ocr2_process_env_maps_cli_args() -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        deepseek_ocr2_provider="openrouter",
        deepseek_ocr2_base_url="http://127.0.0.1:8000/v1",
        deepseek_ocr2_model="community/deepseek-ocr2-awq",
        deepseek_ocr2_prompt="<image>\nmarkdown",
        deepseek_ocr2_max_tokens=4096,
        deepseek_ocr2_timeout_seconds=120.0,
        deepseek_ocr2_request_concurrency=4,
        openrouter_model="openrouter/vision-ocr",
        openrouter_http_referer="https://wendao.local",
        openrouter_title="Wendao OCR Benchmark",
    )

    assert benchmark.deepseek_ocr2_process_env(args) == {
        "WENDAO_DEEPSEEK_OCR2_PROVIDER": "openrouter",
        "WENDAO_DEEPSEEK_OCR2_BASE_URL": "http://127.0.0.1:8000/v1",
        "WENDAO_DEEPSEEK_OCR2_MODEL": "community/deepseek-ocr2-awq",
        "WENDAO_DEEPSEEK_OCR2_PROMPT": "<image>\nmarkdown",
        "WENDAO_DEEPSEEK_OCR2_MAX_TOKENS": "4096",
        "WENDAO_DEEPSEEK_OCR2_TIMEOUT_SECONDS": "120.0",
        "WENDAO_DEEPSEEK_OCR2_REQUEST_CONCURRENCY": "4",
        "WENDAO_OPENROUTER_MODEL": "openrouter/vision-ocr",
        "WENDAO_OPENROUTER_HTTP_REFERER": "https://wendao.local",
        "WENDAO_OPENROUTER_TITLE": "Wendao OCR Benchmark",
    }


def test_deepseek_ocr2_process_env_defaults_openrouter_smoke_model() -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        deepseek_ocr2_provider="openrouter",
        deepseek_ocr2_base_url=None,
        deepseek_ocr2_model=None,
        deepseek_ocr2_prompt=None,
        deepseek_ocr2_max_tokens=None,
        deepseek_ocr2_timeout_seconds=None,
        deepseek_ocr2_request_concurrency=None,
        openrouter_model=None,
        openrouter_http_referer=None,
        openrouter_title=None,
    )

    assert benchmark.deepseek_ocr2_process_env(args) == {
        "WENDAO_DEEPSEEK_OCR2_PROVIDER": "openrouter",
        "WENDAO_OPENROUTER_MODEL": "baidu/qianfan-ocr-fast:free",
    }


def test_openrouter_key_configured_reads_environment(monkeypatch) -> None:
    benchmark = _load_benchmark_module()
    for key in (
        "WENDAO_OPENROUTER_API_KEY",
        "OPENROUTER_API_KEY",
        "OPENROUTE_API_KEY",
        "WENDAO_DEEPSEEK_OCR2_API_KEY",
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
