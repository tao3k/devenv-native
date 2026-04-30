from __future__ import annotations

from pathlib import Path

import pytest

import xiuxian_wendao_analyzer.documents as documents
from xiuxian_wendao_analyzer import (
    DOCLING_COMMON_SOURCE_SUFFIXES,
    DOCLING_SUPPORTED_DOCUMENT_FORMATS,
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
    DocumentStructureBlock,
    default_document_output_dir,
    document_resources_to_table,
    document_structure_to_table,
    document_timing_to_table,
    extract_document_resources,
    extract_document_table,
    extract_pdf_resources,
    is_known_docling_source,
)


class FakeDoclingDocument:
    def __init__(self, markdown: str) -> None:
        self.markdown = markdown

    def export_to_markdown(self) -> str:
        return self.markdown


class FakeDoclingElement:
    def __init__(
        self,
        *,
        text: str = "",
        self_ref: str = "",
        caption: str = "",
        page_no: int = 1,
        resource_path: str = "",
        confidence: float | None = None,
    ) -> None:
        self.text = text
        self.self_ref = self_ref
        self.caption = caption
        self.page_no = page_no
        self.resource_path = resource_path
        self.confidence = confidence


class FakeStructuredDoclingDocument:
    def __init__(self) -> None:
        self.tables = [
            FakeDoclingElement(
                text="| A | B |\n| - | - |\n| 1 | 2 |",
                self_ref="#/tables/0",
                caption="Example table",
                page_no=2,
                confidence=0.97,
            )
        ]
        self.pictures = [
            FakeDoclingElement(
                text="chart image",
                self_ref="#/pictures/0",
                caption="Chart",
                page_no=3,
            )
        ]
        self.formulas = [
            FakeDoclingElement(text="E = mc^2", self_ref="#/formulas/0", page_no=4)
        ]
        self.code_blocks = [
            FakeDoclingElement(text="print('hello')", self_ref="#/code/0", page_no=5)
        ]
        self.audio_segments = [
            FakeDoclingElement(
                text="spoken words",
                self_ref="#/audio/0",
                caption="Audio segment",
                page_no=1,
            )
        ]
        self.subtitles = [
            FakeDoclingElement(
                text="00:00.000 --> 00:01.000\nHello", self_ref="#/cues/0"
            )
        ]

    def export_to_markdown(self) -> str:
        return "# Structured\n"

    def export_to_dict(self) -> dict[str, object]:
        return {"schema_name": "DoclingDocument", "name": "structured"}


class FakeDoclingResult:
    def __init__(self, markdown: str, document: object | None = None) -> None:
        self.document = (
            document if document is not None else FakeDoclingDocument(markdown)
        )


class FakeDoclingConverter:
    def __init__(
        self,
        markdown: str = "# Parsed\n\nBody\n",
        document: object | None = None,
    ) -> None:
        self.markdown = markdown
        self.document = document
        self.calls: list[Path] = []

    def convert(self, source: str | Path) -> FakeDoclingResult:
        self.calls.append(Path(source))
        return FakeDoclingResult(self.markdown, self.document)


class FailingConverter:
    def convert(self, source: str | Path) -> FakeDoclingResult:
        raise RuntimeError(f"cannot parse {source}")


def test_extract_document_resources_writes_markdown_and_arrow_cache(
    tmp_path: Path,
) -> None:
    source = tmp_path / "handbook.docx"
    source.write_bytes(b"docx fixture")
    output_dir = tmp_path / "handbook-output"
    converter = FakeDoclingConverter("# Handbook\n\nText\n")

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
        converter=FakeDoclingConverter("# Image\n"),
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
        converter=FakeDoclingConverter("# Notes\n"),
    )
    cached_rows = extract_document_resources(
        source, output_dir, converter=FailingConverter()
    )

    assert cached_rows == first_rows


