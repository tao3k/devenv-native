"""Document extraction orchestration helpers."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

from .document_cache import (
    _file_sha256,
    _new_docling_converter,
    _read_cached_resources,
    _read_cached_table,
    _write_cached_resources,
    _write_cached_structure,
    _write_document_timing_sidecar,
)
from .document_metrics import DocumentTimingRecorder
from .document_structure import (
    _document_structure_blocks,
    _structured_document_resources,
)
from .document_types import (
    DocumentConverterProtocol,
    DocumentResourceRow,
    default_document_output_dir,
    document_resources_to_table,
)

if TYPE_CHECKING:
    import pyarrow as pa


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
        out = (
            Path(output_dir)
            if output_dir is not None
            else default_document_output_dir(source)
        )
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

    out = (
        Path(output_dir)
        if output_dir is not None
        else default_document_output_dir(source)
    )
    out.mkdir(parents=True, exist_ok=True)

    if not force:
        cached = _read_cached_resources(source, out)
        if cached is not None:
            return cached

    timing = DocumentTimingRecorder(source)
    try:
        if converter is not None:
            resolved_converter = converter
        else:
            with timing.phase("doclingConverterInit"):
                resolved_converter = _new_docling_converter()
        with timing.phase("doclingConvert"):
            document = resolved_converter.convert(source).document
        with timing.phase("doclingMarkdownExport"):
            markdown_text = document.export_to_markdown()
        markdown_path = out / f"{source.stem}.md"
        with timing.phase("writeMarkdown"):
            markdown_path.write_text(markdown_text, encoding="utf-8")
        with timing.phase("sourceHash"):
            source_content_hash = _file_sha256(source)
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
        with timing.phase("resourceRowsBuild"):
            resources.extend(_structured_document_resources(source, out, document))
        with timing.phase("structureRowsBuild"):
            structure = _document_structure_blocks(
                source,
                document,
                resources,
                source_content_hash=source_content_hash,
            )
        with timing.phase("writeStructureArrow"):
            _write_cached_structure(out, structure)
        with timing.phase("writeResourcesArrow"):
            _write_cached_resources(out, resources)
        timing.finish(
            status="ok",
            resource_rows=len(resources),
            structure_rows=len(structure),
        )
        _write_document_timing_sidecar(out, timing)
        return resources
    except Exception as exc:
        timing.finish(status="error", detail=str(exc))
        _write_document_timing_sidecar(out, timing)
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
