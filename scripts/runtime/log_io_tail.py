#!/usr/bin/env python3
"""Tail read helpers for channel log I/O."""

from __future__ import annotations

from collections import deque
from typing import TYPE_CHECKING

from log_io_models import DEFAULT_LOG_TAIL_BYTES
from log_io_stream import iter_log_lines

if TYPE_CHECKING:
    from pathlib import Path


def read_log_tail_text(
    path: Path,
    *,
    tail_bytes: int = DEFAULT_LOG_TAIL_BYTES,
    encoding: str = "utf-8",
    errors: str = "ignore",
) -> str:
    """Read only a bounded tail window from a log file."""
    if not path.exists():
        return ""

    clamped_tail = max(4 * 1024, int(tail_bytes))
    with path.open("rb") as handle:
        size = path.stat().st_size
        if size <= clamped_tail:
            payload = handle.read()
        else:
            handle.seek(size - clamped_tail)
            handle.readline()
            payload = handle.read()
    return payload.decode(encoding, errors=errors)


def read_log_tail_lines(
    path: Path,
    *,
    tail_bytes: int = DEFAULT_LOG_TAIL_BYTES,
    encoding: str = "utf-8",
    errors: str = "ignore",
) -> list[str]:
    """Read bounded log tail and split into normalized lines."""
    return read_log_tail_text(
        path,
        tail_bytes=tail_bytes,
        encoding=encoding,
        errors=errors,
    ).splitlines()


def tail_log_lines(
    path: Path,
    n: int,
    *,
    encoding: str = "utf-8",
    errors: str = "ignore",
) -> list[str]:
    """Return the last `n` lines using streaming iteration."""
    if n <= 0 or not path.exists():
        return []
    buf: deque[str] = deque(maxlen=n)
    for line in iter_log_lines(path, encoding=encoding, errors=errors):
        buf.append(line)
    return list(buf)
