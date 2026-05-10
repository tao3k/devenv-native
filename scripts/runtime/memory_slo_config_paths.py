#!/usr/bin/env python3
"""Path helpers for memory/session SLO configuration."""

from __future__ import annotations

import os
from pathlib import Path


def default_report_path(filename: str) -> Path:
    """Return default report path under PRJ runtime directory."""
    runtime_root = Path(os.environ.get("PRJ_RUNTIME_DIR", ".run"))
    if not runtime_root.is_absolute():
        runtime_root = Path.cwd() / runtime_root
    return runtime_root / "reports" / filename


def project_root_from(start: Path) -> Path:
    """Walk up to git root if available; otherwise keep current start path."""
    for candidate in [start, *start.parents]:
        if (candidate / ".git").exists():
            return candidate
    return start


def resolve_path(path_str: str, project_root: Path) -> Path:
    """Resolve a path relative to project root when input is not absolute."""
    path = Path(path_str).expanduser()
    if not path.is_absolute():
        path = project_root / path
    return path.resolve()
