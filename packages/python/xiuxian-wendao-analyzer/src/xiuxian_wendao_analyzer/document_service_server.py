"""Arrow Flight server implementation for document extraction."""

from __future__ import annotations

import threading
from typing import TYPE_CHECKING, Any

import pyarrow.flight as flight

from .audio_shard_worker_config import (
    AUDIO_HOSTED_BASE_URL_ENV,
    AUDIO_HOSTED_ENDPOINT_ENV,
    AUDIO_HOSTED_MODEL_ENV,
    AUDIO_HOSTED_PROVIDER_ENV,
    hosted_audio_config_from_env,
)
from .audio_shard_worker_registry import build_audio_shard_worker
from .audio_shards import (
    AUDIO_SHARD_RESULT_SCHEMA,
    AudioShardWorkerProtocol,
    build_audio_shard_result_table,
)
from .document_profiles import (
    DOCUMENT_EXTRACT_FULL_PROFILE,
    new_docling_converter_for_profile,
    normalize_document_extract_profile,
)
from .document_service_extract import build_document_extract_table
from .document_service_headers import (
    document_extract_converter_cache_mode,
    document_extract_profile,
)
from .document_service_routes import (
    ANALYSIS_DOCUMENT_EXTRACT_ROUTE,
    ANALYSIS_PDF_OCR_SHARDS_ROUTE,
    DOCUMENT_EXTRACT_CONVERTER_CACHE_PROFILE,
    WENDAO_AUDIO_HOSTED_BASE_URL_HEADER,
    WENDAO_AUDIO_HOSTED_ENDPOINT_HEADER,
    WENDAO_AUDIO_HOSTED_MODEL_HEADER,
    WENDAO_AUDIO_HOSTED_PROVIDER_HEADER,
    WENDAO_AUDIO_WORKER_HEADER,
    WENDAO_AUDIO_WORKERS_HEADER,
    WENDAO_PDF_OCR_WORKERS_HEADER,
    descriptor_route,
    flatten_headers,
    validate_document_extract_route,
    validate_exchange_route,
    validate_route,
)
from .documents import (
    DOCUMENT_RESOURCE_SCHEMA,
    DocumentConverterProtocol,
)
from .pdf_ocr import (
    PDF_OCR_SHARD_RESULT_SCHEMA,
    PdfOcrShardWorkerProtocol,
    build_pdf_ocr_shard_result_table,
)

if TYPE_CHECKING:
    from collections.abc import Mapping


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
        return DocumentExtractMiddleware(flatten_headers(headers))


class DocumentExtractFlightServer(flight.FlightServerBase):
    """Arrow Flight server that exposes Docling-backed document extraction."""

    def __init__(
        self,
        location: str = "grpc://0.0.0.0:50051",
        *,
        converter: DocumentConverterProtocol | None = None,
        ocr_worker: PdfOcrShardWorkerProtocol | None = None,
        audio_worker: AudioShardWorkerProtocol | None = None,
        converter_factory: Any | None = None,
    ) -> None:
        super().__init__(
            location,
            middleware={"document-extract": DocumentExtractMiddlewareFactory()},
        )
        _warm_document_arrow_runtime()
        self._converter = converter
        self._ocr_worker = ocr_worker
        self._audio_worker = audio_worker
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
        validate_document_extract_route(route)

        try:
            headers = self._get_headers(context)
            table = build_document_extract_table(
                headers,
                converter=self._document_extract_converter(headers),
            )
        except ValueError as exc:
            raise flight.FlightServerError(str(exc), extra_info=str(exc).encode("utf-8")) from exc

        return flight.RecordBatchStream(table)

    def _document_extract_converter(
        self,
        headers: Mapping[str, str],
    ) -> DocumentConverterProtocol | None:
        profile = document_extract_profile(headers)
        if self._converter is not None:
            if profile == DOCUMENT_EXTRACT_FULL_PROFILE:
                return self._converter
            return self._document_extract_converter_for_profile(profile)
        if document_extract_converter_cache_mode() != DOCUMENT_EXTRACT_CONVERTER_CACHE_PROFILE:
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
        route = descriptor_route(descriptor)
        validate_exchange_route(route)

        try:
            headers = self._get_headers(context)
            input_table = reader.read_all()
            if route == ANALYSIS_PDF_OCR_SHARDS_ROUTE:
                result_table = build_pdf_ocr_shard_result_table(
                    input_table,
                    worker=self._ocr_worker,
                    max_workers=headers.get(WENDAO_PDF_OCR_WORKERS_HEADER),
                )
            else:
                result_table = build_audio_shard_result_table(
                    input_table,
                    worker=self._audio_worker_for_headers(headers),
                    max_workers=headers.get(WENDAO_AUDIO_WORKERS_HEADER),
                )
        except ValueError as exc:
            raise flight.FlightServerError(str(exc), extra_info=str(exc).encode("utf-8")) from exc

        writer.begin(result_table.schema)
        writer.write_table(result_table)
        writer.close()

    def get_flight_info(
        self,
        context: flight.ServerCallContext,
        descriptor: flight.FlightDescriptor,
    ) -> flight.FlightInfo:
        route = descriptor_route(descriptor)
        validate_route(route)

        ticket = flight.Ticket(route.encode("utf-8"))
        endpoint = flight.FlightEndpoint(ticket=ticket, locations=[])
        schema = (
            DOCUMENT_RESOURCE_SCHEMA
            if route == ANALYSIS_DOCUMENT_EXTRACT_ROUTE
            else (
                PDF_OCR_SHARD_RESULT_SCHEMA
                if route == ANALYSIS_PDF_OCR_SHARDS_ROUTE
                else AUDIO_SHARD_RESULT_SCHEMA
            )
        )
        return flight.FlightInfo(
            schema=schema,
            descriptor=descriptor,
            endpoints=[endpoint],
            total_records=-1,
            total_bytes=-1,
        )

    def _audio_worker_for_headers(
        self,
        headers: Mapping[str, str],
    ) -> AudioShardWorkerProtocol | None:
        requested_worker = headers.get(WENDAO_AUDIO_WORKER_HEADER, "").strip()
        hosted_overrides = hosted_audio_overrides_from_headers(headers)
        if not requested_worker and not hosted_overrides:
            return self._audio_worker
        worker_name = requested_worker or "hosted"
        hosted_config = None
        if worker_name in {"hosted", "hosted-audio-transcript-v1"} or hosted_overrides:
            hosted_config = hosted_audio_config_from_env(hosted_overrides)
        return build_audio_shard_worker(
            worker_name,
            max_workers=headers.get(WENDAO_AUDIO_WORKERS_HEADER),
            hosted_config=hosted_config,
        )


def _warm_document_arrow_runtime() -> None:
    from . import document_service

    document_service.warm_document_arrow_runtime()


def hosted_audio_overrides_from_headers(
    headers: Mapping[str, str],
) -> dict[str, str]:
    overrides = {
        AUDIO_HOSTED_PROVIDER_ENV: headers.get(WENDAO_AUDIO_HOSTED_PROVIDER_HEADER, ""),
        AUDIO_HOSTED_BASE_URL_ENV: headers.get(WENDAO_AUDIO_HOSTED_BASE_URL_HEADER, ""),
        AUDIO_HOSTED_ENDPOINT_ENV: headers.get(WENDAO_AUDIO_HOSTED_ENDPOINT_HEADER, ""),
        AUDIO_HOSTED_MODEL_ENV: headers.get(WENDAO_AUDIO_HOSTED_MODEL_HEADER, ""),
    }
    return {key: value.strip() for key, value in overrides.items() if value and value.strip()}
