#!/usr/bin/env python3
"""Unit tests for memory benchmark config helpers."""

from __future__ import annotations

import argparse
import importlib
import json
import sys
from pathlib import Path

import pytest

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

config_module = importlib.import_module("memory_benchmark_config")
models_module = importlib.import_module("memory_benchmark_models")


def test_parse_args_uses_script_relative_defaults(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    monkeypatch.setattr(sys, "argv", ["memory_benchmark_config.py"])
    args = config_module.parse_args(
        script_dir=tmp_path,
        default_log_file="runtime.log",
        default_max_wait=44,
        default_max_idle_secs=22,
    )

    assert args.dataset == str(
        tmp_path / "fixtures" / "memory_benchmark_scenarios.json"
    )
    assert args.blackbox_script == str(tmp_path / "agent_channel_blackbox.py")
    assert args.max_wait == 44
    assert args.max_idle_secs == 22


def test_build_config_infers_session_ids_and_creates_report_dirs(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.delenv("OMNI_TEST_CHAT_ID", raising=False)
    monkeypatch.delenv("OMNI_TEST_USER_ID", raising=False)
    monkeypatch.delenv("OMNI_TEST_THREAD_ID", raising=False)

    dataset_path = tmp_path / "dataset.json"
    dataset_path.write_text(
        '{"scenarios":[{"id":"s1","queries":[{"prompt":"q"}]}]}', encoding="utf-8"
    )
    blackbox_script = tmp_path / "blackbox.py"
    blackbox_script.write_text("#!/usr/bin/env python3\n", encoding="utf-8")

    args = argparse.Namespace(
        mode=None,
        max_wait=30,
        max_idle_secs=20,
        iterations=2,
        feedback_down_threshold=0.4,
        chat_id=None,
        user_id=None,
        thread_id=None,
        dataset=str(dataset_path),
        log_file=str(tmp_path / "runtime.log"),
        blackbox_script=str(blackbox_script),
        username="tester",
        skip_reset=False,
        output_json=str(tmp_path / "reports" / "out.json"),
        output_markdown=str(tmp_path / "reports" / "out.md"),
        fail_on_tool_error=True,
        feedback_policy="deadband",
    )

    cfg = config_module.build_config(
        args,
        config_cls=models_module.BenchmarkConfig,
        infer_session_ids_fn=lambda _path: (101, 202, 303),
        resolve_runtime_partition_mode_fn=lambda _path: "chat_user",
    )

    assert cfg.chat_id == 101
    assert cfg.user_id == 202
    assert cfg.thread_id == 303
    assert cfg.runtime_partition_mode == "chat_user"
    assert cfg.output_json.parent.exists()
    assert cfg.output_markdown.parent.exists()


def test_load_scenarios_parses_dataset_payload(tmp_path: Path) -> None:
    payload = {
        "scenarios": [
            {
                "id": "alpha",
                "description": "first",
                "setup_prompts": ["seed"],
                "queries": [
                    {
                        "prompt": "ask",
                        "expected_keywords": ["one", "two"],
                        "required_ratio": 0.5,
                    }
                ],
            }
        ]
    }
    dataset = tmp_path / "scenarios.json"
    dataset.write_text(json.dumps(payload), encoding="utf-8")

    scenarios = config_module.load_scenarios(
        dataset,
        query_spec_cls=models_module.QuerySpec,
        scenario_spec_cls=models_module.ScenarioSpec,
    )

    assert len(scenarios) == 1
    assert scenarios[0].scenario_id == "alpha"
    assert scenarios[0].queries[0].expected_keywords == ("one", "two")


def test_load_scenarios_rejects_invalid_required_ratio(tmp_path: Path) -> None:
    payload = {
        "scenarios": [
            {
                "id": "bad",
                "queries": [{"prompt": "ask", "required_ratio": 1.2}],
            }
        ]
    }
    dataset = tmp_path / "scenarios.json"
    dataset.write_text(json.dumps(payload), encoding="utf-8")

    with pytest.raises(ValueError, match="required_ratio must be in"):
        config_module.load_scenarios(
            dataset,
            query_spec_cls=models_module.QuerySpec,
            scenario_spec_cls=models_module.ScenarioSpec,
        )
