#!/usr/bin/env python3
"""Unit tests for Discord ACL runtime helpers."""

from __future__ import annotations

import importlib
import sys

_SCRIPT_DIR = __import__("pathlib").Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

runtime_module = importlib.import_module("discord_acl_events_runtime")


def test_expected_session_scopes_uses_prefix() -> None:
    scopes = runtime_module.expected_session_scopes(
        "guild_channel_user",
        "3001",
        "2001",
        "1001",
        session_scope_prefix="discord:",
    )
    assert scopes == ("discord:3001:2001:1001",)


def test_reply_json_field_matches_supports_target_placeholder() -> None:
    matched = runtime_module.reply_json_field_matches(
        key="json_session_scope",
        expected="__target_session_scope__",
        observation={"json_session_scope": "discord:3001:2001:1001"},
        expected_session_scopes_values=("discord:3001:2001:1001",),
        target_session_scope_placeholder="__target_session_scope__",
    )
    assert matched is True
