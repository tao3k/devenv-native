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

    monkeypatch.setattr(benchmark._workers, "can_bind_port", lambda host, port: True)
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
        audio_worker="hosted",
        audio_workers="2",
        python_uv_package="xiuxian-wendao-analyzer",
        python_uv_extras=[],
        hosted_vlm_ocr_env={
            "WENDAO_HOSTED_VLM_OCR_MODEL": "community/hosted-vlm-awq",
            "WENDAO_PDF_OCR_PREWARM_PROFILES": "docling-fast-text-ocr",
            "WENDAO_PDF_OCR_PREWARM_SOURCE_PATH": "tests/fixtures/document.pdf",
            "WENDAO_PDF_OCR_PREWARM_PAGE_INDICES": "5,11",
            "WENDAO_PDF_OCR_PREWARM_PAGE_INDEX": "5",
        },
        audio_worker_env={
            "WENDAO_AUDIO_HOSTED_PROVIDER": "openrouter",
            "WENDAO_AUDIO_HOSTED_MODEL": "qwen/qwen3-asr-1.7b",
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
    assert [call[2]["hosted_vlm_ocr_env"]["WENDAO_HOSTED_VLM_OCR_MODEL"] for call in calls] == [
        "community/hosted-vlm-awq",
        "community/hosted-vlm-awq",
        "community/hosted-vlm-awq",
    ]
    assert [
        call[2]["hosted_vlm_ocr_env"].get("WENDAO_PDF_OCR_PREWARM_PROFILES") for call in calls
    ] == ["docling-fast-text-ocr", None, None]
    assert [
        call[2]["hosted_vlm_ocr_env"].get("WENDAO_PDF_OCR_PREWARM_SOURCE_PATH") for call in calls
    ] == ["tests/fixtures/document.pdf", None, None]
    assert [
        call[2]["hosted_vlm_ocr_env"].get("WENDAO_PDF_OCR_PREWARM_PAGE_INDEX") for call in calls
    ] == ["5", None, None]
    assert [
        call[2]["hosted_vlm_ocr_env"].get("WENDAO_PDF_OCR_PREWARM_PAGE_INDICES") for call in calls
    ] == ["5,11", None, None]
    assert [
        Path(call[2]["hosted_vlm_ocr_env"]["WENDAO_HOSTED_VLM_OCR_TRACE_PATH"]).name
        for call in calls
    ] == [
        "python-worker-0.hosted-vlm-ocr.jsonl",
        "python-worker-1.hosted-vlm-ocr.jsonl",
        "python-worker-2.hosted-vlm-ocr.jsonl",
    ]
    assert [call[2]["audio_worker"] for call in calls] == [
        "hosted",
        "hosted",
        "hosted",
    ]
    assert [call[2]["audio_workers"] for call in calls] == ["2", "2", "2"]
    assert [call[2]["audio_worker_env"]["WENDAO_AUDIO_HOSTED_PROVIDER"] for call in calls] == [
        "openrouter",
        "openrouter",
        "openrouter",
    ]
    assert [call[2]["audio_worker_env"]["WENDAO_AUDIO_HOSTED_MODEL"] for call in calls] == [
        "qwen/qwen3-asr-1.7b",
        "qwen/qwen3-asr-1.7b",
        "qwen/qwen3-asr-1.7b",
    ]
    assert [
        Path(call[2]["audio_worker_env"]["WENDAO_AUDIO_HOSTED_TRACE_PATH"]).name for call in calls
    ] == [
        "python-worker-0.hosted-audio.jsonl",
        "python-worker-1.hosted-audio.jsonl",
        "python-worker-2.hosted-audio.jsonl",
    ]


def test_resolve_worker_ports_replaces_occupied_default_base_port(
    monkeypatch,
) -> None:
    benchmark = _load_benchmark_module()
    free_ports = iter([62051, 62052])

    monkeypatch.setattr(
        benchmark._workers,
        "can_bind_port",
        lambda host, port: port != 50051,
    )
    monkeypatch.setattr(benchmark._workers, "pick_free_port", lambda host: next(free_ports))

    assert benchmark.resolve_worker_ports(
        "127.0.0.1",
        50051,
        endpoint_count=2,
        allow_base_port_fallback=True,
    ) == [62051, 62052]


def test_resolve_worker_ports_rejects_occupied_explicit_base_port(
    monkeypatch,
) -> None:
    benchmark = _load_benchmark_module()
    monkeypatch.setattr(benchmark._workers, "can_bind_port", lambda host, port: False)

    try:
        benchmark.resolve_worker_ports(
            "127.0.0.1",
            50051,
            endpoint_count=1,
            allow_base_port_fallback=False,
        )
    except SystemExit as exc:
        assert "--port 50051 is already in use" in str(exc)
    else:
        raise AssertionError("expected occupied explicit base port to exit")


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


def test_wait_for_document_extract_flight_endpoint_requires_process_ready_marker(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    calls = []
    stdout_log = tmp_path / "worker.stdout.log"
    stdout_log.write_text("READY grpc://127.0.0.1:50051\n", encoding="utf-8")

    class FakeProcess:
        wendao_stdout_log = stdout_log

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


def test_wait_for_rust_provider_ready_waits_for_port_and_ready_marker(
    monkeypatch,
) -> None:
    benchmark = _load_benchmark_module()
    calls = []

    class FakeProcess:
        pass

    def fake_wait_for_port(
        host: str,
        port: int,
        server: FakeProcess,
        *,
        timeout_seconds: float,
    ) -> None:
        calls.append(("port", host, port, server, timeout_seconds))

    def fake_wait_for_process_stdout_contains(
        server: FakeProcess,
        needle: str,
        *,
        timeout_seconds: float,
    ) -> None:
        calls.append(("ready", server, needle, timeout_seconds))

    monkeypatch.setattr(benchmark._provider_lifecycle, "wait_for_port", fake_wait_for_port)
    monkeypatch.setattr(
        benchmark._provider_lifecycle,
        "wait_for_process_stdout_contains",
        fake_wait_for_process_stdout_contains,
    )
    server = FakeProcess()

    benchmark.wait_for_rust_provider_ready(
        "127.0.0.1",
        50052,
        server,
        timeout_seconds=3.0,
    )

    assert calls == [
        ("port", "127.0.0.1", 50052, server, 3.0),
        ("ready", server, "READY http://127.0.0.1:50052", 3.0),
    ]
