"""PDF OCR worker environment and conversion helpers."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from .pdf_ocr_contracts import (
    PDF_OCR_DEFAULT_PROFILE,
    PDF_OCR_PAGE_BREAK_SENTINEL,
)

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


PDF_OCR_FAST_TEXT_THREADS_ENV = "WENDAO_PDF_OCR_FAST_TEXT_THREADS"
PDF_OCR_FAST_TEXT_DEFAULT_THREADS = 1
PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_ENV = "WENDAO_PDF_OCR_FAST_TEXT_SOURCE_CONVERTER"
PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_DEFAULT = "default"
PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_BACKEND_TABLE = "backend-table"
PDF_OCR_PREWARM_PROFILES_ENV = "WENDAO_PDF_OCR_PREWARM_PROFILES"
PDF_OCR_PREWARM_SOURCE_PATH_ENV = "WENDAO_PDF_OCR_PREWARM_SOURCE_PATH"
PDF_OCR_PREWARM_PAGE_INDICES_ENV = "WENDAO_PDF_OCR_PREWARM_PAGE_INDICES"
PDF_OCR_PREWARM_PAGE_INDEX_ENV = "WENDAO_PDF_OCR_PREWARM_PAGE_INDEX"
PDF_OCR_BACKEND_TEXT_PAGE_FALLBACK_ENV = "WENDAO_PDF_OCR_BACKEND_TEXT_PAGE_FALLBACK"
PDF_OCR_BACKEND_TEXT_PAGE_FALLBACK_COMPATIBLE = "compatible-page"
PDF_OCR_BACKEND_TEXT_EMPTY_PAGE_ENV = "WENDAO_PDF_OCR_BACKEND_TEXT_EMPTY_PAGE"
PDF_OCR_BACKEND_TEXT_EMPTY_PAGE_VERIFIED = "verified-empty"


def _try_export_source_page_batch_markdown(
    document: Any,
    input_rows: Sequence[Mapping[str, Any]],
    *,
    allow_empty: bool = False,
) -> list[str] | None:
    try:
        markdown = document.export_to_markdown(
            page_break_placeholder=PDF_OCR_PAGE_BREAK_SENTINEL
        )
    except (AttributeError, TypeError, ValueError):
        return None
    if not isinstance(markdown, str):
        return None
    parts = [part.strip() for part in markdown.split(PDF_OCR_PAGE_BREAK_SENTINEL)]
    if len(parts) != len(input_rows):
        return None
    if not allow_empty and any(not part for part in parts):
        return None
    return parts


def _ocr_profile(input_row: Mapping[str, Any]) -> str:
    profile = str(input_row.get("ocrProfile", "")).strip()
    return profile or PDF_OCR_DEFAULT_PROFILE


def fast_text_source_converter_mode() -> str:
    from os import environ

    return fast_text_source_converter_mode_with_lookup(environ.get)


def fast_text_source_converter_mode_with_lookup(
    lookup: Any,
) -> str:
    value = str(lookup(PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_ENV) or "").strip()
    if not value:
        return PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_DEFAULT
    normalized = value.replace("_", "-").lower()
    if normalized in {
        PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_DEFAULT,
        PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_BACKEND_TABLE,
    }:
        return normalized
    return PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_DEFAULT
