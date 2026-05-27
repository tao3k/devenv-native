"""Document extraction schemas and public row types."""

from __future__ import annotations

from dataclasses import asdict, dataclass
from pathlib import Path
from typing import TYPE_CHECKING, Any, Protocol, runtime_checkable

import pyarrow as pa

from .arrow_schema_contracts import ArrowSchemaColumn, build_arrow_schema

if TYPE_CHECKING:
    from collections.abc import Iterable, Mapping


DOCUMENT_RESOURCE_FIELDS = (
    "sourcePath",
    "resourceType",
    "resourcePath",
    "pageIndex",
    "caption",
    "content",
    "mimeType",
    "status",
    "elementId",
)

DOCUMENT_RESOURCE_ARROW_CACHE_NAME = "_resources.arrow"

DOCUMENT_STRUCTURE_ARROW_CACHE_NAME = "_structure.arrow"

DOCUMENT_STRUCTURE_SCHEMA_VERSION = "xiuxian_wendao.document_structure.v1"
DOCUMENT_RESOURCE_TABLE = "document_resource"
DOCUMENT_STRUCTURE_TABLE = "pdf_document_structure"

DOCLING_SUPPORTED_DOCUMENT_FORMATS = (
    "PDF",
    "DOCX",
    "DOC (via Rust gateway legacy Office parser)",
    "XLSX",
    "PPTX",
    "Markdown",
    "AsciiDoc",
    "HTML",
    "XHTML",
    "CSV",
    "PNG",
    "JPEG",
    "TIFF",
    "BMP",
    "WEBP",
    "USPTO XML",
    "JATS XML",
    "XBRL XML",
    "METS GBS",
    "Docling JSON",
    "WebVTT",
    "LaTeX",
    "Plain Text",
    "Audio",
)

DOCLING_COMMON_SOURCE_SUFFIXES = (
    ".pdf",
    ".docx",
    ".doc",
    ".xlsx",
    ".pptx",
    ".md",
    ".adoc",
    ".asciidoc",
    ".html",
    ".htm",
    ".xhtml",
    ".csv",
    ".png",
    ".jpg",
    ".jpeg",
    ".tif",
    ".tiff",
    ".bmp",
    ".webp",
    ".xml",
    ".xbrl",
    ".mets",
    ".json",
    ".vtt",
    ".webvtt",
    ".tex",
    ".latex",
    ".txt",
    ".text",
    ".qmd",
    ".rmd",
    ".mp3",
    ".wav",
    ".m4a",
)

DOCUMENT_RESOURCE_SCHEMA = build_arrow_schema(
    DOCUMENT_RESOURCE_TABLE,
    (
        ArrowSchemaColumn("sourcePath", pa.utf8()),
        ArrowSchemaColumn("resourceType", pa.utf8()),
        ArrowSchemaColumn("resourcePath", pa.utf8()),
        ArrowSchemaColumn("pageIndex", pa.int32()),
        ArrowSchemaColumn("caption", pa.utf8()),
        ArrowSchemaColumn("content", pa.utf8()),
        ArrowSchemaColumn("mimeType", pa.utf8()),
        ArrowSchemaColumn("status", pa.utf8()),
        ArrowSchemaColumn("elementId", pa.utf8()),
    ),
)

DOCUMENT_STRUCTURE_SCHEMA = build_arrow_schema(
    DOCUMENT_STRUCTURE_TABLE,
    (
        ArrowSchemaColumn("contractVersion", pa.utf8()),
        ArrowSchemaColumn("sourcePath", pa.utf8()),
        ArrowSchemaColumn("sourceContentHash", pa.utf8()),
        ArrowSchemaColumn("blockId", pa.utf8()),
        ArrowSchemaColumn("parentBlockId", pa.utf8()),
        ArrowSchemaColumn("pageIndex", pa.int32()),
        ArrowSchemaColumn("blockIndex", pa.int32()),
        ArrowSchemaColumn("readingOrderKey", pa.utf8()),
        ArrowSchemaColumn("blockType", pa.utf8()),
        ArrowSchemaColumn("resourceElementId", pa.utf8()),
        ArrowSchemaColumn("content", pa.utf8()),
        ArrowSchemaColumn("mimeType", pa.utf8()),
        ArrowSchemaColumn("status", pa.utf8()),
        ArrowSchemaColumn("engine", pa.utf8()),
        ArrowSchemaColumn("confidence", pa.float64()),
        ArrowSchemaColumn("bboxLeft", pa.float64()),
        ArrowSchemaColumn("bboxTop", pa.float64()),
        ArrowSchemaColumn("bboxRight", pa.float64()),
        ArrowSchemaColumn("bboxBottom", pa.float64()),
        ArrowSchemaColumn("provenance", pa.utf8()),
    ),
)


