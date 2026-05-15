"""document_service test slice 3."""

from __future__ import annotations

import json

from .support import (
    DoclingPdfOcrShardWorker,
    FakeDoclingConverter,
    Path,
    _sample_pdf_ocr_input_table,
    build_pdf_ocr_shard_result_table,
    pa,
)


def _ocr2_region_marker(row: dict[str, object]) -> str:
    return (
        "<!-- xiuxian-wendao-hosted-vlm-region:"
        f"{row['pageIndex']}:{row['regionIndex']}:{row['shardElementId']}"
        " -->"
    )


def _write_ppm_image(path: Path, color: bytes = b"\x00\x00\x00") -> None:
    path.write_bytes(b"P6\n2 2\n255\n" + color * 4)


def _write_ocr2_region_scaffold_sidecar(
    directory: Path,
    rows: list[dict[str, object]],
    *,
    raster_sha256: str = "rasterhash",
) -> None:
    items = []
    for row in rows:
        items.append(
            {
                "scaffoldKind": "table_candidate",
                "shardElementId": row["shardElementId"],
                "parentShardElementId": row["parentShardElementId"],
                "pageIndex": row["pageIndex"],
                "regionIndex": row["regionIndex"],
                "sourceContentHash": row["sourceContentHash"],
                "rasterSha256": raster_sha256,
                "renderDpi": row["renderDpi"],
                "cropBox": {
                    "left": row["cropLeft"],
                    "bottom": row["cropBottom"],
                    "right": row["cropRight"],
                    "top": row["cropTop"],
                },
                "sourcePagePixelBox": {
                    "left": row["sourcePagePixelLeft"],
                    "top": row["sourcePagePixelTop"],
                    "right": row["sourcePagePixelRight"],
                    "bottom": row["sourcePagePixelBottom"],
                },
                "sourcePageProfile": None,
            }
        )
    (directory / "_hosted_vlm_region_scaffolds.json").write_text(
        json.dumps(
            {
                "schema": "xiuxian_wendao.hosted_vlm_region_scaffold.v1",
                "mode": "region-table-json",
                "items": items,
            }
        ),
        encoding="utf-8",
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
