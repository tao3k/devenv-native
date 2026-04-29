from __future__ import annotations

import threading
from pathlib import Path

import pyarrow as pa
import pyarrow.flight as flight
import pytest

from xiuxian_wendao_analyzer import (
    ANALYSIS_DOCUMENT_EXTRACT_ROUTE,
    ANALYSIS_PDF_OCR_SHARDS_ROUTE,
    DOCUMENT_RESOURCE_SCHEMA,
    EXPECTED_SCHEMA_VERSION,
    PDF_OCR_SHARD_INPUT_SCHEMA,
    PDF_OCR_SHARD_INPUT_SCHEMA_VERSION,
    PDF_OCR_SHARD_RESULT_SCHEMA,
    PDF_OCR_SHARD_RESULT_SCHEMA_VERSION,
    SUPPORTED_DOCUMENT_ROUTES,
    WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER,
    WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER,
    WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
    WENDAO_SCHEMA_VERSION_HEADER,
    DoclingPdfOcrShardWorker,
    DocumentExtractFlightServer,
    build_document_extract_table,
    build_pdf_ocr_shard_result_table,
    succeeded_pdf_ocr_shard_result,
)
from xiuxian_wendao_analyzer.document_service import _build_pdf_ocr_worker
from xiuxian_wendao_analyzer.pdf_ocr import SkippingPdfOcrShardWorker


class FakeDoclingDocument:
    def __init__(self, markdown: str) -> None:
        self.markdown = markdown

    def export_to_markdown(self) -> str:
        return self.markdown


class FakeDoclingResult:
    def __init__(self, markdown: str) -> None:
        self.document = FakeDoclingDocument(markdown)


class FakeDoclingConverter:
    def __init__(self, markdown: str = "# Service\n") -> None:
        self.markdown = markdown
        self.calls: list[Path] = []

    def convert(self, source: str | Path) -> FakeDoclingResult:
        self.calls.append(Path(source))
        return FakeDoclingResult(self.markdown)


class FailingDoclingConverter:
    def convert(self, source: str | Path) -> FakeDoclingResult:
        raise RuntimeError(f"cannot OCR {source}")


def test_document_extract_table_uses_document_headers(tmp_path: Path) -> None:
    source = tmp_path / "manual.docx"
    source.write_bytes(b"docx fixture")
    output_dir = tmp_path / "out"
    converter = FakeDoclingConverter("# Manual\n")

    table = build_document_extract_table(
        {
            WENDAO_SCHEMA_VERSION_HEADER: EXPECTED_SCHEMA_VERSION,
            WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER: str(source),
            WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER: str(output_dir),
            WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER: "true",
        },
        converter=converter,
    )

    assert converter.calls == [source]
    assert table.schema == DOCUMENT_RESOURCE_SCHEMA
    row = table.to_pylist()[0]
    assert row["sourcePath"] == str(source)
    assert row["resourcePath"] == str(output_dir / "manual.md")
    assert row["content"] == "# Manual\n"


def test_document_extract_table_can_return_error_rows(tmp_path: Path) -> None:
    missing = tmp_path / "missing.pdf"

    table = build_document_extract_table(
        {
            WENDAO_SCHEMA_VERSION_HEADER: EXPECTED_SCHEMA_VERSION,
            WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER: str(missing),
            WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER: "true",
        },
        converter=FakeDoclingConverter(),
    )

    row = table.to_pylist()[0]
    assert row["resourceType"] == "error"
    assert row["status"] == "error"
    assert "does not exist" in row["content"]


def test_document_extract_table_validates_required_headers(tmp_path: Path) -> None:
    source = tmp_path / "manual.pdf"
    source.write_bytes(b"pdf fixture")

    with pytest.raises(ValueError, match="Unexpected schema version"):
        build_document_extract_table(
            {
                WENDAO_SCHEMA_VERSION_HEADER: "v1",
                WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER: str(source),
            },
            converter=FakeDoclingConverter(),
        )

    with pytest.raises(ValueError, match="Missing document source path header"):
        build_document_extract_table(
            {WENDAO_SCHEMA_VERSION_HEADER: EXPECTED_SCHEMA_VERSION},
            converter=FakeDoclingConverter(),
        )


def test_document_extract_routes_include_only_primary_document_route() -> None:
    assert SUPPORTED_DOCUMENT_ROUTES == (
        ANALYSIS_DOCUMENT_EXTRACT_ROUTE,
        ANALYSIS_PDF_OCR_SHARDS_ROUTE,
    )


class FakePdfOcrShardWorker:
    def __init__(self) -> None:
        self.inputs: list[dict[str, object]] = []

    def recognize(self, inputs):
        self.inputs = list(inputs)
        return [succeeded_pdf_ocr_shard_result(inputs[0], "page text", 0.91)]


def test_pdf_ocr_shard_result_table_defaults_to_skipped_rows() -> None:
    table = build_pdf_ocr_shard_result_table(_sample_pdf_ocr_input_table())

    assert table.schema == PDF_OCR_SHARD_RESULT_SCHEMA
    row = table.to_pylist()[0]
    assert row["contractVersion"] == PDF_OCR_SHARD_RESULT_SCHEMA_VERSION
    assert row["status"] == "skipped"
    assert row["text"] is None
    assert row["confidence"] is None
    assert row["errorMessage"] == "OCR shard worker is not configured"
    assert len(row["elementId"]) == 64


