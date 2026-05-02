"""document_service test slice 4."""

from __future__ import annotations

from .support import (
    PDF_OCR_SHARD_RESULT_SCHEMA,
    WENDAO_PDF_OCR_WORKERS_HEADER,
    DoclingPdfOcrShardWorker,
    DocumentExtractFlightServer,
    FakePdfOcrShardWorker,
    SkippingPdfOcrShardWorker,
    _build_pdf_ocr_worker,
    _sample_pdf_ocr_input_table,
    flight,
    pytest,
    resolve_pdf_ocr_worker_count,
    threading,
)


def test_document_service_pdf_ocr_worker_selection_is_explicit() -> None:
    assert isinstance(_build_pdf_ocr_worker("skip"), SkippingPdfOcrShardWorker)
    assert isinstance(_build_pdf_ocr_worker("docling"), DoclingPdfOcrShardWorker)


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
