"""Document extraction Flight route constants and validators."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from collections.abc import Mapping

import pyarrow.flight as flight

ANALYSIS_DOCUMENT_EXTRACT_ROUTE = "/analysis/document-extract"
ANALYSIS_PDF_OCR_SHARDS_ROUTE = "/analysis/pdf-ocr-shards"
ANALYSIS_AUDIO_SHARDS_ROUTE = "/analysis/audio-shards"

WENDAO_SCHEMA_VERSION_HEADER = "x-wendao-schema-version"
WENDAO_PDF_OCR_WORKERS_HEADER = "x-wendao-pdf-ocr-workers"
WENDAO_AUDIO_WORKERS_HEADER = "x-wendao-audio-workers"
WENDAO_AUDIO_WORKER_HEADER = "x-wendao-audio-worker"
WENDAO_AUDIO_HOSTED_PROVIDER_HEADER = "x-wendao-audio-hosted-provider"
WENDAO_AUDIO_HOSTED_BASE_URL_HEADER = "x-wendao-audio-hosted-base-url"
WENDAO_AUDIO_HOSTED_ENDPOINT_HEADER = "x-wendao-audio-hosted-endpoint"
WENDAO_AUDIO_HOSTED_MODEL_HEADER = "x-wendao-audio-hosted-model"
WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER = "x-wendao-document-extract-source-path"
WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_UTF8_HEX_HEADER = (
    "x-wendao-document-extract-source-path-utf8-hex"
)
WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER = "x-wendao-document-extract-output-dir"
WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER = "x-wendao-document-extract-force"
WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER = "x-wendao-document-extract-error-row"
WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER = "x-wendao-document-extract-profile"
WENDAO_DOCUMENT_EXTRACT_PAGE_RANGE_HEADER = "x-wendao-document-extract-page-range"
WENDAO_DOCUMENT_EXTRACT_SOURCE_PREPARATION_HEADER = "x-wendao-document-extract-source-preparation"
WENDAO_DOCUMENT_EXTRACT_CONVERTER_CACHE_ENV = "WENDAO_DOCUMENT_EXTRACT_CONVERTER_CACHE"

EXPECTED_SCHEMA_VERSION = "v2"
DOCUMENT_EXTRACT_CONVERTER_CACHE_DISABLED = "disabled"
DOCUMENT_EXTRACT_CONVERTER_CACHE_PROFILE = "profile"
SUPPORTED_DOCUMENT_ROUTES = (
    ANALYSIS_DOCUMENT_EXTRACT_ROUTE,
    ANALYSIS_PDF_OCR_SHARDS_ROUTE,
    ANALYSIS_AUDIO_SHARDS_ROUTE,
)


def validate_route(route: str) -> None:
    """Validate any supported document extraction route."""

    if route not in SUPPORTED_DOCUMENT_ROUTES:
        raise flight.FlightServerError(
            f"Unexpected document extraction route: {route}",
            extra_info=route.encode("utf-8"),
        )


def validate_document_extract_route(route: str) -> None:
    """Validate do_get document extraction routes."""

    if route != ANALYSIS_DOCUMENT_EXTRACT_ROUTE:
        raise flight.FlightServerError(
            f"Unexpected document extraction do_get route: {route}",
            extra_info=route.encode("utf-8"),
        )


def validate_exchange_route(route: str) -> None:
    """Validate do_exchange document extraction routes."""

    if route not in {ANALYSIS_PDF_OCR_SHARDS_ROUTE, ANALYSIS_AUDIO_SHARDS_ROUTE}:
        raise flight.FlightServerError(
            f"Unexpected document extraction exchange route: {route}",
            extra_info=route.encode("utf-8"),
        )


def descriptor_route(descriptor: flight.FlightDescriptor) -> str:
    """Return a slash-prefixed route from a Flight descriptor."""

    return "/" + "/".join(
        part.decode("utf-8") if isinstance(part, bytes) else part for part in descriptor.path
    )


def flatten_headers(headers: dict[str, Any] | Any) -> dict[str, str]:
    """Flatten Flight middleware headers to string values."""

    flat: dict[str, str] = {}
    for key, value in headers.items():
        if isinstance(value, list) and value:
            flat[key] = header_value_to_string(value[0])
        else:
            flat[key] = header_value_to_string(value)
    return flat


def header_value_to_string(value: Any) -> str:
    """Convert Flight header values to strings."""

    if isinstance(value, bytes):
        return value.decode("utf-8")
    return str(value)


def document_extract_source_path_from_headers(headers: Mapping[str, str]) -> str:
    """Return the document source path from raw or UTF-8 hex metadata."""

    encoded = headers.get(WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_UTF8_HEX_HEADER, "")
    if encoded.strip():
        try:
            return bytes.fromhex(encoded.strip()).decode("utf-8")
        except ValueError as error:
            raise ValueError("Invalid document source path encoded header") from error
        except UnicodeDecodeError as error:
            raise ValueError("Invalid document source path encoded header") from error
    return headers.get(WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER, "")
