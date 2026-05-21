#!/usr/bin/env python3
"""Environment profile parsing helpers for retained config resolution."""

from __future__ import annotations

import os
import re
from pathlib import Path

from config_resolver_core_scalars import strip_inline_comment, unquote

ENV_ASSIGNMENT_RE = re.compile(r"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)$")


def group_env_file(repo_root: Path) -> Path:
    """Resolve group profile .env file path with environment override."""
    explicit = os.environ.get("OMNI_TEST_GROUP_ENV_FILE", "").strip()
    if explicit:
        return Path(explicit)
    return repo_root / ".run" / "config" / "lingchong-channel-groups.env"


def dotenv_file(repo_root: Path) -> Path:
    """Resolve dotenv file path with environment override."""
    explicit = os.environ.get("OMNI_TEST_DOTENV_FILE", "").strip()
    if explicit:
        return Path(explicit)
    return repo_root / ".env"


def read_env_profile(path: Path) -> dict[str, str]:
    """Parse .env style key/value file into dict."""
    if not path.exists():
        return {}

    values: dict[str, str] = {}
    for raw in path.read_text(encoding="utf-8", errors="ignore").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        match = ENV_ASSIGNMENT_RE.match(line)
        if not match:
            continue
        key = match.group(1).strip()
        payload = unquote(strip_inline_comment(match.group(2)))
        values[key] = payload
    return values
