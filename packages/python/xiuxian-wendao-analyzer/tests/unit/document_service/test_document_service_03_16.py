"""document_service test slice 3."""

from __future__ import annotations

import json

from xiuxian_wendao_analyzer.pdf_ocr import (
    HOSTED_VLM_OCR_API_KEY_ENV,
    HOSTED_VLM_OCR_BASE_URL_ENV,
    HOSTED_VLM_OCR_MODEL_ENV,
    HOSTED_VLM_OCR_TRACE_PATH_ENV,
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


def test_docling_pdf_ocr_worker_writes_hosted_vlm_ocr_request_trace(
    tmp_path: Path,
    monkeypatch,
) -> None:
    image = tmp_path / "page-00000.png"
    image.write_bytes(b"png fixture")
    trace_path = tmp_path / "hosted-vlm-ocr.jsonl"

    class FakeResponse:
        status = 200

        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps(
                {"choices": [{"message": {"content": "# traced\n"}}]}
            ).encode("utf-8")

    monkeypatch.setenv(HOSTED_VLM_OCR_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(HOSTED_VLM_OCR_MODEL_ENV, "community/hosted-vlm-awq")
    monkeypatch.setenv(HOSTED_VLM_OCR_API_KEY_ENV, "secret-key")
    monkeypatch.setenv(HOSTED_VLM_OCR_TRACE_PATH_ENV, str(trace_path))
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        lambda request, *, timeout: FakeResponse(),
    )

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(
            image_path=str(image),
            page_index=7,
            shard_element_id="hosted-vlm-trace-shard",
            ocr_profile=PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
        ),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert table.to_pylist()[0]["text"] == "# traced\n"
    trace_text = trace_path.read_text(encoding="utf-8")
    assert "secret-key" not in trace_text
    records = [json.loads(line) for line in trace_text.splitlines()]
    assert records == [
        {
            "endpoint": "http://127.0.0.1:8999/v1/chat/completions",
            "endedUnixMs": records[0]["endedUnixMs"],
            "errorMessage": None,
            "errorType": None,
            "httpAttemptCount": 1,
            "httpStatus": 200,
            "imageBytes": len(b"png fixture"),
            "imageOptimizationMode": "disabled",
            "latencyMs": records[0]["latencyMs"],
            "markdownChars": len("# traced\n"),
            "maxTokens": 8192,
            "model": "community/hosted-vlm-awq",
            "ocrProfile": PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
            "parentShardElementId": "",
            "pageCount": 1,
            "pageIndex": 7,
            "rasterHeightPx": 3100,
            "rasterWidthPx": 2400,
            "readingOrderKey": "000000.000000",
            "regionIndex": 0,
            "renderDpi": 300,
            "requestKind": "page",
            "schema": "xiuxian_wendao.hosted_vlm_ocr_request_trace.v1",
            "canonicalMarkdownChars": 0,
            "scaffoldAppliedCount": 0,
            "scaffoldJsonChars": 0,
            "scaffoldMode": "disabled",
            "scaffoldValidationFailureCount": 0,
            "shardCount": 1,
            "shardElementId": "hosted-vlm-trace-shard",
            "shardType": "page",
            "shardTypeCounts": {"page": 1},
            "sourcePagePixelBottom": 3100,
            "sourcePagePixelLeft": 0,
            "sourcePagePixelRight": 2400,
            "sourcePagePixelTop": 0,
            "sourcePixelArea": 7_440_000,
            "startedUnixMs": records[0]["startedUnixMs"],
            "status": "succeeded",
            "timestampUnixMs": records[0]["timestampUnixMs"],
        }
    ]
    assert records[0]["latencyMs"] >= 0
    assert records[0]["startedUnixMs"] <= records[0]["endedUnixMs"]
    assert records[0]["timestampUnixMs"] == records[0]["endedUnixMs"]
    assert records[0]["timestampUnixMs"] > 0
