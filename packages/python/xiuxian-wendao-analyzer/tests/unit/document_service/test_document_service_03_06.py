"""document_service test slice 3."""

from __future__ import annotations

import json

from xiuxian_wendao_analyzer.pdf_ocr import (
    PDF_OCR_BACKEND_TEXT_PROFILE,
    PDF_OCR_DEFAULT_PROFILE,
    PDF_OCR_FAST_TEXT_PROFILE,
)
from xiuxian_wendao_analyzer.pdf_ocr_workers import (
    PDF_OCR_BACKEND_TEXT_PAGE_FALLBACK_COMPATIBLE,
    PDF_OCR_BACKEND_TEXT_PAGE_FALLBACK_ENV,
)

from .support import (
    DoclingPdfOcrShardWorker,
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


def test_backend_text_source_range_canary_topups_empty_pages_with_compatible_page(
    tmp_path: Path,
    monkeypatch,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    requested_profiles: list[str] = []

    class BackendDocument:
        def export_to_markdown(self, **kwargs: object) -> str:
            if "page_break_placeholder" in kwargs:
                separator = str(kwargs["page_break_placeholder"])
                return separator.join(["backend page 1", ""])
            return ""

    class EmptyDocument:
        def export_to_markdown(self, **kwargs: object) -> str:
            _ = kwargs
            return ""

    class CompatibleDocument:
        def export_to_markdown(self, **kwargs: object) -> str:
            _ = kwargs
            return "compatible page 2"

    class Result:
        def __init__(self, document: object) -> None:
            self.document = document

    class Converter:
        def __init__(self, profile: str) -> None:
            self.profile = profile

        def convert(self, source: str | Path, **kwargs: object) -> Result:
            _ = source, kwargs
            if self.profile == PDF_OCR_BACKEND_TEXT_PROFILE:
                return Result(BackendDocument())
            if self.profile == PDF_OCR_DEFAULT_PROFILE:
                return Result(CompatibleDocument())
            return Result(EmptyDocument())

    def converter_factory(profile: str) -> Converter:
        requested_profiles.append(profile)
        return Converter(profile)

    monkeypatch.setenv(
        PDF_OCR_BACKEND_TEXT_PAGE_FALLBACK_ENV,
        PDF_OCR_BACKEND_TEXT_PAGE_FALLBACK_COMPATIBLE,
    )
    input_tables = [
        _sample_pdf_ocr_input_table(
            source_path=str(source),
            page_index=page_index,
            shard_element_id=f"shard-{page_index}",
            ocr_profile=PDF_OCR_BACKEND_TEXT_PROFILE,
        )
        for page_index in range(2)
    ]

    table = build_pdf_ocr_shard_result_table(
        pa.concat_tables(input_tables),
        worker=DoclingPdfOcrShardWorker(
            converter_factory=converter_factory,
            max_workers=1,
        ),
    )

    assert requested_profiles == [
        PDF_OCR_BACKEND_TEXT_PROFILE,
        PDF_OCR_FAST_TEXT_PROFILE,
        PDF_OCR_DEFAULT_PROFILE,
    ]
    assert [row["text"] for row in table.to_pylist()] == [
        "backend page 1",
        "compatible page 2",
    ]
