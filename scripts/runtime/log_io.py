#!/usr/bin/env python3
"""
Shared log I/O helpers for channel scripts.

Goals:
- avoid full-file log payload allocations for large runtime logs;
- keep APIs small and script-friendly (no external dependencies).
"""

from __future__ import annotations

from log_io_models import DEFAULT_LOG_TAIL_BYTES, LogCursor, LogCursorKind
from log_io_stream import (
    count_log_bytes,
    count_log_lines,
    init_log_cursor,
    iter_log_lines,
    read_new_log_lines,
    read_new_log_lines_by_offset,
    read_new_log_lines_with_cursor,
)
from log_io_tail import read_log_tail_lines, read_log_tail_text, tail_log_lines

__all__ = [
    "DEFAULT_LOG_TAIL_BYTES",
    "LogCursor",
    "LogCursorKind",
    "count_log_bytes",
    "count_log_lines",
    "init_log_cursor",
    "iter_log_lines",
    "read_log_tail_lines",
    "read_log_tail_text",
    "read_new_log_lines",
    "read_new_log_lines_by_offset",
    "read_new_log_lines_with_cursor",
    "tail_log_lines",
]
