#!/usr/bin/env python3
"""Group profile and dotenv-backed value resolvers."""

from __future__ import annotations

import os
from pathlib import Path

from config_resolver_core import (
    dotenv_file,
    group_env_file,
    read_env_profile,
    repo_root_from,
)


def group_profile_value(key: str, repo_root: Path | None = None) -> str | None:
    """Resolve value from process env first, then group profile env file."""
    in_process = os.environ.get(key, "").strip()
    if in_process:
        return in_process

    root = repo_root or repo_root_from(Path(__file__).resolve())
    profile_values = read_env_profile(group_env_file(root))
    value = profile_values.get(key, "").strip()
    if not value or value in {"null", "None", "~"}:
        return None
    return value


def group_profile_int(key: str, repo_root: Path | None = None) -> int | None:
    """Resolve integer value from group profile."""
    raw = group_profile_value(key, repo_root)
    if raw is None:
        return None
    try:
        return int(raw)
    except ValueError as error:
        raise ValueError(f"{key} must be an integer, got '{raw}'.") from error


def group_profile_chat_ids(repo_root: Path | None = None) -> tuple[int, ...]:
    """Resolve unique chat ids from standard group profile keys."""
    ordered: list[int] = []
    for key in ("OMNI_TEST_CHAT_ID", "OMNI_TEST_CHAT_B", "OMNI_TEST_CHAT_C"):
        value = group_profile_int(key, repo_root)
        if value is None:
            continue
        if value not in ordered:
            ordered.append(value)
    return tuple(ordered)


def env_or_dotenv_value(key: str, repo_root: Path | None = None) -> str | None:
    """Resolve value from process env first, then .env file."""
    in_process = os.environ.get(key, "").strip()
    if in_process:
        return in_process

    root = repo_root or repo_root_from(Path(__file__).resolve())
    dotenv_values = read_env_profile(dotenv_file(root))
    value = dotenv_values.get(key, "").strip()
    if not value or value in {"null", "None", "~"}:
        return None
    return value
