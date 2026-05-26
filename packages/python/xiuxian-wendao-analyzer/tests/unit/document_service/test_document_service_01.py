"""document_service test slice 1."""

from __future__ import annotations

from xiuxian_wendao_analyzer import document_service

from .support import (
    ANALYSIS_AUDIO_SHARDS_ROUTE,
    ANALYSIS_DOCUMENT_EXTRACT_ROUTE,
    ANALYSIS_PDF_OCR_SHARDS_ROUTE,
    AUDIO_SHARD_RESULT_SCHEMA,
    AUDIO_SHARD_RESULT_SCHEMA_VERSION,
    DOCUMENT_RESOURCE_SCHEMA,
    EXPECTED_SCHEMA_VERSION,
    PDF_OCR_SHARD_RESULT_SCHEMA,
    PDF_OCR_SHARD_RESULT_SCHEMA_VERSION,
    SUPPORTED_DOCUMENT_ROUTES,
    WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER,
    WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER,
    WENDAO_DOCUMENT_EXTRACT_PAGE_RANGE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
    WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_UTF8_HEX_HEADER,
    WENDAO_SCHEMA_VERSION_HEADER,
    DocumentExtractFlightServer,
    FakeDoclingConverter,
    Path,
    _sample_audio_shard_input_table,
    _sample_pdf_ocr_input_table,
    build_audio_shard_result_table,
    build_document_extract_table,
    build_pdf_ocr_shard_result_table,
    pytest,
)


def test_document_extract_table_uses_document_headers(tmp_path: Path) -> None:
    source = tmp_path / "manual.docx"
    source.write_bytes(b"docx fixture")
    output_dir = tmp_path / "out"
    converter = FakeDoclingConverter("# Manual\n")

    table = build_document_extract_table(
        {
            WENDAO_SCHEMA_VERSION_HEADER: EXPECTED_SCHEMA_VERSION,
            WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER: str(source),
            WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER: str(output_dir),
            WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER: "true",
        },
        converter=converter,
    )

    assert converter.calls == [source]
    assert table.schema == DOCUMENT_RESOURCE_SCHEMA
    row = table.to_pylist()[0]
    assert row["sourcePath"] == str(source)
    assert row["resourcePath"] == str(output_dir / "manual.md")
    assert row["content"] == "# Manual\n"


def test_document_extract_table_accepts_utf8_hex_source_path_header(
    tmp_path: Path,
) -> None:
    source = tmp_path / "private-\u97f3\u9891.mp3"
    source.write_bytes(b"audio fixture")
    output_dir = tmp_path / "out"
    converter = FakeDoclingConverter("transcript\n")

    table = build_document_extract_table(
        {
            WENDAO_SCHEMA_VERSION_HEADER: EXPECTED_SCHEMA_VERSION,
            WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_UTF8_HEX_HEADER: (
                str(source).encode("utf-8").hex()
            ),
            WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER: str(output_dir),
        },
        converter=converter,
    )

    assert converter.calls == [source]
    assert table.to_pylist()[0]["sourcePath"] == str(source)


def test_document_extract_table_uses_fast_text_profile_header(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    source = tmp_path / "manual.pdf"
    source.write_bytes(b"pdf fixture")
    output_dir = tmp_path / "out"
    profiles: list[str | None] = []

    def fake_converter_factory(profile: str | None = None) -> FakeDoclingConverter:
        profiles.append(profile)
        return FakeDoclingConverter("# Fast\n")

    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.document_extract._new_docling_converter",
        fake_converter_factory,
    )

    table = build_document_extract_table(
        {
            WENDAO_SCHEMA_VERSION_HEADER: EXPECTED_SCHEMA_VERSION,
            WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER: str(source),
            WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER: str(output_dir),
            WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER: "attachment",
        },
    )

    assert profiles == ["fast-text"]
    assert table.to_pylist()[0]["content"] == "# Fast\n"


def test_document_extract_table_uses_page_range_header(tmp_path: Path) -> None:
    source = tmp_path / "manual.pdf"
    source.write_bytes(b"pdf fixture")
    output_dir = tmp_path / "out"
    converter = FakeDoclingConverter("# Pages\n")

    table = build_document_extract_table(
        {
            WENDAO_SCHEMA_VERSION_HEADER: EXPECTED_SCHEMA_VERSION,
            WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER: str(source),
            WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER: str(output_dir),
            WENDAO_DOCUMENT_EXTRACT_PAGE_RANGE_HEADER: "2:3",
        },
        converter=converter,
    )

    assert converter.kwargs_calls == [{"page_range": (2, 3)}]
    row = table.to_pylist()[0]
    assert row["pageIndex"] == 1
    assert row["elementId"] == "page-range-00002-00003:_main"
    assert row["resourcePath"] == str(output_dir / "manual.pages-00002-00003.md")


def test_document_flight_server_warms_arrow_runtime(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    calls = 0

    def warm_runtime() -> None:
        nonlocal calls
        calls += 1

    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.document_service.warm_document_arrow_runtime",
        warm_runtime,
    )

    DocumentExtractFlightServer("grpc://127.0.0.1:0")

    assert calls == 1


def test_document_extract_converter_cache_mode_accepts_profile_aliases() -> None:
    assert (
        document_service._document_extract_converter_cache_mode_with_lookup(lambda _key: None)
        == "disabled"
    )
    assert (
        document_service._document_extract_converter_cache_mode_with_lookup(
            lambda _key: "profile-cache"
        )
        == "profile"
    )
    assert (
        document_service._document_extract_converter_cache_mode_with_lookup(lambda _key: "unknown")
        == "disabled"
    )


