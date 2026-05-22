"""document_service test slice 3."""

from __future__ import annotations

from xiuxian_wendao_analyzer.pdf_ocr import (
    PDF_OCR_FAST_TEXT_PROFILE,
)
from xiuxian_wendao_analyzer.pdf_ocr_workers import (
    PDF_OCR_PREWARM_PROFILES_ENV,
    PDF_OCR_PREWARM_SOURCE_PATH_ENV,
)

from .support import (
    DoclingPdfOcrShardWorker,
    FakeDoclingResult,
    Path,
    _sample_pdf_ocr_input_table,
    build_pdf_ocr_shard_result_table,
    pa,
)


def test_docling_pdf_ocr_worker_reuses_prewarmed_source_pages(
    monkeypatch,
    tmp_path: Path,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")

    class PageRangeConverter:
        def __init__(self) -> None:
            self.calls: list[tuple[int, int]] = []

        def convert(
            self, source_path: str | Path, **kwargs: object
        ) -> FakeDoclingResult:
            assert Path(source_path) == source
            page_range = kwargs.get("page_range")
            assert isinstance(page_range, tuple)
            assert len(page_range) == 2
            start, end = page_range
            assert isinstance(start, int)
            assert isinstance(end, int)
            self.calls.append((start, end))
            return FakeDoclingResult(f"OCR page {start}\n")

    converter = PageRangeConverter()
    monkeypatch.setenv(PDF_OCR_PREWARM_PROFILES_ENV, PDF_OCR_FAST_TEXT_PROFILE)
    monkeypatch.setenv(PDF_OCR_PREWARM_SOURCE_PATH_ENV, str(source))
    monkeypatch.setenv("WENDAO_PDF_OCR_PREWARM_PAGE_INDICES", "5,6")
    worker = DoclingPdfOcrShardWorker(
        converter_factory=lambda _profile: converter,
        max_workers=1,
    )

    table = build_pdf_ocr_shard_result_table(
        pa.concat_tables(
            [
                _sample_pdf_ocr_input_table(
                    source_path=str(source),
                    page_index=5,
                    shard_element_id="shard-5",
                    ocr_profile=PDF_OCR_FAST_TEXT_PROFILE,
                ),
                _sample_pdf_ocr_input_table(
                    source_path=str(source),
                    page_index=6,
                    shard_element_id="shard-6",
                    ocr_profile=PDF_OCR_FAST_TEXT_PROFILE,
                ),
            ]
        ),
        worker=worker,
    )

    rows = table.to_pylist()
    assert converter.calls == [(6, 6), (7, 7)]
    assert [row["text"] for row in rows] == ["OCR page 6\n", "OCR page 7\n"]


def test_docling_pdf_ocr_worker_rejects_stale_prewarmed_source_page(
    monkeypatch,
    tmp_path: Path,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture v1")

    class PageRangeConverter:
        def __init__(self) -> None:
            self.calls: list[tuple[int, int]] = []

        def convert(
            self, source_path: str | Path, **kwargs: object
        ) -> FakeDoclingResult:
            assert Path(source_path) == source
            page_range = kwargs.get("page_range")
            assert isinstance(page_range, tuple)
            start, end = page_range
            assert isinstance(start, int)
            assert isinstance(end, int)
            self.calls.append((start, end))
            return FakeDoclingResult(f"OCR page {start}\n")

    converter = PageRangeConverter()
    monkeypatch.setenv(PDF_OCR_PREWARM_PROFILES_ENV, PDF_OCR_FAST_TEXT_PROFILE)
    monkeypatch.setenv(PDF_OCR_PREWARM_SOURCE_PATH_ENV, str(source))
    monkeypatch.setenv("WENDAO_PDF_OCR_PREWARM_PAGE_INDICES", "5")
    worker = DoclingPdfOcrShardWorker(
        converter_factory=lambda _profile: converter,
        max_workers=1,
    )
    source.write_bytes(b"%PDF fixture v2 with different size")

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(
            source_path=str(source),
            page_index=5,
            shard_element_id="shard-5",
            ocr_profile=PDF_OCR_FAST_TEXT_PROFILE,
        ),
        worker=worker,
    )

    assert converter.calls == [(6, 6), (6, 6)]
    assert table.to_pylist()[0]["text"] == "OCR page 6\n"
