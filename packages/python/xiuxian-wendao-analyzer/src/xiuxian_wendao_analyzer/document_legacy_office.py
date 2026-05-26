"""Legacy Office source preparation for Docling-backed extraction."""

from __future__ import annotations

import shutil
import subprocess
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable
    from pathlib import Path

LEGACY_OFFICE_CONVERTER_ENV = "WENDAO_DOCUMENT_EXTRACT_LEGACY_OFFICE_CONVERTER"
LEGACY_OFFICE_PREPARATION_MODE = "legacy-office-docx"

_LEGACY_OFFICE_TARGET_SUFFIX = {
    ".doc": ".docx",
}


def prepare_docling_source(
    source: Path,
    output_dir: Path,
    *,
    mode: str,
    env_lookup: Callable[[str], str | None] | None = None,
) -> Path:
    """Return a Docling-readable source path for one input file.

    # Errors

    Raises `RuntimeError` when a legacy Office input requires conversion and no
    converter is available or when conversion fails.
    """

    normalized_mode = mode.strip().lower().replace("_", "-")
    if normalized_mode != LEGACY_OFFICE_PREPARATION_MODE:
        raise ValueError(f"unsupported legacy Office preparation mode `{mode}`")
    target_suffix = _LEGACY_OFFICE_TARGET_SUFFIX.get(source.suffix.lower())
    if target_suffix is None:
        raise ValueError(
            f"legacy Office preparation mode `{mode}` does not support {source.suffix}"
        )
    target = _legacy_office_target_path(source, output_dir, target_suffix)
    if target.exists() and target.stat().st_mtime >= source.stat().st_mtime:
        return target
    target.parent.mkdir(parents=True, exist_ok=True)
    _convert_legacy_doc_with_textutil(source, target, env_lookup=env_lookup)
    if not target.exists() or target.stat().st_size == 0:
        raise RuntimeError(f"legacy Office conversion wrote no output for {source.name}")
    return target


def _legacy_office_target_path(source: Path, output_dir: Path, suffix: str) -> Path:
    from .document_cache import _file_sha256

    digest = _file_sha256(source)[:16]
    return output_dir / "_legacy_office" / f"{source.stem}-{digest}{suffix}"


def _convert_legacy_doc_with_textutil(
    source: Path,
    target: Path,
    *,
    env_lookup: Callable[[str], str | None] | None,
) -> None:
    converter = _legacy_office_converter_command(env_lookup)
    command = [converter, "-convert", "docx", "-output", str(target), str(source)]
    result = subprocess.run(
        command,
        check=False,
        capture_output=True,
        text=True,
        timeout=_legacy_office_converter_timeout_seconds(env_lookup),
    )
    if result.returncode != 0:
        detail = (result.stderr or result.stdout or "").strip()
        raise RuntimeError(
            "legacy Office conversion failed for "
            f"{source.name}: {detail or f'exit code {result.returncode}'}"
        )


def _legacy_office_converter_command(
    env_lookup: Callable[[str], str | None] | None = None,
) -> str:
    import os

    lookup = env_lookup or os.environ.get
    configured = (lookup(LEGACY_OFFICE_CONVERTER_ENV) or "").strip()
    if configured:
        return configured
    discovered = shutil.which("textutil")
    if discovered:
        return discovered
    raise RuntimeError(
        f"legacy Office .doc conversion requires textutil or {LEGACY_OFFICE_CONVERTER_ENV}"
    )


def _legacy_office_converter_timeout_seconds(
    env_lookup: Callable[[str], str | None] | None = None,
) -> float:
    import os

    lookup = env_lookup or os.environ.get
    value = (lookup("WENDAO_DOCUMENT_EXTRACT_LEGACY_OFFICE_TIMEOUT_SECONDS") or "").strip()
    if not value:
        return 120.0
    try:
        timeout = float(value)
    except ValueError as exc:
        raise ValueError(
            "WENDAO_DOCUMENT_EXTRACT_LEGACY_OFFICE_TIMEOUT_SECONDS must be positive"
        ) from exc
    if timeout <= 0:
        raise ValueError("WENDAO_DOCUMENT_EXTRACT_LEGACY_OFFICE_TIMEOUT_SECONDS must be positive")
    return timeout


__all__ = [
    "LEGACY_OFFICE_CONVERTER_ENV",
    "LEGACY_OFFICE_PREPARATION_MODE",
    "prepare_docling_source",
]
