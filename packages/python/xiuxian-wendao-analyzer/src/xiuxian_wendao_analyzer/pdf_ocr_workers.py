"""PDF OCR shard worker facade."""

from __future__ import annotations

from .pdf_ocr_worker_converter import (
    PDF_OCR_FAST_TEXT_SOURCE_BACKEND_TABLE_PROFILE,
    _fast_text_accelerator_threads_with_lookup,
    _new_docling_converter,
)
from .pdf_ocr_worker_docling import DoclingPdfOcrShardWorker, SkippingPdfOcrShardWorker
from .pdf_ocr_worker_fallback import (
    backend_text_empty_page_mode,
    backend_text_empty_page_mode_with_lookup,
    backend_text_page_fallback_mode,
    backend_text_page_fallback_mode_with_lookup,
)
from .pdf_ocr_worker_options import (
    PDF_OCR_BACKEND_TEXT_EMPTY_PAGE_ENV,
    PDF_OCR_BACKEND_TEXT_EMPTY_PAGE_VERIFIED,
    PDF_OCR_BACKEND_TEXT_PAGE_FALLBACK_COMPATIBLE,
    PDF_OCR_BACKEND_TEXT_PAGE_FALLBACK_ENV,
    PDF_OCR_FAST_TEXT_DEFAULT_THREADS,
    PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_BACKEND_TABLE,
    PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_DEFAULT,
    PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_ENV,
    PDF_OCR_FAST_TEXT_THREADS_ENV,
    PDF_OCR_PREWARM_PAGE_INDEX_ENV,
    PDF_OCR_PREWARM_PROFILES_ENV,
    PDF_OCR_PREWARM_SOURCE_PATH_ENV,
    fast_text_source_converter_mode_with_lookup,
)

__all__ = [
    "PDF_OCR_BACKEND_TEXT_EMPTY_PAGE_ENV",
    "PDF_OCR_BACKEND_TEXT_EMPTY_PAGE_VERIFIED",
    "PDF_OCR_BACKEND_TEXT_PAGE_FALLBACK_COMPATIBLE",
    "PDF_OCR_BACKEND_TEXT_PAGE_FALLBACK_ENV",
    "PDF_OCR_FAST_TEXT_DEFAULT_THREADS",
    "PDF_OCR_FAST_TEXT_SOURCE_BACKEND_TABLE_PROFILE",
    "PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_BACKEND_TABLE",
    "PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_DEFAULT",
    "PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_ENV",
    "PDF_OCR_FAST_TEXT_THREADS_ENV",
    "PDF_OCR_PREWARM_PAGE_INDEX_ENV",
    "PDF_OCR_PREWARM_PROFILES_ENV",
    "PDF_OCR_PREWARM_SOURCE_PATH_ENV",
    "DoclingPdfOcrShardWorker",
    "SkippingPdfOcrShardWorker",
    "_fast_text_accelerator_threads_with_lookup",
    "_new_docling_converter",
    "backend_text_empty_page_mode",
    "backend_text_empty_page_mode_with_lookup",
    "backend_text_page_fallback_mode",
    "backend_text_page_fallback_mode_with_lookup",
    "fast_text_source_converter_mode_with_lookup",
]
