"""document_service test slice 3."""

from __future__ import annotations

import json

from xiuxian_wendao_analyzer.pdf_ocr import (
    PDF_OCR_DEFAULT_PROFILE,
    PDF_OCR_FAST_TEXT_PROFILE,
)

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
