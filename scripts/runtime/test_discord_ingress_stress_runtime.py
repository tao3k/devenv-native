#!/usr/bin/env python3
"""Unit tests for Discord ingress stress runtime helpers."""

from __future__ import annotations

import importlib
import json
import sys
from pathlib import Path
from types import SimpleNamespace

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

module = importlib.import_module("discord_ingress_stress_runtime")


def test_p95_handles_empty_and_ordering() -> None:
    assert module.p95([]) == 0.0
    assert module.p95([10.0, 30.0, 20.0, 50.0, 40.0]) == 40.0


def test_log_offset_and_incremental_reads(tmp_path: Path) -> None:
    log_path = tmp_path / "runtime.log"
    offset = module.init_log_offset(log_path)
    assert offset == 0

    log_path.write_text("a\nb\n", encoding="utf-8")
    next_offset, lines = module.read_new_log_lines(log_path, offset)
    assert lines == ["a", "b"]

    next_offset_2, lines_2 = module.read_new_log_lines(log_path, next_offset)
    assert next_offset_2 == next_offset
    assert lines_2 == []


def test_build_ingress_payload_includes_guild_member_fields() -> None:
    cfg = SimpleNamespace(
        channel_id="2001",
        user_id="1001",
        guild_id="3001",
        username="alice",
        role_ids=("r1", "r2"),
    )

    raw = module.build_ingress_payload(cfg, event_id="9", prompt="hello")
    payload = json.loads(raw.decode("utf-8"))

    assert payload["id"] == "9"
    assert payload["channel_id"] == "2001"
    assert payload["guild_id"] == "3001"
    assert payload["author"]["id"] == "1001"
    assert payload["author"]["username"] == "alice"
    assert payload["member"]["roles"] == ["r1", "r2"]
