#!/usr/bin/env python3
"""Session-partition resolution for Telegram channel settings."""

from __future__ import annotations

import os
from pathlib import Path

from config_resolver_core import (
    read_telegram_key_from_toml,
    repo_root_from,
    settings_candidates,
)
from config_resolver_runtime import normalize_telegram_session_partition_mode


def telegram_session_partition_mode(repo_root: Path | None = None) -> str | None:
    """Resolve canonical telegram session partition mode from env/settings."""
    root = repo_root or repo_root_from(Path(__file__).resolve())
    env_mode = os.environ.get("XIUXIAN_DAOCHANG_TELEGRAM_SESSION_PARTITION", "").strip()
    normalized_env_mode = normalize_telegram_session_partition_mode(env_mode)
    if normalized_env_mode:
        return normalized_env_mode

    for settings_path in settings_candidates(root):
        configured = read_telegram_key_from_toml(settings_path, "session_partition")
        if configured is None:
            continue
        normalized = normalize_telegram_session_partition_mode(configured)
        if normalized:
            return normalized
    return None
