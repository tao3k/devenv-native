#!/usr/bin/env python3
"""Unit tests for Discord ACL config helpers."""

from __future__ import annotations

import argparse
import importlib
import sys
from pathlib import Path

import pytest

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

config_module = importlib.import_module("discord_acl_events_config")
models_module = importlib.import_module("discord_acl_events_models")
endpoints = importlib.import_module("channel_test_endpoints")


def test_normalize_partition_mode_aliases() -> None:
    assert (
        config_module.normalize_partition_mode("guild-channel-user")
        == "guild_channel_user"
    )
    assert config_module.normalize_partition_mode("channel_only") == "channel"
    assert config_module.normalize_partition_mode("user") == "user"
    assert config_module.normalize_partition_mode("guild-user") == "guild_user"


def test_build_config_requires_channel_and_user_ids(tmp_path: Path) -> None:
    args = argparse.Namespace(
        ingress_url=endpoints.discord_ingress_url(),
        log_file=str(tmp_path / "runtime.log"),
        max_wait=20,
        max_idle_secs=20,
        channel_id="",
        user_id="",
        guild_id=None,
        username=None,
        role_id=[],
        secret_token=None,
        session_partition="guild_channel_user",
        no_follow=True,
    )
    with pytest.raises(ValueError, match="--channel-id and --user-id are required"):
        config_module.build_config(args, config_cls=models_module.ProbeConfig)


def test_default_ingress_url_from_bind_and_path(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.delenv("OMNI_DISCORD_INGRESS_URL", raising=False)
    monkeypatch.setenv("XIUXIAN_DAOCHANG_DISCORD_INGRESS_BIND", "0.0.0.0:19082")
    monkeypatch.setenv("XIUXIAN_DAOCHANG_DISCORD_INGRESS_PATH", "/ingress/discord")
    expected_bind = config_module.normalize_ingress_bind_for_local_url("0.0.0.0:19082")
    assert (
        config_module.default_ingress_url() == f"http://{expected_bind}/ingress/discord"
    )
