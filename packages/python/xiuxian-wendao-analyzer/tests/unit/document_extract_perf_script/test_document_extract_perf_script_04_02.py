"""document_extract_perf_script test slice 4."""

from __future__ import annotations

import sys
import types

from .support import (
    Path,
    _load_benchmark_module,
)


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


def test_wait_for_document_extract_flight_endpoint_uses_flight_info(
    monkeypatch,
) -> None:
    benchmark = _load_benchmark_module()
    calls = []

    class FakeProcess:
        def poll(self):
            return None

    class FakeFlightClient:
        def __init__(self, location: str):
            calls.append(("client", location))

        def get_flight_info(self, descriptor):
            calls.append(("get_flight_info", descriptor))

    class FakeFlightDescriptor:
        @staticmethod
        def for_path(*parts: str):
            calls.append(("descriptor", parts))
            return ("descriptor", parts)

    fake_pyarrow = types.ModuleType("pyarrow")
    fake_flight = types.ModuleType("pyarrow.flight")
    fake_flight.FlightClient = FakeFlightClient
    fake_flight.FlightDescriptor = FakeFlightDescriptor
    fake_pyarrow.flight = fake_flight
    monkeypatch.setitem(sys.modules, "pyarrow", fake_pyarrow)
    monkeypatch.setitem(sys.modules, "pyarrow.flight", fake_flight)

    benchmark.wait_for_document_extract_flight_endpoint(
        "127.0.0.1",
        50051,
        FakeProcess(),
        timeout_seconds=1,
    )

    assert calls == [
        ("client", "grpc://127.0.0.1:50051"),
        ("descriptor", ("analysis", "document-extract")),
        ("get_flight_info", ("descriptor", ("analysis", "document-extract"))),
    ]
