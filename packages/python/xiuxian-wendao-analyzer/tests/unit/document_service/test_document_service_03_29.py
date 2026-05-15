"""document_service test slice 3."""

from __future__ import annotations

import json

from xiuxian_wendao_analyzer.pdf_ocr import (
    HOSTED_VLM_OCR_BASE_URL_ENV,
    HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE_ENV,
    HOSTED_VLM_OCR_SCAFFOLD_MODE_ENV,
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


def test_docling_pdf_ocr_worker_uses_composite_region_scaffold_json(
    tmp_path: Path,
    monkeypatch,
) -> None:
    images = [tmp_path / f"region-{index:05}.png" for index in range(2)]
    for image in images:
        image.write_bytes(b"region png fixture")
    input_table = pa.concat_tables(
        [
            _sample_pdf_ocr_input_table(
                image_path=str(images[index]),
                shard_element_id=f"region-{index}",
                shard_type="region",
                region_index=index + 1,
                parent_shard_element_id="parent-page",
                ocr_profile=PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
            )
            for index in range(2)
        ]
    )
    input_rows = input_table.to_pylist()
    _write_ocr2_region_scaffold_sidecar(tmp_path, input_rows)
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
                                "content": json.dumps(
                                    {
                                        "regions": [
                                            {
                                                "marker": _ocr2_region_marker(row),
                                                "shardElementId": row["shardElementId"],
                                                "content": f"region text {index}",
                                                "tables": [],
                                            }
                                            for index, row in enumerate(input_rows)
                                        ]
                                    }
                                )
                            }
                        }
                    ]
                }
            ).encode("utf-8")

    monkeypatch.setenv(HOSTED_VLM_OCR_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE_ENV, "2")
    monkeypatch.setenv(HOSTED_VLM_OCR_SCAFFOLD_MODE_ENV, "region-table-json")
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        lambda request, *, timeout: requests.append(request) or FakeResponse(),
    )

    table = build_pdf_ocr_shard_result_table(
        input_table,
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert [row["text"] for row in table.to_pylist()] == [
        "region text 0",
        "region text 1",
    ]
    assert len(requests) == 1
    payload = json.loads(requests[0].data.decode("utf-8"))
    assert (
        sum(
            1
            for part in payload["messages"][0]["content"]
            if part["type"] == "image_url"
        )
        == 2
    )
