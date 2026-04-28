"""Docling-backed document extraction helpers for analyzer workflows."""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import TYPE_CHECKING, Any, Protocol, runtime_checkable

import pyarrow as pa

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

DOCLING_SUPPORTED_DOCUMENT_FORMATS = (
    "PDF",
    "DOCX",
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

DOCUMENT_RESOURCE_SCHEMA = pa.schema(
    [
        pa.field("sourcePath", pa.utf8()),
        pa.field("resourceType", pa.utf8()),
        pa.field("resourcePath", pa.utf8()),
        pa.field("pageIndex", pa.int32()),
        pa.field("caption", pa.utf8()),
        pa.field("content", pa.utf8()),
        pa.field("mimeType", pa.utf8()),
        pa.field("status", pa.utf8()),
        pa.field("elementId", pa.utf8()),
    ]
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

    def convert(self, source: str | Path) -> DoclingConversionResultProtocol: ...


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
        resource.to_mapping() if isinstance(resource, DocumentResourceRow) else dict(resource)
        for resource in resources
    ]
    return pa.Table.from_pylist(rows, schema=DOCUMENT_RESOURCE_SCHEMA)


def extract_document_table(
    source_path: str | Path,
    output_dir: str | Path | None = None,
    *,
    converter: DocumentConverterProtocol | None = None,
    force: bool = False,
    error_row: bool = False,
) -> pa.Table:
    """Extract one document and return Arrow resource rows.

    # Errors

    Raises `FileNotFoundError` when the source path does not exist. Raises
    `RuntimeError` when Docling is not installed and no converter is provided.
    Raises conversion exceptions unless `error_row` is true.
    """

    source = Path(source_path)
    if source.exists() and not force:
        out = Path(output_dir) if output_dir is not None else default_document_output_dir(source)
        cached_table = _read_cached_table(source, out)
        if cached_table is not None:
            return cached_table

    return document_resources_to_table(
        extract_document_resources(
            source,
            output_dir,
            converter=converter,
            force=force,
            error_row=error_row,
        )
    )


def extract_document_resources(
    source_path: str | Path,
    output_dir: str | Path | None = None,
    *,
    converter: DocumentConverterProtocol | None = None,
    force: bool = False,
    error_row: bool = False,
) -> list[DocumentResourceRow]:
    """Extract one local document into Arrow-friendly resource rows.

    # Errors

    Raises `FileNotFoundError` when the source path does not exist. Raises
    `RuntimeError` when Docling is not installed and no converter is provided.
    Raises conversion exceptions unless `error_row` is true.
    """

    source = Path(source_path)
    if not source.exists():
        if error_row:
            return [
                DocumentResourceRow(
                    sourcePath=str(source),
                    resourceType="error",
                    resourcePath="",
                    pageIndex=0,
                    caption="",
                    content=f"document source path does not exist: {source}",
                    mimeType="text/plain",
                    status="error",
                    elementId="",
                )
            ]
        raise FileNotFoundError(f"document source path does not exist: {source}")

    out = Path(output_dir) if output_dir is not None else default_document_output_dir(source)
    out.mkdir(parents=True, exist_ok=True)

    if not force:
        cached = _read_cached_resources(source, out)
        if cached is not None:
            return cached

    try:
        resolved_converter = converter if converter is not None else _new_docling_converter()
        document = resolved_converter.convert(source).document
        markdown_text = document.export_to_markdown()
        markdown_path = out / f"{source.stem}.md"
        markdown_path.write_text(markdown_text, encoding="utf-8")
        resources = [
            DocumentResourceRow(
                sourcePath=str(source),
                resourceType="document",
                resourcePath=str(markdown_path),
                pageIndex=0,
                caption="",
                content=markdown_text,
                mimeType="text/markdown",
                status="ok",
                elementId="_main",
            )
        ]
        resources.extend(_structured_document_resources(source, out, document))
        _write_cached_resources(out, resources)
        return resources
    except Exception as exc:
        if not error_row:
            raise
        return [
            DocumentResourceRow(
                sourcePath=str(source),
                resourceType="error",
                resourcePath="",
                pageIndex=0,
                caption="",
                content=str(exc),
                mimeType="text/plain",
                status="error",
                elementId="",
            )
        ]


def extract_pdf_resources(
    source_path: str | Path,
    output_dir: str | Path | None = None,
    *,
    converter: DocumentConverterProtocol | None = None,
    force: bool = False,
    error_row: bool = False,
) -> list[DocumentResourceRow]:
    """Compatibility wrapper for PDF callers migrating to document extraction.

    # Errors

    Raises the same errors as `extract_document_resources`.
    """

    return extract_document_resources(
        source_path,
        output_dir,
        converter=converter,
        force=force,
        error_row=error_row,
    )


def extract_pdf_table(
    source_path: str | Path,
    output_dir: str | Path | None = None,
    *,
    converter: DocumentConverterProtocol | None = None,
    force: bool = False,
    error_row: bool = False,
) -> pa.Table:
    """Compatibility wrapper for PDF callers that need an Arrow table.

    # Errors

    Raises the same errors as `extract_document_table`.
    """

    return extract_document_table(
        source_path,
        output_dir,
        converter=converter,
        force=force,
        error_row=error_row,
    )


def _new_docling_converter() -> DocumentConverterProtocol:
    try:
        from docling.document_converter import DocumentConverter
    except ModuleNotFoundError as exc:
        raise RuntimeError(
            "docling is not installed; install xiuxian-wendao-analyzer[documents] "
            "to enable document extraction"
        ) from exc
    return DocumentConverter()


def _read_cached_resources(source: Path, output_dir: Path) -> list[DocumentResourceRow] | None:
    table = _read_cached_table(source, output_dir)
    if table is None:
        return None
    return [_resource_from_mapping(row) for row in table.to_pylist()]


def _read_cached_table(source: Path, output_dir: Path) -> pa.Table | None:
    marker = output_dir / "_complete.marker"
    resources_path = output_dir / DOCUMENT_RESOURCE_ARROW_CACHE_NAME
    if not marker.exists() or not resources_path.exists():
        return None
    if source.stat().st_mtime > marker.stat().st_mtime:
        return None

    try:
        with pa.ipc.open_file(resources_path) as reader:
            table = reader.read_all()
    except (pa.ArrowInvalid, OSError):
        return None
    if table.schema != DOCUMENT_RESOURCE_SCHEMA:
        return None
    return table


def _write_cached_resources(output_dir: Path, resources: list[DocumentResourceRow]) -> None:
    resources_path = output_dir / DOCUMENT_RESOURCE_ARROW_CACHE_NAME
    table = document_resources_to_table(resources)
    with pa.ipc.new_file(resources_path, DOCUMENT_RESOURCE_SCHEMA) as writer:
        writer.write_table(table)
    (output_dir / "_complete.marker").touch()


def _structured_document_resources(
    source: Path,
    output_dir: Path,
    document: DoclingDocumentProtocol,
) -> list[DocumentResourceRow]:
    resources: list[DocumentResourceRow] = []
    resources.extend(_docling_json_resource(source, output_dir, document))

    for resource_type, attribute_names in _STRUCTURED_RESOURCE_COLLECTIONS:
        for index, element in enumerate(
            _iter_document_elements(document, attribute_names),
            start=1,
        ):
            row = _resource_from_element(source, output_dir, resource_type, element, index)
            if row is not None:
                resources.append(row)

    return resources


_STRUCTURED_RESOURCE_COLLECTIONS: tuple[tuple[str, tuple[str, ...]], ...] = (
    ("table", ("tables",)),
    ("image", ("pictures", "images", "figures")),
    ("formula", ("formulas", "equations")),
    ("code", ("code_blocks", "codeblocks")),
    ("audio", ("audio_segments", "audio", "tracks")),
    ("subtitle", ("subtitles", "cues", "vtt_cues")),
)


def _docling_json_resource(
    source: Path,
    output_dir: Path,
    document: DoclingDocumentProtocol,
) -> list[DocumentResourceRow]:
    if not isinstance(document, DoclingDocumentDictExportProtocol):
        return []

    content = json.dumps(document.export_to_dict(), ensure_ascii=False, sort_keys=True)
    resource_path = output_dir / f"{source.stem}.docling.json"
    resource_path.write_text(content, encoding="utf-8")
    return [
        DocumentResourceRow(
            sourcePath=str(source),
            resourceType="docling_json",
            resourcePath=str(resource_path),
            pageIndex=0,
            caption="",
            content="",
            mimeType="application/json",
            status="ok",
            elementId="_docling_json",
        )
    ]


def _iter_document_elements(
    document: DoclingDocumentProtocol,
    attribute_names: tuple[str, ...],
) -> list[Any]:
    elements: list[Any] = []
    seen_ids: set[int] = set()
    for attribute_name in attribute_names:
        value = getattr(document, attribute_name, None)
        if value is None:
            continue
        if isinstance(value, dict):
            candidates = value.values()
        elif isinstance(value, (str, bytes)):
            candidates = (value,)
        else:
            try:
                candidates = iter(value)
            except TypeError:
                candidates = (value,)
        for candidate in candidates:
            candidate_id = id(candidate)
            if candidate_id in seen_ids:
                continue
            seen_ids.add(candidate_id)
            elements.append(candidate)
    return elements


def _resource_from_element(
    source: Path,
    output_dir: Path,
    resource_type: str,
    element: Any,
    index: int,
) -> DocumentResourceRow | None:
    content = _element_content(element)
    resource_path = _element_resource_path(element)
    if content and not resource_path:
        suffix = _resource_file_suffix(resource_type)
        resource_file = output_dir / f"{resource_type}-{index}{suffix}"
        resource_file.write_text(content, encoding="utf-8")
        resource_path = str(resource_file)
    if not content and not resource_path:
        return None

    return DocumentResourceRow(
        sourcePath=str(source),
        resourceType=resource_type,
        resourcePath=resource_path,
        pageIndex=_element_page_index(element),
        caption=_element_caption(element),
        content=content,
        mimeType=_resource_mime_type(resource_type, resource_path),
        status="ok",
        elementId=_element_id(element, resource_type, index),
    )


def _element_content(element: Any) -> str:
    for method_name in ("export_to_markdown", "export_to_html", "export_to_text"):
        method = getattr(element, method_name, None)
        if callable(method):
            try:
                value = method()
            except TypeError:
                continue
            if value:
                return str(value)
    for attribute_name in ("text", "content", "transcript", "caption", "label"):
        value = getattr(element, attribute_name, None)
        if value:
            return str(value)
    if isinstance(element, str):
        return element
    return ""


def _element_resource_path(element: Any) -> str:
    for attribute_name in ("resource_path", "path", "uri"):
        value = getattr(element, attribute_name, None)
        if value:
            return str(value)
    image = getattr(element, "image", None)
    if image is not None:
        uri = getattr(image, "uri", None)
        if uri:
            return str(uri)
    return ""


def _element_caption(element: Any) -> str:
    value = getattr(element, "caption", "")
    if isinstance(value, list):
        return " ".join(str(item) for item in value if item)
    return str(value) if value else ""


def _element_page_index(element: Any) -> int:
    page_no = getattr(element, "page_no", None)
    if page_no is None:
        provenance = getattr(element, "prov", None) or getattr(element, "provenance", None)
        if provenance:
            first = provenance[0] if isinstance(provenance, (list, tuple)) else provenance
            page_no = getattr(first, "page_no", None)
    try:
        return max(int(page_no) - 1, 0) if page_no is not None else 0
    except (TypeError, ValueError):
        return 0


def _element_id(element: Any, resource_type: str, index: int) -> str:
    for attribute_name in ("self_ref", "cref", "id", "name"):
        value = getattr(element, attribute_name, None)
        if value:
            return str(value).strip("#/").replace("/", "-")
    return f"{resource_type}-{index}"


def _resource_file_suffix(resource_type: str) -> str:
    return {
        "docling_json": ".json",
        "table": ".md",
        "image": ".txt",
        "formula": ".tex",
        "code": ".txt",
        "audio": ".txt",
        "subtitle": ".vtt",
    }.get(resource_type, ".txt")


def _resource_mime_type(resource_type: str, resource_path: str) -> str:
    if resource_path:
        suffix = Path(resource_path).suffix.lower()
        if suffix == ".json":
            return "application/json"
        if suffix == ".png":
            return "image/png"
        if suffix in {".jpg", ".jpeg"}:
            return "image/jpeg"
        if suffix in {".tif", ".tiff"}:
            return "image/tiff"
        if suffix == ".bmp":
            return "image/bmp"
        if suffix == ".webp":
            return "image/webp"
        if suffix == ".mp3":
            return "audio/mpeg"
        if suffix == ".wav":
            return "audio/wav"
        if suffix == ".m4a":
            return "audio/mp4"
        if suffix == ".html":
            return "text/html"
        if suffix in {".md", ".markdown"}:
            return "text/markdown"
        if suffix in {".tex", ".latex"}:
            return "application/x-tex"
        if suffix in {".vtt", ".webvtt"}:
            return "text/vtt"
    return {
        "docling_json": "application/json",
        "table": "text/markdown",
        "image": "text/plain",
        "formula": "application/x-tex",
        "code": "text/plain",
        "audio": "text/plain",
        "subtitle": "text/vtt",
    }.get(resource_type, "text/plain")


def _resource_from_mapping(row: Mapping[str, Any]) -> DocumentResourceRow:
    return DocumentResourceRow(
        sourcePath=str(row.get("sourcePath", "")),
        resourceType=str(row.get("resourceType", "")),
        resourcePath=str(row.get("resourcePath", "")),
        pageIndex=int(row.get("pageIndex", 0)),
        caption=str(row.get("caption", "")),
        content=str(row.get("content", "")),
        mimeType=str(row.get("mimeType", "")),
        status=str(row.get("status", "")),
        elementId=str(row.get("elementId", "")),
    )


__all__ = [
    "DOCLING_COMMON_SOURCE_SUFFIXES",
    "DOCLING_SUPPORTED_DOCUMENT_FORMATS",
    "DOCUMENT_RESOURCE_ARROW_CACHE_NAME",
    "DOCUMENT_RESOURCE_FIELDS",
    "DOCUMENT_RESOURCE_SCHEMA",
    "DocumentConverterProtocol",
    "DocumentResourceRow",
    "default_document_output_dir",
    "document_resources_to_table",
    "extract_document_resources",
    "extract_document_table",
    "extract_pdf_resources",
    "extract_pdf_table",
    "is_known_docling_source",
]
