"""DeepSeek-OCR-2 backend management owned by the analyzer package."""

from __future__ import annotations

from .manager import (
    BackendLaunch,
    Ocr2BackendAction,
    Ocr2BackendError,
    Ocr2BackendOptions,
    build_start_backend_launch,
    run_ocr2_backend_action,
)

__all__ = [
    "BackendLaunch",
    "Ocr2BackendAction",
    "Ocr2BackendError",
    "Ocr2BackendOptions",
    "build_start_backend_launch",
    "run_ocr2_backend_action",
]
