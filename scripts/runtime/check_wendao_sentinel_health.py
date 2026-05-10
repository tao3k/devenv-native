#!/usr/bin/env python3
"""Check Wendao sentinel readiness from process liveness and effective watch roots."""

from __future__ import annotations

import argparse
import os
import re
import sys
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

DEFAULT_SENTINEL_WATCH_DIRS = ("docs", "src")
_ENV_PATTERN = re.compile(r"\$(\w+)|\$\{([^}]+)\}")


def _normalize_path_like(raw: object) -> str | None:
    if not isinstance(raw, str):
        return None
    normalized = raw.strip().replace("\\", "/")
    if not normalized:
        return None
    while "//" in normalized:
        normalized = normalized.replace("//", "/")
    while len(normalized) > 1 and normalized.endswith("/"):
        normalized = normalized[:-1]
    return normalized or None


def _normalize_path_buf_like(path: Path) -> Path | None:
    normalized = Path(path.anchor) if path.is_absolute() else Path()
    for part in path.parts[1:] if path.is_absolute() else path.parts:
        if part in {"", "."}:
            continue
        if part == "..":
            normalized = normalized.parent
            continue
        normalized /= part
    if str(normalized) in {"", "."}:
        return None
    return normalized


def _expand_home_path_like(input_path: str) -> Path:
    if input_path == "~":
        return Path.home()
    if input_path.startswith("~/"):
        return Path.home() / input_path[2:]
    return Path(input_path)


def _resolve_path_like(base: Path, input_path: str) -> Path | None:
    normalized = _normalize_path_like(input_path)
    if normalized is None:
        return None
    expanded = _expand_home_path_like(normalized)
    joined = expanded if expanded.is_absolute() else base / expanded
    return _normalize_path_buf_like(joined)


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


def resolve_sentinel_watch_paths(project_root: Path, config_path: Path) -> list[Path]:
    effective_path = _resolve_effective_config_path(config_path)
    document = _load_toml_with_imports(effective_path)
    link_graph = document.get("link_graph", {})
    projects = link_graph.get("projects", {}) if isinstance(link_graph, dict) else {}

    if not isinstance(projects, dict):
        projects = {}

    config_root = effective_path.parent
    resolved_paths: list[Path] = []
    seen = set()

    for raw_project in projects.values():
        if not isinstance(raw_project, dict):
            continue
        project_root_raw = _normalize_path_like(raw_project.get("root")) or "."
        project_base = _resolve_path_like(config_root, project_root_raw)
        if project_base is None:
            continue
        raw_dirs = raw_project.get("dirs", [])
        if not isinstance(raw_dirs, list):
            continue
        for raw_dir in raw_dirs:
            normalized_dir = _normalize_path_like(raw_dir)
            if normalized_dir is None:
                continue
            candidate = _resolve_path_like(project_base, normalized_dir)
            if candidate is None:
                continue
            try:
                candidate.resolve().relative_to(project_root.resolve())
            except ValueError:
                continue
            candidate_key = str(candidate.resolve())
            if candidate_key in seen:
                continue
            seen.add(candidate_key)
            resolved_paths.append(candidate)

    if resolved_paths:
        return resolved_paths

    return [project_root / path for path in DEFAULT_SENTINEL_WATCH_DIRS]


def read_expected_pid(pidfile: Path) -> int:
    contents = pidfile.read_text(encoding="utf-8").strip()
    if not contents:
        raise ValueError(f"pidfile is empty: {pidfile}")
    try:
        return int(contents)
    except ValueError as error:
        raise ValueError(
            f"pidfile does not contain a valid process id: {pidfile}"
        ) from error


def _pid_is_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except OSError:
        return False
    return True


def is_sentinel_healthy(
    *,
    project_root: Path,
    config_path: Path,
    pidfile: Path,
    pid_checker=_pid_is_alive,
) -> tuple[bool, str]:
    try:
        expected_pid = read_expected_pid(pidfile)
    except OSError as error:
        return False, f"failed to read pidfile {pidfile}: {error}"
    except ValueError as error:
        return False, str(error)

    if not pid_checker(expected_pid):
        return False, f"sentinel process is not alive: {expected_pid}"

    try:
        watch_paths = resolve_sentinel_watch_paths(project_root, config_path)
    except (OSError, ValueError, tomllib.TOMLDecodeError) as error:
        return False, f"failed to resolve sentinel watch paths: {error}"

    existing_paths = [path for path in watch_paths if path.is_dir()]
    if not existing_paths:
        rendered = ", ".join(str(path) for path in watch_paths)
        return False, f"sentinel has no existing watch roots: {rendered}"

    return True, "healthy"


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Check Wendao sentinel readiness against pid and watch roots"
    )
    parser.add_argument(
        "--project-root",
        type=Path,
        required=True,
        help="Project root used to scope effective watch roots",
    )
    parser.add_argument(
        "--config",
        type=Path,
        required=True,
        help="Path to the base Wendao TOML config",
    )
    parser.add_argument(
        "--pidfile",
        type=Path,
        required=True,
        help="Pidfile written by the Wendao sentinel launcher",
    )
    args = parser.parse_args()

    healthy, message = is_sentinel_healthy(
        project_root=args.project_root,
        config_path=args.config,
        pidfile=args.pidfile,
    )
    if healthy:
        print(message)
        return 0
    print(f"Error: {message}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
