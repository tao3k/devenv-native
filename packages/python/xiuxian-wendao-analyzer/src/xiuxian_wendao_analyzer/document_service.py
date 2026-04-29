"""Arrow Flight service surface for Wendao document extraction."""

from __future__ import annotations

import argparse
import sys
from typing import TYPE_CHECKING, Any

import pyarrow.flight as flight

from .documents import (
    DOCUMENT_RESOURCE_SCHEMA,
    DocumentConverterProtocol,
    extract_document_table,
)
from .pdf_ocr import (
    PDF_OCR_SHARD_RESULT_SCHEMA,
    DoclingPdfOcrShardWorker,
    PdfOcrShardWorkerProtocol,
    SkippingPdfOcrShardWorker,
    build_pdf_ocr_shard_result_table,
)

if TYPE_CHECKING:
    from collections.abc import Mapping

ANALYSIS_DOCUMENT_EXTRACT_ROUTE = "/analysis/document-extract"
ANALYSIS_PDF_OCR_SHARDS_ROUTE = "/analysis/pdf-ocr-shards"

WENDAO_SCHEMA_VERSION_HEADER = "x-wendao-schema-version"
WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER = "x-wendao-document-extract-source-path"
WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER = "x-wendao-document-extract-output-dir"
WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER = "x-wendao-document-extract-force"
WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER = "x-wendao-document-extract-error-row"

EXPECTED_SCHEMA_VERSION = "v2"
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

    return extract_document_table(
        source_path,
        output_dir or None,
        converter=converter,
        force=force,
        error_row=error_row,
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
    ) -> None:
        super().__init__(
            location,
            middleware={"document-extract": DocumentExtractMiddlewareFactory()},
        )
        self._converter = converter
        self._ocr_worker = ocr_worker

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
            table = build_document_extract_table(
                self._get_headers(context),
                converter=self._converter,
            )
        except ValueError as exc:
            raise flight.FlightServerError(
                str(exc), extra_info=str(exc).encode("utf-8")
            ) from exc

        return flight.RecordBatchStream(table)

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


def main() -> int:
    """Run the Wendao document extraction Arrow Flight service."""

    parser = argparse.ArgumentParser(
        description="Wendao document extraction Arrow Flight service"
    )
    parser.add_argument("--host", default="0.0.0.0", help="Bind host")
    parser.add_argument("--port", type=int, default=50051, help="Bind port")
    parser.add_argument(
        "--pdf-ocr-worker",
        choices=("skip", "docling"),
        default="skip",
        help="OCR worker used by the internal /analysis/pdf-ocr-shards exchange",
    )
    args = parser.parse_args()

    location = f"grpc://{args.host}:{args.port}"
    server = DocumentExtractFlightServer(
        location,
        ocr_worker=_build_pdf_ocr_worker(args.pdf_ocr_worker),
    )
    print(f"Wendao document extraction service listening on {location}")
    server.serve()
    return 0


def _build_pdf_ocr_worker(worker_name: str) -> PdfOcrShardWorkerProtocol:
    if worker_name == "docling":
        return DoclingPdfOcrShardWorker()
    return SkippingPdfOcrShardWorker()


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
    "WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER",
    "WENDAO_SCHEMA_VERSION_HEADER",
    "DocumentExtractFlightServer",
    "DocumentExtractMiddleware",
    "DocumentExtractMiddlewareFactory",
    "build_document_extract_table",
    "main",
]


if __name__ == "__main__":
    sys.exit(main())
