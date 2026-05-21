"""Shared helpers for documents tests."""

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
    warm_document_arrow_runtime,
)


class DocumentsFakeDoclingDocument:
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


class DocumentsFakeDoclingResult:
    def __init__(self, markdown: str, document: object | None = None) -> None:
        self.document = (
            document if document is not None else DocumentsFakeDoclingDocument(markdown)
        )


class DocumentsFakeDoclingConverter:
    def __init__(
        self,
        markdown: str = "# Parsed\n\nBody\n",
        document: object | None = None,
    ) -> None:
        self.markdown = markdown
        self.document = document
        self.calls: list[Path] = []
        self.kwargs_calls: list[dict[str, object]] = []

    def convert(
        self, source: str | Path, **kwargs: object
    ) -> DocumentsFakeDoclingResult:
        self.calls.append(Path(source))
        self.kwargs_calls.append(dict(kwargs))
        return DocumentsFakeDoclingResult(self.markdown, self.document)


class FailingConverter:
    def convert(self, source: str | Path) -> DocumentsFakeDoclingResult:
        raise RuntimeError(f"cannot parse {source}")


__all__ = [
    "DOCLING_COMMON_SOURCE_SUFFIXES",
    "DOCLING_SUPPORTED_DOCUMENT_FORMATS",
    "DOCUMENT_RESOURCE_ARROW_CACHE_NAME",
    "DOCUMENT_RESOURCE_FIELDS",
    "DOCUMENT_RESOURCE_SCHEMA",
    "DOCUMENT_STRUCTURE_ARROW_CACHE_NAME",
    "DOCUMENT_STRUCTURE_SCHEMA",
    "DOCUMENT_STRUCTURE_SCHEMA_VERSION",
    "DOCUMENT_TIMING_ARROW_CACHE_NAME",
    "DOCUMENT_TIMING_SCHEMA",
    "DOCUMENT_TIMING_SCHEMA_VERSION",
    "DocumentResourceRow",
    "DocumentStructureBlock",
    "DocumentsFakeDoclingConverter",
    "DocumentsFakeDoclingDocument",
    "DocumentsFakeDoclingResult",
    "FailingConverter",
    "FakeDoclingElement",
    "FakeStructuredDoclingDocument",
    "Path",
    "default_document_output_dir",
    "document_resources_to_table",
    "document_structure_to_table",
    "document_timing_to_table",
    "documents",
    "extract_document_resources",
    "extract_document_table",
    "extract_pdf_resources",
    "is_known_docling_source",
    "pytest",
    "warm_document_arrow_runtime",
]
