"""Prewarm helpers for PDF OCR workers."""

from __future__ import annotations

import os
from pathlib import Path
from typing import TYPE_CHECKING

from .pdf_ocr_worker_options import (
    PDF_OCR_PREWARM_PAGE_INDEX_ENV,
    PDF_OCR_PREWARM_PAGE_INDICES_ENV,
    PDF_OCR_PREWARM_PROFILES_ENV,
    PDF_OCR_PREWARM_SOURCE_PATH_ENV,
)

if TYPE_CHECKING:
    from collections.abc import Callable

    from .documents import DocumentConverterProtocol


def _prewarm_profiles_from_env() -> list[str]:
    raw = os.environ.get(PDF_OCR_PREWARM_PROFILES_ENV, "")
    profiles = []
    seen = set()
    for part in raw.replace(";", ",").split(","):
        profile = part.strip()
        if not profile or profile in seen:
            continue
        seen.add(profile)
        profiles.append(profile)
    return profiles


def _prewarm_converter_from_env(converter: DocumentConverterProtocol) -> None:
    source_path = os.environ.get(PDF_OCR_PREWARM_SOURCE_PATH_ENV)
    if not source_path:
        return
    for page_index in _prewarm_page_indices_from_env(os.environ.get):
        page_number = page_index + 1
        result = converter.convert(
            Path(source_path),
            page_range=(page_number, page_number),
        )
        markdown = result.document.export_to_markdown()
        if not markdown.strip():
            raise RuntimeError(
                f"Docling OCR prewarm returned empty text for page index {page_index}"
            )


def _prewarm_page_indices_from_env(lookup: Callable[[str], str | None]) -> list[int]:
    raw_indices = lookup(PDF_OCR_PREWARM_PAGE_INDICES_ENV)
    if raw_indices is None or not raw_indices.strip():
        return [_prewarm_page_index_from_env(lookup)]

    indices = []
    seen = set()
    for part in raw_indices.replace(";", ",").split(","):
        raw_index = part.strip()
        if not raw_index:
            continue
        page_index = _parse_prewarm_page_index(
            raw_index, PDF_OCR_PREWARM_PAGE_INDICES_ENV
        )
        if page_index in seen:
            continue
        seen.add(page_index)
        indices.append(page_index)
    if not indices:
        raise RuntimeError(
            f"{PDF_OCR_PREWARM_PAGE_INDICES_ENV} must include a page index"
        )
    return indices


def _prewarm_page_index_from_env(lookup: Callable[[str], str | None]) -> int:
    raw = lookup(PDF_OCR_PREWARM_PAGE_INDEX_ENV)
    if raw is None or not raw.strip():
        return 0
    return _parse_prewarm_page_index(raw.strip(), PDF_OCR_PREWARM_PAGE_INDEX_ENV)


def _parse_prewarm_page_index(raw: str, env_name: str) -> int:
    try:
        value = int(raw)
    except ValueError as exc:
        raise RuntimeError(
            f"{env_name} must contain non-negative integer page indices"
        ) from exc
    if value < 0:
        raise RuntimeError(f"{env_name} must contain non-negative integer page indices")
    return value
