"""document_service test slice 3."""

from __future__ import annotations

import json

from xiuxian_wendao_analyzer.pdf_ocr import (
    HOSTED_VLM_OCR_BASE_URL_ENV,
    HOSTED_VLM_OCR_MODEL_ENV,
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


def test_docling_pdf_ocr_worker_uses_hosted_vlm_ocr_openai_endpoint(
    tmp_path: Path,
    monkeypatch,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    image = tmp_path / "page-00000.png"
    image.write_bytes(b"png fixture")
    requests: list[object] = []

    class FakeResponse:
        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps(
                {"choices": [{"message": {"content": "# OCR2\n\ntext"}}]}
            ).encode("utf-8")

    def fake_urlopen(request: object, *, timeout: float) -> FakeResponse:
        _ = timeout
        requests.append(request)
        return FakeResponse()

    monkeypatch.setenv(HOSTED_VLM_OCR_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(HOSTED_VLM_OCR_MODEL_ENV, "community/hosted-vlm-awq")
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        fake_urlopen,
    )

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(
            source_path=str(source),
            image_path=str(image),
            ocr_profile=PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
        ),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert table.to_pylist()[0]["text"] == "# OCR2\n\ntext"
    request = requests[0]
    assert request.full_url == "http://127.0.0.1:8999/v1/chat/completions"
    payload = json.loads(request.data.decode("utf-8"))
    assert payload["model"] == "community/hosted-vlm-awq"
    assert payload["messages"][0]["content"][1]["image_url"]["url"].startswith(
        "data:image/png;base64,"
    )
