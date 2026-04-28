from __future__ import annotations

from pathlib import Path

import pytest

from xiuxian_wendao_analyzer import (
    ANALYSIS_DOCUMENT_EXTRACT_ROUTE,
    DOCUMENT_RESOURCE_SCHEMA,
    EXPECTED_SCHEMA_VERSION,
    SUPPORTED_DOCUMENT_ROUTES,
    WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER,
    WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER,
    WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
    WENDAO_SCHEMA_VERSION_HEADER,
    build_document_extract_table,
)


class FakeDoclingDocument:
    def __init__(self, markdown: str) -> None:
        self.markdown = markdown

    def export_to_markdown(self) -> str:
        return self.markdown


class FakeDoclingResult:
    def __init__(self, markdown: str) -> None:
        self.document = FakeDoclingDocument(markdown)


class FakeDoclingConverter:
    def __init__(self, markdown: str = "# Service\n") -> None:
        self.markdown = markdown
        self.calls: list[Path] = []

    def convert(self, source: str | Path) -> FakeDoclingResult:
        self.calls.append(Path(source))
        return FakeDoclingResult(self.markdown)


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


def test_document_extract_routes_include_only_primary_document_route() -> None:
    assert SUPPORTED_DOCUMENT_ROUTES == (ANALYSIS_DOCUMENT_EXTRACT_ROUTE,)
