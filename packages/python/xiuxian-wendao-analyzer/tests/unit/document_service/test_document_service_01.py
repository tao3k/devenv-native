"""document_service test slice 1."""

from __future__ import annotations

from .support import (
    ANALYSIS_DOCUMENT_EXTRACT_ROUTE,
    ANALYSIS_PDF_OCR_SHARDS_ROUTE,
    DOCUMENT_RESOURCE_SCHEMA,
    EXPECTED_SCHEMA_VERSION,
    PDF_OCR_SHARD_RESULT_SCHEMA,
    PDF_OCR_SHARD_RESULT_SCHEMA_VERSION,
    SUPPORTED_DOCUMENT_ROUTES,
    WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER,
    WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER,
    WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
    WENDAO_SCHEMA_VERSION_HEADER,
    DocumentExtractFlightServer,
    FakeDoclingConverter,
    Path,
    _sample_pdf_ocr_input_table,
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


def test_document_extract_routes_include_only_primary_document_route() -> None:
    assert SUPPORTED_DOCUMENT_ROUTES == (
        ANALYSIS_DOCUMENT_EXTRACT_ROUTE,
        ANALYSIS_PDF_OCR_SHARDS_ROUTE,
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
