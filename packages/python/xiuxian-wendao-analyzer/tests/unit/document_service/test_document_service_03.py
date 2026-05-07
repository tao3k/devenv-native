"""document_service test slice 3."""

from __future__ import annotations

import json
import urllib.error

from xiuxian_wendao_analyzer.pdf_ocr import (
    DEEPSEEK_OCR2_API_KEY_ENV,
    DEEPSEEK_OCR2_BASE_URL_ENV,
    DEEPSEEK_OCR2_DEFAULT_MAX_TOKENS,
    DEEPSEEK_OCR2_DEFAULT_REGION_MAX_TOKENS,
    DEEPSEEK_OCR2_MAX_TOKENS_ENV,
    DEEPSEEK_OCR2_MODEL_ENV,
    DEEPSEEK_OCR2_OPENROUTE_COMPAT_API_KEY_ENV,
    DEEPSEEK_OCR2_OPENROUTER_API_KEY_ENV,
    DEEPSEEK_OCR2_OPENROUTER_BASE_URL,
    DEEPSEEK_OCR2_OPENROUTER_HTTP_REFERER_ENV,
    DEEPSEEK_OCR2_OPENROUTER_MODEL_ENV,
    DEEPSEEK_OCR2_OPENROUTER_PROVIDER,
    DEEPSEEK_OCR2_OPENROUTER_TEST_MODEL,
    DEEPSEEK_OCR2_OPENROUTER_TITLE_ENV,
    DEEPSEEK_OCR2_PAGE_WINDOW_SIZE_ENV,
    DEEPSEEK_OCR2_PROVIDER_ENV,
    DEEPSEEK_OCR2_REGION_ATLAS_MODE_ENV,
    DEEPSEEK_OCR2_REGION_COMPOSITE_SIZE_ENV,
    DEEPSEEK_OCR2_REGION_MAX_TOKENS_ENV,
    DEEPSEEK_OCR2_REQUEST_CONCURRENCY_ENV,
    DEEPSEEK_OCR2_SCAFFOLD_MODE_ENV,
    DEEPSEEK_OCR2_TRACE_PATH_ENV,
    PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
    PDF_OCR_DEFAULT_PROFILE,
    PDF_OCR_FAST_TEXT_PROFILE,
)

from .support import (
    DoclingPdfOcrShardWorker,
    FailingDoclingConverter,
    FakeDoclingConverter,
    FakeDoclingResult,
    Path,
    _sample_pdf_ocr_input_table,
    build_pdf_ocr_shard_result_table,
    pa,
    threading,
    time,
)


