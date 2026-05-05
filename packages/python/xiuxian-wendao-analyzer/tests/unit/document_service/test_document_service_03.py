"""document_service test slice 3."""

from __future__ import annotations

from xiuxian_wendao_analyzer.pdf_ocr import (
    PDF_OCR_DEFAULT_PROFILE,
    PDF_OCR_FAST_TEXT_PROFILE,
)

from .support import (
    DoclingPdfOcrShardWorker,
    FailingDoclingConverter,
    FakeDoclingConverter,
    FakeDoclingResult,
    Path,
    _sample_pdf_ocr_input_table,
    build_pdf_ocr_shard_result_table,
    pa,
    threading,
    time,
)


def test_docling_pdf_ocr_worker_uses_single_page_break_export_for_ranges(
    tmp_path: Path,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    export_calls: list[dict[str, object]] = []

    class PageBreakDocument:
        def export_to_markdown(self, **kwargs: object) -> str:
            export_calls.append(dict(kwargs))
            if "page_break_placeholder" in kwargs:
                separator = str(kwargs["page_break_placeholder"])
                return separator.join(["OCR page 1", "OCR page 2", "OCR page 3"])
            page_no = kwargs.get("page_no")
            return f"fallback page {page_no}\n"

    class PageBreakResult:
        document = PageBreakDocument()

    class PageBreakConverter(FakeDoclingConverter):
        def convert(self, source: str | Path, **kwargs: object) -> PageBreakResult:
            self.calls.append(Path(source))
            self.kwargs_calls.append(dict(kwargs))
            return PageBreakResult()

    converter = PageBreakConverter()
    input_tables = [
        _sample_pdf_ocr_input_table(
            source_path=str(source),
            image_path=str(tmp_path / f"page-{page_index:05}.png"),
            page_index=page_index,
            shard_element_id=f"shard-{page_index}",
        )
        for page_index in range(3)
    ]

    table = build_pdf_ocr_shard_result_table(
        pa.concat_tables(input_tables),
        worker=DoclingPdfOcrShardWorker(converter, max_workers=4),
    )

    assert converter.calls == [source]
    assert converter.kwargs_calls == [{"page_range": (1, 3)}]
    assert export_calls == [
        {"page_break_placeholder": "<!-- xiuxian-wendao-pdf-ocr-page-break -->"}
    ]
    assert [row["text"] for row in table.to_pylist()] == [
        "OCR page 1",
        "OCR page 2",
        "OCR page 3",
    ]


def test_docling_pdf_ocr_worker_keeps_profile_ranges_separate(tmp_path: Path) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")

    converter = FakeDoclingConverter("OCR\n")
    table = build_pdf_ocr_shard_result_table(
        pa.concat_tables(
            [
                _sample_pdf_ocr_input_table(
                    source_path=str(source),
                    page_index=0,
                    ocr_profile=PDF_OCR_DEFAULT_PROFILE,
                ),
                _sample_pdf_ocr_input_table(
                    source_path=str(source),
                    page_index=1,
                    shard_element_id="shard-1",
                    ocr_profile=PDF_OCR_FAST_TEXT_PROFILE,
                ),
            ]
        ),
        worker=DoclingPdfOcrShardWorker(converter, max_workers=4),
    )

    assert converter.kwargs_calls == [{"page_range": (1, 1)}, {"page_range": (2, 2)}]
    assert [row["ocrProfile"] for row in table.to_pylist()] == [
        PDF_OCR_DEFAULT_PROFILE,
        PDF_OCR_FAST_TEXT_PROFILE,
    ]


def test_docling_pdf_ocr_worker_uses_fast_converter_for_fast_profile(
    tmp_path: Path,
    monkeypatch,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    requested_profiles: list[str] = []

    def fake_converter_factory(profile: str) -> FakeDoclingConverter:
        requested_profiles.append(profile)
        return FakeDoclingConverter(f"OCR {profile}\n")

    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_workers._new_docling_converter",
        fake_converter_factory,
    )

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(
            source_path=str(source),
            ocr_profile=PDF_OCR_FAST_TEXT_PROFILE,
        ),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert requested_profiles == [PDF_OCR_FAST_TEXT_PROFILE]
    assert table.to_pylist()[0]["text"] == f"OCR {PDF_OCR_FAST_TEXT_PROFILE}\n"


def test_docling_pdf_ocr_worker_passes_profile_to_converter_factory(
    tmp_path: Path,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    requested_profiles: list[str] = []

    def fake_converter_factory(profile: str) -> FakeDoclingConverter:
        requested_profiles.append(profile)
        return FakeDoclingConverter(f"OCR {profile}\n")

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(
            source_path=str(source),
            ocr_profile=PDF_OCR_FAST_TEXT_PROFILE,
        ),
        worker=DoclingPdfOcrShardWorker(
            converter_factory=fake_converter_factory,
            max_workers=1,
        ),
    )

    assert requested_profiles == [PDF_OCR_FAST_TEXT_PROFILE]
    assert table.to_pylist()[0]["text"] == f"OCR {PDF_OCR_FAST_TEXT_PROFILE}\n"


def test_docling_pdf_ocr_worker_preserves_order_with_concurrent_shards(
    tmp_path: Path,
) -> None:
    records: list[tuple[int, str]] = []
    records_lock = threading.Lock()

    class ThreadLocalConverter:
        def convert(self, source: str | Path, **kwargs: object) -> FakeDoclingResult:
            _ = kwargs
            time.sleep(0.005)
            with records_lock:
                records.append((threading.get_ident(), Path(source).name))
            return FakeDoclingResult(f"OCR {Path(source).stem}\n")

    tables = []
    for page_index in range(20):
        image = tmp_path / f"page-{page_index:05}.png"
        image.write_bytes(b"png fixture")
        tables.append(
            _sample_pdf_ocr_input_table(
                source_path=str(tmp_path / "missing-source.pdf"),
                image_path=str(image),
                page_index=page_index,
                shard_element_id=f"shard-{page_index}",
            )
        )

    table = build_pdf_ocr_shard_result_table(
        pa.concat_tables(tables),
        worker=DoclingPdfOcrShardWorker(
            converter_factory=ThreadLocalConverter,
            max_workers=4,
        ),
    )

    rows = table.to_pylist()
    assert [row["pageIndex"] for row in rows] == list(range(20))
    assert [row["text"] for row in rows] == [
        f"OCR page-{page_index:05}\n" for page_index in range(20)
    ]
    assert len({thread_id for thread_id, _ in records}) > 1


def test_docling_pdf_ocr_worker_failure_isolated_per_shard(tmp_path: Path) -> None:
    class PartiallyFailingConverter:
        def convert(self, source: str | Path, **kwargs: object) -> FakeDoclingResult:
            _ = kwargs
            if Path(source).name == "page-00001.png":
                raise RuntimeError("selected shard failed")
            return FakeDoclingResult(f"OCR {Path(source).stem}\n")

    tables = []
    for page_index in range(3):
        image = tmp_path / f"page-{page_index:05}.png"
        image.write_bytes(b"png fixture")
        tables.append(
            _sample_pdf_ocr_input_table(
                source_path=str(tmp_path / "missing-source.pdf"),
                image_path=str(image),
                page_index=page_index,
                shard_element_id=f"shard-{page_index}",
            )
        )

    table = build_pdf_ocr_shard_result_table(
        pa.concat_tables(tables),
        worker=DoclingPdfOcrShardWorker(
            converter_factory=PartiallyFailingConverter,
            max_workers=3,
        ),
    )

    rows = table.to_pylist()
    assert [row["status"] for row in rows] == ["succeeded", "failed", "succeeded"]
    assert "Docling OCR failed" in rows[1]["errorMessage"]


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
