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
    assert "document_extract_full_threads_from_env" in code
    assert "DOCUMENT_EXTRACT_FULL_PROFILE" in code
    assert "DOCUMENT_EXTRACT_STRUCTURE_TEXT_PROFILE" in code
    assert "prewarm_document_extract_converter(converter)" in code
    assert "WENDAO_DOCUMENT_EXTRACT_PREWARM_SOURCE_PATH" in code
    assert "WENDAO_DOCUMENT_EXTRACT_PREWARM_PAGE_RANGES" in code
    assert "PDF_OCR_FAST_TEXT_PROFILE" in code
    assert "AcceleratorOptions" in code
    assert "WENDAO_PDF_OCR_FAST_TEXT_THREADS" in code
    assert "AcceleratorDevice.CPU" in code
    assert "TableFormerMode.FAST" in code
    assert "PDF_OCR_DOCLING_VLM_DEEPSEEK_OCR_PROFILE" in code
    assert 'VlmConvertOptions.from_preset("deepseek_ocr")' in code
    assert "DoclingPdfOcrShardWorker(" in code
    assert "converter_factory=make_converter" in code
    assert code.count("converter_factory=make_converter") == 2
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
