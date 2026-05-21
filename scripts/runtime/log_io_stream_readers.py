#!/usr/bin/env python3
"""Incremental readers for channel log I/O."""

from __future__ import annotations

from typing import TYPE_CHECKING

from log_io_models import LogCursor
from log_io_stream_counting import count_log_bytes

if TYPE_CHECKING:
    from pathlib import Path


def read_new_log_lines(
    path: Path,
    cursor: int,
    *,
    encoding: str = "utf-8",
    errors: str = "ignore",
) -> tuple[int, list[str]]:
    """
    Read lines after a line-number cursor.

    Returns `(next_cursor, lines_since_cursor)` where cursor is 0-based line count.
    """
    start_cursor = max(0, int(cursor))
    next_cursor = start_cursor
    if not path.exists():
        return next_cursor, []

    lines: list[str] = []
    with path.open("r", encoding=encoding, errors=errors) as handle:
        for index, raw_line in enumerate(handle):
            if index >= start_cursor:
                lines.append(raw_line.rstrip("\n").rstrip("\r"))
            next_cursor = index + 1
    return next_cursor, lines


def read_new_log_lines_by_offset(
    path: Path,
    offset: int,
    *,
    encoding: str = "utf-8",
    errors: str = "ignore",
) -> tuple[int, list[str]]:
    """
    Read newly appended lines after a byte-offset cursor.

    Returns `(next_offset, lines_since_offset)`. When file is truncated/rotated
    and `offset` exceeds current size, reading restarts from 0.
    """
    start_offset = max(0, int(offset))
    if not path.exists():
        return start_offset, []

    size = count_log_bytes(path)
    if start_offset > size:
        start_offset = 0

    with path.open("rb") as handle:
        if start_offset > 0:
            handle.seek(start_offset - 1)
            prev = handle.read(1)
            handle.seek(start_offset)
            skip_fragment = prev not in (b"\n", b"\r")
            if skip_fragment:
                handle.readline()
            payload = handle.read()
            if skip_fragment and not payload and start_offset < size:
                handle.seek(0)
                payload = handle.read()
        else:
            handle.seek(0)
            payload = handle.read()

    lines = payload.decode(encoding, errors=errors).splitlines()
    return size, lines


def read_new_log_lines_with_cursor(
    path: Path,
    cursor: LogCursor,
    *,
    encoding: str = "utf-8",
    errors: str = "ignore",
) -> tuple[LogCursor, list[str]]:
    """Read appended log lines based on cursor mode and return the next cursor."""
    value = max(0, int(cursor.value))
    if cursor.kind == "line":
        next_value, lines = read_new_log_lines(
            path, value, encoding=encoding, errors=errors
        )
        return LogCursor(kind="line", value=next_value), lines
    if cursor.kind == "offset":
        next_value, lines = read_new_log_lines_by_offset(
            path,
            value,
            encoding=encoding,
            errors=errors,
        )
        return LogCursor(kind="offset", value=next_value), lines
    raise ValueError(f"unsupported log cursor kind: {cursor.kind}")
