"""Backend-text fallback options for PDF OCR workers."""

from __future__ import annotations

import os
from typing import TYPE_CHECKING, Any

from .pdf_ocr_worker_options import (
    PDF_OCR_BACKEND_TEXT_EMPTY_PAGE_ENV,
    PDF_OCR_BACKEND_TEXT_EMPTY_PAGE_VERIFIED,
    PDF_OCR_BACKEND_TEXT_PAGE_FALLBACK_COMPATIBLE,
    PDF_OCR_BACKEND_TEXT_PAGE_FALLBACK_ENV,
)

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping


def backend_text_page_fallback_mode() -> str:
    return backend_text_page_fallback_mode_with_lookup(os.environ.get)


def backend_text_page_fallback_mode_with_lookup(
    lookup: Callable[[str], str | None],
) -> str:
    mode = (lookup(PDF_OCR_BACKEND_TEXT_PAGE_FALLBACK_ENV) or "").strip()
    mode = mode.replace("_", "-").lower()
    if mode == PDF_OCR_BACKEND_TEXT_PAGE_FALLBACK_COMPATIBLE:
        return PDF_OCR_BACKEND_TEXT_PAGE_FALLBACK_COMPATIBLE
    return "disabled"


def backend_text_empty_page_mode() -> str:
    return backend_text_empty_page_mode_with_lookup(os.environ.get)


def backend_text_empty_page_mode_with_lookup(
    lookup: Callable[[str], str | None],
) -> str:
    mode = (lookup(PDF_OCR_BACKEND_TEXT_EMPTY_PAGE_ENV) or "").strip()
    mode = mode.replace("_", "-").lower()
    if mode == PDF_OCR_BACKEND_TEXT_EMPTY_PAGE_VERIFIED:
        return PDF_OCR_BACKEND_TEXT_EMPTY_PAGE_VERIFIED
    return "disabled"


def _verified_empty_backend_text_result() -> Mapping[str, Any]:
    return {
        "status": "succeeded",
        "text": "",
        "textMimeType": "text/markdown",
        "confidence": None,
        "errorMessage": None,
    }
