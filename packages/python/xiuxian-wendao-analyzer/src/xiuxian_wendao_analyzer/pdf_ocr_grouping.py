"""PDF OCR shard grouping helpers."""

from __future__ import annotations

from itertools import pairwise
from pathlib import Path
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


def _group_pdf_ocr_inputs(
    input_rows: Sequence[Mapping[str, Any]],
) -> list[tuple[list[int], list[Mapping[str, Any]]]]:
    groups: list[tuple[list[int], list[Mapping[str, Any]]]] = []
    current_indexes: list[int] = []
    current_rows: list[Mapping[str, Any]] = []
    for index, input_row in enumerate(input_rows):
        if current_rows and _can_extend_source_page_group(current_rows[-1], input_row):
            current_indexes.append(index)
            current_rows.append(input_row)
            continue
        if current_rows:
            groups.append((current_indexes, current_rows))
        current_indexes = [index]
        current_rows = [input_row]
    if current_rows:
        groups.append((current_indexes, current_rows))
    return groups


def _can_extend_source_page_group(
    previous_row: Mapping[str, Any],
    input_row: Mapping[str, Any],
) -> bool:
    if not _should_try_source_pdf_page_range(previous_row):
        return False
    if not _should_try_source_pdf_page_range(input_row):
        return False
    if str(previous_row["sourcePath"]) != str(input_row["sourcePath"]):
        return False
    return int(input_row["pageIndex"]) == int(previous_row["pageIndex"]) + 1


def _is_source_pdf_page_range_group(
    input_rows: Sequence[Mapping[str, Any]],
) -> bool:
    return all(_should_try_source_pdf_page_range(row) for row in input_rows) and all(
        _can_extend_source_page_group(previous_row, input_row)
        for previous_row, input_row in pairwise(input_rows)
    )


def _flatten_group_results(
    input_count: int,
    group_results: Sequence[Sequence[tuple[int, Mapping[str, Any]]]],
) -> list[Mapping[str, Any]]:
    ordered: list[Mapping[str, Any] | None] = [None] * input_count
    for group in group_results:
        for index, result in group:
            ordered[index] = result
    return [
        (
            result
            if result is not None
            else {"status": "failed", "errorMessage": "missing result"}
        )
        for result in ordered
    ]


def _should_try_source_pdf_page_range(input_row: Mapping[str, Any]) -> bool:
    if str(input_row.get("shardType", "")) != "page":
        return False
    source_path = Path(str(input_row.get("sourcePath", "")))
    return source_path.suffix.lower() == ".pdf" and source_path.is_file()
