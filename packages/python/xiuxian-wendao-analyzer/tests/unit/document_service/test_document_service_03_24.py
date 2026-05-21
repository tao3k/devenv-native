"""document_service test slice 3."""

from __future__ import annotations

import json

from xiuxian_wendao_analyzer.pdf_ocr import (
    HOSTED_VLM_OCR_BASE_URL_ENV,
    HOSTED_VLM_OCR_REGION_ATLAS_MODE_ENV,
    HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE_ENV,
    HOSTED_VLM_OCR_TRACE_PATH_ENV,
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


def test_docling_pdf_ocr_worker_uses_region_atlas_json(
    tmp_path: Path,
    monkeypatch,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    region_images = [tmp_path / f"region-{index:05}.png" for index in range(2)]
    _write_ppm_image(region_images[0], b"\xff\x00\x00")
    _write_ppm_image(region_images[1], b"\x00\x00\xff")
    trace_path = tmp_path / "hosted-vlm-region-atlas.jsonl"
    requests: list[object] = []

    input_table = pa.concat_tables(
        [
            _sample_pdf_ocr_input_table(
                source_path=str(source),
                image_path=str(region_images[index]),
                page_index=0,
                shard_element_id=f"region-{index}",
                shard_type="region",
                region_index=index + 1,
                parent_shard_element_id="parent-page",
                reading_order_key=f"000000.0{index + 1}0000",
                ocr_profile=PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
            )
            for index in range(2)
        ]
    )
    input_rows = input_table.to_pylist()

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
                                                "panel": f"REGION {index + 1}",
                                                "marker": _ocr2_region_marker(row),
                                                "shardElementId": row["shardElementId"],
                                                "text": f"atlas text {index}",
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

    def fake_urlopen(request: object, *, timeout: float) -> FakeResponse:
        _ = timeout
        requests.append(request)
        return FakeResponse()

    monkeypatch.setenv(HOSTED_VLM_OCR_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE_ENV, "2")
    monkeypatch.setenv(HOSTED_VLM_OCR_REGION_ATLAS_MODE_ENV, "same-page-json")
    monkeypatch.setenv(HOSTED_VLM_OCR_TRACE_PATH_ENV, str(trace_path))
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        fake_urlopen,
    )

    table = build_pdf_ocr_shard_result_table(
        input_table,
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert [row["text"] for row in table.to_pylist()] == [
        "atlas text 0",
        "atlas text 1",
    ]
    assert len(requests) == 1
    payload = json.loads(requests[0].data.decode("utf-8"))
    content = payload["messages"][0]["content"]
    assert sum(1 for part in content if part["type"] == "image_url") == 1
    assert "Atlas panel mapping" in content[0]["text"]
    assert '"panel": "REGION 1"' in content[0]["text"]
    assert _ocr2_region_marker(input_rows[0]) in content[0]["text"]
    records = [
        json.loads(line) for line in trace_path.read_text(encoding="utf-8").splitlines()
    ]
    assert records[0]["requestKind"] == "region-atlas"
    assert records[0]["shardCount"] == 2
    assert records[0]["canonicalMarkdownChars"] == len("atlas text 0atlas text 1")
