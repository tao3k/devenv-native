"""Docling converter profiles for Wendao document extraction."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .document_types import DocumentConverterProtocol

DOCUMENT_EXTRACT_PROFILE_ENV = "WENDAO_DOCUMENT_EXTRACT_PROFILE"
DOCUMENT_EXTRACT_FULL_PROFILE = "full"
DOCUMENT_EXTRACT_FAST_TEXT_PROFILE = "fast-text"
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
    raise ValueError(f"unsupported document extract profile `{value}`")


def new_docling_converter_for_profile(
    profile: str | None = None,
) -> "DocumentConverterProtocol":
    """Build a Docling converter for a Wendao extraction profile.

    # Errors

    Raises `RuntimeError` when Docling is not installed. Raises `ValueError`
    when the profile is not recognized.
    """

    try:
        from docling.document_converter import DocumentConverter
    except ModuleNotFoundError as exc:
        raise RuntimeError(
            "docling is not installed; install xiuxian-wendao-analyzer[documents] "
            "to enable document extraction"
        ) from exc

    match normalize_document_extract_profile(profile):
        case "fast-text":
            return _new_fast_text_docling_converter()
        case _:
            return DocumentConverter()


def _new_fast_text_docling_converter() -> "DocumentConverterProtocol":
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


__all__ = [
    "DOCUMENT_EXTRACT_DEFAULT_PROFILE",
    "DOCUMENT_EXTRACT_FAST_TEXT_PROFILE",
    "DOCUMENT_EXTRACT_FULL_PROFILE",
    "DOCUMENT_EXTRACT_PROFILE_ENV",
    "new_docling_converter_for_profile",
    "normalize_document_extract_profile",
]