def _ocr2_region_marker(row: dict[str, object]) -> str:
    return (
        "<!-- xiuxian-wendao-ocr2-region:"
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
    (directory / "_ocr2_region_scaffolds.json").write_text(
        json.dumps(
            {
                "schema": "xiuxian_wendao.ocr2_region_scaffold.v1",
                "mode": "region-table-json",
                "items": items,
            }
        ),
        encoding="utf-8",
    )


def test_docling_pdf_ocr_worker_uses_single_page_break_export_for_ranges(
    tmp_path: Path,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    export_calls: list[dict[str, object]] = []

    class PageBreakDocument:
        def export_to_markdown(self, **kwargs: object) -> str:
            export_calls.append(dict(kwargs))
            if "page_break_placeholder" in kwargs:
                separator = str(kwargs["page_break_placeholder"])
                return separator.join(["OCR page 1", "OCR page 2", "OCR page 3"])
            page_no = kwargs.get("page_no")
            return f"fallback page {page_no}\n"

    class PageBreakResult:
        document = PageBreakDocument()

    class PageBreakConverter(FakeDoclingConverter):
        def convert(self, source: str | Path, **kwargs: object) -> PageBreakResult:
            self.calls.append(Path(source))
            self.kwargs_calls.append(dict(kwargs))
            return PageBreakResult()

    converter = PageBreakConverter()
    input_tables = [
        _sample_pdf_ocr_input_table(
            source_path=str(source),
            image_path=str(tmp_path / f"page-{page_index:05}.png"),
            page_index=page_index,
            shard_element_id=f"shard-{page_index}",
        )
        for page_index in range(3)
    ]

    table = build_pdf_ocr_shard_result_table(
        pa.concat_tables(input_tables),
        worker=DoclingPdfOcrShardWorker(converter, max_workers=4),
    )

    assert converter.calls == [source]
    assert converter.kwargs_calls == [{"page_range": (1, 3)}]
    assert export_calls == [
        {"page_break_placeholder": "<!-- xiuxian-wendao-pdf-ocr-page-break -->"}
    ]
    assert [row["text"] for row in table.to_pylist()] == [
        "OCR page 1",
        "OCR page 2",
        "OCR page 3",
    ]


def test_docling_pdf_ocr_worker_keeps_profile_ranges_separate(tmp_path: Path) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")

    converter = FakeDoclingConverter("OCR\n")
    table = build_pdf_ocr_shard_result_table(
        pa.concat_tables(
            [
                _sample_pdf_ocr_input_table(
                    source_path=str(source),
                    page_index=0,
                    ocr_profile=PDF_OCR_DEFAULT_PROFILE,
                ),
                _sample_pdf_ocr_input_table(
                    source_path=str(source),
                    page_index=1,
                    shard_element_id="shard-1",
                    ocr_profile=PDF_OCR_FAST_TEXT_PROFILE,
                ),
            ]
        ),
        worker=DoclingPdfOcrShardWorker(converter, max_workers=4),
    )

    assert converter.kwargs_calls == [{"page_range": (1, 1)}, {"page_range": (2, 2)}]
    assert [row["ocrProfile"] for row in table.to_pylist()] == [
        PDF_OCR_DEFAULT_PROFILE,
        PDF_OCR_FAST_TEXT_PROFILE,
    ]


def test_docling_pdf_ocr_worker_uses_fast_converter_for_fast_profile(
    tmp_path: Path,
    monkeypatch,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    requested_profiles: list[str] = []

    def fake_converter_factory(profile: str) -> FakeDoclingConverter:
        requested_profiles.append(profile)
        return FakeDoclingConverter(f"OCR {profile}\n")

    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_workers._new_docling_converter",
        fake_converter_factory,
    )

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(
            source_path=str(source),
            ocr_profile=PDF_OCR_FAST_TEXT_PROFILE,
        ),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert requested_profiles == [PDF_OCR_FAST_TEXT_PROFILE]
    assert table.to_pylist()[0]["text"] == f"OCR {PDF_OCR_FAST_TEXT_PROFILE}\n"


def test_docling_pdf_ocr_worker_uses_deepseek_ocr2_openai_endpoint(
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

    monkeypatch.setenv(DEEPSEEK_OCR2_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(DEEPSEEK_OCR2_MODEL_ENV, "community/deepseek-ocr2-awq")
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        fake_urlopen,
    )

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(
            source_path=str(source),
            image_path=str(image),
            ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
        ),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert table.to_pylist()[0]["text"] == "# OCR2\n\ntext"
    request = requests[0]
    assert request.full_url == "http://127.0.0.1:8999/v1/chat/completions"
    payload = json.loads(request.data.decode("utf-8"))
    assert payload["model"] == "community/deepseek-ocr2-awq"
    assert payload["messages"][0]["content"][1]["image_url"]["url"].startswith(
        "data:image/png;base64,"
    )


def test_docling_pdf_ocr_worker_retries_transient_deepseek_ocr2_http_error(
    tmp_path: Path,
    monkeypatch,
) -> None:
    image = tmp_path / "page-00000.png"
    image.write_bytes(b"png fixture")
    requests: list[object] = []
    sleeps: list[float] = []

    class FakeResponse:
        status = 200

        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps(
                {"choices": [{"message": {"content": "retry ok"}}]}
            ).encode("utf-8")

    def fake_urlopen(request: object, *, timeout: float) -> FakeResponse:
        _ = timeout
        requests.append(request)
        if len(requests) == 1:
            raise urllib.error.HTTPError(
                url="http://127.0.0.1:8999/v1/chat/completions",
                code=502,
                msg="Bad Gateway",
                hdrs={},
                fp=None,
            )
        return FakeResponse()

    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        fake_urlopen,
    )
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.sleep",
        lambda seconds: sleeps.append(seconds),
    )

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(
            image_path=str(image),
            ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
        ),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    row = table.to_pylist()[0]
    assert row["status"] == "succeeded"
    assert row["text"] == "retry ok"
    assert len(requests) == 2
    assert sleeps == [0.25]


def test_docling_pdf_ocr_worker_honors_ocr2_retry_after(
    tmp_path: Path,
    monkeypatch,
) -> None:
    image = tmp_path / "page-00000.png"
    image.write_bytes(b"png fixture")
    requests: list[object] = []
    sleeps: list[float] = []

    class FakeResponse:
        status = 200

        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps(
                {"choices": [{"message": {"content": "retry ok"}}]}
            ).encode("utf-8")

    def fake_urlopen(request: object, *, timeout: float) -> FakeResponse:
        _ = timeout
        requests.append(request)
        if len(requests) == 1:
            raise urllib.error.HTTPError(
                url="http://127.0.0.1:8999/v1/chat/completions",
                code=429,
                msg="Too Many Requests",
                hdrs={"Retry-After": "3"},
                fp=None,
            )
        return FakeResponse()

    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        fake_urlopen,
    )
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.sleep",
        lambda seconds: sleeps.append(seconds),
    )

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(
            image_path=str(image),
            ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
        ),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    row = table.to_pylist()[0]
    assert row["status"] == "succeeded"
    assert row["text"] == "retry ok"
    assert len(requests) == 2
    assert sleeps == [3.0]


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
                    ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
                ),
                _sample_pdf_ocr_input_table(
                    image_path=str(images[1]),
                    page_index=1,
                    shard_element_id="shard-1",
                    ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
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


def test_docling_pdf_ocr_worker_writes_deepseek_ocr2_request_trace(
    tmp_path: Path,
    monkeypatch,
) -> None:
    image = tmp_path / "page-00000.png"
    image.write_bytes(b"png fixture")
    trace_path = tmp_path / "ocr2.jsonl"

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

    monkeypatch.setenv(DEEPSEEK_OCR2_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(DEEPSEEK_OCR2_MODEL_ENV, "community/deepseek-ocr2-awq")
    monkeypatch.setenv(DEEPSEEK_OCR2_API_KEY_ENV, "secret-key")
    monkeypatch.setenv(DEEPSEEK_OCR2_TRACE_PATH_ENV, str(trace_path))
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        lambda request, *, timeout: FakeResponse(),
    )

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(
            image_path=str(image),
            page_index=7,
            shard_element_id="ocr2-trace-shard",
            ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
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
            "httpStatus": 200,
            "imageBytes": len(b"png fixture"),
            "latencyMs": records[0]["latencyMs"],
            "markdownChars": len("# traced\n"),
            "maxTokens": 8192,
            "model": "community/deepseek-ocr2-awq",
            "ocrProfile": PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
            "parentShardElementId": "",
            "pageCount": 1,
            "pageIndex": 7,
            "rasterHeightPx": 3100,
            "rasterWidthPx": 2400,
            "readingOrderKey": "000000.000000",
            "regionIndex": 0,
            "renderDpi": 300,
            "requestKind": "page",
            "schema": "xiuxian_wendao.deepseek_ocr2_request_trace.v1",
            "canonicalMarkdownChars": 0,
            "scaffoldAppliedCount": 0,
            "scaffoldJsonChars": 0,
            "scaffoldMode": "disabled",
            "scaffoldValidationFailureCount": 0,
            "shardCount": 1,
            "shardElementId": "ocr2-trace-shard",
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

    monkeypatch.delenv(DEEPSEEK_OCR2_MAX_TOKENS_ENV, raising=False)
    monkeypatch.delenv(DEEPSEEK_OCR2_REGION_MAX_TOKENS_ENV, raising=False)
    monkeypatch.setenv(DEEPSEEK_OCR2_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
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
                    shard_element_id="ocr2-page",
                    ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
                ),
                _sample_pdf_ocr_input_table(
                    source_path=str(source),
                    image_path=str(region_image),
                    page_index=0,
                    shard_element_id="ocr2-region",
                    shard_type="region",
                    region_index=1,
                    parent_shard_element_id="ocr2-page",
                    ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
                ),
            ]
        ),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert [row["status"] for row in table.to_pylist()] == ["succeeded", "succeeded"]
    assert [payload["max_tokens"] for payload in payloads] == [
        DEEPSEEK_OCR2_DEFAULT_MAX_TOKENS,
        DEEPSEEK_OCR2_DEFAULT_REGION_MAX_TOKENS,
    ]


