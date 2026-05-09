"""Arrow Flight service surface for Wendao document extraction."""

from __future__ import annotations

import os
import threading
from typing import TYPE_CHECKING, Any

import pyarrow.flight as flight

from .document_profiles import (
    DOCUMENT_EXTRACT_DEFAULT_PROFILE,
    DOCUMENT_EXTRACT_FULL_PROFILE,
    DOCUMENT_EXTRACT_PROFILE_ENV,
    new_docling_converter_for_profile,
    normalize_document_extract_profile,
)
from .documents import (
    DOCUMENT_RESOURCE_SCHEMA,
    DocumentConverterProtocol,
    extract_document_table,
    warm_document_arrow_runtime,
)
from .pdf_ocr import (
    PDF_OCR_SHARD_RESULT_SCHEMA,
    PdfOcrShardWorkerProtocol,
    build_pdf_ocr_shard_result_table,
)

if TYPE_CHECKING:
    from collections.abc import Mapping

ANALYSIS_DOCUMENT_EXTRACT_ROUTE = "/analysis/document-extract"
ANALYSIS_PDF_OCR_SHARDS_ROUTE = "/analysis/pdf-ocr-shards"

WENDAO_SCHEMA_VERSION_HEADER = "x-wendao-schema-version"
WENDAO_PDF_OCR_WORKERS_HEADER = "x-wendao-pdf-ocr-workers"
WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER = "x-wendao-document-extract-source-path"
WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER = "x-wendao-document-extract-output-dir"
WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER = "x-wendao-document-extract-force"
WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER = "x-wendao-document-extract-error-row"
WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER = "x-wendao-document-extract-profile"
WENDAO_DOCUMENT_EXTRACT_PAGE_RANGE_HEADER = "x-wendao-document-extract-page-range"
WENDAO_DOCUMENT_EXTRACT_CONVERTER_CACHE_ENV = "WENDAO_DOCUMENT_EXTRACT_CONVERTER_CACHE"

EXPECTED_SCHEMA_VERSION = "v2"
DOCUMENT_EXTRACT_CONVERTER_CACHE_DISABLED = "disabled"
DOCUMENT_EXTRACT_CONVERTER_CACHE_PROFILE = "profile"
SUPPORTED_DOCUMENT_ROUTES = (
    ANALYSIS_DOCUMENT_EXTRACT_ROUTE,
    ANALYSIS_PDF_OCR_SHARDS_ROUTE,
)


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

    source_path = headers.get(WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER, "")
    if not source_path:
        raise ValueError("Missing document source path header")

    output_dir = headers.get(WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER, "")
    force = _header_bool(headers, WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER, False)
    error_row = _header_bool(headers, WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER, True)
    profile = _document_extract_profile(headers)
    page_range = _document_extract_page_range(headers)

    return extract_document_table(
        source_path,
        output_dir or None,
        converter=converter,
        profile=profile,
        force=force,
        error_row=error_row,
        page_range=page_range,
    )


class DocumentExtractMiddleware(flight.ServerMiddleware):
    """Per-call middleware that captures incoming request headers."""

    def __init__(self, headers: Mapping[str, str]) -> None:
        self.headers = dict(headers)

    def call_completed(self, exception: Exception | None) -> None:
        pass

    def sending_headers(self) -> None:
        pass


class DocumentExtractMiddlewareFactory(flight.ServerMiddlewareFactory):
    """Factory that injects document extraction headers on every RPC."""

    def start_call(
        self,
        info: flight.CallInfo,
        headers: Mapping[str, list[str] | bytes | str],
    ) -> DocumentExtractMiddleware:
        return DocumentExtractMiddleware(_flatten_headers(headers))


