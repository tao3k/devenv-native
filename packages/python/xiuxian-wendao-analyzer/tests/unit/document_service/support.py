"""Shared helpers for document_service tests."""

from __future__ import annotations

import threading
import time
from pathlib import Path

import pyarrow as pa
import pyarrow.flight as flight
import pytest

from xiuxian_wendao_analyzer import (
    ANALYSIS_AUDIO_SHARDS_ROUTE,
    ANALYSIS_DOCUMENT_EXTRACT_ROUTE,
    ANALYSIS_PDF_OCR_SHARDS_ROUTE,
    AUDIO_SHARD_INPUT_SCHEMA,
    AUDIO_SHARD_INPUT_SCHEMA_VERSION,
    AUDIO_SHARD_RESULT_SCHEMA,
    AUDIO_SHARD_RESULT_SCHEMA_VERSION,
    DOCUMENT_RESOURCE_SCHEMA,
    EXPECTED_SCHEMA_VERSION,
    PDF_OCR_SHARD_INPUT_SCHEMA,
    PDF_OCR_SHARD_INPUT_SCHEMA_VERSION,
    PDF_OCR_SHARD_RESULT_SCHEMA,
    PDF_OCR_SHARD_RESULT_SCHEMA_VERSION,
    SUPPORTED_DOCUMENT_ROUTES,
    WENDAO_AUDIO_WORKERS_HEADER,
    WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER,
    WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER,
    WENDAO_DOCUMENT_EXTRACT_PAGE_RANGE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
    WENDAO_PDF_OCR_WORKERS_HEADER,
    WENDAO_SCHEMA_VERSION_HEADER,
    DoclingAudioShardWorker,
    DoclingPdfOcrShardWorker,
    DocumentExtractFlightServer,
    HostedAudioConfig,
    HostedAudioShardWorker,
    SkippingAudioShardWorker,
    UnsupportedAudioShardWorker,
    build_audio_shard_result_table,
    build_audio_shard_worker,
    build_document_extract_table,
    build_pdf_ocr_shard_result_table,
    hosted_audio_payload,
    normalize_audio_worker_name,
    resolve_audio_shard_worker_count,
    succeeded_audio_shard_result,
    succeeded_pdf_ocr_shard_result,
)
from xiuxian_wendao_analyzer.document_service import (
    _build_audio_shard_worker,
    _build_pdf_ocr_worker,
)
from xiuxian_wendao_analyzer.pdf_ocr import (
    SkippingPdfOcrShardWorker,
    resolve_pdf_ocr_worker_count,
)

from .support_fakes import (
    FailingDoclingConverter,
    FakeAudioShardWorker,
    FakeDoclingConverter,
    FakeDoclingDocument,
    FakeDoclingResult,
    FakePdfOcrShardWorker,
)
from .support_tables import (
    _sample_audio_shard_input_table,
    _sample_pdf_ocr_input_table,
)

__all__ = [
    "ANALYSIS_AUDIO_SHARDS_ROUTE",
    "ANALYSIS_DOCUMENT_EXTRACT_ROUTE",
    "ANALYSIS_PDF_OCR_SHARDS_ROUTE",
    "AUDIO_SHARD_INPUT_SCHEMA",
    "AUDIO_SHARD_INPUT_SCHEMA_VERSION",
    "AUDIO_SHARD_RESULT_SCHEMA",
    "AUDIO_SHARD_RESULT_SCHEMA_VERSION",
    "DOCUMENT_RESOURCE_SCHEMA",
    "EXPECTED_SCHEMA_VERSION",
    "PDF_OCR_SHARD_INPUT_SCHEMA",
    "PDF_OCR_SHARD_INPUT_SCHEMA_VERSION",
    "PDF_OCR_SHARD_RESULT_SCHEMA",
    "PDF_OCR_SHARD_RESULT_SCHEMA_VERSION",
    "SUPPORTED_DOCUMENT_ROUTES",
    "WENDAO_AUDIO_WORKERS_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_PAGE_RANGE_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER",
    "WENDAO_PDF_OCR_WORKERS_HEADER",
    "WENDAO_SCHEMA_VERSION_HEADER",
    "DoclingAudioShardWorker",
    "DoclingPdfOcrShardWorker",
    "DocumentExtractFlightServer",
    "FailingDoclingConverter",
    "FakeAudioShardWorker",
    "FakeDoclingConverter",
    "FakeDoclingDocument",
    "FakeDoclingResult",
    "FakePdfOcrShardWorker",
    "HostedAudioConfig",
    "HostedAudioShardWorker",
    "Path",
    "SkippingAudioShardWorker",
    "SkippingPdfOcrShardWorker",
    "UnsupportedAudioShardWorker",
    "_build_audio_shard_worker",
    "_build_pdf_ocr_worker",
    "_sample_audio_shard_input_table",
    "_sample_pdf_ocr_input_table",
    "build_audio_shard_result_table",
    "build_audio_shard_worker",
    "build_document_extract_table",
    "build_pdf_ocr_shard_result_table",
    "flight",
    "hosted_audio_payload",
    "normalize_audio_worker_name",
    "pa",
    "pytest",
    "resolve_audio_shard_worker_count",
    "resolve_pdf_ocr_worker_count",
    "succeeded_audio_shard_result",
    "succeeded_pdf_ocr_shard_result",
    "threading",
    "time",
]
