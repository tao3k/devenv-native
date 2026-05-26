"""Document extraction orchestration helpers."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

from .document_cache import (
    _new_docling_converter,
    _read_cached_resources,
    _read_cached_table,
)
from .document_extract_inline import (
    _document_extract_error_row,
    _extract_document_resources_inline,
    _should_isolate_document_extract,
    _write_extract_error_timing,
)
from .document_profiles import (
    DOCUMENT_EXTRACT_FULL_PROFILE,
)
from .document_types import (
    DocumentConverterProtocol,
    DocumentResourceRow,
    default_document_output_dir,
    document_resources_to_table,
)

if TYPE_CHECKING:
    import pyarrow as pa

__all__ = [
    "_new_docling_converter",
    "extract_document_resources",
    "extract_document_table",
    "extract_pdf_resources",
    "extract_pdf_table",
]


def extract_document_table(
    source_path: str | Path,
    output_dir: str | Path | None = None,
    *,
    converter: DocumentConverterProtocol | None = None,
    profile: str | None = None,
    force: bool = False,
    error_row: bool = False,
    page_range: tuple[int, int] | None = None,
    source_preparation: str | None = None,
) -> pa.Table:
    """Extract one document and return Arrow resource rows.

    # Errors

    Raises `FileNotFoundError` when the source path does not exist. Raises
    `RuntimeError` when Docling is not installed and no converter is provided.
    Raises conversion exceptions unless `error_row` is true.
    """

    source = Path(source_path)
    if source.exists() and not force and page_range is None:
        out = Path(output_dir) if output_dir is not None else default_document_output_dir(source)
        cached_table = _read_cached_table(source, out)
        if cached_table is not None:
            return cached_table

    return document_resources_to_table(
        extract_document_resources(
            source,
            output_dir,
            converter=converter,
            profile=profile,
            force=force,
            error_row=error_row,
            page_range=page_range,
            source_preparation=source_preparation,
        )
    )


def extract_document_resources(
    source_path: str | Path,
    output_dir: str | Path | None = None,
    *,
    converter: DocumentConverterProtocol | None = None,
    profile: str | None = None,
    force: bool = False,
    error_row: bool = False,
    page_range: tuple[int, int] | None = None,
    source_preparation: str | None = None,
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
                _document_extract_error_row(
                    source,
                    f"document source path does not exist: {source}",
                )
            ]
        raise FileNotFoundError(f"document source path does not exist: {source}")

    out = Path(output_dir) if output_dir is not None else default_document_output_dir(source)
    out.mkdir(parents=True, exist_ok=True)

    if not force and page_range is None:
        cached = _read_cached_resources(source, out)
        if cached is not None:
            return cached

    if page_range is None and _should_isolate_document_extract(
        converter=converter, profile=profile
    ):
        try:
            from .document_isolation import run_isolated_document_extract

            run_isolated_document_extract(
                source,
                out,
                profile=DOCUMENT_EXTRACT_FULL_PROFILE,
                force=force,
                source_preparation=source_preparation,
            )
            cached = _read_cached_resources(source, out)
            if cached is None:
                raise RuntimeError(
                    "isolated document extraction completed without a resource cache"
                )
            return cached
        except Exception as exc:
            _write_extract_error_timing(source, out, exc)
            if not error_row:
                raise
            return [_document_extract_error_row(source, str(exc))]

    return _extract_document_resources_inline(
        source,
        out,
        converter=converter,
        profile=profile,
        error_row=error_row,
        page_range=page_range,
        source_preparation=source_preparation,
    )


def extract_pdf_resources(
    source_path: str | Path,
    output_dir: str | Path | None = None,
    *,
    converter: DocumentConverterProtocol | None = None,
    profile: str | None = None,
    force: bool = False,
    error_row: bool = False,
    source_preparation: str | None = None,
) -> list[DocumentResourceRow]:
    """Compatibility wrapper for PDF callers migrating to document extraction.

    # Errors

    Raises the same errors as `extract_document_resources`.
    """

    return extract_document_resources(
        source_path,
        output_dir,
        converter=converter,
        profile=profile,
        force=force,
        error_row=error_row,
        source_preparation=source_preparation,
    )


def extract_pdf_table(
    source_path: str | Path,
    output_dir: str | Path | None = None,
    *,
    converter: DocumentConverterProtocol | None = None,
    profile: str | None = None,
    force: bool = False,
    error_row: bool = False,
    source_preparation: str | None = None,
) -> pa.Table:
    """Compatibility wrapper for PDF callers that need an Arrow table.

    # Errors

    Raises the same errors as `extract_document_table`.
    """

    return extract_document_table(
        source_path,
        output_dir,
        converter=converter,
        profile=profile,
        force=force,
        error_row=error_row,
        source_preparation=source_preparation,
    )
