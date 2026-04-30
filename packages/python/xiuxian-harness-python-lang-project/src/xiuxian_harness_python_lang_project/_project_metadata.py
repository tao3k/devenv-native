"""Minimal pyproject metadata reader for Python project policy."""

from __future__ import annotations

import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True, slots=True)
class PythonProjectMetadata:
    """Compact project metadata needed by harness policy rules."""

    project_root: Path
    pyproject_path: Path
    project_name: str | None
    requires_python: str | None
    build_backend: str | None
    wheel_packages: tuple[str, ...]
    package_roots: tuple[Path, ...]


def read_python_project_metadata(
    project_root: str | Path,
) -> PythonProjectMetadata | None:
    """Read minimal project metadata from `pyproject.toml` when present."""

    root = Path(project_root)
    pyproject_path = root / "pyproject.toml"
    if not pyproject_path.exists():
        return None

    try:
        payload = tomllib.loads(pyproject_path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError, UnicodeDecodeError):
        return None

    project = _table(payload.get("project"))
    build_system = _table(payload.get("build-system"))
    wheel = _table(
        _table(_table(_table(payload.get("tool")).get("hatch")).get("build"))
        .get("targets")
        .get("wheel")
    )
    wheel_packages = _string_tuple(wheel.get("packages"))

    return PythonProjectMetadata(
        project_root=root,
        pyproject_path=pyproject_path,
        project_name=_string_or_none(project.get("name")),
        requires_python=_string_or_none(project.get("requires-python")),
        build_backend=_string_or_none(build_system.get("build-backend")),
        wheel_packages=wheel_packages,
        package_roots=tuple(
            _resolve_package_path(root, item) for item in wheel_packages
        ),
    )


def _table(value: object) -> dict[str, Any]:
    if isinstance(value, dict):
        return value
    return {}


def _string_or_none(value: object) -> str | None:
    if isinstance(value, str):
        return value
    return None


def _string_tuple(value: object) -> tuple[str, ...]:
    if not isinstance(value, list):
        return ()
    seen: set[str] = set()
    values: list[str] = []
    for item in value:
        if not isinstance(item, str):
            continue
        package_path = Path(item).as_posix()
        if package_path in seen:
            continue
        seen.add(package_path)
        values.append(package_path)
    return tuple(values)


def _resolve_package_path(project_root: Path, package_path: str) -> Path:
    path = Path(package_path)
    if path.is_absolute():
        return path
    return project_root / path
