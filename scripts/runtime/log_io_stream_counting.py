#!/usr/bin/env python3
"""Counting and cursor-initialization helpers for channel log I/O."""

from __future__ import annotations

from typing import TYPE_CHECKING

from log_io_models import LogCursor, LogCursorKind

if TYPE_CHECKING:
    from collections.abc import Iterator
    from pathlib import Path


def iter_log_lines(
    path: Path, *, encoding: str = "utf-8", errors: str = "ignore"
) -> Iterator[str]:
    """Yield log lines without trailing newlines."""
    if not path.exists():
        return
    with path.open("r", encoding=encoding, errors=errors) as handle:
        for raw_line in handle:
            yield raw_line.rstrip("\n")


def count_log_lines(
    path: Path, *, encoding: str = "utf-8", errors: str = "ignore"
) -> int:
    """Count log lines using streaming iteration."""
    if not path.exists():
        return 0
    with path.open("r", encoding=encoding, errors=errors) as handle:
        return sum(1 for _ in handle)


def count_log_bytes(path: Path) -> int:
    """Return file size in bytes for offset-based log cursors."""
    if not path.exists():
        return 0
    return int(path.stat().st_size)


def init_log_cursor(
    path: Path,
    *,
    kind: LogCursorKind = "offset",
    encoding: str = "utf-8",
    errors: str = "ignore",
) -> LogCursor:
    """Initialize a cursor from the current file position for the requested mode."""
    if kind == "line":
        return LogCursor(
            kind="line", value=count_log_lines(path, encoding=encoding, errors=errors)
        )
    return LogCursor(kind="offset", value=count_log_bytes(path))
