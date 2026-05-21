"""Path resolution for Wendao Episteme source-contract artifacts."""

from __future__ import annotations

import os
from pathlib import Path


def episteme_root() -> Path:
    """Return the Wendao Episteme repository root.

    The compiler package lives in the parent workspace, while source-contract
    artifacts live in the user-facing `wendao-episteme` repository. Resolution
    therefore starts from an explicit environment override, then falls back to
    the current working directory and its parents.
    """

    configured = os.environ.get("WENDAO_EPISTEME_ROOT")
    if configured:
        return Path(configured).expanduser().resolve()

    cwd = Path.cwd().resolve()
    candidates = [cwd, *cwd.parents]
    for candidate in candidates:
        if (candidate / "ontology" / "manifest.toml").is_file():
            return candidate
        nested = candidate / "wendao-episteme"
        if (nested / "ontology" / "manifest.toml").is_file():
            return nested

    raise RuntimeError(
        "could not locate wendao-episteme; set WENDAO_EPISTEME_ROOT "
        "or run from the episteme repository"
    )


def ontology_root() -> Path:
    """Return the ontology source-contract artifact root."""

    return episteme_root() / "ontology"


def cache_root() -> Path:
    """Return the project cache root used by episteme alignment fetches."""

    configured = os.environ.get("PRJ_CACHE_HOME", ".cache")
    path = Path(configured)
    if path.is_absolute():
        return path
    return episteme_root().parent / path


__all__ = ["cache_root", "episteme_root", "ontology_root"]
