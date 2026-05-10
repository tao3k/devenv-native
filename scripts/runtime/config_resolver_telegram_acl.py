#!/usr/bin/env python3
"""ACL and username resolution from Telegram settings."""

from __future__ import annotations

from pathlib import Path

from config_resolver_core import (
    read_telegram_acl_allow_users,
    repo_root_from,
    settings_candidates,
)


def allowed_users_from_settings(repo_root: Path | None = None) -> list[str]:
    """Resolve `telegram.acl.allow.users` from merged settings (user overrides system)."""
    root = repo_root or repo_root_from(Path(__file__).resolve())
    for settings_path in settings_candidates(root):
        allowed_users = read_telegram_acl_allow_users(settings_path)
        if allowed_users is not None:
            return allowed_users
    return []


def username_from_settings(repo_root: Path | None = None) -> str | None:
    """Resolve first explicit username from allowlist."""
    for first in allowed_users_from_settings(repo_root):
        if first in {"*", "''", '""'}:
            return None
        return first
    return None
