#!/usr/bin/env python3
"""Unit tests for command-events runtime context helpers."""

from __future__ import annotations

import importlib
import sys
from pathlib import Path

import pytest

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

module = importlib.import_module("command_events_runtime_context")


def test_parse_optional_int_env_parses_and_validates(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv("X_TEST", "42")
    assert module.parse_optional_int_env("X_TEST") == 42

    monkeypatch.setenv("X_TEST", "abc")
    with pytest.raises(ValueError, match="must be an integer"):
        module.parse_optional_int_env("X_TEST")


def test_resolve_topic_thread_pair_requires_distinct() -> None:
    assert (
        module.resolve_topic_thread_pair(
            primary_thread_id=None, secondary_thread_id=None
        )
        is None
    )
    assert module.resolve_topic_thread_pair(
        primary_thread_id=10, secondary_thread_id=None
    ) == (
        10,
        11,
    )
    with pytest.raises(ValueError, match="distinct thread ids"):
        module.resolve_topic_thread_pair(primary_thread_id=10, secondary_thread_id=10)


def test_resolve_allow_chat_ids_prefers_cli(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("OMNI_BLACKBOX_ALLOWED_CHAT_IDS", "-200,-201")
    result = module.resolve_allow_chat_ids(
        ("123",), group_profile_chat_ids_fn=lambda: [-100]
    )
    assert result == ("123",)
