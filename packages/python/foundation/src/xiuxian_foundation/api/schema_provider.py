"""Centralized schema provider backed by Rust resource files."""

from __future__ import annotations

import json
from functools import lru_cache
from pathlib import Path
from typing import Any

from .schema_locator import resolve_schema_file_path


def _schema_filename(name: str) -> str:
    raw = str(name).strip()
    if not raw:
        raise ValueError("schema name must not be empty")
    return raw if raw.endswith(".json") else f"{raw}.schema.json"


def _preferred_crates(name: str) -> tuple[str, ...]:
    raw = str(name).strip()
    if raw.startswith("xiuxian_wendao.link_graph."):
        return ("xiuxian-wendao",)
    if (
        raw.startswith("xiuxian.runtime.")
        or raw.startswith("xiuxian.router.")
        or raw.startswith("xiuxian.discover.")
    ):
        return ("xiuxian-wendao",)
    if raw.startswith("xiuxian.memory."):
        return ("xiuxian-memory-engine", "xiuxian-wendao")
    if raw.startswith("xiuxian.skill."):
        return ("xiuxian-wendao",)
    return ("xiuxian-wendao",)


@lru_cache(maxsize=None)
def get_schema(name: str) -> dict[str, Any]:
    """
    Load a schema by name from Rust crate resource files.

    Args:
        name: The canonical schema identifier (e.g., 'xiuxian_wendao.link_graph.record.v1')

    Returns:
        The parsed JSON schema as a dictionary.

    Raises:
        ValueError: If the schema name is empty or the payload is invalid JSON.
        FileNotFoundError: If the schema file cannot be resolved from Rust resources.
    """
    schema_name = _schema_filename(name)
    schema_path = resolve_schema_file_path(
        schema_name,
        preferred_crates=_preferred_crates(name),
    )
    if not schema_path.exists():
        raise FileNotFoundError(f"Unknown schema identifier: {name} ({schema_path})")
    return json.loads(Path(schema_path).read_text(encoding="utf-8"))


def get_schema_id(name: str) -> str:
    """Return the $id field from a schema."""
    schema = get_schema(name)
    return str(schema.get("$id", "")).strip()
