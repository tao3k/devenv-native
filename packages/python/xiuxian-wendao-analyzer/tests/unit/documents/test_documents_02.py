"""documents test slice 2."""

from __future__ import annotations

from .support import (
    DOCUMENT_RESOURCE_SCHEMA,
    DOCUMENT_STRUCTURE_SCHEMA,
    DOCUMENT_STRUCTURE_SCHEMA_VERSION,
    DOCUMENT_TIMING_SCHEMA,
    DOCUMENT_TIMING_SCHEMA_VERSION,
    DocumentsFakeDoclingConverter,
    DocumentStructureBlock,
    FailingConverter,
    Path,
    document_resources_to_table,
    document_structure_to_table,
    document_timing_to_table,
    extract_document_resources,
    pytest,
    warm_document_arrow_runtime,
)


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


def test_warm_document_arrow_runtime_is_idempotent() -> None:
    warm_document_arrow_runtime()
    warm_document_arrow_runtime()


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
            tmp_path / "missing.pdf", converter=DocumentsFakeDoclingConverter()
        )
