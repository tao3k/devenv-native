"""document_service test slice 4."""

from __future__ import annotations

from xiuxian_wendao_analyzer.audio_shard_worker_config import (
    hosted_audio_config_from_env,
)
from xiuxian_wendao_analyzer.document_service_cli import (
    build_audio_worker,
    build_document_extract_argument_parser,
)

from .support import (
    AUDIO_SHARD_RESULT_SCHEMA,
    PDF_OCR_SHARD_RESULT_SCHEMA,
    WENDAO_AUDIO_WORKERS_HEADER,
    WENDAO_PDF_OCR_WORKERS_HEADER,
    DoclingAudioShardWorker,
    DoclingPdfOcrShardWorker,
    DocumentExtractFlightServer,
    FakeAudioShardWorker,
    FakeDoclingConverter,
    FakePdfOcrShardWorker,
    HostedAudioConfig,
    HostedAudioShardWorker,
    SkippingAudioShardWorker,
    SkippingPdfOcrShardWorker,
    UnsupportedAudioShardWorker,
    _build_audio_shard_worker,
    _build_pdf_ocr_worker,
    _sample_audio_shard_input_table,
    _sample_pdf_ocr_input_table,
    build_audio_shard_result_table,
    flight,
    hosted_audio_payload,
    normalize_audio_worker_name,
    pa,
    pytest,
    resolve_pdf_ocr_worker_count,
    threading,
    time,
)


def test_document_service_pdf_ocr_worker_selection_is_explicit() -> None:
    assert isinstance(_build_pdf_ocr_worker("skip"), SkippingPdfOcrShardWorker)
    assert isinstance(_build_pdf_ocr_worker("docling"), DoclingPdfOcrShardWorker)


def test_document_service_audio_worker_selection_is_explicit() -> None:
    assert isinstance(_build_audio_shard_worker("skip"), SkippingAudioShardWorker)
    assert isinstance(_build_audio_shard_worker("docling"), DoclingAudioShardWorker)
    assert isinstance(_build_audio_shard_worker("hosted"), HostedAudioShardWorker)
    assert isinstance(_build_audio_shard_worker("unknown"), UnsupportedAudioShardWorker)
    assert normalize_audio_worker_name("hosted") == "hosted-audio-transcript-v1"


def test_document_service_cli_accepts_audio_worker_flags() -> None:
    args = build_document_extract_argument_parser().parse_args(
        ["--audio-worker", "hosted", "--audio-workers", "2"]
    )

    assert isinstance(
        build_audio_worker(args.audio_worker, args.audio_workers),
        HostedAudioShardWorker,
    )


