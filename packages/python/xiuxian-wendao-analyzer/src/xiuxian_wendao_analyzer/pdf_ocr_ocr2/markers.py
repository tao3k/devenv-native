"""Hosted VLM/OCR page and region markers."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence

_HOSTED_VLM_OCR_PAGE_MARKER_PREFIX = "<!-- xiuxian-wendao-hosted-vlm-page:"
_HOSTED_VLM_OCR_PAGE_MARKER_SUFFIX = " -->"
_HOSTED_VLM_OCR_REGION_MARKER_PREFIX = "<!-- xiuxian-wendao-hosted-vlm-region:"
_HOSTED_VLM_OCR_REGION_MARKER_SUFFIX = " -->"


def ocr2_page_marker(input_row: Mapping[str, Any]) -> str:
    return (
        f"{_HOSTED_VLM_OCR_PAGE_MARKER_PREFIX}"
        f"{input_row.get('pageIndex')}"
        f"{_HOSTED_VLM_OCR_PAGE_MARKER_SUFFIX}"
    )


def ocr2_region_marker(input_row: Mapping[str, Any]) -> str:
    return (
        f"{_HOSTED_VLM_OCR_REGION_MARKER_PREFIX}"
        f"{input_row.get('pageIndex')}:"
        f"{input_row.get('regionIndex')}:"
        f"{input_row.get('shardElementId')}"
        f"{_HOSTED_VLM_OCR_REGION_MARKER_SUFFIX}"
    )


def extract_ocr2_page_window_markdown(
    markdown: str,
    input_rows: Sequence[Mapping[str, Any]],
) -> list[str]:
    return extract_ocr2_marked_sections(
        markdown,
        [ocr2_page_marker(row) for row in input_rows],
        "page-window",
    )


def extract_ocr2_region_composite_markdown(
    markdown: str,
    input_rows: Sequence[Mapping[str, Any]],
) -> list[str]:
    return extract_ocr2_marked_sections(
        markdown,
        [ocr2_region_marker(row) for row in input_rows],
        "region-composite",
    )


def extract_ocr2_marked_sections(
    markdown: str,
    markers: Sequence[str],
    label: str,
) -> list[str]:
    if not markdown.strip():
        raise ValueError(f"Hosted VLM/OCR {label} response returned empty text")
    sections = []
    cursor = 0
    for index, marker in enumerate(markers):
        marker_position = markdown.find(marker, cursor)
        if marker_position < 0:
            raise ValueError(
                f"Hosted VLM/OCR {label} response is missing a section marker"
            )
        content_start = marker_position + len(marker)
        if index + 1 < len(markers):
            next_position = markdown.find(markers[index + 1], content_start)
            if next_position < 0:
                raise ValueError(
                    f"Hosted VLM/OCR {label} response is missing the next section marker"
                )
            content_end = next_position
        else:
            content_end = len(markdown)
        text = markdown[content_start:content_end].strip()
        if not text:
            raise ValueError(
                f"Hosted VLM/OCR {label} response returned an empty section"
            )
        sections.append(text)
        cursor = content_end
    return sections
