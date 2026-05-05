"""document_service test slice 3."""

from __future__ import annotations

import json

from xiuxian_wendao_analyzer.pdf_ocr import (
    DEEPSEEK_OCR2_BASE_URL_ENV,
    DEEPSEEK_OCR2_MODEL_ENV,
    DEEPSEEK_OCR2_OPENROUTE_COMPAT_API_KEY_ENV,
    DEEPSEEK_OCR2_OPENROUTER_API_KEY_ENV,
    DEEPSEEK_OCR2_OPENROUTER_BASE_URL,
    DEEPSEEK_OCR2_OPENROUTER_HTTP_REFERER_ENV,
    DEEPSEEK_OCR2_OPENROUTER_MODEL_ENV,
    DEEPSEEK_OCR2_OPENROUTER_PROVIDER,
    DEEPSEEK_OCR2_OPENROUTER_TEST_MODEL,
    DEEPSEEK_OCR2_OPENROUTER_TITLE_ENV,
    DEEPSEEK_OCR2_PROVIDER_ENV,
    DEEPSEEK_OCR2_REQUEST_CONCURRENCY_ENV,
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
        "xiuxian_wendao_analyzer.pdf_ocr_workers.urllib.request.urlopen",
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
        "xiuxian_wendao_analyzer.pdf_ocr_workers.urllib.request.urlopen",
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
        "xiuxian_wendao_analyzer.pdf_ocr_workers.urllib.request.urlopen",
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
        "xiuxian_wendao_analyzer.pdf_ocr_workers.urllib.request.urlopen",
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
        "xiuxian_wendao_analyzer.pdf_ocr_workers.urllib.request.urlopen",
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
        "xiuxian_wendao_analyzer.pdf_ocr_workers.urllib.request.urlopen",
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
