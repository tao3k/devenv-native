#!/usr/bin/env python3
"""Unit tests for complex scenario config helpers."""

from __future__ import annotations

import importlib
import sys
from pathlib import Path

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

module = importlib.import_module("complex_scenarios_config")
endpoints = importlib.import_module("channel_test_endpoints")


def test_env_int_handles_unset_and_blank(monkeypatch) -> None:
    monkeypatch.delenv("OMNI_TEST_CHAT_ID", raising=False)
    assert module.env_int("OMNI_TEST_CHAT_ID") is None

    monkeypatch.setenv("OMNI_TEST_CHAT_ID", "  ")
    assert module.env_int("OMNI_TEST_CHAT_ID") is None

    monkeypatch.setenv("OMNI_TEST_CHAT_ID", "123")
    assert module.env_int("OMNI_TEST_CHAT_ID") == 123


def test_parse_args_uses_injected_env_defaults(monkeypatch, tmp_path: Path) -> None:
    values = {
        "OMNI_TEST_CHAT_ID": 11,
        "OMNI_TEST_CHAT_B": 12,
        "OMNI_TEST_CHAT_C": 13,
        "OMNI_TEST_USER_ID": 21,
        "OMNI_TEST_USER_B": 22,
        "OMNI_TEST_USER_C": 23,
        "OMNI_TEST_THREAD_ID": 31,
        "OMNI_TEST_THREAD_B": 32,
        "OMNI_TEST_THREAD_C": 33,
    }
    monkeypatch.setattr(sys, "argv", ["complex_scenarios_config.py"])

    args = module.parse_args(
        script_dir=tmp_path,
        webhook_url_default=endpoints.http_url(80, "/webhook"),
        default_log_file="runtime.log",
        default_max_wait=30,
        default_max_idle_secs=20,
        env_int_fn=lambda name: values.get(name),
    )

    assert args.chat_a == 11
    assert args.chat_b == 12
    assert args.chat_c == 13
    assert args.user_a == 21
    assert args.thread_c == 33
    assert args.dataset == str(
        tmp_path / "fixtures" / "complex_blackbox_scenarios.json"
    )
    assert args.blackbox_script == str(tmp_path / "agent_channel_blackbox.py")
    assert args.max_wait == 30
    assert args.max_idle_secs == 20
