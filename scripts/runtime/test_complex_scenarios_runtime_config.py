#!/usr/bin/env python3
"""Unit tests for complex scenario runtime config helpers."""

from __future__ import annotations

import importlib
import sys
from dataclasses import dataclass
from pathlib import Path

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

module = importlib.import_module("complex_scenarios_runtime_config")


@dataclass(frozen=True)
class Session:
    alias: str
    thread_id: int | None


def test_parse_numeric_user_ids_deduplicates_and_ignores_invalid() -> None:
    assert module.parse_numeric_user_ids(["1", " 2 ", "abc", "1", "-3"]) == [1, 2, -3]


def test_apply_runtime_partition_defaults_sets_zero_thread() -> None:
    sessions = (
        Session(alias="a", thread_id=None),
        Session(alias="b", thread_id=42),
    )
    updated = module.apply_runtime_partition_defaults(sessions, "chat_thread_user")
    assert updated[0].thread_id == 0
    assert updated[1].thread_id == 42


def test_resolve_runtime_partition_mode_prefers_override(tmp_path: Path) -> None:
    mode = module.resolve_runtime_partition_mode(
        tmp_path / "runtime.log",
        env_get_fn=lambda _key, _default="": "chat-user",
        normalize_partition_fn=lambda value: "chat_user" if value else None,
        partition_mode_from_runtime_log_fn=lambda _path: "user",
        partition_mode_from_settings_fn=lambda: "chat_thread_user",
    )
    assert mode == "chat_user"