def test_extract_document_table_uses_resource_schema(tmp_path: Path) -> None:
    source = tmp_path / "report.xlsx"
    source.write_bytes(b"xlsx fixture")

    table = extract_document_table(
        source, converter=FakeDoclingConverter("# Workbook\n")
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
        converter=FakeDoclingConverter("# Cached\n"),
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
        converter=FakeDoclingConverter(document=FakeStructuredDoclingDocument()),
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


def test_document_resources_to_table_accepts_mappings() -> None:
    table = document_resources_to_table(
        [
            {
                "sourcePath": "source.pdf",
                "resourceType": "document",
                "resourcePath": "source.md",
                "pageIndex": 0,
                "caption": "",
                "content": "# Source\n",
                "mimeType": "text/markdown",
                "status": "ok",
                "elementId": "_main",
            }
        ]
    )

    assert table.schema == DOCUMENT_RESOURCE_SCHEMA
    assert table.to_pylist()[0]["sourcePath"] == "source.pdf"


def test_document_structure_to_table_sorts_reading_order() -> None:
    table = document_structure_to_table(
        [
            DocumentStructureBlock(
                contractVersion=DOCUMENT_STRUCTURE_SCHEMA_VERSION,
                sourcePath="source.pdf",
                sourceContentHash="hash",
                blockId="b",
                parentBlockId="",
                pageIndex=1,
                blockIndex=2,
                readingOrderKey="000001.000002",
                blockType="ocr_text",
                resourceElementId="b",
                content="second",
                mimeType="text/plain",
                status="succeeded",
                engine="docling",
                confidence=None,
                bboxLeft=None,
                bboxTop=None,
                bboxRight=None,
                bboxBottom=None,
                provenance="{}",
            ),
            DocumentStructureBlock(
                contractVersion=DOCUMENT_STRUCTURE_SCHEMA_VERSION,
                sourcePath="source.pdf",
                sourceContentHash="hash",
                blockId="a",
                parentBlockId="",
                pageIndex=0,
                blockIndex=1,
                readingOrderKey="000000.000001",
                blockType="text_page",
                resourceElementId="a",
                content="first",
                mimeType="text/markdown",
                status="ok",
                engine="wendao-hybrid",
                confidence=None,
                bboxLeft=None,
                bboxTop=None,
                bboxRight=None,
                bboxBottom=None,
                provenance="{}",
            ),
        ]
    )

    assert table.schema == DOCUMENT_STRUCTURE_SCHEMA
    assert [row["blockId"] for row in table.to_pylist()] == ["a", "b"]


def test_document_timing_to_table_uses_stable_schema() -> None:
    table = document_timing_to_table(
        [
            {
                "contractVersion": DOCUMENT_TIMING_SCHEMA_VERSION,
                "sourcePath": "source.png",
                "sourceSuffix": ".png",
                "phase": "doclingConvert",
                "elapsedMs": 12.5,
                "status": "ok",
                "detail": "",
                "resourceRows": 1,
                "structureRows": 1,
            }
        ]
    )

    assert table.schema == DOCUMENT_TIMING_SCHEMA
    assert table.to_pylist()[0]["phase"] == "doclingConvert"


def test_extract_document_resources_can_return_error_row(tmp_path: Path) -> None:
    source = tmp_path / "broken.pdf"
    source.write_bytes(b"bad fixture")

    rows = extract_document_resources(
        source, converter=FailingConverter(), error_row=True
    )

    assert rows[0].resourceType == "error"
    assert rows[0].status == "error"
    assert "cannot parse" in rows[0].content


def test_extract_document_resources_raises_for_missing_source(tmp_path: Path) -> None:
    with pytest.raises(FileNotFoundError):
        extract_document_resources(
            tmp_path / "missing.pdf", converter=FakeDoclingConverter()
        )


def test_pdf_compatibility_wrapper_delegates_to_document_extraction(
    tmp_path: Path,
) -> None:
    source = tmp_path / "legacy.pdf"
    source.write_bytes(b"%PDF fixture")

    rows = extract_pdf_resources(source, converter=FakeDoclingConverter("# Legacy\n"))

    assert rows[0].sourcePath == str(source)
    assert rows[0].content == "# Legacy\n"


def test_default_document_output_dir_preserves_source_suffix() -> None:
    assert default_document_output_dir("manual.docx") == Path("manual.docx.extracted")


def test_known_docling_source_suffixes_cover_common_document_formats() -> None:
    expected_suffixes = {
        ".pdf",
        ".docx",
        ".xlsx",
        ".pptx",
        ".md",
        ".html",
        ".csv",
        ".png",
        ".jpg",
        ".tiff",
        ".webp",
        ".xml",
        ".json",
        ".xbrl",
        ".vtt",
        ".tex",
        ".txt",
        ".qmd",
        ".mp3",
        ".wav",
    }

    assert expected_suffixes.issubset(set(DOCLING_COMMON_SOURCE_SUFFIXES))
    assert {
        "PDF",
        "DOCX",
        "XLSX",
        "PPTX",
        "HTML",
        "CSV",
        "PNG",
        "Docling JSON",
        "XBRL XML",
        "METS GBS",
        "WebVTT",
        "LaTeX",
        "Plain Text",
        "Audio",
    }.issubset(set(DOCLING_SUPPORTED_DOCUMENT_FORMATS))


def test_is_known_docling_source_is_a_case_insensitive_suffix_helper() -> None:
    assert is_known_docling_source("deck.PPTX")
    assert is_known_docling_source("scan.TIFF")
    assert is_known_docling_source("captions.VTT")
    assert is_known_docling_source("lecture.MP3")
    assert is_known_docling_source("paper.TEX")
    assert not is_known_docling_source("archive.zip")
