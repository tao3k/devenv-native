"""Optional startup prewarm for the document extraction Flight service."""

from __future__ import annotations

import os
from pathlib import Path
from typing import TYPE_CHECKING

from .document_profiles import (
    DOCUMENT_EXTRACT_FULL_PROFILE,
    normalize_document_extract_profile,
)

if TYPE_CHECKING:
    from collections.abc import Callable

    from .documents import DocumentConverterProtocol

DOCUMENT_EXTRACT_PREWARM_SOURCE_PATH_ENV = "WENDAO_DOCUMENT_EXTRACT_PREWARM_SOURCE_PATH"
DOCUMENT_EXTRACT_PREWARM_PAGE_RANGES_ENV = "WENDAO_DOCUMENT_EXTRACT_PREWARM_PAGE_RANGES"
DOCUMENT_EXTRACT_PREWARM_PROFILE_ENV = "WENDAO_DOCUMENT_EXTRACT_PREWARM_PROFILE"


def document_extract_prewarm_page_ranges(
    lookup: "Callable[[str], str | None]" = os.environ.get,
) -> list[tuple[int, int]]:
    """Return 1-based inclusive page ranges used for document prewarm."""

    raw = (lookup(DOCUMENT_EXTRACT_PREWARM_PAGE_RANGES_ENV) or "").strip()
    if not raw:
        return [(1, 1)]
    ranges: list[tuple[int, int]] = []
    for part in raw.split(","):
        normalized = part.strip()
        if not normalized:
            continue
        if ":" in normalized:
            start_text, end_text = normalized.split(":", 1)
        else:
            start_text = end_text = normalized
        start = _parse_positive_page_index(
            start_text, DOCUMENT_EXTRACT_PREWARM_PAGE_RANGES_ENV
        )
        end = _parse_positive_page_index(
            end_text, DOCUMENT_EXTRACT_PREWARM_PAGE_RANGES_ENV
        )
        if end < start:
            raise ValueError(
                f"{DOCUMENT_EXTRACT_PREWARM_PAGE_RANGES_ENV} must use 1-based inclusive ranges"
            )
        ranges.append((start, end))
    if not ranges:
        raise ValueError(
            f"{DOCUMENT_EXTRACT_PREWARM_PAGE_RANGES_ENV} must include a range"
        )
    return ranges


def prewarm_document_extract_converter_from_env(
    *,
    converter_factory: "Callable[[str | None], DocumentConverterProtocol]",
    lookup: "Callable[[str], str | None]" = os.environ.get,
) -> "DocumentConverterProtocol | None":
    """Build and prewarm the configured document converter.

    Returns `None` when no prewarm source is configured.
    """

    source_path = (lookup(DOCUMENT_EXTRACT_PREWARM_SOURCE_PATH_ENV) or "").strip()
    if not source_path:
        return None
    source = Path(source_path).expanduser()
    if not source.exists():
        raise FileNotFoundError("document extract prewarm source path does not exist")
    profile = normalize_document_extract_profile(
        lookup(DOCUMENT_EXTRACT_PREWARM_PROFILE_ENV) or DOCUMENT_EXTRACT_FULL_PROFILE
    )
    converter = converter_factory(profile)
    for page_range in document_extract_prewarm_page_ranges(lookup):
        document = converter.convert(source, page_range=page_range).document
        markdown = document.export_to_markdown()
        if not markdown.strip():
            raise RuntimeError("document extract prewarm returned empty markdown")
    return converter


def _parse_positive_page_index(raw: str, env_name: str) -> int:
    try:
        value = int(raw.strip())
    except ValueError as exc:
        raise ValueError(f"{env_name} must contain integer page indices") from exc
    if value < 1:
        raise ValueError(f"{env_name} must use 1-based positive page indices")
    return value


__all__ = [
    "DOCUMENT_EXTRACT_PREWARM_PAGE_RANGES_ENV",
    "DOCUMENT_EXTRACT_PREWARM_PROFILE_ENV",
    "DOCUMENT_EXTRACT_PREWARM_SOURCE_PATH_ENV",
    "document_extract_prewarm_page_ranges",
    "prewarm_document_extract_converter_from_env",
]
