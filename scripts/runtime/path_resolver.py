#!/usr/bin/env python3
"""Shared path resolution helpers for channel scripts."""

from __future__ import annotations

import os
from pathlib import Path


def default_report_path(filename: str) -> Path:
    """Build report path under runtime reports directory."""
    runtime_root = Path(os.environ.get("PRJ_RUNTIME_DIR", ".run"))
    if not runtime_root.is_absolute():
        runtime_root = Path.cwd() / runtime_root
    return runtime_root / "reports" / filename


def project_root_from(start: Path) -> Path:
    """Walk up parent chain to find git root (fallback to start)."""
    for candidate in [start, *start.parents]:
        if (candidate / ".git").exists():
            return candidate
    return start


def resolve_path(path_str: str, project_root: Path) -> Path:
    """Resolve user path against project root when relative."""
    path = Path(path_str).expanduser()
    if not path.is_absolute():
        path = project_root / path
    return path.resolve()