class DoclingDocumentProtocol(Protocol):
    """Docling document behavior used by this module."""

    def export_to_markdown(self) -> str: ...


@runtime_checkable
class DoclingDocumentDictExportProtocol(Protocol):
    """Optional Docling behavior for lossless JSON resource export."""

    def export_to_dict(self) -> dict[str, Any]: ...


class DoclingConversionResultProtocol(Protocol):
    """Docling conversion result shape used by this module."""

    document: DoclingDocumentProtocol


class DocumentConverterProtocol(Protocol):
    """Minimal converter seam for Docling and tests."""

    def convert(
        self,
        source: str | Path,
        **kwargs: Any,
    ) -> DoclingConversionResultProtocol: ...


@dataclass(frozen=True, slots=True)
class DocumentResourceRow:
    """Arrow-friendly row for one extracted document resource."""

    sourcePath: str
    resourceType: str
    resourcePath: str
    pageIndex: int
    caption: str
    content: str
    mimeType: str
    status: str
    elementId: str

    def to_mapping(self) -> dict[str, object]:
        """Return a mapping with the stable Arrow resource columns."""

        return asdict(self)


@dataclass(frozen=True, slots=True)
class DocumentStructureBlock:
    """Internal Arrow-friendly structure block used to preserve read order."""

    contractVersion: str
    sourcePath: str
    sourceContentHash: str
    blockId: str
    parentBlockId: str
    pageIndex: int
    blockIndex: int
    readingOrderKey: str
    blockType: str
    resourceElementId: str
    content: str
    mimeType: str
    status: str
    engine: str
    confidence: float | None
    bboxLeft: float | None
    bboxTop: float | None
    bboxRight: float | None
    bboxBottom: float | None
    provenance: str

    def to_mapping(self) -> dict[str, object]:
        """Return a mapping with the internal structure Arrow columns."""

        return asdict(self)


def default_document_output_dir(source_path: str | Path) -> Path:
    """Return the default extraction directory for one source document."""

    return Path(source_path).with_suffix(Path(source_path).suffix + ".extracted")


def is_known_docling_source(source_path: str | Path) -> bool:
    """Return whether the path has a common Docling-supported source suffix.

    This is a UX helper, not a parser gate. Docling remains the authority for
    actual conversion support.
    """

    return Path(source_path).suffix.lower() in DOCLING_COMMON_SOURCE_SUFFIXES


def document_resources_to_table(
    resources: Iterable[DocumentResourceRow | Mapping[str, object]],
) -> pa.Table:
    """Convert document resource rows into the stable Arrow table shape."""

    rows = [
        (resource.to_mapping() if isinstance(resource, DocumentResourceRow) else dict(resource))
        for resource in resources
    ]
    return pa.Table.from_pylist(rows, schema=DOCUMENT_RESOURCE_SCHEMA)


def document_structure_to_table(
    blocks: Iterable[DocumentStructureBlock | Mapping[str, object]],
) -> pa.Table:
    """Convert structure blocks into the internal Arrow sidecar table."""

    rows = [
        block.to_mapping() if isinstance(block, DocumentStructureBlock) else dict(block)
        for block in sorted(
            blocks,
            key=lambda item: _structure_sort_key(
                item.to_mapping() if isinstance(item, DocumentStructureBlock) else item
            ),
        )
    ]
    return pa.Table.from_pylist(rows, schema=DOCUMENT_STRUCTURE_SCHEMA)


def _structure_sort_key(row: Mapping[str, object]) -> tuple[int, str, int, str]:
    return (
        int(row.get("pageIndex") or 0),
        str(row.get("readingOrderKey") or ""),
        int(row.get("blockIndex") or 0),
        str(row.get("blockId") or ""),
    )
