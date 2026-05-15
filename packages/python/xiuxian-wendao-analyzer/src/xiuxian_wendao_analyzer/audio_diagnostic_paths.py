"""Audio diagnostic source discovery and evidence path helpers."""

from __future__ import annotations

from datetime import UTC, datetime
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping
    from pathlib import Path

from xiuxian_wendao_analyzer.audio_diagnostic_identity import SUPPORTED_AUDIO_SUFFIXES

DEFAULT_OUTPUT_DIR = "audio_asr_diagnostic"
PRIVATE_INPUT_PRIVACY = "private-local"
SHAREABLE_INPUT_PRIVACY = "shareable"


def discover_audio_sources(source_root: Path, *, limit_files: int | None) -> list[Path]:
    """Return supported audio files below ``source_root`` in deterministic order."""

    if source_root.is_file():
        candidates = [source_root]
    else:
        candidates = [
            path
            for path in source_root.rglob("*")
            if path.is_file() and path.suffix.lower() in SUPPORTED_AUDIO_SUFFIXES
        ]
    sources = sorted(candidates, key=lambda path: str(path).lower())
    if limit_files is not None:
        return sources[:limit_files]
    return sources


def resolve_repo_root(start: Path) -> Path:
    """Find the nearest git repository root for default evidence placement."""

    for candidate in [start, *start.parents]:
        if (candidate / ".git").exists():
            return candidate
    return start


def default_output_dir(start: Path) -> Path:
    """Return a timestamped diagnostic output directory below the repo cache."""

    stamp = datetime.now(tz=UTC).strftime("%Y%m%dT%H%M%SZ")
    return evidence_root(start) / DEFAULT_OUTPUT_DIR / stamp


def evidence_root(start: Path) -> Path:
    """Return the cache-local evidence root for private diagnostics."""

    return resolve_repo_root(start) / ".cache" / "agent" / "evidence"


def is_relative_to(child: Path, parent: Path) -> bool:
    """Return whether ``child`` is within ``parent`` after path resolution."""

    try:
        child.resolve().relative_to(parent.resolve())
    except ValueError:
        return False
    return True


def validate_private_output_dir(
    output_dir: Path,
    *,
    start: Path,
    input_privacy: str,
    allow_private_output_outside_cache: bool,
) -> None:
    """Guard private diagnostic transcripts from being written to repo fixtures."""

    if input_privacy != PRIVATE_INPUT_PRIVACY:
        return
    if is_relative_to(output_dir, evidence_root(start)):
        return
    if allow_private_output_outside_cache:
        return
    raise ValueError(
        "private audio diagnostics must write under .cache/agent/evidence; "
        "pass --allow-private-output-outside-cache only for local scratch paths "
        "that will not be committed"
    )


def read_env_file(path: Path) -> dict[str, str]:
    """Read a simple dotenv file without adding a runtime dependency."""

    if not path.exists():
        return {}
    values: dict[str, str] = {}
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        key = key.strip()
        value = normalize_env_value(value)
        if key:
            values[key] = value
    return values


def normalize_env_value(value: str) -> str:
    """Normalize simple dotenv/env values used by the diagnostic."""

    return value.strip().strip("\"'")


def resolve_openrouter_api_key(
    env: Mapping[str, str], *, env_file: Path | None
) -> str | None:
    """Return the standard OpenRouter key from env or dotenv."""

    if env.get("OPENROUTER_API_KEY"):
        return normalize_env_value(env["OPENROUTER_API_KEY"])
    if env_file is None:
        return None
    return read_env_file(env_file).get("OPENROUTER_API_KEY")