def test_docling_pdf_ocr_worker_uses_lower_global_region_ocr2_token_limit(
    tmp_path: Path,
    monkeypatch,
) -> None:
    image = tmp_path / "region-00000.png"
    image.write_bytes(b"region png fixture")
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

    monkeypatch.setenv(DEEPSEEK_OCR2_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(DEEPSEEK_OCR2_MAX_TOKENS_ENV, "1024")
    monkeypatch.delenv(DEEPSEEK_OCR2_REGION_MAX_TOKENS_ENV, raising=False)
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        lambda request, *, timeout: payloads.append(
            json.loads(request.data.decode("utf-8"))
        )
        or FakeResponse(),
    )

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(
            image_path=str(image),
            shard_type="region",
            region_index=1,
            parent_shard_element_id="ocr2-page",
            ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
        ),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert table.to_pylist()[0]["status"] == "succeeded"
    assert payloads[0]["max_tokens"] == 1024


def test_docling_pdf_ocr_worker_composites_same_page_ocr2_regions(
    tmp_path: Path,
    monkeypatch,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    region_images = [tmp_path / f"region-{index:05}.png" for index in range(2)]
    for index, image in enumerate(region_images):
        image.write_bytes(f"region png fixture {index}".encode())
    trace_path = tmp_path / "ocr2-region-composite.jsonl"
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
                                    "<!-- xiuxian-wendao-ocr2-region:0:1:region-a -->\n"
                                    "| A | B |\n"
                                    "<!-- xiuxian-wendao-ocr2-region:0:2:region-b -->\n"
                                    "$x^2$\n"
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

    monkeypatch.setenv(DEEPSEEK_OCR2_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(DEEPSEEK_OCR2_REGION_COMPOSITE_SIZE_ENV, "2")
    monkeypatch.setenv(DEEPSEEK_OCR2_TRACE_PATH_ENV, str(trace_path))
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        fake_urlopen,
    )
    input_table = pa.concat_tables(
        [
            _sample_pdf_ocr_input_table(
                source_path=str(source),
                image_path=str(region_images[0]),
                page_index=0,
                shard_element_id="region-a",
                shard_type="region",
                region_index=1,
                parent_shard_element_id="parent-page",
                reading_order_key="000000.010000",
                ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
            ),
            _sample_pdf_ocr_input_table(
                source_path=str(source),
                image_path=str(region_images[1]),
                page_index=0,
                shard_element_id="region-b",
                shard_type="region",
                region_index=2,
                parent_shard_element_id="parent-page",
                reading_order_key="000000.020000",
                ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
            ),
        ]
    )

    table = build_pdf_ocr_shard_result_table(
        input_table,
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert [row["text"] for row in table.to_pylist()] == ["| A | B |", "$x^2$"]
    assert len(requests) == 1
    payload = json.loads(requests[0].data.decode("utf-8"))
    content = payload["messages"][0]["content"]
    assert payload["max_tokens"] == DEEPSEEK_OCR2_DEFAULT_REGION_MAX_TOKENS * 2
    assert sum(1 for part in content if part["type"] == "image_url") == 2
    assert "<!-- xiuxian-wendao-ocr2-region:0:1:region-a -->" in content[0]["text"]
    assert "<!-- xiuxian-wendao-ocr2-region:0:2:region-b -->" in content[0]["text"]
    records = [
        json.loads(line) for line in trace_path.read_text(encoding="utf-8").splitlines()
    ]
    assert records[0]["requestKind"] == "region-composite"
    assert records[0]["shardCount"] == 2
    assert records[0]["shardTypeCounts"] == {"region": 2}
    assert records[0]["sourcePixelArea"] == 14_880_000
    assert records[0]["maxTokens"] == DEEPSEEK_OCR2_DEFAULT_REGION_MAX_TOKENS * 2


def test_docling_pdf_ocr_worker_falls_back_when_region_composite_is_invalid(
    tmp_path: Path,
    monkeypatch,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    region_images = [tmp_path / f"region-{index:05}.png" for index in range(2)]
    for image in region_images:
        image.write_bytes(b"region png fixture")
    request_image_counts: list[int] = []

    class FakeResponse:
        status = 200

        def __init__(self, markdown: str) -> None:
            self._markdown = markdown

        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps(
                {"choices": [{"message": {"content": self._markdown}}]}
            ).encode("utf-8")

    def fake_urlopen(request: object, *, timeout: float) -> FakeResponse:
        _ = timeout
        payload = json.loads(request.data.decode("utf-8"))
        image_count = sum(
            1
            for part in payload["messages"][0]["content"]
            if part["type"] == "image_url"
        )
        request_image_counts.append(image_count)
        if image_count > 1:
            return FakeResponse("missing region markers")
        return FakeResponse(f"# fallback region {len(request_image_counts)}")

    monkeypatch.setenv(DEEPSEEK_OCR2_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(DEEPSEEK_OCR2_REGION_COMPOSITE_SIZE_ENV, "2")
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        fake_urlopen,
    )
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
                ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
            )
            for index in range(2)
        ]
    )

    table = build_pdf_ocr_shard_result_table(
        input_table,
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert [row["status"] for row in table.to_pylist()] == ["succeeded", "succeeded"]
    assert [row["text"] for row in table.to_pylist()] == [
        "# fallback region 2",
        "# fallback region 3",
    ]
    assert request_image_counts == [2, 1, 1]


def test_docling_pdf_ocr_worker_disables_invalid_region_composite_canary(
    tmp_path: Path,
    monkeypatch,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    region_images = [tmp_path / f"region-{index:05}.png" for index in range(4)]
    for image in region_images:
        image.write_bytes(b"region png fixture")
    request_image_counts: list[int] = []

    class FakeResponse:
        status = 200

        def __init__(self, markdown: str) -> None:
            self._markdown = markdown

        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps(
                {"choices": [{"message": {"content": self._markdown}}]}
            ).encode("utf-8")

    def fake_urlopen(request: object, *, timeout: float) -> FakeResponse:
        _ = timeout
        payload = json.loads(request.data.decode("utf-8"))
        image_count = sum(
            1
            for part in payload["messages"][0]["content"]
            if part["type"] == "image_url"
        )
        request_image_counts.append(image_count)
        if image_count > 1:
            return FakeResponse("missing region markers")
        return FakeResponse(f"# fallback region {len(request_image_counts)}")

    monkeypatch.setenv(DEEPSEEK_OCR2_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(DEEPSEEK_OCR2_REGION_COMPOSITE_SIZE_ENV, "2")
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        fake_urlopen,
    )
    input_table = pa.concat_tables(
        [
            _sample_pdf_ocr_input_table(
                source_path=str(source),
                image_path=str(region_images[index]),
                page_index=index // 2,
                shard_element_id=f"region-{index}",
                shard_type="region",
                region_index=(index % 2) + 1,
                parent_shard_element_id=f"parent-page-{index // 2}",
                reading_order_key=f"00000{index // 2}.0{index + 1}0000",
                ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
            )
            for index in range(4)
        ]
    )

    table = build_pdf_ocr_shard_result_table(
        input_table,
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert [row["status"] for row in table.to_pylist()] == ["succeeded"] * 4
    assert request_image_counts == [2, 1, 1, 1, 1]


def test_docling_pdf_ocr_worker_uses_region_atlas_json(
    tmp_path: Path,
    monkeypatch,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    region_images = [tmp_path / f"region-{index:05}.png" for index in range(2)]
    _write_ppm_image(region_images[0], b"\xff\x00\x00")
    _write_ppm_image(region_images[1], b"\x00\x00\xff")
    trace_path = tmp_path / "ocr2-region-atlas.jsonl"
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
                ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
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

    monkeypatch.setenv(DEEPSEEK_OCR2_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(DEEPSEEK_OCR2_REGION_COMPOSITE_SIZE_ENV, "2")
    monkeypatch.setenv(DEEPSEEK_OCR2_REGION_ATLAS_MODE_ENV, "same-page-json")
    monkeypatch.setenv(DEEPSEEK_OCR2_TRACE_PATH_ENV, str(trace_path))
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


def test_docling_pdf_ocr_worker_falls_back_when_region_atlas_json_is_invalid(
    tmp_path: Path,
    monkeypatch,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    region_images = [tmp_path / f"region-{index:05}.png" for index in range(2)]
    for image in region_images:
        _write_ppm_image(image)
    request_kinds: list[str] = []

    class FakeResponse:
        status = 200

        def __init__(self, content: str) -> None:
            self._content = content

        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps(
                {"choices": [{"message": {"content": self._content}}]}
            ).encode("utf-8")

    def fake_urlopen(request: object, *, timeout: float) -> FakeResponse:
        _ = timeout
        payload = json.loads(request.data.decode("utf-8"))
        prompt_text = payload["messages"][0]["content"][0]["text"]
        if "Atlas panel mapping" in prompt_text:
            request_kinds.append("atlas")
            return FakeResponse('{"regions":[]}')
        request_kinds.append("single")
        return FakeResponse(f"# fallback atlas region {len(request_kinds)}")

    monkeypatch.setenv(DEEPSEEK_OCR2_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(DEEPSEEK_OCR2_REGION_COMPOSITE_SIZE_ENV, "2")
    monkeypatch.setenv(DEEPSEEK_OCR2_REGION_ATLAS_MODE_ENV, "same-page-json")
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        fake_urlopen,
    )
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
                ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
            )
            for index in range(2)
        ]
    )

    table = build_pdf_ocr_shard_result_table(
        input_table,
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert [row["status"] for row in table.to_pylist()] == ["succeeded", "succeeded"]
    assert [row["text"] for row in table.to_pylist()] == [
        "# fallback atlas region 2",
        "# fallback atlas region 3",
    ]
    assert request_kinds == ["atlas", "single", "single"]


def test_docling_pdf_ocr_worker_disables_invalid_region_atlas_canary(
    tmp_path: Path,
    monkeypatch,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    region_images = [tmp_path / f"region-{index:05}.png" for index in range(4)]
    for image in region_images:
        _write_ppm_image(image)
    request_kinds: list[str] = []

    class FakeResponse:
        status = 200

        def __init__(self, content: str) -> None:
            self._content = content

        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps(
                {"choices": [{"message": {"content": self._content}}]}
            ).encode("utf-8")

    def fake_urlopen(request: object, *, timeout: float) -> FakeResponse:
        _ = timeout
        payload = json.loads(request.data.decode("utf-8"))
        prompt_text = payload["messages"][0]["content"][0]["text"]
        if "Atlas panel mapping" in prompt_text:
            request_kinds.append("atlas")
            return FakeResponse('{"regions":[]}')
        request_kinds.append("single")
        return FakeResponse(f"# fallback atlas region {len(request_kinds)}")

    monkeypatch.setenv(DEEPSEEK_OCR2_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(DEEPSEEK_OCR2_REGION_COMPOSITE_SIZE_ENV, "2")
    monkeypatch.setenv(DEEPSEEK_OCR2_REGION_ATLAS_MODE_ENV, "same-page-json")
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        fake_urlopen,
    )
    input_table = pa.concat_tables(
        [
            _sample_pdf_ocr_input_table(
                source_path=str(source),
                image_path=str(region_images[index]),
                page_index=index // 2,
                shard_element_id=f"region-{index}",
                shard_type="region",
                region_index=(index % 2) + 1,
                parent_shard_element_id=f"parent-page-{index // 2}",
                reading_order_key=f"00000{index // 2}.0{index + 1}0000",
                ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
            )
            for index in range(4)
        ]
    )

    table = build_pdf_ocr_shard_result_table(
        input_table,
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert [row["status"] for row in table.to_pylist()] == ["succeeded"] * 4
    assert request_kinds == ["atlas", "single", "single", "single", "single"]


def test_docling_pdf_ocr_worker_shares_invalid_region_atlas_canary_state(
    tmp_path: Path,
    monkeypatch,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    region_images = [tmp_path / f"region-{index:05}.png" for index in range(4)]
    for image in region_images:
        _write_ppm_image(image)
    request_kinds: list[str] = []

    class FakeResponse:
        status = 200

        def __init__(self, content: str) -> None:
            self._content = content

        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps(
                {"choices": [{"message": {"content": self._content}}]}
            ).encode("utf-8")

    def fake_urlopen(request: object, *, timeout: float) -> FakeResponse:
        _ = timeout
        payload = json.loads(request.data.decode("utf-8"))
        prompt_text = payload["messages"][0]["content"][0]["text"]
        if "Atlas panel mapping" in prompt_text:
            request_kinds.append("atlas")
            return FakeResponse('{"regions":[]}')
        request_kinds.append("single")
        return FakeResponse(f"# fallback atlas region {len(request_kinds)}")

    def input_table_for_page(page_index: int, offset: int) -> pa.Table:
        return pa.concat_tables(
            [
                _sample_pdf_ocr_input_table(
                    source_path=str(source),
                    image_path=str(region_images[offset + index]),
                    page_index=page_index,
                    shard_element_id=f"region-{offset + index}",
                    shard_type="region",
                    region_index=index + 1,
                    parent_shard_element_id=f"parent-page-{page_index}",
                    reading_order_key=f"00000{page_index}.0{index + 1}0000",
                    ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
                )
                for index in range(2)
            ]
        )

    monkeypatch.setenv(DEEPSEEK_OCR2_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(DEEPSEEK_OCR2_REGION_COMPOSITE_SIZE_ENV, "2")
    monkeypatch.setenv(DEEPSEEK_OCR2_REGION_ATLAS_MODE_ENV, "same-page-json")
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        fake_urlopen,
    )
    log_dir = tmp_path / "logs"
    log_dir.mkdir()

    monkeypatch.setenv(DEEPSEEK_OCR2_TRACE_PATH_ENV, str(log_dir / "worker-0.jsonl"))
    first = build_pdf_ocr_shard_result_table(
        input_table_for_page(0, 0),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )
    monkeypatch.setenv(DEEPSEEK_OCR2_TRACE_PATH_ENV, str(log_dir / "worker-1.jsonl"))
    second = build_pdf_ocr_shard_result_table(
        input_table_for_page(1, 2),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert [row["status"] for row in first.to_pylist()] == ["succeeded"] * 2
    assert [row["status"] for row in second.to_pylist()] == ["succeeded"] * 2
    assert request_kinds == ["atlas", "single", "single", "single", "single"]


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
        ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
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

    monkeypatch.setenv(DEEPSEEK_OCR2_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(DEEPSEEK_OCR2_SCAFFOLD_MODE_ENV, "region-table-json")
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
    assert payload["max_tokens"] == DEEPSEEK_OCR2_DEFAULT_REGION_MAX_TOKENS


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
                ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
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

    monkeypatch.setenv(DEEPSEEK_OCR2_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(DEEPSEEK_OCR2_REGION_COMPOSITE_SIZE_ENV, "2")
    monkeypatch.setenv(DEEPSEEK_OCR2_SCAFFOLD_MODE_ENV, "region-table-json")
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


def test_docling_pdf_ocr_worker_fails_invalid_region_scaffolds(
    tmp_path: Path,
    monkeypatch,
) -> None:
    cases = [
        ("missing sidecar", None, None),
        ("fingerprint mismatch", "wrong-raster", None),
        ("malformed json", "rasterhash", "{not-json"),
        (
            "missing marker",
            "rasterhash",
            json.dumps(
                {
                    "regions": [
                        {"marker": "wrong", "shardElementId": "region-a", "text": "x"}
                    ]
                }
            ),
        ),
        (
            "row mismatch",
            "rasterhash",
            json.dumps({"regions": []}),
        ),
        (
            "empty output",
            "rasterhash",
            json.dumps(
                {
                    "regions": [
                        {
                            "marker": "<!-- xiuxian-wendao-ocr2-region:0:1:region-a -->",
                            "shardElementId": "region-a",
                            "text": "",
                            "tables": [],
                        }
                    ]
                }
            ),
        ),
        (
            "invalid table shape",
            "rasterhash",
            json.dumps(
                {
                    "regions": [
                        {
                            "marker": "<!-- xiuxian-wendao-ocr2-region:0:1:region-a -->",
                            "shardElementId": "region-a",
                            "tables": [{"rows": [["A", "B"], ["1"]]}],
                        }
                    ]
                }
            ),
        ),
    ]

    class FakeResponse:
        status = 200

        def __init__(self, content: str) -> None:
            self._content = content

        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps(
                {"choices": [{"message": {"content": self._content}}]}
            ).encode("utf-8")

    requests: list[object] = []
    responses: list[str] = []
    trace_path = tmp_path / "invalid-scaffold-trace.jsonl"

    def fake_urlopen(request: object, *, timeout: float) -> FakeResponse:
        _ = timeout
        requests.append(request)
        return FakeResponse(responses.pop(0))

    monkeypatch.setenv(DEEPSEEK_OCR2_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(DEEPSEEK_OCR2_SCAFFOLD_MODE_ENV, "region-table-json")
    monkeypatch.setenv(DEEPSEEK_OCR2_TRACE_PATH_ENV, str(trace_path))
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        fake_urlopen,
    )

    for index, (_label, raster_sha256, response) in enumerate(cases):
        case_dir = tmp_path / f"case-{index}"
        case_dir.mkdir()
        image = case_dir / "region-00001.png"
        image.write_bytes(b"region png fixture")
        input_table = _sample_pdf_ocr_input_table(
            image_path=str(image),
            shard_element_id="region-a",
            shard_type="region",
            region_index=1,
            parent_shard_element_id="parent-page",
            ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
        )
        input_row = input_table.to_pylist()[0]
        if raster_sha256 is not None:
            _write_ocr2_region_scaffold_sidecar(
                case_dir,
                [input_row],
                raster_sha256=raster_sha256,
            )
        if response is not None:
            responses.append(response)

        table = build_pdf_ocr_shard_result_table(
            input_table,
            worker=DoclingPdfOcrShardWorker(max_workers=1),
        )

        row = table.to_pylist()[0]
        assert row["status"] == "failed"
        assert "DeepSeek-OCR-2 OCR failed" in row["errorMessage"]

    trace_records = [
        json.loads(line) for line in trace_path.read_text(encoding="utf-8").splitlines()
    ]
    response_validation_records = [
        record
        for record in trace_records
        if record["status"] == "failed" and record["httpStatus"] == 200
    ]
    assert response_validation_records
    assert all(
        record["scaffoldJsonChars"] > 0 for record in response_validation_records
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
                                    "<!-- xiuxian-wendao-ocr2-page:0 -->\n"
                                    "# Page zero\n"
                                    "<!-- xiuxian-wendao-ocr2-page:1 -->\n"
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

    monkeypatch.setenv(DEEPSEEK_OCR2_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(DEEPSEEK_OCR2_MODEL_ENV, "unit/window-success")
    monkeypatch.setenv(DEEPSEEK_OCR2_PAGE_WINDOW_SIZE_ENV, "2")
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
                shard_element_id=f"ocr2-shard-{page_index}",
                ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
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
    assert "<!-- xiuxian-wendao-ocr2-page:0 -->" in content[0]["text"]
    assert "<!-- xiuxian-wendao-ocr2-page:1 -->" in content[0]["text"]


def test_docling_pdf_ocr_worker_falls_back_when_ocr2_page_window_is_invalid(
    tmp_path: Path,
    monkeypatch,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    image_paths = [tmp_path / f"page-{page_index:05}.png" for page_index in range(4)]
    for image_path in image_paths:
        image_path.write_bytes(b"png fixture")
    request_image_counts: list[int] = []

    class FakeResponse:
        status = 200

        def __init__(self, markdown: str) -> None:
            self._markdown = markdown

        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps(
                {"choices": [{"message": {"content": self._markdown}}]}
            ).encode("utf-8")

    def fake_urlopen(request: object, *, timeout: float) -> FakeResponse:
        _ = timeout
        payload = json.loads(request.data.decode("utf-8"))
        image_count = sum(
            1
            for part in payload["messages"][0]["content"]
            if part["type"] == "image_url"
        )
        request_image_counts.append(image_count)
        if image_count > 1:
            return FakeResponse("missing page markers")
        return FakeResponse(f"# fallback {len(request_image_counts)}")

    monkeypatch.setenv(DEEPSEEK_OCR2_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(DEEPSEEK_OCR2_MODEL_ENV, "unit/window-fallback")
    monkeypatch.setenv(DEEPSEEK_OCR2_PAGE_WINDOW_SIZE_ENV, "2")
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
                shard_element_id=f"ocr2-shard-{page_index}",
                ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
            )
            for page_index, image_path in enumerate(image_paths)
        ]
    )

    table = build_pdf_ocr_shard_result_table(
        input_table,
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert [row["status"] for row in table.to_pylist()] == [
        "succeeded",
        "succeeded",
        "succeeded",
        "succeeded",
    ]
    assert [row["text"] for row in table.to_pylist()] == [
        "# fallback 2",
        "# fallback 3",
        "# fallback 4",
        "# fallback 5",
    ]
    assert request_image_counts == [2, 1, 1, 1, 1]


def test_docling_pdf_ocr_worker_uses_openrouter_provider_preset(
    tmp_path: Path,
    monkeypatch,
) -> None:
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
                {"choices": [{"message": {"content": "# OpenRouter OCR\n"}}]}
            ).encode("utf-8")

    def fake_urlopen(request: object, *, timeout: float) -> FakeResponse:
        _ = timeout
        requests.append(request)
        return FakeResponse()

    monkeypatch.setenv(DEEPSEEK_OCR2_PROVIDER_ENV, DEEPSEEK_OCR2_OPENROUTER_PROVIDER)
    monkeypatch.setenv(DEEPSEEK_OCR2_OPENROUTER_API_KEY_ENV, "or-key")
    monkeypatch.setenv(DEEPSEEK_OCR2_OPENROUTER_MODEL_ENV, "openrouter/vision-ocr")
    monkeypatch.setenv(
        DEEPSEEK_OCR2_OPENROUTER_HTTP_REFERER_ENV, "https://wendao.local"
    )
    monkeypatch.setenv(DEEPSEEK_OCR2_OPENROUTER_TITLE_ENV, "Wendao OCR Benchmark")
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        fake_urlopen,
    )

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(
            image_path=str(image),
            ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
        ),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert table.to_pylist()[0]["text"] == "# OpenRouter OCR\n"
    request = requests[0]
    assert request.full_url == f"{DEEPSEEK_OCR2_OPENROUTER_BASE_URL}/chat/completions"
    headers = {key.lower(): value for key, value in request.header_items()}
    assert headers["authorization"] == "Bearer or-key"
    assert headers["http-referer"] == "https://wendao.local"
    assert headers["x-openrouter-title"] == "Wendao OCR Benchmark"
    payload = json.loads(request.data.decode("utf-8"))
    assert payload["model"] == "openrouter/vision-ocr"


def test_docling_pdf_ocr_worker_uses_openrouter_smoke_model_by_default(
    tmp_path: Path,
    monkeypatch,
) -> None:
    image = tmp_path / "page-00000.png"
    image.write_bytes(b"png fixture")
    requests: list[object] = []

    class FakeResponse:
        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps({"choices": [{"message": {"content": "OCR"}}]}).encode(
                "utf-8"
            )

    monkeypatch.setenv(DEEPSEEK_OCR2_PROVIDER_ENV, DEEPSEEK_OCR2_OPENROUTER_PROVIDER)
    monkeypatch.setenv(DEEPSEEK_OCR2_OPENROUTER_API_KEY_ENV, "or-key")
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        lambda request, *, timeout: requests.append(request) or FakeResponse(),
    )

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(
            image_path=str(image),
            ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
        ),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert table.to_pylist()[0]["text"] == "OCR"
    payload = json.loads(requests[0].data.decode("utf-8"))
    assert payload["model"] == DEEPSEEK_OCR2_OPENROUTER_TEST_MODEL


def test_docling_pdf_ocr_worker_parallelizes_direct_ocr2_requests(
    tmp_path: Path,
    monkeypatch,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    image_paths = [tmp_path / f"page-{page_index:05}.png" for page_index in range(2)]
    for image_path in image_paths:
        image_path.write_bytes(b"png fixture")
    active = 0
    max_active = 0
    lock = threading.Lock()

    class FakeResponse:
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
        nonlocal active, max_active
        _ = request, timeout
        with lock:
            active += 1
            max_active = max(max_active, active)
        try:
            time.sleep(0.05)
            return FakeResponse(f"OCR active {max_active}")
        finally:
            with lock:
                active -= 1

    monkeypatch.setenv(DEEPSEEK_OCR2_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(DEEPSEEK_OCR2_REQUEST_CONCURRENCY_ENV, "2")
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
                shard_element_id=f"ocr2-shard-{page_index}",
                ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
            )
            for page_index, image_path in enumerate(image_paths)
        ]
    )

    table = build_pdf_ocr_shard_result_table(
        input_table,
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert [row["status"] for row in table.to_pylist()] == ["succeeded", "succeeded"]
    assert max_active == 2


def test_docling_pdf_ocr_worker_coalesces_noncontiguous_direct_ocr2_requests(
    tmp_path: Path,
    monkeypatch,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    image_paths = [tmp_path / f"page-{page_index:05}.png" for page_index in range(3)]
    for image_path in image_paths:
        image_path.write_bytes(b"png fixture")
    active = 0
    max_active = 0
    lock = threading.Lock()

    class FakeResponse:
        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps({"choices": [{"message": {"content": "OCR2"}}]}).encode(
                "utf-8"
            )

    def fake_urlopen(request: object, *, timeout: float) -> FakeResponse:
        nonlocal active, max_active
        _ = request, timeout
        with lock:
            active += 1
            max_active = max(max_active, active)
        try:
            time.sleep(0.05)
            return FakeResponse()
        finally:
            with lock:
                active -= 1

    monkeypatch.setenv(DEEPSEEK_OCR2_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.delenv(DEEPSEEK_OCR2_REQUEST_CONCURRENCY_ENV, raising=False)
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        fake_urlopen,
    )
    input_table = pa.concat_tables(
        [
            _sample_pdf_ocr_input_table(
                source_path=str(source),
                image_path=str(image_paths[0]),
                page_index=0,
                shard_element_id="ocr2-shard-0",
                ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
            ),
            _sample_pdf_ocr_input_table(
                source_path=str(source),
                image_path=str(image_paths[1]),
                page_index=1,
                shard_element_id="fast-shard-1",
                ocr_profile=PDF_OCR_FAST_TEXT_PROFILE,
            ),
            _sample_pdf_ocr_input_table(
                source_path=str(source),
                image_path=str(image_paths[2]),
                page_index=2,
                shard_element_id="ocr2-shard-2",
                ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
            ),
        ]
    )

    table = build_pdf_ocr_shard_result_table(
        input_table,
        worker=DoclingPdfOcrShardWorker(
            FakeDoclingConverter("fast text"), max_workers=1
        ),
        max_workers=2,
    )

    assert [row["status"] for row in table.to_pylist()] == [
        "succeeded",
        "succeeded",
        "succeeded",
    ]
    assert [row["text"] for row in table.to_pylist()] == ["OCR2", "fast text", "OCR2"]
    assert max_active == 2


def test_docling_pdf_ocr_worker_reports_missing_openrouter_key(
    tmp_path: Path,
    monkeypatch,
) -> None:
    image = tmp_path / "page-00000.png"
    image.write_bytes(b"png fixture")
    monkeypatch.setenv(DEEPSEEK_OCR2_PROVIDER_ENV, DEEPSEEK_OCR2_OPENROUTER_PROVIDER)

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(
            image_path=str(image),
            ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
        ),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    row = table.to_pylist()[0]
    assert row["status"] == "failed"
    assert "OpenRouter OCR provider requires" in row["errorMessage"]


def test_docling_pdf_ocr_worker_accepts_openroute_key_compat_alias(
    tmp_path: Path,
    monkeypatch,
) -> None:
    image = tmp_path / "page-00000.png"
    image.write_bytes(b"png fixture")
    requests: list[object] = []

    class FakeResponse:
        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps({"choices": [{"message": {"content": "OCR"}}]}).encode(
                "utf-8"
            )

    monkeypatch.setenv(DEEPSEEK_OCR2_PROVIDER_ENV, DEEPSEEK_OCR2_OPENROUTER_PROVIDER)
    monkeypatch.setenv(DEEPSEEK_OCR2_OPENROUTE_COMPAT_API_KEY_ENV, "or-compat-key")
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        lambda request, *, timeout: requests.append(request) or FakeResponse(),
    )

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(
            image_path=str(image),
            ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
        ),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert table.to_pylist()[0]["text"] == "OCR"
    headers = {key.lower(): value for key, value in requests[0].header_items()}
    assert headers["authorization"] == "Bearer or-compat-key"


def test_docling_pdf_ocr_worker_reports_empty_deepseek_ocr2_response(
    tmp_path: Path,
    monkeypatch,
) -> None:
    image = tmp_path / "page-00000.png"
    image.write_bytes(b"png fixture")

    class FakeResponse:
        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps({"choices": [{"message": {"content": ""}}]}).encode(
                "utf-8"
            )

    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        lambda request, *, timeout: FakeResponse(),
    )

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(
            image_path=str(image),
            ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
        ),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    row = table.to_pylist()[0]
    assert row["status"] == "failed"
    assert "returned empty text" in row["errorMessage"]


def test_docling_pdf_ocr_worker_reports_missing_deepseek_ocr2_image(
    tmp_path: Path,
) -> None:
    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(
            image_path=str(tmp_path / "missing.png"),
            ocr_profile=PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
        ),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    row = table.to_pylist()[0]
    assert row["status"] == "failed"
    assert "shard image does not exist" in row["errorMessage"]


def test_docling_pdf_ocr_worker_passes_profile_to_converter_factory(
    tmp_path: Path,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    requested_profiles: list[str] = []

    def fake_converter_factory(profile: str) -> FakeDoclingConverter:
        requested_profiles.append(profile)
        return FakeDoclingConverter(f"OCR {profile}\n")

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(
            source_path=str(source),
            ocr_profile=PDF_OCR_FAST_TEXT_PROFILE,
        ),
        worker=DoclingPdfOcrShardWorker(
            converter_factory=fake_converter_factory,
            max_workers=1,
        ),
    )

    assert requested_profiles == [PDF_OCR_FAST_TEXT_PROFILE]
    assert table.to_pylist()[0]["text"] == f"OCR {PDF_OCR_FAST_TEXT_PROFILE}\n"


def test_docling_pdf_ocr_worker_preserves_order_with_concurrent_shards(
    tmp_path: Path,
) -> None:
    records: list[tuple[int, str]] = []
    records_lock = threading.Lock()

    class ThreadLocalConverter:
        def convert(self, source: str | Path, **kwargs: object) -> FakeDoclingResult:
            _ = kwargs
            time.sleep(0.005)
            with records_lock:
                records.append((threading.get_ident(), Path(source).name))
            return FakeDoclingResult(f"OCR {Path(source).stem}\n")

    tables = []
    for page_index in range(20):
        image = tmp_path / f"page-{page_index:05}.png"
        image.write_bytes(b"png fixture")
        tables.append(
            _sample_pdf_ocr_input_table(
                source_path=str(tmp_path / "missing-source.pdf"),
                image_path=str(image),
                page_index=page_index,
                shard_element_id=f"shard-{page_index}",
            )
        )

    table = build_pdf_ocr_shard_result_table(
        pa.concat_tables(tables),
        worker=DoclingPdfOcrShardWorker(
            converter_factory=ThreadLocalConverter,
            max_workers=4,
        ),
    )

    rows = table.to_pylist()
    assert [row["pageIndex"] for row in rows] == list(range(20))
    assert [row["text"] for row in rows] == [
        f"OCR page-{page_index:05}\n" for page_index in range(20)
    ]
    assert len({thread_id for thread_id, _ in records}) > 1


def test_docling_pdf_ocr_worker_failure_isolated_per_shard(tmp_path: Path) -> None:
    class PartiallyFailingConverter:
        def convert(self, source: str | Path, **kwargs: object) -> FakeDoclingResult:
            _ = kwargs
            if Path(source).name == "page-00001.png":
                raise RuntimeError("selected shard failed")
            return FakeDoclingResult(f"OCR {Path(source).stem}\n")

    tables = []
    for page_index in range(3):
        image = tmp_path / f"page-{page_index:05}.png"
        image.write_bytes(b"png fixture")
        tables.append(
            _sample_pdf_ocr_input_table(
                source_path=str(tmp_path / "missing-source.pdf"),
                image_path=str(image),
                page_index=page_index,
                shard_element_id=f"shard-{page_index}",
            )
        )

    table = build_pdf_ocr_shard_result_table(
        pa.concat_tables(tables),
        worker=DoclingPdfOcrShardWorker(
            converter_factory=PartiallyFailingConverter,
            max_workers=3,
        ),
    )

    rows = table.to_pylist()
    assert [row["status"] for row in rows] == ["succeeded", "failed", "succeeded"]
    assert "Docling OCR failed" in rows[1]["errorMessage"]


def test_docling_pdf_ocr_worker_reports_missing_images() -> None:
    converter = FakeDoclingConverter("OCR\n")

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(image_path="/tmp/missing-page.png"),
        worker=DoclingPdfOcrShardWorker(converter),
    )

    row = table.to_pylist()[0]
    assert converter.calls == []
    assert row["status"] == "failed"
    assert "does not exist" in row["errorMessage"]


def test_docling_pdf_ocr_worker_rejects_empty_output(tmp_path: Path) -> None:
    image = tmp_path / "page-00000.png"
    image.write_bytes(b"png fixture")

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(image_path=str(image)),
        worker=DoclingPdfOcrShardWorker(FakeDoclingConverter(" \n")),
    )

    row = table.to_pylist()[0]
    assert row["status"] == "failed"
    assert row["errorMessage"] == "Docling OCR returned empty text"


def test_docling_pdf_ocr_worker_reports_converter_errors(tmp_path: Path) -> None:
    image = tmp_path / "page-00000.png"
    image.write_bytes(b"png fixture")

    table = build_pdf_ocr_shard_result_table(
        _sample_pdf_ocr_input_table(image_path=str(image)),
        worker=DoclingPdfOcrShardWorker(FailingDoclingConverter()),
    )

    row = table.to_pylist()[0]
    assert row["status"] == "failed"
    assert "Docling OCR failed" in row["errorMessage"]
