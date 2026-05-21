"""document_service test slice 3."""

from __future__ import annotations

import json

from xiuxian_wendao_analyzer.pdf_ocr import (
    HOSTED_VLM_OCR_BASE_URL_ENV,
    HOSTED_VLM_OCR_MODEL_ENV,
    HOSTED_VLM_OCR_PAGE_WINDOW_SIZE_ENV,
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


def test_docling_pdf_ocr_worker_batches_direct_ocr2_page_window(
    tmp_path: Path,
    monkeypatch,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    image_paths = [tmp_path / f"page-{page_index:05}.png" for page_index in range(2)]
    for page_index, image_path in enumerate(image_paths):
        image_path.write_bytes(f"png fixture {page_index}".encode())
    requests: list[object] = []

    class FakeResponse:
        status = 200

        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps(
                {
                    "choices": [
                        {
                            "message": {
                                "content": (
                                    "<!-- xiuxian-wendao-hosted-vlm-page:0 -->\n"
                                    "# Page zero\n"
                                    "<!-- xiuxian-wendao-hosted-vlm-page:1 -->\n"
                                    "# Page one\n"
                                )
                            }
                        }
                    ]
                }
            ).encode("utf-8")

    def fake_urlopen(request: object, *, timeout: float) -> FakeResponse:
        _ = timeout
        requests.append(request)
        return FakeResponse()

    monkeypatch.setenv(HOSTED_VLM_OCR_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(HOSTED_VLM_OCR_MODEL_ENV, "unit/window-success")
    monkeypatch.setenv(HOSTED_VLM_OCR_PAGE_WINDOW_SIZE_ENV, "2")
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        fake_urlopen,
    )
    input_table = pa.concat_tables(
        [
            _sample_pdf_ocr_input_table(
                source_path=str(source),
                image_path=str(image_path),
                page_index=page_index,
                shard_element_id=f"hosted-vlm-shard-{page_index}",
                ocr_profile=PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
            )
            for page_index, image_path in enumerate(image_paths)
        ]
    )

    table = build_pdf_ocr_shard_result_table(
        input_table,
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert [row["text"] for row in table.to_pylist()] == [
        "# Page zero",
        "# Page one",
    ]
    assert len(requests) == 1
    payload = json.loads(requests[0].data.decode("utf-8"))
    content = payload["messages"][0]["content"]
    assert sum(1 for part in content if part["type"] == "image_url") == 2
    assert "<!-- xiuxian-wendao-hosted-vlm-page:0 -->" in content[0]["text"]
    assert "<!-- xiuxian-wendao-hosted-vlm-page:1 -->" in content[0]["text"]
