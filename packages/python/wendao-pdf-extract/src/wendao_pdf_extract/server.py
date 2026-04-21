"""Arrow Flight server for Wendao PDF extraction."""

from __future__ import annotations

import argparse
import sys
from typing import Mapping

import pyarrow as pa
import pyarrow.flight as flight

from .extractor import extract_pdf

ANALYSIS_PDF_EXTRACT_ROUTE = "/analysis/pdf-extract"

WENDAO_SCHEMA_VERSION_HEADER = "x-wendao-schema-version"
WENDAO_PDF_EXTRACT_SOURCE_PATH_HEADER = "x-wendao-pdf-extract-source-path"
WENDAO_PDF_EXTRACT_OUTPUT_DIR_HEADER = "x-wendao-pdf-extract-output-dir"
WENDAO_PDF_EXTRACT_IMAGES_HEADER = "x-wendao-pdf-extract-images"
WENDAO_PDF_EXTRACT_TABLES_HEADER = "x-wendao-pdf-extract-tables"
WENDAO_PDF_EXTRACT_FORMULAS_HEADER = "x-wendao-pdf-extract-formulas"

EXPECTED_SCHEMA_VERSION = "v2"

_RESULT_SCHEMA = pa.schema(
    [
        pa.field("sourcePath", pa.utf8()),
        pa.field("resourceType", pa.utf8()),
        pa.field("resourcePath", pa.utf8()),
        pa.field("pageIndex", pa.int32()),
        pa.field("caption", pa.utf8()),
        pa.field("content", pa.utf8()),
        pa.field("mimeType", pa.utf8()),
        pa.field("status", pa.utf8()),
        pa.field("elementId", pa.utf8()),
    ]
)


def _header_bool(headers: Mapping[str, str], key: str, default: bool = True) -> bool:
    value = headers.get(key, "")
    if value.lower() in ("true", "1"):
        return True
    if value.lower() in ("false", "0"):
        return False
    return default


class PdfExtractMiddleware(flight.ServerMiddleware):
    """Per-call middleware that captures incoming request headers."""

    def __init__(self, headers: Mapping[str, str]) -> None:
        self.headers = headers

    def call_completed(self, exception):
        pass

    def sending_headers(self):
        pass


class PdfExtractMiddlewareFactory(flight.ServerMiddlewareFactory):
    """Factory that injects PdfExtractMiddleware on every RPC."""

    def start_call(self, info, headers):
        # headers is dict[str, list[str] | bytes]
        flat: dict[str, str] = {}
        for key, value in headers.items():
            if isinstance(value, list) and value:
                flat[key] = value[0]
            elif isinstance(value, bytes):
                flat[key] = value.decode("utf-8")
            elif isinstance(value, str):
                flat[key] = value
        return PdfExtractMiddleware(flat)


class PdfExtractFlightServer(flight.FlightServerBase):
    """Arrow Flight server that exposes PDF extraction via OpenDataLoader."""

    def __init__(self, location: str = "grpc://0.0.0.0:50051") -> None:
        super().__init__(location, middleware={"pdf-extract": PdfExtractMiddlewareFactory()})

    def _get_headers(self, context: flight.ServerCallContext) -> dict[str, str]:
        middleware = context.get_middleware("pdf-extract")
        if middleware is None:
            return {}
        return middleware.headers

    def do_get(
        self, context: flight.ServerCallContext, ticket: flight.Ticket
    ) -> flight.RecordBatchStream:
        route = ticket.ticket.decode("utf-8")
        if route != ANALYSIS_PDF_EXTRACT_ROUTE:
            raise flight.FlightServerError(
                f"Unexpected ticket route: {route}",
                extra_info=route.encode("utf-8"),
            )

        headers = self._get_headers(context)
        schema_version = headers.get(WENDAO_SCHEMA_VERSION_HEADER, "")
        if schema_version != EXPECTED_SCHEMA_VERSION:
            raise flight.FlightServerError(
                f"Unexpected schema version: {schema_version}",
                extra_info=schema_version.encode("utf-8"),
            )

        source_path = headers.get(WENDAO_PDF_EXTRACT_SOURCE_PATH_HEADER, "")
        if not source_path:
            raise flight.FlightServerError(
                "Missing source path header",
            )

        output_dir = headers.get(WENDAO_PDF_EXTRACT_OUTPUT_DIR_HEADER, "")
        if not output_dir:
            output_dir = f"{source_path}.extracted"

        extract_images = _header_bool(headers, WENDAO_PDF_EXTRACT_IMAGES_HEADER, True)
        extract_tables = _header_bool(headers, WENDAO_PDF_EXTRACT_TABLES_HEADER, True)
        extract_formulas = _header_bool(headers, WENDAO_PDF_EXTRACT_FORMULAS_HEADER, True)

        try:
            resources = extract_pdf(
                source_path,
                output_dir,
                extract_images=extract_images,
                extract_tables=extract_tables,
                extract_formulas=extract_formulas,
            )
        except Exception as exc:
            # Return one error row
            resources = [
                {
                    "sourcePath": source_path,
                    "resourceType": "error",
                    "resourcePath": "",
                    "pageIndex": 0,
                    "caption": "",
                    "content": str(exc),
                    "mimeType": "text/plain",
                    "status": "error",
                    "elementId": "",
                }
            ]

        if not resources:
            resources = [
                {
                    "sourcePath": source_path,
                    "resourceType": "document",
                    "resourcePath": "",
                    "pageIndex": 0,
                    "caption": "",
                    "content": "",
                    "mimeType": "text/plain",
                    "status": "skipped",
                    "elementId": "",
                }
            ]

        table = pa.Table.from_pylist(resources, schema=_RESULT_SCHEMA)
        return flight.RecordBatchStream(table)

    def get_flight_info(
        self, context: flight.ServerCallContext, descriptor: flight.FlightDescriptor
    ) -> flight.FlightInfo:
        route = "/" + "/".join(
            p.decode("utf-8") if isinstance(p, bytes) else p for p in descriptor.path
        )
        if route != ANALYSIS_PDF_EXTRACT_ROUTE:
            raise flight.FlightServerError(
                f"Unexpected descriptor route: {route}",
                extra_info=route.encode("utf-8"),
            )

        ticket = flight.Ticket(ANALYSIS_PDF_EXTRACT_ROUTE.encode("utf-8"))
        endpoint = flight.FlightEndpoint(ticket=ticket, locations=[])
        return flight.FlightInfo(
            schema=_RESULT_SCHEMA,
            descriptor=descriptor,
            endpoints=[endpoint],
            total_records=-1,
            total_bytes=-1,
        )


def main() -> int:
    parser = argparse.ArgumentParser(description="Wendao PDF Extract Arrow Flight Server")
    parser.add_argument(
        "--host", default="0.0.0.0", help="Bind host (default: 0.0.0.0)"
    )
    parser.add_argument(
        "--port", type=int, default=50051, help="Bind port (default: 50051)"
    )
    args = parser.parse_args()

    location = f"grpc://{args.host}:{args.port}"
    server = PdfExtractFlightServer(location)
    print(f"🚀 Wendao PDF Extract server listening on {location}")
    server.serve()
    return 0


if __name__ == "__main__":
    sys.exit(main())
