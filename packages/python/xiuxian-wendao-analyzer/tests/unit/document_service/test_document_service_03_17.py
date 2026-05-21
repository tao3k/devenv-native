"""document_service test slice 3."""

from __future__ import annotations

import json

from xiuxian_wendao_analyzer.pdf_ocr import (
    HOSTED_VLM_OCR_BASE_URL_ENV,
    HOSTED_VLM_OCR_DEFAULT_MAX_TOKENS,
    HOSTED_VLM_OCR_DEFAULT_REGION_MAX_TOKENS,
    HOSTED_VLM_OCR_MAX_TOKENS_ENV,
    HOSTED_VLM_OCR_REGION_MAX_TOKENS_ENV,
    PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
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


def test_docling_pdf_ocr_worker_caps_region_ocr2_tokens(
    tmp_path: Path,
    monkeypatch,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    page_image = tmp_path / "page-00000.png"
    region_image = tmp_path / "region-00000.png"
    page_image.write_bytes(b"page png fixture")
    region_image.write_bytes(b"region png fixture")
    payloads: list[dict[str, object]] = []

    class FakeResponse:
        status = 200

        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps(
                {"choices": [{"message": {"content": "# OCR2\n"}}]}
            ).encode("utf-8")

    def fake_urlopen(request: object, *, timeout: float) -> FakeResponse:
        _ = timeout
        payloads.append(json.loads(request.data.decode("utf-8")))
        return FakeResponse()

    monkeypatch.delenv(HOSTED_VLM_OCR_MAX_TOKENS_ENV, raising=False)
    monkeypatch.delenv(HOSTED_VLM_OCR_REGION_MAX_TOKENS_ENV, raising=False)
    monkeypatch.setenv(HOSTED_VLM_OCR_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        fake_urlopen,
    )

    table = build_pdf_ocr_shard_result_table(
        pa.concat_tables(
            [
                _sample_pdf_ocr_input_table(
                    source_path=str(source),
                    image_path=str(page_image),
                    page_index=0,
                    shard_element_id="hosted-vlm-page",
                    ocr_profile=PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
                ),
                _sample_pdf_ocr_input_table(
                    source_path=str(source),
                    image_path=str(region_image),
                    page_index=0,
                    shard_element_id="hosted-vlm-region",
                    shard_type="region",
                    region_index=1,
                    parent_shard_element_id="hosted-vlm-page",
                    ocr_profile=PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
                ),
            ]
        ),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert [row["status"] for row in table.to_pylist()] == ["succeeded", "succeeded"]
    assert [payload["max_tokens"] for payload in payloads] == [
        HOSTED_VLM_OCR_DEFAULT_MAX_TOKENS,
        HOSTED_VLM_OCR_DEFAULT_REGION_MAX_TOKENS,
    ]
