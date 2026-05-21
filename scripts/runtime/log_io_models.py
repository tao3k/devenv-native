#!/usr/bin/env python3
"""Shared data models for channel log I/O helpers."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

DEFAULT_LOG_TAIL_BYTES = 256 * 1024
LogCursorKind = Literal["line", "offset"]


@dataclass(frozen=True)
class LogCursor:
    """Generic log cursor that can represent either line or byte offsets."""

    kind: LogCursorKind
    value: int
