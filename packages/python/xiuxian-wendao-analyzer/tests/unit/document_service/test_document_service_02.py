"""document_service test slice 2."""

from __future__ import annotations

from .support import (
    DoclingPdfOcrShardWorker,
    FakeDoclingConverter,
    FakeDoclingResult,
    FakePdfOcrShardWorker,
    Path,
    _sample_pdf_ocr_input_table,
    build_pdf_ocr_shard_result_table,
    pa,
)


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


def test_docling_pdf_ocr_worker_prefers_source_pdf_page_range(
    tmp_path: Path,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    image = tmp_path / "page-00000.png"
    converter = FakeDoclingConverter("OCR from source page\n")

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(source_path=str(source), image_path=str(image)),
        worker=DoclingPdfOcrShardWorker(converter),
    )

    assert converter.calls == [source]
    assert converter.kwargs_calls == [{"page_range": (1, 1)}]
    row = table.to_pylist()[0]
    assert row["status"] == "succeeded"
    assert row["text"] == "OCR from source page\n"


def test_docling_pdf_ocr_worker_falls_back_to_image_after_page_range_failure(
    tmp_path: Path,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    image = tmp_path / "page-00000.png"
    image.write_bytes(b"png fixture")

    class PageRangeFailingConverter(FakeDoclingConverter):
        def convert(self, source: str | Path, **kwargs: object) -> FakeDoclingResult:
            self.calls.append(Path(source))
            self.kwargs_calls.append(dict(kwargs))
            if "page_range" in kwargs:
                raise RuntimeError("page range unavailable")
            return FakeDoclingResult("OCR from image\n")

    converter = PageRangeFailingConverter()

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(source_path=str(source), image_path=str(image)),
        worker=DoclingPdfOcrShardWorker(converter),
    )

    assert converter.calls == [source, image]
    assert converter.kwargs_calls == [{"page_range": (1, 1)}, {}]
    row = table.to_pylist()[0]
    assert row["status"] == "succeeded"
    assert row["text"] == "OCR from image\n"


def test_docling_pdf_ocr_worker_batches_contiguous_source_pdf_pages(
    tmp_path: Path,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")

    class PageBatchConverter(FakeDoclingConverter):
        def convert(self, source: str | Path, **kwargs: object) -> FakeDoclingResult:
            self.calls.append(Path(source))
            self.kwargs_calls.append(dict(kwargs))
            return FakeDoclingResult(
                "all pages\n",
                markdown_by_page={
                    1: "OCR page 1\n",
                    2: "OCR page 2\n",
                    3: "OCR page 3\n",
                },
            )

    converter = PageBatchConverter()
    input_tables = []
    for page_index in range(3):
        input_tables.append(
            _sample_pdf_ocr_input_table(
                source_path=str(source),
                image_path=str(tmp_path / f"page-{page_index:05}.png"),
                page_index=page_index,
                shard_element_id=f"shard-{page_index}",
            )
        )

    table = build_pdf_ocr_shard_result_table(
        pa.concat_tables(input_tables),
        worker=DoclingPdfOcrShardWorker(converter, max_workers=4),
    )

    assert converter.calls == [source]
    assert converter.kwargs_calls == [{"page_range": (1, 3)}]
    assert [row["text"] for row in table.to_pylist()] == [
        "OCR page 1\n",
        "OCR page 2\n",
        "OCR page 3\n",
    ]
