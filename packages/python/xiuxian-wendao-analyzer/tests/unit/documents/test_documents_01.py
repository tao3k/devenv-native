"""documents test slice 1."""

from __future__ import annotations

from .support import (
    DOCUMENT_RESOURCE_ARROW_CACHE_NAME,
    DOCUMENT_RESOURCE_FIELDS,
    DOCUMENT_RESOURCE_SCHEMA,
    DOCUMENT_STRUCTURE_ARROW_CACHE_NAME,
    DOCUMENT_STRUCTURE_SCHEMA,
    DOCUMENT_STRUCTURE_SCHEMA_VERSION,
    DOCUMENT_TIMING_ARROW_CACHE_NAME,
    DOCUMENT_TIMING_SCHEMA,
    DOCUMENT_TIMING_SCHEMA_VERSION,
    DocumentResourceRow,
    DocumentsFakeDoclingConverter,
    FailingConverter,
    FakeStructuredDoclingDocument,
    Path,
    documents,
    extract_document_resources,
    extract_document_table,
    pytest,
)


def test_extract_document_resources_writes_markdown_and_arrow_cache(
    tmp_path: Path,
) -> None:
    source = tmp_path / "handbook.docx"
    source.write_bytes(b"docx fixture")
    output_dir = tmp_path / "handbook-output"
    converter = DocumentsFakeDoclingConverter("# Handbook\n\nText\n")

    rows = extract_document_resources(source, output_dir, converter=converter)

    assert converter.calls == [source]
    assert rows == [
        DocumentResourceRow(
            sourcePath=str(source),
            resourceType="document",
            resourcePath=str(output_dir / "handbook.md"),
            pageIndex=0,
            caption="",
            content="# Handbook\n\nText\n",
            mimeType="text/markdown",
            status="ok",
            elementId="_main",
        )
    ]
    assert (output_dir / "handbook.md").read_text(
        encoding="utf-8"
    ) == "# Handbook\n\nText\n"
    assert (output_dir / DOCUMENT_RESOURCE_ARROW_CACHE_NAME).exists()
    assert not (output_dir / "_metadata.json").exists()
    assert (output_dir / "_complete.marker").exists()


def test_extract_document_resources_writes_timing_sidecar(tmp_path: Path) -> None:
    source = tmp_path / "image.png"
    source.write_bytes(b"png fixture")
    output_dir = tmp_path / "image-output"

    extract_document_resources(
        source,
        output_dir,
        converter=DocumentsFakeDoclingConverter("# Image\n"),
    )

    timing_path = output_dir / DOCUMENT_TIMING_ARROW_CACHE_NAME
    assert timing_path.exists()
    with documents.pa.ipc.open_file(timing_path) as reader:
        timing = reader.read_all()

    assert timing.schema == DOCUMENT_TIMING_SCHEMA
    rows = timing.to_pylist()
    phases = {row["phase"] for row in rows}
    assert {
        "doclingConvert",
        "doclingMarkdownExport",
        "writeMarkdown",
        "sourceHash",
        "resourceRowsBuild",
        "structureRowsBuild",
        "writeStructureArrow",
        "writeResourcesArrow",
        "total",
    }.issubset(phases)
    total = next(row for row in rows if row["phase"] == "total")
    assert total["contractVersion"] == DOCUMENT_TIMING_SCHEMA_VERSION
    assert total["sourceSuffix"] == ".png"
    assert total["status"] == "ok"
    assert total["resourceRows"] == 1
    assert total["structureRows"] == 1
    assert total["elapsedMs"] >= 0.0


def test_extract_document_resources_uses_fresh_cache(tmp_path: Path) -> None:
    source = tmp_path / "notes.docx"
    source.write_bytes(b"docx fixture")
    output_dir = tmp_path / "notes-output"

    first_rows = extract_document_resources(
        source,
        output_dir,
        converter=DocumentsFakeDoclingConverter("# Notes\n"),
    )
    cached_rows = extract_document_resources(
        source, output_dir, converter=FailingConverter()
    )

    assert cached_rows == first_rows


def test_extract_document_table_uses_resource_schema(tmp_path: Path) -> None:
    source = tmp_path / "report.xlsx"
    source.write_bytes(b"xlsx fixture")

    table = extract_document_table(
        source, converter=DocumentsFakeDoclingConverter("# Workbook\n")
    )

    assert table.schema == DOCUMENT_RESOURCE_SCHEMA
    assert table.column_names == list(DOCUMENT_RESOURCE_FIELDS)
    assert table.to_pylist()[0]["content"] == "# Workbook\n"


def test_extract_document_table_returns_cached_arrow_table_without_row_roundtrip(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    source = tmp_path / "cached.docx"
    source.write_bytes(b"docx fixture")
    output_dir = tmp_path / "cached-output"
    extract_document_resources(
        source,
        output_dir,
        converter=DocumentsFakeDoclingConverter("# Cached\n"),
    )

    def fail_row_roundtrip(row: object) -> object:
        raise AssertionError(f"unexpected row roundtrip: {row}")

    monkeypatch.setattr(documents, "_resource_from_mapping", fail_row_roundtrip)

    table = extract_document_table(source, output_dir, converter=FailingConverter())

    assert table.schema == DOCUMENT_RESOURCE_SCHEMA
    assert table.to_pylist()[0]["content"] == "# Cached\n"


def test_extract_document_resources_emits_structured_docling_rows(
    tmp_path: Path,
) -> None:
    source = tmp_path / "lecture.mp3"
    source.write_bytes(b"audio fixture")
    output_dir = tmp_path / "lecture-output"

    rows = extract_document_resources(
        source,
        output_dir,
        converter=DocumentsFakeDoclingConverter(
            document=FakeStructuredDoclingDocument()
        ),
    )

    row_by_type = {row.resourceType: row for row in rows}
    expected_resource_types = {
        "document",
        "docling_json",
        "table",
        "image",
        "formula",
        "code",
        "audio",
        "subtitle",
    }
    assert expected_resource_types.issubset(row_by_type)
    assert row_by_type["docling_json"].mimeType == "application/json"
    assert row_by_type["docling_json"].content == ""
    assert (output_dir / "lecture.docling.json").read_text(
        encoding="utf-8"
    ) == '{"name": "structured", "schema_name": "DoclingDocument"}'
    assert row_by_type["table"].pageIndex == 1
    assert row_by_type["formula"].mimeType == "application/x-tex"
    assert row_by_type["subtitle"].mimeType == "text/vtt"
    assert (output_dir / "lecture.docling.json").exists()
    structure_path = output_dir / DOCUMENT_STRUCTURE_ARROW_CACHE_NAME
    assert structure_path.exists()
    with documents.pa.ipc.open_file(structure_path) as reader:
        structure = reader.read_all()
    assert structure.schema == DOCUMENT_STRUCTURE_SCHEMA
    structure_rows = structure.to_pylist()
    assert [row["contractVersion"] for row in structure_rows] == [
        DOCUMENT_STRUCTURE_SCHEMA_VERSION
    ] * len(structure_rows)
    assert [row["blockType"] for row in structure_rows[:2]] == ["document", "audio"]
    table_row = next(row for row in structure_rows if row["blockType"] == "table")
    assert table_row["pageIndex"] == 1
    assert table_row["confidence"] == 0.97
    assert table_row["resourceElementId"] == "tables-0"