def test_pdf_ocr_shard_result_table_uses_injected_worker() -> None:
    worker = FakePdfOcrShardWorker()
    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(), worker=worker
    )

    assert worker.inputs[0]["imagePath"] == "/tmp/page-00000.png"
    assert worker.inputs[0]["shardType"] == "page"
    assert worker.inputs[0]["readingOrderKey"] == "000000.000000"
    row = table.to_pylist()[0]
    assert row["status"] == "succeeded"
    assert row["text"] == "page text"
    assert row["confidence"] == 0.91


def test_pdf_ocr_shard_result_table_accepts_region_inputs() -> None:
    worker = FakePdfOcrShardWorker()
    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(
            shard_type="region",
            region_index=2,
            parent_shard_element_id="parent-page",
            reading_order_key="000000.000002",
        ),
        worker=worker,
    )

    assert worker.inputs[0]["shardType"] == "region"
    assert worker.inputs[0]["regionIndex"] == 2
    assert worker.inputs[0]["parentShardElementId"] == "parent-page"
    assert worker.inputs[0]["readingOrderKey"] == "000000.000002"
    assert table.to_pylist()[0]["status"] == "succeeded"


def test_docling_pdf_ocr_worker_converts_page_images(tmp_path: Path) -> None:
    image = tmp_path / "page-00000.png"
    image.write_bytes(b"png fixture")
    converter = FakeDoclingConverter("OCR **page**\n")

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(image_path=str(image)),
        worker=DoclingPdfOcrShardWorker(converter),
    )

    assert converter.calls == [image]
    row = table.to_pylist()[0]
    assert row["status"] == "succeeded"
    assert row["text"] == "OCR **page**\n"
    assert row["textMimeType"] == "text/markdown"
    assert row["confidence"] is None


def test_docling_pdf_ocr_worker_reports_missing_images() -> None:
    converter = FakeDoclingConverter("OCR\n")

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(image_path="/tmp/missing-page.png"),
        worker=DoclingPdfOcrShardWorker(converter),
    )

    row = table.to_pylist()[0]
    assert converter.calls == []
    assert row["status"] == "failed"
    assert "does not exist" in row["errorMessage"]


def test_docling_pdf_ocr_worker_rejects_empty_output(tmp_path: Path) -> None:
    image = tmp_path / "page-00000.png"
    image.write_bytes(b"png fixture")

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(image_path=str(image)),
        worker=DoclingPdfOcrShardWorker(FakeDoclingConverter(" \n")),
    )

    row = table.to_pylist()[0]
    assert row["status"] == "failed"
    assert row["errorMessage"] == "Docling OCR returned empty text"


def test_docling_pdf_ocr_worker_reports_converter_errors(tmp_path: Path) -> None:
    image = tmp_path / "page-00000.png"
    image.write_bytes(b"png fixture")

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(image_path=str(image)),
        worker=DoclingPdfOcrShardWorker(FailingDoclingConverter()),
    )

    row = table.to_pylist()[0]
    assert row["status"] == "failed"
    assert "Docling OCR failed" in row["errorMessage"]


def test_document_service_pdf_ocr_worker_selection_is_explicit() -> None:
    assert isinstance(_build_pdf_ocr_worker("skip"), SkippingPdfOcrShardWorker)
    assert isinstance(_build_pdf_ocr_worker("docling"), DoclingPdfOcrShardWorker)


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


def _sample_pdf_ocr_input_table(
    image_path: str = "/tmp/page-00000.png",
    *,
    shard_type: str = "page",
    region_index: int = 0,
    parent_shard_element_id: str = "",
    reading_order_key: str = "000000.000000",
):
    return pa.Table.from_pylist(
        [
            {
                "contractVersion": PDF_OCR_SHARD_INPUT_SCHEMA_VERSION,
                "sourcePath": "/tmp/source.pdf",
                "sourceContentHash": "sourcehash",
                "pageIndex": 0,
                "imagePath": image_path,
                "imageMimeType": "image/png",
                "rasterSha256": "rasterhash",
                "renderProfile": "pdfium-render-page-shards-v1",
                "ocrProfile": "docling-compatible-page-ocr-v1",
                "ocrEngine": "docling-compatible-ocr",
                "preferredLanguages": "auto",
                "minConfidence": 0.0,
                "preserveLayout": True,
                "rasterWidthPx": 2400,
                "rasterHeightPx": 3100,
                "renderDpi": 300,
                "rotationDegrees": 0,
                "cropLeft": 0.0,
                "cropBottom": 0.0,
                "cropRight": 612.0,
                "cropTop": 792.0,
                "pointToPixelScaleX": 3.921568627,
                "pointToPixelScaleY": 3.914141414,
                "shardElementId": "shard-id",
                "shardType": shard_type,
                "regionIndex": region_index,
                "parentShardElementId": parent_shard_element_id,
                "readingOrderKey": reading_order_key,
                "sourcePagePixelLeft": 0,
                "sourcePagePixelTop": 0,
                "sourcePagePixelRight": 2400,
                "sourcePagePixelBottom": 3100,
            }
        ],
        schema=PDF_OCR_SHARD_INPUT_SCHEMA,
    )
