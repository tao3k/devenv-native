#!/usr/bin/env python3
"""Unit tests for session matrix config helpers."""

from __future__ import annotations

import argparse
import importlib
import sys
from pathlib import Path

import pytest

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

config_module = importlib.import_module("session_matrix_config")
models_module = importlib.import_module("session_matrix_models")
endpoints = importlib.import_module("channel_test_endpoints")


def test_session_context_result_fields_uses_expected_key() -> None:
    fields = config_module.session_context_result_fields(
        1,
        2,
        None,
        "chat_user",
        expected_session_key_fn=lambda *_args: "1:2",
    )
    assert fields[0] == "json_kind=session_context"
    assert "json_partition_key=1:2" in fields


def test_build_config_rejects_non_positive_wait() -> None:
    args = argparse.Namespace(
        max_wait=0,
        max_idle_secs=10,
        log_file=".run/log.log",
        chat_id=1,
        chat_b=1,
        chat_c=1,
        user_a=2,
        user_b=3,
        user_c=4,
        thread_a=None,
        thread_b=None,
        thread_c=None,
        username="u",
        mixed_plain_prompt="p",
        secret_token=None,
        output_json=".run/a.json",
        output_markdown=".run/a.md",
        forbid_log_regex=[],
        webhook_url=endpoints.http_url(8080, "/webhook/telegram"),
    )

    with pytest.raises(ValueError, match="--max-wait"):
        config_module.build_config(
            args,
            config_cls=models_module.ProbeConfig,
            resolve_runtime_partition_mode_fn=lambda _log: None,
            group_profile_int_fn=lambda _key: None,
            session_ids_from_runtime_log_fn=lambda _log: (None, None, None),
            username_from_settings_fn=lambda: None,
            username_from_runtime_log_fn=lambda _log: None,
            expected_session_key_fn=lambda *_args: "x",
        )
