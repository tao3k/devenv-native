#!/usr/bin/env python3
"""Resolve Valkey endpoint values from xiuxian.toml and env."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
from urllib.parse import urlparse

try:
    import tomllib
except ModuleNotFoundError:
    try:
        import tomli as tomllib  # type: ignore[no-redef]
    except ModuleNotFoundError as exc:  # pragma: no cover - environment guard
        raise ModuleNotFoundError(
            "No TOML parser available. Use Python 3.11+ or install tomli."
        ) from exc

DEFAULT_SCHEME = "redis"
DEFAULT_HOST = (
    os.environ.get("XIUXIAN_WENDAO_LOCAL_HOST", "localhost").strip() or "localhost"
)
DEFAULT_PORT = 6379
DEFAULT_DB = 0
NAMESPACED_VALKEY_URL_ENV = "XIUXIAN_WENDAO_VALKEY_URL"


def _normalize_port(value: object) -> int | None:
    if isinstance(value, bool):
        return None
    if isinstance(value, int):
        parsed = value
    elif isinstance(value, str):
        text = value.strip()
        if not text:
            return None
        try:
            parsed = int(text)
        except ValueError:
            return None
    else:
        return None
    if 1 <= parsed <= 65535:
        return parsed
    return None


def _normalize_db(path: str) -> int:
    text = path.strip().lstrip("/")
    if not text:
        return DEFAULT_DB
    first = text.split("/", 1)[0]
    try:
        parsed = int(first)
    except ValueError:
        return DEFAULT_DB
    return parsed if parsed >= 0 else DEFAULT_DB


def _format_netloc(
    host: str, port: int, username: str | None, password: str | None
) -> str:
    auth = ""
    if username is not None:
        auth = username
        if password is not None:
            auth = f"{auth}:{password}"
        auth = f"{auth}@"
    if ":" in host and not host.startswith("["):
        host = f"[{host}]"
    return f"{auth}{host}:{port}"


def _parse_candidate(value: object) -> dict[str, str] | None:
    if not isinstance(value, str):
        return None
    text = value.strip()
    if not text:
        return None
    parsed = urlparse(text)
    if not parsed.scheme:
        return None
    scheme = parsed.scheme.strip() or DEFAULT_SCHEME
    host = (parsed.hostname or DEFAULT_HOST).strip() or DEFAULT_HOST
    port = _normalize_port(parsed.port) or DEFAULT_PORT
    db = _normalize_db(parsed.path)
    netloc = _format_netloc(host, port, parsed.username, parsed.password)
    return {
        "scheme": scheme,
        "host": host,
        "port": str(port),
        "db": str(db),
        "url": f"{scheme}://{netloc}/{db}",
    }


def _repo_root_from(start: Path) -> Path:
    for candidate in [start, *start.parents]:
        if (candidate / ".git").exists():
            return candidate
    return start


def _xiuxian_toml_candidates() -> list[Path]:
    repo_root = _repo_root_from(Path(__file__).resolve())
    prj_config_home = Path(
        os.environ.get("PRJ_CONFIG_HOME", str(repo_root / ".config"))
    )
    return [
        prj_config_home / "xiuxian-artisan-workshop" / "xiuxian.toml",
        repo_root
        / "packages"
        / "rust"
        / "crates"
        / "xiuxian-wendao"
        / "resources"
        / "config"
        / "xiuxian.toml",
    ]


def _dig(mapping: object, *keys: str) -> object | None:
    cursor = mapping
    for key in keys:
        if not isinstance(cursor, dict) or key not in cursor:
            return None
        cursor = cursor[key]
    return cursor


def _read_first_xiuxian_value(*paths: tuple[str, ...]) -> object | None:
    for config_path in _xiuxian_toml_candidates():
        if not config_path.exists():
            continue
        try:
            data = tomllib.loads(
                config_path.read_text(encoding="utf-8", errors="ignore")
            )
        except tomllib.TOMLDecodeError:
            continue
        for key_path in paths:
            value = _dig(data, *key_path)
            if value is None:
                continue
            return value
    return None


def resolve_valkey_endpoint() -> dict[str, str]:
    resolved = _parse_candidate(os.environ.get(NAMESPACED_VALKEY_URL_ENV))
    if resolved is not None:
        return resolved

    resolved = _parse_candidate(
        _read_first_xiuxian_value(
            ("memory", "persistence_valkey_url"),
            ("session", "valkey_url"),
            ("wendao", "link_graph", "cache", "valkey_url"),
        )
    )
    if resolved is not None:
        return resolved

    return {
        "scheme": DEFAULT_SCHEME,
        "host": DEFAULT_HOST,
        "port": str(DEFAULT_PORT),
        "db": str(DEFAULT_DB),
        "url": f"{DEFAULT_SCHEME}://{DEFAULT_HOST}:{DEFAULT_PORT}/{DEFAULT_DB}",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Resolve Valkey endpoint values")
    parser.add_argument(
        "--field",
        default="url",
        choices=("scheme", "host", "port", "db", "url"),
        help="Endpoint field to print",
    )
    args = parser.parse_args()
    print(resolve_valkey_endpoint()[str(args.field)], end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
