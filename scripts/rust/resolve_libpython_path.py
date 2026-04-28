#!/usr/bin/env python3
"""Resolve active Python shared library path for xiuxian-core-rs tests."""

from __future__ import annotations

import os
import sys
import sysconfig
from ctypes import util
from pathlib import Path


def resolve_libpython_path() -> str:
    for candidate in candidate_libpython_paths():
        if candidate.is_file():
            return str(candidate)
    return ""


def candidate_libpython_paths() -> list[Path]:
    version = f"{sys.version_info.major}.{sys.version_info.minor}"
    names = configured_library_names(version)
    paths: list[Path] = []

    for libdir_key in ("LIBDIR", "LIBPL"):
        libdir = sysconfig.get_config_var(libdir_key)
        if libdir:
            paths.extend(Path(str(libdir)) / name for name in names)

    framework = sysconfig.get_config_var("PYTHONFRAMEWORK")
    framework_prefix = sysconfig.get_config_var("PYTHONFRAMEWORKPREFIX")
    framework_version = sysconfig.get_config_var("PYTHONFRAMEWORKVERSION") or version
    if framework and framework_prefix:
        paths.append(
            Path(str(framework_prefix))
            / f"{framework}.framework"
            / "Versions"
            / str(framework_version)
            / str(framework)
        )

    base_prefix = Path(sys.base_prefix)
    paths.extend(base_prefix / "lib" / name for name in names)
    executable_prefix = Path(sys.executable).resolve().parent.parent
    paths.extend(executable_prefix / "lib" / name for name in names)

    discovered = util.find_library(f"python{version}")
    if discovered:
        discovered_path = Path(discovered)
        if discovered_path.is_absolute():
            paths.append(discovered_path)
        else:
            paths.extend(Path(search_dir) / discovered for search_dir in library_search_dirs())

    return dedupe_paths(paths)


def configured_library_names(version: str) -> list[str]:
    names = [
        value
        for key in ("LDLIBRARY", "LIBRARY")
        if (value := sysconfig.get_config_var(key))
    ]
    names.extend(
        [
            f"libpython{version}.dylib",
            f"libpython{version}.so",
            f"libpython{version}m.so",
        ]
    )
    return dedupe_strings(str(name) for name in names)


def library_search_dirs() -> list[Path]:
    values = []
    for env_name in ("DYLD_LIBRARY_PATH", "LD_LIBRARY_PATH"):
        values.extend(os.environ.get(env_name, "").split(os.pathsep))
    values.extend(
        str(value)
        for value in (
            sysconfig.get_config_var("LIBDIR"),
            sysconfig.get_config_var("LIBPL"),
            Path(sys.base_prefix) / "lib",
            Path(sys.executable).resolve().parent.parent / "lib",
        )
        if value
    )
    return [Path(value) for value in dedupe_strings(values) if value]


def dedupe_paths(paths: list[Path]) -> list[Path]:
    seen: set[str] = set()
    deduped: list[Path] = []
    for path in paths:
        key = str(path)
        if key in seen:
            continue
        seen.add(key)
        deduped.append(path)
    return deduped


def dedupe_strings(values) -> list[str]:
    seen: set[str] = set()
    deduped: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        deduped.append(value)
    return deduped


def main() -> int:
    path = resolve_libpython_path()
    if not path:
        return 1
    print(path, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
