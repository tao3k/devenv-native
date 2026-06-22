"""Hosted VLM/OCR page-window and region-composite task grouping."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, Any

from ..pdf_ocr_contracts import (
    HOSTED_VLM_OCR_REGION_COMPOSITE_ADAPTIVE_SMALL_REGION_MODE,
    HOSTED_VLM_OCR_REGION_COMPOSITE_DISABLED_MODE,
    HOSTED_VLM_OCR_REGION_COMPOSITE_FIXED_MODE,
)

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


def ocr2_page_windows(
    input_rows: Sequence[Mapping[str, Any]],
    window_size: int,
) -> list[list[Mapping[str, Any]]]:
    windows: list[list[Mapping[str, Any]]] = []
    current: list[Mapping[str, Any]] = []
    for row in input_rows:
        if not is_page_window_candidate(row):
            if current:
                windows.append(current)
                current = []
            windows.append([row])
            continue
        if not current:
            current.append(row)
            continue
        if len(current) >= window_size or not can_extend_ocr2_page_window(
            current[-1],
            row,
        ):
            windows.append(current)
            current = [row]
            continue
        current.append(row)
    if current:
        windows.append(current)
    return windows


def ocr2_region_composite_tasks(
    input_rows: Sequence[Mapping[str, Any]],
    composite_size: int,
    *,
    composite_mode: str = HOSTED_VLM_OCR_REGION_COMPOSITE_FIXED_MODE,
    max_source_pixel_area: int = 0,
    max_image_bytes: int = 0,
) -> list[list[Mapping[str, Any]]]:
    if (
        composite_size <= 1
        or composite_mode == HOSTED_VLM_OCR_REGION_COMPOSITE_DISABLED_MODE
    ):
        return [[row] for row in input_rows]
    tasks: list[list[Mapping[str, Any]]] = []
    current: list[Mapping[str, Any]] = []
    for row in input_rows:
        if not is_region_composite_candidate(row):
            if current:
                tasks.append(current)
                current = []
            tasks.append([row])
            continue
        if not current:
            current.append(row)
            continue
        if (
            len(current) >= composite_size
            or not can_extend_ocr2_region_composite(
                current[-1],
                row,
            )
            or not can_fit_ocr2_region_composite(
                [*current, row],
                composite_mode=composite_mode,
                max_source_pixel_area=max_source_pixel_area,
                max_image_bytes=max_image_bytes,
            )
        ):
            tasks.append(current)
            current = [row]
            continue
        current.append(row)
    if current:
        tasks.append(current)
    return tasks


def is_page_window_candidate(row: Mapping[str, Any]) -> bool:
    return str(row.get("shardType") or "") == "page"


def is_region_composite_candidate(row: Mapping[str, Any]) -> bool:
    return str(row.get("shardType") or "") == "region"


def can_extend_ocr2_page_window(
    previous: Mapping[str, Any],
    current: Mapping[str, Any],
) -> bool:
    if str(previous.get("sourcePath")) != str(current.get("sourcePath")):
        return False
    if str(previous.get("sourceContentHash")) != str(current.get("sourceContentHash")):
        return False
    previous_page = previous.get("pageIndex")
    current_page = current.get("pageIndex")
    return (
        isinstance(previous_page, int)
        and isinstance(current_page, int)
        and current_page == previous_page + 1
    )


def can_extend_ocr2_region_composite(
    previous: Mapping[str, Any],
    current: Mapping[str, Any],
) -> bool:
    return (
        str(previous.get("sourcePath")) == str(current.get("sourcePath"))
        and str(previous.get("sourceContentHash"))
        == str(current.get("sourceContentHash"))
        and previous.get("pageIndex") == current.get("pageIndex")
        and str(previous.get("parentShardElementId"))
        == str(current.get("parentShardElementId"))
    )


def can_fit_ocr2_region_composite(
    rows: Sequence[Mapping[str, Any]],
    *,
    composite_mode: str,
    max_source_pixel_area: int,
    max_image_bytes: int,
) -> bool:
    if composite_mode == HOSTED_VLM_OCR_REGION_COMPOSITE_FIXED_MODE:
        return True
    if composite_mode != HOSTED_VLM_OCR_REGION_COMPOSITE_ADAPTIVE_SMALL_REGION_MODE:
        return False
    return (
        region_rows_source_pixel_area(rows) <= max_source_pixel_area
        and region_rows_image_bytes(rows) <= max_image_bytes
    )


def region_rows_source_pixel_area(rows: Sequence[Mapping[str, Any]]) -> int:
    return sum(region_row_source_pixel_area(row) for row in rows)


def region_row_source_pixel_area(row: Mapping[str, Any]) -> int:
    try:
        left = int(row.get("sourcePagePixelLeft") or 0)
        top = int(row.get("sourcePagePixelTop") or 0)
        right = int(row.get("sourcePagePixelRight") or 0)
        bottom = int(row.get("sourcePagePixelBottom") or 0)
    except (TypeError, ValueError):
        return 0
    return max(0, right - left) * max(0, bottom - top)


def region_rows_image_bytes(rows: Sequence[Mapping[str, Any]]) -> int:
    total = 0
    for row in rows:
        try:
            total += Path(str(row.get("imagePath") or "")).stat().st_size
        except OSError:
            return max_image_byte_sentinel()
    return total


def max_image_byte_sentinel() -> int:
    return 2**63 - 1


def flatten_page_window_results(
    window_results: Sequence[Sequence[Mapping[str, Any]]],
) -> list[Mapping[str, Any]]:
    return [result for window in window_results for result in window]
