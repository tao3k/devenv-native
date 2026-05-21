"""
Schema path resolver for Python-side contract validation modules.

The resolver searches Rust crate `resources/` directories in deterministic order.
"""

from __future__ import annotations

from functools import lru_cache
from pathlib import Path


def _project_roots() -> tuple[Path, ...]:
    roots: list[Path] = []
    seen: set[str] = set()

    try:
        from xiuxian_foundation.config.prj import get_project_root

        root = get_project_root()
        key = str(root)
        if key not in seen:
            seen.add(key)
            roots.append(root)
    except Exception:
        pass

    if roots:
        return tuple(roots)
    return (Path.cwd(),)


def _resource_dirs_for_root(
    root: Path, preferred_crates: tuple[str, ...]
) -> list[Path]:
    out: list[Path] = []
    seen: set[str] = set()
    crates_root = root / "packages" / "rust" / "crates"

    def _append(path: Path) -> None:
        key = str(path)
        if key in seen:
            return
        seen.add(key)
        out.append(path)

    for crate in preferred_crates:
        _append(crates_root / crate / "resources")

    if crates_root.exists():
        for path in sorted(crates_root.glob("*/resources")):
            _append(path)

    return out


@lru_cache(maxsize=512)
def resolve_schema_file_path(
    schema_name: str,
    *,
    preferred_crates: tuple[str, ...] = (),
) -> Path:
    """Resolve schema file path from Rust crate resources."""
    name = Path(str(schema_name)).name
    if not name:
        raise ValueError("schema_name must not be empty")

    roots = _project_roots()
    for root in roots:
        for directory in _resource_dirs_for_root(root, preferred_crates):
            candidate = directory / name
            if candidate.exists():
                return candidate
            for nested in sorted(directory.glob(f"*/{name}")):
                if nested.exists():
                    return nested

    # Deterministic not-found path for clear error messages.
    root = roots[0]
    if preferred_crates:
        return (
            root
            / "packages"
            / "rust"
            / "crates"
            / preferred_crates[0]
            / "resources"
            / name
        )
    return root / "packages" / "rust" / "crates" / "xiuxian-wendao" / "resources" / name


__all__ = ["resolve_schema_file_path"]
