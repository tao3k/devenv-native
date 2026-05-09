"""Docling converter profiles for Wendao document extraction."""

from __future__ import annotations

import os
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .document_types import DocumentConverterProtocol

DOCUMENT_EXTRACT_PROFILE_ENV = "WENDAO_DOCUMENT_EXTRACT_PROFILE"
DOCUMENT_EXTRACT_FULL_THREADS_ENV = "WENDAO_DOCUMENT_EXTRACT_FULL_THREADS"
DOCUMENT_EXTRACT_FULL_PROFILE = "full"
DOCUMENT_EXTRACT_FAST_TEXT_PROFILE = "fast-text"
DOCUMENT_EXTRACT_STRUCTURE_TEXT_PROFILE = "structure-text"
DOCUMENT_EXTRACT_DEFAULT_PROFILE = DOCUMENT_EXTRACT_FULL_PROFILE

_FAST_TEXT_ALIASES = {
    "attachment",
    "attachment-fast-text",
    "fast",
    "fast_text",
    "fast-text",
    "text",
}
_FULL_ALIASES = {"", "default", "docling", "full", "full-docling"}
_STRUCTURE_TEXT_ALIASES = {
    "docling-structure-text",
    "structure",
    "structure_text",
    "structure-text",
    "text-structure",
}


def normalize_document_extract_profile(value: str | None) -> str:
    """Normalize a user supplied document extraction profile.

    # Errors

    Raises `ValueError` when the profile is not recognized.
    """

    normalized = (value or DOCUMENT_EXTRACT_DEFAULT_PROFILE).strip().lower()
    if normalized in _FULL_ALIASES:
        return DOCUMENT_EXTRACT_FULL_PROFILE
    if normalized in _FAST_TEXT_ALIASES:
        return DOCUMENT_EXTRACT_FAST_TEXT_PROFILE
    if normalized in _STRUCTURE_TEXT_ALIASES:
        return DOCUMENT_EXTRACT_STRUCTURE_TEXT_PROFILE
    raise ValueError(f"unsupported document extract profile `{value}`")


def new_docling_converter_for_profile(
    profile: str | None = None,
) -> DocumentConverterProtocol:
    """Build a Docling converter for a Wendao extraction profile.

    # Errors

    Raises `RuntimeError` when Docling is not installed. Raises `ValueError`
    when the profile is not recognized.
    """

    try:
        import docling.document_converter  # noqa: F401
    except ModuleNotFoundError as exc:
        raise RuntimeError(
            "docling is not installed; install xiuxian-wendao-analyzer[documents] "
            "to enable document extraction"
        ) from exc

    match normalize_document_extract_profile(profile):
        case "fast-text":
            return _new_fast_text_docling_converter()
        case "structure-text":
            return _new_structure_text_docling_converter()
        case _:
            return _new_full_docling_converter()


def document_extract_full_threads_from_env(
    environ: dict[str, str] | None = None,
) -> int | None:
    """Return the optional Docling full-profile thread cap from the environment."""

    env = environ if environ is not None else os.environ
    raw_value = env.get(DOCUMENT_EXTRACT_FULL_THREADS_ENV, "").strip()
    if not raw_value:
        return None
    try:
        value = int(raw_value)
    except ValueError:
        return None
    return value if value > 0 else None


def _new_full_docling_converter() -> DocumentConverterProtocol:
    from docling.datamodel.base_models import InputFormat
    from docling.datamodel.pipeline_options import (
        AcceleratorOptions,
        PdfPipelineOptions,
    )
    from docling.document_converter import DocumentConverter, PdfFormatOption

    thread_count = document_extract_full_threads_from_env()
    if thread_count is None:
        return DocumentConverter()

    options = PdfPipelineOptions()
    options.accelerator_options = AcceleratorOptions(num_threads=thread_count)
    return DocumentConverter(
        format_options={InputFormat.PDF: PdfFormatOption(pipeline_options=options)}
    )


def _new_fast_text_docling_converter() -> DocumentConverterProtocol:
    from docling.datamodel.accelerator_options import (
        AcceleratorDevice,
        AcceleratorOptions,
    )
    from docling.datamodel.base_models import InputFormat
    from docling.datamodel.pipeline_options import PdfPipelineOptions
    from docling.document_converter import DocumentConverter, PdfFormatOption

    options = PdfPipelineOptions()
    options.accelerator_options = AcceleratorOptions(
        num_threads=1,
        device=AcceleratorDevice.CPU,
    )
    options.do_ocr = False
    options.do_table_structure = False
    options.force_backend_text = True
    options.ocr_batch_size = 1
    options.layout_batch_size = 1
    options.table_batch_size = 1

    return DocumentConverter(
        format_options={InputFormat.PDF: PdfFormatOption(pipeline_options=options)}
    )


def _new_structure_text_docling_converter() -> DocumentConverterProtocol:
    from docling.datamodel.accelerator_options import AcceleratorDevice
    from docling.datamodel.base_models import InputFormat
    from docling.datamodel.pipeline_options import (
        AcceleratorOptions,
        PdfPipelineOptions,
    )
    from docling.document_converter import DocumentConverter, PdfFormatOption

    options = PdfPipelineOptions()
    options.accelerator_options = AcceleratorOptions(
        num_threads=1,
        device=AcceleratorDevice.CPU,
    )
    options.do_ocr = False
    options.do_table_structure = True

    return DocumentConverter(
        format_options={InputFormat.PDF: PdfFormatOption(pipeline_options=options)}
    )


__all__ = [
    "DOCUMENT_EXTRACT_DEFAULT_PROFILE",
    "DOCUMENT_EXTRACT_FAST_TEXT_PROFILE",
    "DOCUMENT_EXTRACT_FULL_PROFILE",
    "DOCUMENT_EXTRACT_FULL_THREADS_ENV",
    "DOCUMENT_EXTRACT_PROFILE_ENV",
    "DOCUMENT_EXTRACT_STRUCTURE_TEXT_PROFILE",
    "document_extract_full_threads_from_env",
    "new_docling_converter_for_profile",
    "normalize_document_extract_profile",
]
