"""Document extraction header parsing."""

from __future__ import annotations

import os
from typing import Any

from .document_profiles import (
    DOCUMENT_EXTRACT_DEFAULT_PROFILE,
    DOCUMENT_EXTRACT_PROFILE_ENV,
    normalize_document_extract_profile,
)
from .document_service_routes import (
    DOCUMENT_EXTRACT_CONVERTER_CACHE_DISABLED,
    DOCUMENT_EXTRACT_CONVERTER_CACHE_PROFILE,
    WENDAO_DOCUMENT_EXTRACT_CONVERTER_CACHE_ENV,
    WENDAO_DOCUMENT_EXTRACT_PAGE_RANGE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER,
)


def header_bool(headers: dict[str, str] | Any, key: str, default: bool) -> bool:
    """Parse a boolean document extraction header."""

    value = headers.get(key, "")
    if value.lower() in {"true", "1", "yes"}:
        return True
    if value.lower() in {"false", "0", "no"}:
        return False
    return default


def document_extract_profile(headers: dict[str, str] | Any) -> str:
    """Resolve the requested document extraction profile."""

    requested_profile = headers.get(WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER)
    default_profile = os.environ.get(
        DOCUMENT_EXTRACT_PROFILE_ENV,
        DOCUMENT_EXTRACT_DEFAULT_PROFILE,
    )
    return normalize_document_extract_profile(requested_profile or default_profile)


def document_extract_converter_cache_mode() -> str:
    """Resolve converter cache mode from process environment."""

    return document_extract_converter_cache_mode_with_lookup(os.environ.get)


def document_extract_converter_cache_mode_with_lookup(lookup: Any) -> str:
    """Resolve converter cache mode using an injectable lookup."""

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


def document_extract_page_range(
    headers: dict[str, str] | Any,
) -> tuple[int, int] | None:
    """Parse an optional 1-based inclusive document page range."""

    value = headers.get(WENDAO_DOCUMENT_EXTRACT_PAGE_RANGE_HEADER, "").strip()
    if not value:
        return None
    parts = value.split(":")
    if len(parts) != 2:
        raise ValueError("document extract page range must use 1-based inclusive `start:end`")
    try:
        start, end = (int(part) for part in parts)
    except ValueError as exc:
        raise ValueError("document extract page range must use integer page numbers") from exc
    if start <= 0 or end <= 0 or start > end:
        raise ValueError("document extract page range must satisfy 1 <= start <= end")
    return (start, end)

