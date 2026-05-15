"""document_service test slice 3."""

from __future__ import annotations

import json

from .support import (
    DoclingPdfOcrShardWorker,
    FakeDoclingResult,
    Path,
    _sample_pdf_ocr_input_table,
    build_pdf_ocr_shard_result_table,
    pa,
    threading,
    time,
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
