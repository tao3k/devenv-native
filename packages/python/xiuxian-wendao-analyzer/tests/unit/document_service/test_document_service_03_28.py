"""document_service test slice 3."""

from __future__ import annotations

import json

from xiuxian_wendao_analyzer.pdf_ocr import (
    HOSTED_VLM_OCR_BASE_URL_ENV,
    HOSTED_VLM_OCR_DEFAULT_REGION_MAX_TOKENS,
    HOSTED_VLM_OCR_SCAFFOLD_MODE_ENV,
    PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
)

from .support import (
    DoclingPdfOcrShardWorker,
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


def test_docling_pdf_ocr_worker_uses_region_scaffold_json(
    tmp_path: Path,
    monkeypatch,
) -> None:
    image = tmp_path / "ocr-shards" / "region-hash" / "region-00001.png"
    image.parent.mkdir(parents=True)
    image.write_bytes(b"region png fixture")
    requests: list[object] = []
    input_table = _sample_pdf_ocr_input_table(
        image_path=str(image),
        shard_element_id="region-a",
        shard_type="region",
        region_index=1,
        parent_shard_element_id="parent-page",
        ocr_profile=PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
    )
    input_row = input_table.to_pylist()[0]
    _write_ocr2_region_scaffold_sidecar(tmp_path, [input_row])

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
                                "content": json.dumps(
                                    {
                                        "regions": [
                                            {
                                                "marker": _ocr2_region_marker(
                                                    input_row
                                                ),
                                                "shardElementId": "region-a",
                                                "text": "Table title",
                                                "tables": [
                                                    {
                                                        "rows": [
                                                            ["A", "B"],
                                                            ["1", "2"],
                                                        ]
                                                    }
                                                ],
                                            }
                                        ]
                                    }
                                )
                            }
                        }
                    ]
                }
            ).encode("utf-8")

    monkeypatch.setenv(HOSTED_VLM_OCR_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(HOSTED_VLM_OCR_SCAFFOLD_MODE_ENV, "region-table-json")
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        lambda request, *, timeout: requests.append(request) or FakeResponse(),
    )

    table = build_pdf_ocr_shard_result_table(
        input_table,
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    row = table.to_pylist()[0]
    assert row["status"] == "succeeded"
    assert row["text"] == "Table title\n\n| A | B |\n| --- | --- |\n| 1 | 2 |"
    payload = json.loads(requests[0].data.decode("utf-8"))
    prompt_text = payload["messages"][0]["content"][0]["text"]
    assert "Return JSON only" in prompt_text
    assert "Every region must contain non-empty recognized content" in prompt_text
    assert '"scaffoldKind": "table_candidate"' in prompt_text
    assert _ocr2_region_marker(input_row) in prompt_text
    assert payload["max_tokens"] == HOSTED_VLM_OCR_DEFAULT_REGION_MAX_TOKENS
