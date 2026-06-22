"""document_service test slice 3."""

from __future__ import annotations

import json
from typing import TYPE_CHECKING

from xiuxian_wendao_analyzer.pdf_ocr_workers import (
    PDF_OCR_FAST_TEXT_DEFAULT_THREADS,
    PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_BACKEND_TABLE,
    PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_DEFAULT,
    PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_ENV,
    PDF_OCR_FAST_TEXT_THREADS_ENV,
    _fast_text_accelerator_threads_with_lookup,
    fast_text_source_converter_mode_with_lookup,
)

if TYPE_CHECKING:
    from .support import Path


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


def test_fast_text_docling_threads_default_to_rust_scheduler_friendly_single_thread() -> (
    None
):
    assert _fast_text_accelerator_threads_with_lookup(lambda _key: None) == 1
    assert PDF_OCR_FAST_TEXT_DEFAULT_THREADS == 1
    assert (
        _fast_text_accelerator_threads_with_lookup(
            lambda key: "3" if key == PDF_OCR_FAST_TEXT_THREADS_ENV else None
        )
        == 3
    )
    assert (
        _fast_text_accelerator_threads_with_lookup(
            lambda key: "0" if key == PDF_OCR_FAST_TEXT_THREADS_ENV else None
        )
        == 1
    )


def test_fast_text_source_converter_mode_accepts_only_bounded_modes() -> None:
    assert (
        fast_text_source_converter_mode_with_lookup(lambda _key: None)
        == PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_DEFAULT
    )
    assert (
        fast_text_source_converter_mode_with_lookup(
            lambda key: (
                "backend_table"
                if key == PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_ENV
                else None
            )
        )
        == PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_BACKEND_TABLE
    )
    assert (
        fast_text_source_converter_mode_with_lookup(
            lambda key: (
                "invalid" if key == PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_ENV else None
            )
        )
        == PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_DEFAULT
    )
