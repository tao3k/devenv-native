#!/usr/bin/env python3
"""Resolve the effective Wendao gateway port from TOML config."""

from __future__ import annotations

import argparse
import os
import re
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

DEFAULT_PORT = 9517
_ENV_PATTERN = re.compile(r"\$(\w+)|\$\{([^}]+)\}")


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


def _parse_bind_port(value: object) -> int | None:
    if not isinstance(value, str):
        return None
    text = value.strip()
    if not text:
        return None
    direct = _normalize_port(text)
    if direct is not None:
        return direct
    _, _, port_text = text.rpartition(":")
    return _normalize_port(port_text)


def _merge_toml_values(base: object, overlay: object) -> object:
    if isinstance(base, dict) and isinstance(overlay, dict):
        merged = dict(base)
        for key, value in overlay.items():
            if key in merged:
                merged[key] = _merge_toml_values(merged[key], value)
            else:
                merged[key] = value
        return merged
    return overlay


def _expand_import_path(raw_path: str, environment: dict[str, str]) -> str:
    def replacer(match: re.Match[str]) -> str:
        variable_name = match.group(1) or match.group(2) or ""
        if variable_name not in environment:
            raise ValueError(
                f"unresolved environment variable in import path: {variable_name}"
            )
        return environment[variable_name]

    return _ENV_PATTERN.sub(replacer, raw_path)


def _resolve_import_path(config_path: Path, raw_import: object) -> Path:
    if not isinstance(raw_import, str):
        raise ValueError("imports entries must be strings")
    expanded = _expand_import_path(raw_import, dict(os.environ))
    import_path = Path(expanded)
    if import_path.is_absolute():
        return import_path
    return config_path.parent / import_path


def _load_toml_document(path: Path) -> dict[str, Any]:
    with path.open("rb") as handle:
        loaded = tomllib.load(handle)
    if not isinstance(loaded, dict):
        raise ValueError(f"TOML document must decode to a table: {path}")
    return loaded


def _resolve_effective_config_path(config_path: Path) -> Path:
    if config_path.name != "wendao.toml":
        return config_path
    overlay_path = config_path.with_name("wendao.studio.overlay.toml")
    if overlay_path.is_file():
        return overlay_path
    return config_path


def _load_toml_with_imports(
    config_path: Path, stack: tuple[Path, ...] = ()
) -> dict[str, Any]:
    resolved_path = config_path.resolve()
    if resolved_path in stack:
        cycle = " -> ".join(str(path) for path in (*stack, resolved_path))
        raise ValueError(f"cyclic config imports detected: {cycle}")

    document = _load_toml_document(resolved_path)
    imports = document.get("imports", [])
    if imports is None:
        imports = []
    if not isinstance(imports, list):
        raise ValueError(f"imports must be a list in {resolved_path}")

    merged: object = {}
    next_stack = (*stack, resolved_path)
    for raw_import in imports:
        import_path = _resolve_import_path(resolved_path, raw_import)
        imported = _load_toml_with_imports(import_path, next_stack)
        merged = _merge_toml_values(merged, imported)

    local_document = {key: value for key, value in document.items() if key != "imports"}
    merged = _merge_toml_values(merged, local_document)
    if not isinstance(merged, dict):
        raise ValueError(f"merged config must decode to a table: {resolved_path}")
    return merged


def resolve_gateway_port(config_path: Path) -> int:
    effective_path = _resolve_effective_config_path(config_path)
    document = _load_toml_with_imports(effective_path)
    gateway = document.get("gateway", {})
    if not isinstance(gateway, dict):
        return DEFAULT_PORT
    configured_port = _normalize_port(gateway.get("port"))
    if configured_port is not None:
        return configured_port
    return _parse_bind_port(gateway.get("bind")) or DEFAULT_PORT


def _default_config_path() -> Path:
    project_root = Path(os.environ.get("PRJ_ROOT", Path.cwd()))
    return project_root / "wendao.toml"


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Resolve the effective Wendao gateway port"
    )
    parser.add_argument(
        "--config",
        type=Path,
        default=_default_config_path(),
        help="Path to the base Wendao TOML config",
    )
    args = parser.parse_args()
    print(resolve_gateway_port(Path(args.config)), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
