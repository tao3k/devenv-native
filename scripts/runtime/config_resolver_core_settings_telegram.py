#!/usr/bin/env python3
"""Telegram config readers for channel resolver (xiuxian.toml)."""

from __future__ import annotations

import os
from pathlib import Path
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:
    try:
        import tomli as tomllib  # type: ignore[no-redef]
    except ModuleNotFoundError as exc:  # pragma: no cover - environment guard
        raise ModuleNotFoundError(
            "No TOML parser available. Use Python 3.11+ or install tomli."
        ) from exc


def repo_root_from(start: Path) -> Path:
    """Resolve repository root by walking upward until .git is found."""
    for candidate in [start, *start.parents]:
        if (candidate / ".git").exists():
            return candidate
    return start


def settings_candidates(repo_root: Path) -> list[Path]:
    """Return xiuxian.toml candidates in precedence order."""
    prj_config_home = Path(
        os.environ.get("PRJ_CONFIG_HOME", str(repo_root / ".config"))
    )
    user_xiuxian = prj_config_home / "xiuxian-artisan-workshop" / "xiuxian.toml"
    system_xiuxian = (
        repo_root
        / "packages"
        / "rust"
        / "crates"
        / "xiuxian-wendao"
        / "resources"
        / "config"
        / "xiuxian.toml"
    )
    return [user_xiuxian, system_xiuxian]


def _normalize_toml_scalar(value: object) -> str:
    if value is None:
        return ""
    text = str(value).strip()
    if text in {"", "null", "None", "~"}:
        return ""
    return text


def _dig(mapping: dict[str, Any], *keys: str) -> object:
    cursor: object = mapping
    for key in keys:
        if not isinstance(cursor, dict) or key not in cursor:
            return None
        cursor = cursor[key]
    return cursor


def read_telegram_key_from_toml(path: Path, key: str) -> str | None:
    """Read telegram-related scalar from xiuxian TOML."""
    if not path.exists():
        return None
    try:
        document = tomllib.loads(path.read_text(encoding="utf-8", errors="ignore"))
    except tomllib.TOMLDecodeError:
        return None

    key_paths: dict[str, tuple[tuple[str, ...], ...]] = {
        "webhook_secret_token": (
            ("telegram", "webhook_secret_token"),
            ("telegram", "webhook", "secret_token"),
            ("telegram", "secret_token"),
        ),
        "webhook_bind": (
            ("telegram", "webhook_bind"),
            ("telegram", "webhook", "bind"),
        ),
        "session_partition": (
            ("telegram", "session_partition"),
            ("telegram", "session", "partition"),
        ),
    }
    candidates = key_paths.get(key, (("telegram", key),))
    for path_keys in candidates:
        value = _dig(document, *path_keys)
        if value is None:
            continue
        return _normalize_toml_scalar(value)
    return None
