"""Hosted VLM/OCR page-window and region-composite task grouping."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

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
) -> list[list[Mapping[str, Any]]]:
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
        if len(current) >= composite_size or not can_extend_ocr2_region_composite(
            current[-1],
            row,
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


def flatten_page_window_results(
    window_results: Sequence[Sequence[Mapping[str, Any]]],
) -> list[Mapping[str, Any]]:
    return [result for window in window_results for result in window]
