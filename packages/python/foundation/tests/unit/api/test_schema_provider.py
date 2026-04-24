"""Tests for schema provider resource-backed loading."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

import xiuxian_foundation.api.schema_provider as schema_provider
from xiuxian_foundation.api.schema_provider import get_schema


@pytest.fixture(autouse=True)
def _clear_schema_cache() -> None:
    schema_provider.get_schema.cache_clear()
    yield
    schema_provider.get_schema.cache_clear()


def test_get_schema_loads_from_rust_resource_file() -> None:
    """Schema id should resolve from Rust crate resources."""
    schema = get_schema("xiuxian.runtime.server_info.v1")
    assert schema["type"] == "object"
    assert schema["$id"].endswith("xiuxian.runtime.server_info.v1.schema.json")


def test_get_schema_loads_json_filename_directly(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
):
    """Explicit schema filenames should resolve through the resource locator."""
    resource_dir = (
        tmp_path / "packages" / "rust" / "crates" / "xiuxian-wendao" / "resources"
    )
    resource_dir.mkdir(parents=True)
    resource_file = resource_dir / "custom.schema.v1.schema.json"
    resource_file.write_text(
        json.dumps({"$id": "custom", "type": "object"}), encoding="utf-8"
    )

    monkeypatch.setattr(
        schema_provider,
        "resolve_schema_file_path",
        lambda name, preferred_crates=(): resource_dir / name,
    )

    schema = get_schema("custom.schema.v1.schema.json")
    assert schema["$id"] == "custom"


def test_get_schema_raises_for_unmapped_schema() -> None:
    """Unmapped schema ids should fail without any bindings fallback."""
    with pytest.raises(FileNotFoundError, match="Unknown schema identifier"):
        schema_provider.get_schema("xiuxian.vector.unknown.v1")


def test_vector_schema_ids_do_not_probe_retired_vector_resources(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Retired vector schemas should fall back to Wendao-owned resources only."""
    seen: list[tuple[str, tuple[str, ...]]] = []

    def capture(name: str, preferred_crates: tuple[str, ...] = ()) -> Path:
        seen.append((name, preferred_crates))
        return Path("/missing") / name

    monkeypatch.setattr(schema_provider, "resolve_schema_file_path", capture)

    with pytest.raises(FileNotFoundError):
        schema_provider.get_schema("xiuxian.vector.unknown.v1")

    assert seen == [
        (
            "xiuxian.vector.unknown.v1.schema.json",
            ("xiuxian-wendao",),
        )
    ]


def test_skill_schema_ids_do_not_probe_retired_skills_resources(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Retired skill schemas should resolve through Wendao-owned resources."""
    seen: list[tuple[str, tuple[str, ...]]] = []

    def capture(name: str, preferred_crates: tuple[str, ...] = ()) -> Path:
        seen.append((name, preferred_crates))
        return Path("/missing") / name

    monkeypatch.setattr(schema_provider, "resolve_schema_file_path", capture)

    with pytest.raises(FileNotFoundError):
        schema_provider.get_schema("xiuxian.skill.unknown.v1")

    assert seen == [
        (
            "xiuxian.skill.unknown.v1.schema.json",
            ("xiuxian-wendao",),
        )
    ]
