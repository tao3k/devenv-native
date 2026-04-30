"""Python project path discovery for embedded harness runs."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

from ._constants import IGNORED_DIR_NAMES
from ._model import PythonProjectHarnessScope

if TYPE_CHECKING:
    from collections.abc import Iterable, Sequence


def discover_python_files(
    paths: Sequence[str | Path],
    *,
    ignored_dir_names: Iterable[str] | None = None,
) -> tuple[Path, ...]:
    """Discover Python files below the provided paths."""

    ignored_names = (
        IGNORED_DIR_NAMES if ignored_dir_names is None else frozenset(ignored_dir_names)
    )
    discovered: list[Path] = []
    for raw_path in paths:
        path = Path(raw_path)
        if path.is_file():
            if path.suffix == ".py":
                discovered.append(path)
            continue
        if path.is_dir():
            discovered.extend(
                candidate
                for candidate in path.rglob("*.py")
                if is_scannable_python_file(candidate, ignored_dir_names=ignored_names)
            )
    return tuple(sorted(discovered, key=lambda item: item.as_posix()))


def python_project_harness_paths(
    project_root: str | Path,
    *,
    include_tests: bool = True,
    source_dir_names: Sequence[str] = ("src",),
    test_dir_names: Sequence[str] = ("tests",),
) -> tuple[Path, ...]:
    """Return conventional project paths for embedded pytest harness checks."""

    return python_project_harness_scope(
        project_root,
        include_tests=include_tests,
        source_dir_names=source_dir_names,
        test_dir_names=test_dir_names,
    ).monitored_paths


def python_project_harness_scope(
    project_root: str | Path,
    *,
    include_tests: bool = True,
    source_dir_names: Sequence[str] = ("src",),
    test_dir_names: Sequence[str] = ("tests",),
) -> PythonProjectHarnessScope:
    """Return the default project monitoring scope for `src/**` and `tests/**`."""

    root = Path(project_root)
    source_paths = tuple(
        root / name for name in source_dir_names if (root / name).exists()
    )
    test_paths = (
        tuple(root / name for name in test_dir_names if (root / name).exists())
        if include_tests
        else ()
    )
    fallback_paths = _fallback_project_paths(
        root,
        source_paths=source_paths,
        test_paths=test_paths,
        include_tests=include_tests,
        test_dir_names=test_dir_names,
    )
    return PythonProjectHarnessScope(
        project_root=root,
        source_paths=source_paths,
        test_paths=test_paths,
        fallback_paths=fallback_paths,
    )


def _fallback_project_paths(
    root: Path,
    *,
    source_paths: tuple[Path, ...],
    test_paths: tuple[Path, ...],
    include_tests: bool,
    test_dir_names: Sequence[str],
) -> tuple[Path, ...]:
    if source_paths or test_paths:
        return ()
    if include_tests:
        return (root,)

    excluded_test_dir_names = {
        name for name in test_dir_names if (root / name).is_dir()
    }
    if not excluded_test_dir_names:
        return (root,)

    candidates = (
        child
        for child in root.iterdir()
        if child.name not in excluded_test_dir_names
        and (
            child.suffix == ".py"
            or (child.is_dir() and (child / "__init__.py").is_file())
        )
    )
    return tuple(sorted(candidates, key=lambda path: path.as_posix()))


def is_scannable_python_file(
    path: Path,
    *,
    ignored_dir_names: frozenset[str],
) -> bool:
    """Return whether a Python file belongs to the harness-owned scan scope."""

    return not any(part in ignored_dir_names for part in path.parts)
