"""document_service test slice 3."""

from __future__ import annotations

import json
import urllib.error

from xiuxian_wendao_analyzer.pdf_ocr import (
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


def test_docling_pdf_ocr_worker_retries_transient_failed_group_rows(
    tmp_path: Path,
    monkeypatch,
) -> None:
    images = [tmp_path / f"page-{page_index:05}.png" for page_index in range(2)]
    for image in images:
        image.write_bytes(b"png fixture")
    requests: list[object] = []
    sleeps: list[float] = []

    class FakeResponse:
        status = 200

        def __init__(self, text: str) -> None:
            self._text = text

        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps(
                {"choices": [{"message": {"content": self._text}}]}
            ).encode("utf-8")

    def fake_urlopen(request: object, *, timeout: float) -> FakeResponse:
        _ = timeout
        requests.append(request)
        if len(requests) <= 4:
            raise urllib.error.HTTPError(
                url="http://127.0.0.1:8999/v1/chat/completions",
                code=502,
                msg="Bad Gateway",
                hdrs={},
                fp=None,
            )
        return FakeResponse(f"retry ok {len(requests)}")

    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        fake_urlopen,
    )
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.sleep",
        lambda seconds: sleeps.append(seconds),
    )

    table = build_pdf_ocr_shard_result_table(
        pa.concat_tables(
            [
                _sample_pdf_ocr_input_table(
                    image_path=str(images[0]),
                    page_index=0,
                    ocr_profile=PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
                ),
                _sample_pdf_ocr_input_table(
                    image_path=str(images[1]),
                    page_index=1,
                    shard_element_id="shard-1",
                    ocr_profile=PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
                ),
            ]
        ),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    rows = table.to_pylist()
    assert [row["status"] for row in rows] == ["succeeded", "succeeded"]
    assert [row["text"] for row in rows] == ["retry ok 6", "retry ok 5"]
    assert len(requests) == 6
    assert sleeps == [0.25, 0.5, 1.0, 8.0]
