"""Arrow contracts for PDF OCR shard workers."""

from __future__ import annotations

from .pdf_ocr_contracts import (
    PDF_OCR_DEFAULT_PROFILE,
    PDF_OCR_FAST_TEXT_PROFILE,
    PDF_OCR_MAX_WORKERS_ENV,
    PDF_OCR_SHARD_INPUT_SCHEMA,
    PDF_OCR_SHARD_INPUT_SCHEMA_VERSION,
    PDF_OCR_SHARD_RESULT_SCHEMA,
    PDF_OCR_SHARD_RESULT_SCHEMA_VERSION,
    PDF_OCR_WORKERS_ENV,
    PdfOcrShardWorkerProtocol,
)
from .pdf_ocr_results import (
    failed_pdf_ocr_shard_result,
    normalize_pdf_ocr_shard_result,
    skipped_pdf_ocr_shard_result,
    succeeded_pdf_ocr_shard_result,
)
from .pdf_ocr_tables import (
    build_pdf_ocr_shard_result_table,
    resolve_pdf_ocr_worker_count,
    validate_pdf_ocr_shard_input_table,
)
from .pdf_ocr_workers import DoclingPdfOcrShardWorker, SkippingPdfOcrShardWorker

__all__ = [
    "PDF_OCR_DEFAULT_PROFILE",
    "PDF_OCR_FAST_TEXT_PROFILE",
    "PDF_OCR_MAX_WORKERS_ENV",
    "PDF_OCR_SHARD_INPUT_SCHEMA",
    "PDF_OCR_SHARD_INPUT_SCHEMA_VERSION",
    "PDF_OCR_SHARD_RESULT_SCHEMA",
    "PDF_OCR_SHARD_RESULT_SCHEMA_VERSION",
    "PDF_OCR_WORKERS_ENV",
    "DoclingPdfOcrShardWorker",
    "PdfOcrShardWorkerProtocol",
    "SkippingPdfOcrShardWorker",
    "build_pdf_ocr_shard_result_table",
    "failed_pdf_ocr_shard_result",
    "normalize_pdf_ocr_shard_result",
    "resolve_pdf_ocr_worker_count",
    "skipped_pdf_ocr_shard_result",
    "succeeded_pdf_ocr_shard_result",
    "validate_pdf_ocr_shard_input_table",
]