def test_document_flight_server_can_reuse_profile_converter(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv(
        document_service.WENDAO_DOCUMENT_EXTRACT_CONVERTER_CACHE_ENV,
        "profile",
    )
    factory_calls: list[str | None] = []

    def converter_factory(profile: str | None = None) -> FakeDoclingConverter:
        factory_calls.append(profile)
        return FakeDoclingConverter(f"# {profile}\n")

    server = DocumentExtractFlightServer(
        "grpc://127.0.0.1:0",
        converter_factory=converter_factory,
    )

    first = server._document_extract_converter({WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER: "full"})
    second = server._document_extract_converter({WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER: "full"})
    fast = server._document_extract_converter({WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER: "fast-text"})

    assert first is second
    assert first is not fast
    assert factory_calls == ["full", "fast-text"]


def test_document_flight_server_profile_header_overrides_fixed_full_converter() -> None:
    fixed_converter = FakeDoclingConverter("# Full\n")
    factory_calls: list[str | None] = []

    def converter_factory(profile: str | None = None) -> FakeDoclingConverter:
        factory_calls.append(profile)
        return FakeDoclingConverter(f"# {profile}\n")

    server = DocumentExtractFlightServer(
        "grpc://127.0.0.1:0",
        converter=fixed_converter,
        converter_factory=converter_factory,
    )

    full = server._document_extract_converter({WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER: "full"})
    structure = server._document_extract_converter(
        {WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER: "structure-text"}
    )
    structure_again = server._document_extract_converter(
        {WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER: "docling-structure-text"}
    )

    assert full is fixed_converter
    assert structure is structure_again
    assert structure is not fixed_converter
    assert factory_calls == ["structure-text"]


def test_document_flight_server_converter_cache_is_opt_in() -> None:
    calls = 0

    def converter_factory(profile: str | None = None) -> FakeDoclingConverter:
        nonlocal calls
        calls += 1
        return FakeDoclingConverter(f"# {profile}\n")

    server = DocumentExtractFlightServer(
        "grpc://127.0.0.1:0",
        converter_factory=converter_factory,
    )

    assert (
        server._document_extract_converter({WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER: "full"}) is None
    )
    assert calls == 0


def test_document_extract_table_can_return_error_rows(tmp_path: Path) -> None:
    missing = tmp_path / "missing.pdf"

    table = build_document_extract_table(
        {
            WENDAO_SCHEMA_VERSION_HEADER: EXPECTED_SCHEMA_VERSION,
            WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER: str(missing),
            WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER: "true",
        },
        converter=FakeDoclingConverter(),
    )

    row = table.to_pylist()[0]
    assert row["resourceType"] == "error"
    assert row["status"] == "error"
    assert "does not exist" in row["content"]


def test_document_extract_table_validates_required_headers(tmp_path: Path) -> None:
    source = tmp_path / "manual.pdf"
    source.write_bytes(b"pdf fixture")

    with pytest.raises(ValueError, match="Unexpected schema version"):
        build_document_extract_table(
            {
                WENDAO_SCHEMA_VERSION_HEADER: "v1",
                WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER: str(source),
            },
            converter=FakeDoclingConverter(),
        )

    with pytest.raises(ValueError, match="Missing document source path header"):
        build_document_extract_table(
            {WENDAO_SCHEMA_VERSION_HEADER: EXPECTED_SCHEMA_VERSION},
            converter=FakeDoclingConverter(),
        )

    with pytest.raises(ValueError, match="unsupported document extract profile"):
        build_document_extract_table(
            {
                WENDAO_SCHEMA_VERSION_HEADER: EXPECTED_SCHEMA_VERSION,
                WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER: str(source),
                WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER: "expensive-magic",
            },
            converter=FakeDoclingConverter(),
        )

    with pytest.raises(ValueError, match="1 <= start <= end"):
        build_document_extract_table(
            {
                WENDAO_SCHEMA_VERSION_HEADER: EXPECTED_SCHEMA_VERSION,
                WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER: str(source),
                WENDAO_DOCUMENT_EXTRACT_PAGE_RANGE_HEADER: "3:2",
            },
            converter=FakeDoclingConverter(),
        )


def test_document_extract_routes_include_document_and_internal_arrow_routes() -> None:
    assert SUPPORTED_DOCUMENT_ROUTES == (
        ANALYSIS_DOCUMENT_EXTRACT_ROUTE,
        ANALYSIS_PDF_OCR_SHARDS_ROUTE,
        ANALYSIS_AUDIO_SHARDS_ROUTE,
    )


def test_pdf_ocr_shard_result_table_defaults_to_skipped_rows() -> None:
    table = build_pdf_ocr_shard_result_table(_sample_pdf_ocr_input_table())

    assert table.schema == PDF_OCR_SHARD_RESULT_SCHEMA
    row = table.to_pylist()[0]
    assert row["contractVersion"] == PDF_OCR_SHARD_RESULT_SCHEMA_VERSION
    assert row["status"] == "skipped"
    assert row["text"] is None
    assert row["confidence"] is None
    assert row["errorMessage"] == "OCR shard worker is not configured"
    assert len(row["elementId"]) == 64


def test_audio_shard_result_table_defaults_to_skipped_rows() -> None:
    table = build_audio_shard_result_table(_sample_audio_shard_input_table())

    assert table.schema == AUDIO_SHARD_RESULT_SCHEMA
    row = table.to_pylist()[0]
    assert row["contractVersion"] == AUDIO_SHARD_RESULT_SCHEMA_VERSION
    assert row["status"] == "skipped"
    assert row["text"] is None
    assert row["confidence"] is None
    assert row["errorMessage"] == "audio shard worker is not configured"
    assert len(row["elementId"]) == 64
