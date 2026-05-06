#!/usr/bin/env python3
"""Resolve the effective Wendao document extraction endpoint from TOML config."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Any
from urllib.parse import urlparse

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from resolve_wendao_gateway_port import (  # noqa: E402
    _load_toml_with_imports,
    _resolve_effective_config_path,
)

DEFAULT_ENDPOINT = "http://127.0.0.1:50051"


def _normalize_endpoint(value: object) -> str | None:
    if not isinstance(value, str):
        return None
    endpoint = value.strip().rstrip("/")
    if not endpoint:
        return None
    parsed = urlparse(endpoint)
    if parsed.scheme not in {"http", "https"} or not parsed.hostname:
        raise ValueError(f"invalid document_extract.endpoint: {value!r}")
    return endpoint


def _document_extract_section(document: dict[str, Any]) -> dict[str, Any]:
    section = document.get("document_extract", {})
    if isinstance(section, dict):
        return section
    return {}


def resolve_document_extract_endpoint(config_path: Path) -> str:
    effective_path = _resolve_effective_config_path(config_path)
    document = _load_toml_with_imports(effective_path)
    endpoint = _normalize_endpoint(_document_extract_section(document).get("endpoint"))
    return endpoint or DEFAULT_ENDPOINT


def document_extract_endpoint_host(endpoint: str) -> str:
    parsed = urlparse(endpoint)
    return parsed.hostname or "127.0.0.1"


def document_extract_endpoint_port(endpoint: str) -> int:
    parsed = urlparse(endpoint)
    if parsed.port is not None:
        return parsed.port
    if parsed.scheme == "https":
        return 443
    return 80


def _default_config_path() -> Path:
    project_root = Path.cwd()
    return project_root / "wendao.toml"


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Resolve the effective Wendao document extraction endpoint"
    )
    parser.add_argument(
        "--config",
        type=Path,
        default=_default_config_path(),
        help="Path to the base Wendao TOML config",
    )
    parser.add_argument(
        "--field",
        choices=("endpoint", "host", "port"),
        default="endpoint",
        help="Resolved endpoint field to print",
    )
    args = parser.parse_args()
    endpoint = resolve_document_extract_endpoint(Path(args.config))
    if args.field == "host":
        print(document_extract_endpoint_host(endpoint), end="")
    elif args.field == "port":
        print(document_extract_endpoint_port(endpoint), end="")
    else:
        print(endpoint, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
