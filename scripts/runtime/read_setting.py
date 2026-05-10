#!/usr/bin/env python3
"""Read arbitrary config value via xiuxian.toml resolver."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Any

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

from config_resolver_core import repo_root_from, settings_candidates  # noqa: E402

try:
    import tomllib
except ModuleNotFoundError:
    try:
        import tomli as tomllib  # type: ignore[no-redef]
    except ModuleNotFoundError as exc:  # pragma: no cover - environment guard
        raise ModuleNotFoundError(
            "No TOML parser available. Use Python 3.11+ or install tomli."
        ) from exc


def _dig(mapping: object, *keys: str) -> object | None:
    cursor = mapping
    for key in keys:
        if not isinstance(cursor, dict) or key not in cursor:
            return None
        cursor = cursor[key]
    return cursor


def _normalize_scalar(value: object) -> str:
    if value is None:
        return ""
    text = str(value).strip()
    if text in {"", "null", "None", "~"}:
        return ""
    return text


def _read_from_toml_candidates(key: str) -> str:
    repo_root = repo_root_from(Path(__file__).resolve())
    key_parts = tuple(part for part in key.split(".") if part)
    if not key_parts:
        return ""
    for candidate_path in settings_candidates(repo_root):
        if candidate_path.suffix.lower() != ".toml" or not candidate_path.exists():
            continue
        try:
            document: dict[str, Any] = tomllib.loads(
                candidate_path.read_text(encoding="utf-8", errors="ignore")
            )
        except tomllib.TOMLDecodeError:
            continue
        value = _dig(document, *key_parts)
        if value is None:
            continue
        return _normalize_scalar(value)
    return ""


def read_setting(key: str) -> str:
    return _read_from_toml_candidates(key)


def main() -> int:
    parser = argparse.ArgumentParser(description="Read setting value")
    parser.add_argument(
        "--key", required=True, help="dot-path setting key (for example: gateway.bind)"
    )
    args = parser.parse_args()
    print(read_setting(str(args.key)), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
