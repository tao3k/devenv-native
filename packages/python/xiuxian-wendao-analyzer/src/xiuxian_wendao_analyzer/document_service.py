"""Arrow Flight service surface for Wendao document extraction."""

from __future__ import annotations

from typing import TYPE_CHECKING

from .document_service_extract import build_document_extract_table
from .document_service_headers import document_extract_converter_cache_mode_with_lookup
from .document_service_routes import (
    ANALYSIS_AUDIO_SHARDS_ROUTE,
    ANALYSIS_DOCUMENT_EXTRACT_ROUTE,
    ANALYSIS_PDF_OCR_SHARDS_ROUTE,
    DOCUMENT_EXTRACT_CONVERTER_CACHE_DISABLED,
    DOCUMENT_EXTRACT_CONVERTER_CACHE_PROFILE,
    EXPECTED_SCHEMA_VERSION,
    SUPPORTED_DOCUMENT_ROUTES,
    WENDAO_AUDIO_HOSTED_BASE_URL_HEADER,
    WENDAO_AUDIO_HOSTED_ENDPOINT_HEADER,
    WENDAO_AUDIO_HOSTED_MODEL_HEADER,
    WENDAO_AUDIO_HOSTED_PROVIDER_HEADER,
    WENDAO_AUDIO_WORKER_HEADER,
    WENDAO_AUDIO_WORKERS_HEADER,
    WENDAO_DOCUMENT_EXTRACT_CONVERTER_CACHE_ENV,
    WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER,
    WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER,
    WENDAO_DOCUMENT_EXTRACT_PAGE_RANGE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
    WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_UTF8_HEX_HEADER,
    WENDAO_DOCUMENT_EXTRACT_SOURCE_PREPARATION_HEADER,
    WENDAO_PDF_OCR_WORKERS_HEADER,
    WENDAO_SCHEMA_VERSION_HEADER,
)
from .document_service_server import (
    DocumentExtractFlightServer,
    DocumentExtractMiddleware,
    DocumentExtractMiddlewareFactory,
)
from .documents import warm_document_arrow_runtime

if TYPE_CHECKING:
    from .audio_shards import AudioShardWorkerProtocol
    from .pdf_ocr import PdfOcrShardWorkerProtocol

DOCUMENT_EXTRACT_CONVERTER_CACHE_ENV = WENDAO_DOCUMENT_EXTRACT_CONVERTER_CACHE_ENV


def _build_pdf_ocr_worker(
    worker_name: str,
    max_workers: int | str | None = "auto",
) -> PdfOcrShardWorkerProtocol:
    from .document_service_cli import build_pdf_ocr_worker

    return build_pdf_ocr_worker(worker_name, max_workers)


def _build_audio_shard_worker(
    worker_name: str | None = None,
    max_workers: int | str | None = "auto",
) -> AudioShardWorkerProtocol:
    from .document_service_cli import build_audio_worker

    return build_audio_worker(worker_name, max_workers)


_document_extract_converter_cache_mode_with_lookup = (
    document_extract_converter_cache_mode_with_lookup
)

__all__ = [
    "ANALYSIS_AUDIO_SHARDS_ROUTE",
    "ANALYSIS_DOCUMENT_EXTRACT_ROUTE",
    "ANALYSIS_PDF_OCR_SHARDS_ROUTE",
    "DOCUMENT_EXTRACT_CONVERTER_CACHE_DISABLED",
    "DOCUMENT_EXTRACT_CONVERTER_CACHE_ENV",
    "DOCUMENT_EXTRACT_CONVERTER_CACHE_PROFILE",
    "EXPECTED_SCHEMA_VERSION",
    "SUPPORTED_DOCUMENT_ROUTES",
    "WENDAO_AUDIO_HOSTED_BASE_URL_HEADER",
    "WENDAO_AUDIO_HOSTED_ENDPOINT_HEADER",
    "WENDAO_AUDIO_HOSTED_MODEL_HEADER",
    "WENDAO_AUDIO_HOSTED_PROVIDER_HEADER",
    "WENDAO_AUDIO_WORKERS_HEADER",
    "WENDAO_AUDIO_WORKER_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_PAGE_RANGE_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_UTF8_HEX_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_SOURCE_PREPARATION_HEADER",
    "WENDAO_PDF_OCR_WORKERS_HEADER",
    "WENDAO_SCHEMA_VERSION_HEADER",
    "DocumentExtractFlightServer",
    "DocumentExtractMiddleware",
    "DocumentExtractMiddlewareFactory",
    "_build_audio_shard_worker",
    "_build_pdf_ocr_worker",
    "_document_extract_converter_cache_mode_with_lookup",
    "build_document_extract_table",
    "warm_document_arrow_runtime",
]