class DocumentExtractFlightServer(flight.FlightServerBase):
    """Arrow Flight server that exposes Docling-backed document extraction."""

    def __init__(
        self,
        location: str = "grpc://0.0.0.0:50051",
        *,
        converter: DocumentConverterProtocol | None = None,
        ocr_worker: PdfOcrShardWorkerProtocol | None = None,
        converter_factory: Any | None = None,
    ) -> None:
        super().__init__(
            location,
            middleware={"document-extract": DocumentExtractMiddlewareFactory()},
        )
        warm_document_arrow_runtime()
        self._converter = converter
        self._ocr_worker = ocr_worker
        self._converter_factory = converter_factory or new_docling_converter_for_profile
        self._converter_cache_lock = threading.Lock()
        self._converter_cache: dict[str, DocumentConverterProtocol] = {}

    def _get_headers(self, context: flight.ServerCallContext) -> dict[str, str]:
        middleware = context.get_middleware("document-extract")
        if middleware is None:
            return {}
        return dict(middleware.headers)

    def do_get(
        self,
        context: flight.ServerCallContext,
        ticket: flight.Ticket,
    ) -> flight.RecordBatchStream:
        route = ticket.ticket.decode("utf-8")
        _validate_document_extract_route(route)

        try:
            headers = self._get_headers(context)
            table = build_document_extract_table(
                headers,
                converter=self._document_extract_converter(headers),
            )
        except ValueError as exc:
            raise flight.FlightServerError(
                str(exc), extra_info=str(exc).encode("utf-8")
            ) from exc

        return flight.RecordBatchStream(table)

    def _document_extract_converter(
        self,
        headers: Mapping[str, str],
    ) -> DocumentConverterProtocol | None:
        profile = _document_extract_profile(headers)
        if self._converter is not None:
            if profile == DOCUMENT_EXTRACT_FULL_PROFILE:
                return self._converter
            return self._document_extract_converter_for_profile(profile)
        if (
            _document_extract_converter_cache_mode()
            != DOCUMENT_EXTRACT_CONVERTER_CACHE_PROFILE
        ):
            return None
        return self._document_extract_converter_for_profile(profile)

    def _document_extract_converter_for_profile(
        self,
        profile: str | None,
    ) -> DocumentConverterProtocol:
        normalized_profile = normalize_document_extract_profile(profile)
        with self._converter_cache_lock:
            converter = self._converter_cache.get(normalized_profile)
            if converter is None:
                converter = self._converter_factory(normalized_profile)
                self._converter_cache[normalized_profile] = converter
            return converter

    def do_exchange(
        self,
        context: flight.ServerCallContext,
        descriptor: flight.FlightDescriptor,
        reader: flight.MetadataRecordBatchReader,
        writer: flight.MetadataRecordBatchWriter,
    ) -> None:
        route = _descriptor_route(descriptor)
        _validate_pdf_ocr_shards_route(route)

        try:
            result_table = build_pdf_ocr_shard_result_table(
                reader.read_all(),
                worker=self._ocr_worker,
                max_workers=self._get_headers(context).get(
                    WENDAO_PDF_OCR_WORKERS_HEADER
                ),
            )
        except ValueError as exc:
            raise flight.FlightServerError(
                str(exc), extra_info=str(exc).encode("utf-8")
            ) from exc

        writer.begin(result_table.schema)
        writer.write_table(result_table)
        writer.close()

    def get_flight_info(
        self,
        context: flight.ServerCallContext,
        descriptor: flight.FlightDescriptor,
    ) -> flight.FlightInfo:
        route = _descriptor_route(descriptor)
        _validate_route(route)

        ticket = flight.Ticket(route.encode("utf-8"))
        endpoint = flight.FlightEndpoint(ticket=ticket, locations=[])
        schema = (
            DOCUMENT_RESOURCE_SCHEMA
            if route == ANALYSIS_DOCUMENT_EXTRACT_ROUTE
            else PDF_OCR_SHARD_RESULT_SCHEMA
        )
        return flight.FlightInfo(
            schema=schema,
            descriptor=descriptor,
            endpoints=[endpoint],
            total_records=-1,
            total_bytes=-1,
        )


def _build_pdf_ocr_worker(
    worker_name: str,
    max_workers: int | str | None = "auto",
) -> PdfOcrShardWorkerProtocol:
    from .document_service_cli import build_pdf_ocr_worker

    return build_pdf_ocr_worker(worker_name, max_workers)


def _validate_route(route: str) -> None:
    if route not in SUPPORTED_DOCUMENT_ROUTES:
        raise flight.FlightServerError(
            f"Unexpected document extraction route: {route}",
            extra_info=route.encode("utf-8"),
        )


