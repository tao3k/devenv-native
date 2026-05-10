#!/usr/bin/env python3
"""Resolve tool-runtime endpoint values from xiuxian.toml and env."""

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

DEFAULT_SCHEME = "http"
DEFAULT_HOST = (
    os.environ.get("XIUXIAN_WENDAO_LOCAL_HOST", "localhost").strip() or "localhost"
)
DEFAULT_PORT = 3002
NAMESPACED_TOOL_BASE_URL_ENV = "XIUXIAN_WENDAO_TOOL_BASE_URL"
NAMESPACED_TOOL_PORT_ENV = "XIUXIAN_WENDAO_TOOL_PORT"


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


def _parse_url(url_value: object) -> tuple[str | None, str | None, int | None]:
    if not isinstance(url_value, str):
        return None, None, None
    text = url_value.strip()
    if not text:
        return None, None, None
    parsed = urlparse(text)
    if not parsed.scheme:
        return None, None, None
    return parsed.scheme.strip(), parsed.hostname, _normalize_port(parsed.port)


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


def resolve_tool_endpoint() -> dict[str, str]:
    scheme: str | None = None
    host: str | None = None
    port: int | None = None

    env_base_url = os.environ.get(NAMESPACED_TOOL_BASE_URL_ENV, "").strip()
    if env_base_url:
        candidate_scheme, candidate_host, candidate_port = _parse_url(env_base_url)
        if candidate_scheme is not None:
            scheme = candidate_scheme
            host = candidate_host
            port = candidate_port

    if scheme is None:
        xiuxian_url = _read_first_xiuxian_value(
            ("tool_runtime", "base_url"),
            ("embedding", "client_url"),
        )
        candidate_scheme, candidate_host, candidate_port = _parse_url(xiuxian_url)
        if candidate_scheme is not None:
            scheme = candidate_scheme
            host = candidate_host
            port = candidate_port

    preferred_port = _normalize_port(os.environ.get(NAMESPACED_TOOL_PORT_ENV))
    if preferred_port is None:
        preferred_port = _normalize_port(
            _read_first_xiuxian_value(
                ("tool_runtime", "preferred_embed_port"),
                ("tool_runtime", "port"),
            )
        )
    resolved_scheme = scheme or DEFAULT_SCHEME
    resolved_host = (host or DEFAULT_HOST).strip() or DEFAULT_HOST
    resolved_port = preferred_port or port or DEFAULT_PORT

    base_url = f"{resolved_scheme}://{resolved_host}:{resolved_port}"
    health_url = f"{base_url}/health"
    return {
        "scheme": resolved_scheme,
        "host": resolved_host,
        "port": str(resolved_port),
        "base_url": base_url,
        "health_url": health_url,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Resolve tool endpoint values")
    parser.add_argument(
        "--field",
        default="base_url",
        choices=("scheme", "host", "port", "base_url", "health_url"),
        help="Endpoint field to print",
    )
    args = parser.parse_args()
    print(resolve_tool_endpoint()[str(args.field)], end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
