"""Inline Docling document extraction execution."""

from __future__ import annotations

import sys
from dataclasses import replace
from typing import TYPE_CHECKING

from .document_cache import (
    _file_sha256,
    _new_docling_converter,
    _write_cached_resources,
    _write_cached_structure,
    _write_document_timing_sidecar,
)
from .document_legacy_office import prepare_docling_source
from .document_metrics import DocumentTimingRecorder
from .document_profiles import (
    DOCUMENT_EXTRACT_FULL_PROFILE,
    normalize_document_extract_profile,
)
from .document_structure import (
    _document_structure_blocks,
    _structured_document_resources,
)
from .document_types import DocumentConverterProtocol, DocumentResourceRow

if TYPE_CHECKING:
    from pathlib import Path


def _extract_document_resources_inline(
    source: Path,
    output_dir: Path,
    *,
    converter: DocumentConverterProtocol | None = None,
    profile: str | None = None,
    error_row: bool = False,
    page_range: tuple[int, int] | None = None,
    source_preparation: str | None = None,
) -> list[DocumentResourceRow]:
    output_dir.mkdir(parents=True, exist_ok=True)
    timing = DocumentTimingRecorder(source)
    try:
        if converter is not None:
            resolved_converter = converter
        else:
            with timing.phase("doclingConverterInit"):
                resolved_converter = _document_extract_converter_factory()(profile)
        docling_source = source
        if source_preparation is not None:
            with timing.phase("sourcePreparation"):
                docling_source = prepare_docling_source(
                    source,
                    output_dir,
                    mode=source_preparation,
                )
        with timing.phase("doclingConvert"):
            convert_kwargs = {"page_range": page_range} if page_range else {}
            document = resolved_converter.convert(docling_source, **convert_kwargs).document
        with timing.phase("doclingMarkdownExport"):
            markdown_text = document.export_to_markdown()
        page_range_slug = _page_range_slug(page_range)
        markdown_path = output_dir / f"{source.stem}{page_range_slug}.md"
        with timing.phase("writeMarkdown"):
            markdown_path.write_text(markdown_text, encoding="utf-8")
        with timing.phase("sourceHash"):
            source_content_hash = _file_sha256(source)
        element_id_prefix = _page_range_element_id_prefix(page_range)
        page_index = page_range[0] - 1 if page_range is not None else 0
        resources = [
            DocumentResourceRow(
                sourcePath=str(source),
                resourceType="document",
                resourcePath=str(markdown_path),
                pageIndex=page_index,
                caption="",
                content=markdown_text,
                mimeType="text/markdown",
                status="ok",
                elementId=f"{element_id_prefix}_main",
            )
        ]
        with timing.phase("resourceRowsBuild"):
            resources.extend(
                _structured_document_resources(
                    source,
                    output_dir,
                    document,
                    element_id_prefix=element_id_prefix,
                    resource_file_prefix=_page_range_resource_file_prefix(page_range),
                )
            )
        if page_range is not None:
            resources = [
                _normalize_page_range_resource_page_index(row, page_range) for row in resources
            ]
        with timing.phase("structureRowsBuild"):
            structure = _document_structure_blocks(
                source,
                document,
                resources,
                source_content_hash=source_content_hash,
                element_id_prefix=element_id_prefix,
            )
        with timing.phase("writeStructureArrow"):
            _write_cached_structure(output_dir, structure)
        with timing.phase("writeResourcesArrow"):
            _write_cached_resources(output_dir, resources)
        timing.finish(
            status="ok",
            resource_rows=len(resources),
            structure_rows=len(structure),
        )
        _write_document_timing_sidecar(output_dir, timing)
        return resources
    except Exception as exc:
        _finish_extract_error_timing(timing, output_dir, exc)
        if not error_row:
            raise
        return [_document_extract_error_row(source, str(exc))]


def _should_isolate_document_extract(
    *,
    converter: DocumentConverterProtocol | None,
    profile: str | None,
) -> bool:
    if converter is not None:
        return False
    if normalize_document_extract_profile(profile) != DOCUMENT_EXTRACT_FULL_PROFILE:
        return False

    from .document_isolation import full_profile_isolation_enabled

    return full_profile_isolation_enabled()


def _write_extract_error_timing(
    source: Path,
    output_dir: Path,
    exc: Exception,
) -> None:
    timing = DocumentTimingRecorder(source)
    _finish_extract_error_timing(timing, output_dir, exc)


def _finish_extract_error_timing(
    timing: DocumentTimingRecorder,
    output_dir: Path,
    exc: Exception,
) -> None:
    timing.finish(status="error", detail=str(exc))
    _write_document_timing_sidecar(output_dir, timing)


def _document_extract_error_row(source: Path, content: str) -> DocumentResourceRow:
    return DocumentResourceRow(
        sourcePath=str(source),
        resourceType="error",
        resourcePath="",
        pageIndex=0,
        caption="",
        content=content,
        mimeType="text/plain",
        status="error",
        elementId="",
    )


def _document_extract_converter_factory():
    facade = sys.modules.get("xiuxian_wendao_analyzer.document_extract")
    if facade is not None:
        return getattr(facade, "_new_docling_converter", _new_docling_converter)
    return _new_docling_converter


def _page_range_slug(page_range: tuple[int, int] | None) -> str:
    if page_range is None:
        return ""
    start, end = page_range
    return f".pages-{start:05}-{end:05}"


def _page_range_element_id_prefix(page_range: tuple[int, int] | None) -> str:
    if page_range is None:
        return ""
    start, end = page_range
    return f"page-range-{start:05}-{end:05}:"


def _page_range_resource_file_prefix(page_range: tuple[int, int] | None) -> str:
    if page_range is None:
        return ""
    start, end = page_range
    return f"pages-{start:05}-{end:05}-"


def _normalize_page_range_resource_page_index(
    row: DocumentResourceRow,
    page_range: tuple[int, int],
) -> DocumentResourceRow:
    start, end = page_range
    if start - 1 <= row.pageIndex <= end - 1:
        return row
    return replace(row, pageIndex=start - 1)