def _validate_document_extract_route(route: str) -> None:
    if route != ANALYSIS_DOCUMENT_EXTRACT_ROUTE:
        raise flight.FlightServerError(
            f"Unexpected document extraction do_get route: {route}",
            extra_info=route.encode("utf-8"),
        )


def _validate_pdf_ocr_shards_route(route: str) -> None:
    if route != ANALYSIS_PDF_OCR_SHARDS_ROUTE:
        raise flight.FlightServerError(
            f"Unexpected PDF OCR shard exchange route: {route}",
            extra_info=route.encode("utf-8"),
        )


def _descriptor_route(descriptor: flight.FlightDescriptor) -> str:
    return "/" + "/".join(
        part.decode("utf-8") if isinstance(part, bytes) else part
        for part in descriptor.path
    )


def _header_bool(headers: Mapping[str, str], key: str, default: bool) -> bool:
    value = headers.get(key, "")
    if value.lower() in {"true", "1", "yes"}:
        return True
    if value.lower() in {"false", "0", "no"}:
        return False
    return default


def _document_extract_profile(headers: Mapping[str, str]) -> str:
    requested_profile = headers.get(WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER)
    default_profile = os.environ.get(
        DOCUMENT_EXTRACT_PROFILE_ENV,
        DOCUMENT_EXTRACT_DEFAULT_PROFILE,
    )
    return normalize_document_extract_profile(requested_profile or default_profile)


def _document_extract_converter_cache_mode() -> str:
    return _document_extract_converter_cache_mode_with_lookup(os.environ.get)


def _document_extract_converter_cache_mode_with_lookup(
    lookup: Any,
) -> str:
    value = str(lookup(WENDAO_DOCUMENT_EXTRACT_CONVERTER_CACHE_ENV) or "").strip()
    normalized = value.lower().replace("_", "-")
    if normalized in {
        "profile",
        "profile-cache",
        "shared-profile",
        "shared-profile-cache",
    }:
        return DOCUMENT_EXTRACT_CONVERTER_CACHE_PROFILE
    return DOCUMENT_EXTRACT_CONVERTER_CACHE_DISABLED


def _document_extract_page_range(headers: Mapping[str, str]) -> tuple[int, int] | None:
    value = headers.get(WENDAO_DOCUMENT_EXTRACT_PAGE_RANGE_HEADER, "").strip()
    if not value:
        return None
    parts = value.split(":")
    if len(parts) != 2:
        raise ValueError(
            "document extract page range must use 1-based inclusive `start:end`"
        )
    try:
        start, end = (int(part) for part in parts)
    except ValueError as exc:
        raise ValueError(
            "document extract page range must use integer page numbers"
        ) from exc
    if start <= 0 or end <= 0 or start > end:
        raise ValueError("document extract page range must satisfy 1 <= start <= end")
    return (start, end)


def _flatten_headers(headers: Mapping[str, Any]) -> dict[str, str]:
    flat: dict[str, str] = {}
    for key, value in headers.items():
        if isinstance(value, list) and value:
            flat[key] = _header_value_to_string(value[0])
        else:
            flat[key] = _header_value_to_string(value)
    return flat


def _header_value_to_string(value: Any) -> str:
    if isinstance(value, bytes):
        return value.decode("utf-8")
    return str(value)


__all__ = [
    "ANALYSIS_DOCUMENT_EXTRACT_ROUTE",
    "ANALYSIS_PDF_OCR_SHARDS_ROUTE",
    "EXPECTED_SCHEMA_VERSION",
    "SUPPORTED_DOCUMENT_ROUTES",
    "WENDAO_DOCUMENT_EXTRACT_CONVERTER_CACHE_ENV",
    "WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_PAGE_RANGE_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER",
    "WENDAO_PDF_OCR_WORKERS_HEADER",
    "WENDAO_SCHEMA_VERSION_HEADER",
    "DocumentExtractFlightServer",
    "DocumentExtractMiddleware",
    "DocumentExtractMiddlewareFactory",
    "build_document_extract_table",
]
