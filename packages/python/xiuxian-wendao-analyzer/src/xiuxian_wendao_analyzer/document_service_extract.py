"""Document extraction table builder."""

from __future__ import annotations

from typing import TYPE_CHECKING

from .document_service_headers import (
    document_extract_page_range,
    document_extract_profile,
    document_extract_source_preparation,
    header_bool,
)
from .document_service_routes import (
    EXPECTED_SCHEMA_VERSION,
    WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER,
    WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER,
    WENDAO_SCHEMA_VERSION_HEADER,
    document_extract_source_path_from_headers,
)
from .documents import extract_document_table

if TYPE_CHECKING:
    from collections.abc import Mapping

    from .documents import DocumentConverterProtocol


def build_document_extract_table(
    headers: Mapping[str, str],
    *,
    converter: DocumentConverterProtocol | None = None,
):
    """Build one Arrow table from Wendao document extraction headers.

    # Errors

    Raises `ValueError` when required headers are missing or invalid. Raises
    document conversion errors unless the error-row header requests table-shaped
    error rows.
    """

    schema_version = headers.get(WENDAO_SCHEMA_VERSION_HEADER, "")
    if schema_version != EXPECTED_SCHEMA_VERSION:
        raise ValueError(f"Unexpected schema version: {schema_version}")

    source_path = document_extract_source_path_from_headers(headers)
    if not source_path:
        raise ValueError("Missing document source path header")

    output_dir = headers.get(WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER, "")
    force = header_bool(headers, WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER, False)
    error_row = header_bool(headers, WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER, True)
    profile = document_extract_profile(headers)
    page_range = document_extract_page_range(headers)
    source_preparation = document_extract_source_preparation(headers)

    return extract_document_table(
        source_path,
        output_dir or None,
        converter=converter,
        profile=profile,
        force=force,
        error_row=error_row,
        page_range=page_range,
        source_preparation=source_preparation,
    )
