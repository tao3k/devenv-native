#!/usr/bin/env python3
"""Compatibility facade for channel log I/O stream helpers."""

from __future__ import annotations

import log_io_stream_counting as _counting
import log_io_stream_readers as _readers

iter_log_lines = _counting.iter_log_lines
count_log_lines = _counting.count_log_lines
count_log_bytes = _counting.count_log_bytes
init_log_cursor = _counting.init_log_cursor

read_new_log_lines = _readers.read_new_log_lines
read_new_log_lines_by_offset = _readers.read_new_log_lines_by_offset
read_new_log_lines_with_cursor = _readers.read_new_log_lines_with_cursor
