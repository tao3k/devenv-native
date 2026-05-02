"""Docling-backed document extraction helpers for analyzer workflows."""

from __future__ import annotations

import pyarrow as pa

from .document_cache import _resource_from_mapping, warm_document_arrow_runtime
from .document_extract import (
    extract_document_resources,
    extract_document_table,
    extract_pdf_resources,
    extract_pdf_table,
)
from .document_types import (
    DOCLING_COMMON_SOURCE_SUFFIXES,
    DOCLING_SUPPORTED_DOCUMENT_FORMATS,
    DOCUMENT_RESOURCE_ARROW_CACHE_NAME,
    DOCUMENT_RESOURCE_FIELDS,
    DOCUMENT_RESOURCE_SCHEMA,
    DOCUMENT_STRUCTURE_ARROW_CACHE_NAME,
    DOCUMENT_STRUCTURE_SCHEMA,
    DOCUMENT_STRUCTURE_SCHEMA_VERSION,
    DocumentConverterProtocol,
    DocumentResourceRow,
    DocumentStructureBlock,
    default_document_output_dir,
    document_resources_to_table,
    document_structure_to_table,
    is_known_docling_source,
)

__all__ = [
    "DOCLING_COMMON_SOURCE_SUFFIXES",
    "DOCLING_SUPPORTED_DOCUMENT_FORMATS",
    "DOCUMENT_RESOURCE_ARROW_CACHE_NAME",
    "DOCUMENT_RESOURCE_FIELDS",
    "DOCUMENT_RESOURCE_SCHEMA",
    "DOCUMENT_STRUCTURE_ARROW_CACHE_NAME",
    "DOCUMENT_STRUCTURE_SCHEMA",
    "DOCUMENT_STRUCTURE_SCHEMA_VERSION",
    "DocumentConverterProtocol",
    "DocumentResourceRow",
    "DocumentStructureBlock",
    "_resource_from_mapping",
    "default_document_output_dir",
    "document_resources_to_table",
    "document_structure_to_table",
    "extract_document_resources",
    "extract_document_table",
    "extract_pdf_resources",
    "extract_pdf_table",
    "is_known_docling_source",
    "pa",
    "warm_document_arrow_runtime",
]
