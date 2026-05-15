"""document_service test slice 3."""

from __future__ import annotations

import json

from .support import (
    DoclingPdfOcrShardWorker,
    FailingDoclingConverter,
    FakeDoclingConverter,
    Path,
    _sample_pdf_ocr_input_table,
    build_pdf_ocr_shard_result_table,
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