def test_pdf_ocr_worker_count_is_adaptive(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("WENDAO_PDF_OCR_WORKERS", raising=False)
    monkeypatch.delenv("WENDAO_PDF_OCR_MAX_WORKERS", raising=False)

    assert resolve_pdf_ocr_worker_count(2, 8) == 2
    assert resolve_pdf_ocr_worker_count(8, "3") == 3
    assert resolve_pdf_ocr_worker_count(8, "invalid") >= 1


def test_pdf_ocr_worker_count_respects_max_cap(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv("WENDAO_PDF_OCR_MAX_WORKERS", "2")

    assert resolve_pdf_ocr_worker_count(8, 6) == 2


def test_document_service_exchanges_pdf_ocr_shards_over_arrow_flight() -> None:
    worker = FakePdfOcrShardWorker()
    server = DocumentExtractFlightServer("grpc://127.0.0.1:0", ocr_worker=worker)
    thread = threading.Thread(target=server.serve, daemon=True)
    thread.start()
    client = flight.FlightClient(f"grpc://127.0.0.1:{server.port}")
    descriptor = flight.FlightDescriptor.for_path("analysis", "pdf-ocr-shards")
    writer, reader = client.do_exchange(descriptor)
    input_table = _sample_pdf_ocr_input_table()

    try:
        writer.begin(input_table.schema)
        writer.write_table(input_table)
        writer.done_writing()
        result = reader.read_all()
    finally:
        writer.close()
        server.shutdown()
        thread.join(timeout=5)

    assert result.schema == PDF_OCR_SHARD_RESULT_SCHEMA
    row = result.to_pylist()[0]
    assert row["status"] == "succeeded"
    assert row["text"] == "page text"
    assert worker.inputs[0]["sourcePath"] == "/tmp/source.pdf"


def test_document_service_forwards_pdf_ocr_worker_budget_header() -> None:
    worker = FakePdfOcrShardWorker()
    server = DocumentExtractFlightServer("grpc://127.0.0.1:0", ocr_worker=worker)
    thread = threading.Thread(target=server.serve, daemon=True)
    thread.start()
    client = flight.FlightClient(f"grpc://127.0.0.1:{server.port}")
    descriptor = flight.FlightDescriptor.for_path("analysis", "pdf-ocr-shards")
    options = flight.FlightCallOptions(
        headers=[(WENDAO_PDF_OCR_WORKERS_HEADER.encode("utf-8"), b"3")]
    )
    writer, reader = client.do_exchange(descriptor, options=options)
    input_table = _sample_pdf_ocr_input_table()

    try:
        writer.begin(input_table.schema)
        writer.write_table(input_table)
        writer.done_writing()
        result = reader.read_all()
    finally:
        writer.close()
        server.shutdown()
        thread.join(timeout=5)

    assert result.schema == PDF_OCR_SHARD_RESULT_SCHEMA
    assert worker.max_workers == "3"


def test_document_service_exchanges_audio_shards_over_arrow_flight() -> None:
    worker = FakeAudioShardWorker()
    server = DocumentExtractFlightServer("grpc://127.0.0.1:0", audio_worker=worker)
    thread = threading.Thread(target=server.serve, daemon=True)
    thread.start()
    client = flight.FlightClient(f"grpc://127.0.0.1:{server.port}")
    descriptor = flight.FlightDescriptor.for_path("analysis", "audio-shards")
    writer, reader = client.do_exchange(descriptor)
    input_table = _sample_audio_shard_input_table()

    try:
        writer.begin(input_table.schema)
        writer.write_table(input_table)
        writer.done_writing()
        result = reader.read_all()
    finally:
        writer.close()
        server.shutdown()
        thread.join(timeout=5)

    assert result.schema == AUDIO_SHARD_RESULT_SCHEMA
    row = result.to_pylist()[0]
    assert row["status"] == "succeeded"
    assert row["text"] == "audio text"
    assert worker.inputs[0]["sourcePath"] == "/tmp/source.mp3"
    assert worker.inputs[0]["shardPath"] == "/tmp/chunk.wav"


def test_document_service_forwards_audio_worker_budget_header() -> None:
    worker = FakeAudioShardWorker()
    server = DocumentExtractFlightServer("grpc://127.0.0.1:0", audio_worker=worker)
    thread = threading.Thread(target=server.serve, daemon=True)
    thread.start()
    client = flight.FlightClient(f"grpc://127.0.0.1:{server.port}")
    descriptor = flight.FlightDescriptor.for_path("analysis", "audio-shards")
    options = flight.FlightCallOptions(
        headers=[(WENDAO_AUDIO_WORKERS_HEADER.encode("utf-8"), b"4")]
    )
    writer, reader = client.do_exchange(descriptor, options=options)
    input_table = _sample_audio_shard_input_table()

    try:
        writer.begin(input_table.schema)
        writer.write_table(input_table)
        writer.done_writing()
        result = reader.read_all()
    finally:
        writer.close()
        server.shutdown()
        thread.join(timeout=5)

    assert result.schema == AUDIO_SHARD_RESULT_SCHEMA
    assert worker.max_workers == "4"


def test_docling_audio_worker_normalizes_transcript(tmp_path) -> None:
    shard = tmp_path / "chunk.wav"
    shard.write_bytes(b"wav")
    input_table = _sample_audio_shard_input_table(str(shard))
    worker = DoclingAudioShardWorker(
        converter_factory=lambda: FakeDoclingConverter("  transcript text  ")
    )

    result = build_audio_shard_result_table(input_table, worker=worker)

    row = result.to_pylist()[0]
    assert row["status"] == "succeeded"
    assert row["text"] == "transcript text"
    assert row["textMimeType"] == "text/plain"


def test_hosted_audio_worker_builds_openai_compatible_payload(tmp_path) -> None:
    shard = tmp_path / "chunk.wav"
    shard.write_bytes(b"abc")
    input_row = _sample_audio_shard_input_table(str(shard)).to_pylist()[0]

    payload = hosted_audio_payload(input_row, "audio-model")

    assert payload["model"] == "audio-model"
    content = payload["messages"][0]["content"]
    assert content[1]["type"] == "input_audio"
    assert content[1]["input_audio"]["format"] == "wav"
    assert content[1]["input_audio"]["data"] == "YWJj"


def test_hosted_audio_worker_payload_accepts_primary_language_hint(tmp_path) -> None:
    shard = tmp_path / "chunk.wav"
    shard.write_bytes(b"abc")
    input_row = _sample_audio_shard_input_table(str(shard)).to_pylist()[0]

    payload = hosted_audio_payload(input_row, "audio-model", primary_language="zh")

    content = payload["messages"][0]["content"]
    assert "PRIMARY_LANGUAGE=zh" in content[0]["text"]
    assert "Infer the actual spoken language from the audio" in content[0]["text"]


def test_hosted_audio_config_strips_wrapping_quotes_from_env(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv("WENDAO_AUDIO_HOSTED_PROVIDER", '"openrouter"')
    monkeypatch.setenv("WENDAO_AUDIO_HOSTED_MODEL", '"xiaomi/mimo-v2.5"')
    monkeypatch.setenv("OPENROUTER_API_KEY", '"or-key"')

    config = hosted_audio_config_from_env()

    assert config.provider == "openrouter"
    assert config.model == "xiaomi/mimo-v2.5"
    assert config.api_key == "or-key"


def test_hosted_audio_worker_normalizes_successful_response(tmp_path) -> None:
    shard = tmp_path / "chunk.wav"
    shard.write_bytes(b"abc")
    input_table = _sample_audio_shard_input_table(str(shard))
    config = HostedAudioConfig(
        provider="openai-compatible",
        base_url="https://example.test/v1",
        model="audio-model",
        api_key="key",
        timeout_seconds=5.0,
        request_concurrency=1,
    )

    worker = HostedAudioShardWorker(
        config=config,
        request_sender=lambda _config, _payload: {
            "choices": [{"message": {"content": "云端转写"}}]
        },
    )

    result = build_audio_shard_result_table(input_table, worker=worker)

    row = result.to_pylist()[0]
    assert row["status"] == "succeeded"
    assert row["text"] == "云端转写"


def test_hosted_audio_worker_retries_transient_request_failure(tmp_path) -> None:
    shard = tmp_path / "chunk.wav"
    shard.write_bytes(b"abc")
    input_table = _sample_audio_shard_input_table(str(shard))
    calls = 0

    def request_sender(_config, _payload):
        nonlocal calls
        calls += 1
        if calls == 1:
            raise RuntimeError("temporary upstream failure")
        return {"choices": [{"message": {"content": "重试文本"}}]}

    worker = HostedAudioShardWorker(
        config=HostedAudioConfig(
            provider="openai-compatible",
            base_url="https://example.test/v1",
            model="audio-model",
            api_key="key",
            timeout_seconds=5.0,
            request_concurrency=1,
            max_attempts=2,
        ),
        request_sender=request_sender,
    )

    result = build_audio_shard_result_table(input_table, worker=worker)

    row = result.to_pylist()[0]
    assert row["status"] == "succeeded"
    assert row["text"] == "重试文本"
    assert calls == 2


def test_hosted_audio_worker_uses_configured_flight_request_concurrency(
    tmp_path,
) -> None:
    rows = []
    for index in range(3):
        shard = tmp_path / f"chunk-{index}.wav"
        shard.write_bytes(f"audio-{index}".encode())
        row = _sample_audio_shard_input_table(
            str(shard),
            shard_element_id=f"audio-shard-{index}",
        ).to_pylist()[0]
        row["readingOrderKey"] = f"{index:06d}.000000000000"
        rows.append(row)
    input_table = pa.Table.from_pylist(
        rows, schema=_sample_audio_shard_input_table().schema
    )
    barrier = threading.Barrier(3)
    seen: list[str] = []
    seen_lock = threading.Lock()

    def request_sender(_config, payload):
        audio_bytes = payload["messages"][0]["content"][1]["input_audio"]["data"]
        with seen_lock:
            seen.append(audio_bytes)
        barrier.wait(timeout=1.0)
        return {"choices": [{"message": {"content": f"转写-{len(seen)}"}}]}

    worker = HostedAudioShardWorker(
        config=HostedAudioConfig(
            provider="openai-compatible",
            base_url="https://example.test/v1",
            model="audio-model",
            api_key="key",
            timeout_seconds=5.0,
            request_concurrency=3,
        ),
        request_sender=request_sender,
    )

    result = build_audio_shard_result_table(input_table, worker=worker)

    output_rows = result.to_pylist()
    assert [row["shardElementId"] for row in output_rows] == [
        "audio-shard-0",
        "audio-shard-1",
        "audio-shard-2",
    ]
    assert all(row["status"] == "succeeded" for row in output_rows)
    assert len(seen) == 3


def test_hosted_audio_worker_caps_concurrency_by_flight_budget(tmp_path) -> None:
    rows = []
    for index in range(3):
        shard = tmp_path / f"chunk-{index}.wav"
        shard.write_bytes(f"audio-{index}".encode())
        row = _sample_audio_shard_input_table(
            str(shard),
            shard_element_id=f"audio-shard-{index}",
        ).to_pylist()[0]
        row["readingOrderKey"] = f"{index:06d}.000000000000"
        rows.append(row)
    input_table = pa.Table.from_pylist(
        rows, schema=_sample_audio_shard_input_table().schema
    )
    active_count = 0
    max_seen_active_count = 0
    seen_lock = threading.Lock()

    def request_sender(_config, _payload):
        nonlocal active_count, max_seen_active_count
        with seen_lock:
            active_count += 1
            max_seen_active_count = max(max_seen_active_count, active_count)
        time.sleep(0.05)
        with seen_lock:
            active_count -= 1
        return {"choices": [{"message": {"content": "转写文本"}}]}

    worker = HostedAudioShardWorker(
        config=HostedAudioConfig(
            provider="openai-compatible",
            base_url="https://example.test/v1",
            model="audio-model",
            api_key="key",
            timeout_seconds=5.0,
            request_concurrency=3,
        ),
        request_sender=request_sender,
    )

    result = build_audio_shard_result_table(input_table, worker=worker, max_workers=2)

    assert all(row["status"] == "succeeded" for row in result.to_pylist())
    assert max_seen_active_count == 2


def test_hosted_audio_worker_reports_malformed_response(tmp_path) -> None:
    shard = tmp_path / "chunk.wav"
    shard.write_bytes(b"abc")
    input_table = _sample_audio_shard_input_table(str(shard))
    config = HostedAudioConfig(
        provider="openai-compatible",
        base_url="https://example.test/v1",
        model="audio-model",
        api_key="key",
        timeout_seconds=5.0,
        request_concurrency=1,
    )

    worker = HostedAudioShardWorker(
        config=config,
        request_sender=lambda _config, _payload: {"choices": []},
    )

    result = build_audio_shard_result_table(input_table, worker=worker)

    row = result.to_pylist()[0]
    assert row["status"] == "failed"
    assert "does not contain choices" in row["errorMessage"]


def test_hosted_audio_worker_reports_empty_text(tmp_path) -> None:
    shard = tmp_path / "chunk.wav"
    shard.write_bytes(b"abc")
    input_table = _sample_audio_shard_input_table(str(shard))
    config = HostedAudioConfig(
        provider="openai-compatible",
        base_url="https://example.test/v1",
        model="audio-model",
        api_key="key",
        timeout_seconds=5.0,
        request_concurrency=1,
    )

    worker = HostedAudioShardWorker(
        config=config,
        request_sender=lambda _config, _payload: {
            "choices": [{"message": {"content": "   "}}]
        },
    )

    result = build_audio_shard_result_table(input_table, worker=worker)

    row = result.to_pylist()[0]
    assert row["status"] == "failed"
    assert row["errorMessage"] == "Hosted audio worker returned empty text"


def test_unsupported_audio_worker_returns_failed_rows() -> None:
    table = build_audio_shard_result_table(
        _sample_audio_shard_input_table(),
        worker=_build_audio_shard_worker("missing-backend"),
    )

    row = table.to_pylist()[0]
    assert row["status"] == "failed"
    assert row["errorMessage"] == "unsupported audio shard worker: missing-backend"
