#!/usr/bin/env python3
"""Unit tests for Discord ingress stress config helpers."""

from __future__ import annotations

import argparse
import importlib
import sys
from pathlib import Path

import pytest

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

config_module = importlib.import_module("discord_ingress_stress_config")
models_module = importlib.import_module("discord_ingress_stress_models")
endpoints = importlib.import_module("channel_test_endpoints")


def _base_args(tmp_path: Path) -> argparse.Namespace:
    return argparse.Namespace(
        rounds=2,
        warmup_rounds=1,
        parallel=2,
        requests_per_worker=3,
        timeout_secs=10.0,
        cooldown_secs=0.1,
        ingress_url=endpoints.discord_ingress_url(),
        secret_token="",
        channel_id="2001",
        user_id="1001",
        guild_id="3001",
        username="tester",
        role_id=["role-a", "role-a", "role-b"],
        prompt="stress",
        log_file=str(tmp_path / "runtime.log"),
        project_root=str(tmp_path),
        output_json=str(tmp_path / "report.json"),
        output_markdown=str(tmp_path / "report.md"),
        quality_max_failure_rate=0.0,
        quality_max_p95_ms=0.0,
        quality_min_rps=0.0,
    )


def test_build_config_requires_channel_id(tmp_path: Path) -> None:
    args = _base_args(tmp_path)
    args.channel_id = ""

    with pytest.raises(ValueError, match="--channel-id is required"):
        config_module.build_config(args, config_cls=models_module.StressConfig)


def test_build_config_resolves_and_dedups(tmp_path: Path) -> None:
    args = _base_args(tmp_path)
    cfg = config_module.build_config(args, config_cls=models_module.StressConfig)

    assert cfg.rounds == 2
    assert cfg.warmup_rounds == 1
    assert cfg.parallel == 2
    assert cfg.requests_per_worker == 3
    assert cfg.role_ids == ("role-a", "role-b")
    assert cfg.output_json == (tmp_path / "report.json").resolve()
    assert cfg.output_markdown == (tmp_path / "report.md").resolve()
    assert cfg.log_file == (tmp_path / "runtime.log").resolve()
    assert cfg.quality_max_p95_ms is None
    assert cfg.quality_min_rps is None
