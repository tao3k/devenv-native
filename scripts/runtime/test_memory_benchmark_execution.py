#!/usr/bin/env python3
"""Unit tests for memory benchmark execution helpers."""

from __future__ import annotations

import importlib
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from types import SimpleNamespace

import pytest

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

_execution_module = importlib.import_module("memory_benchmark_execution")


@dataclass(frozen=True)
class _Query:
    prompt: str
    expected_keywords: tuple[str, ...]
    required_ratio: float


@dataclass(frozen=True)
class _Scenario:
    scenario_id: str
    setup_prompts: tuple[str, ...]
    queries: tuple[_Query, ...]
    reset_before: bool
    reset_after: bool


@dataclass(frozen=True)
class _Turn:
    keyword_success: bool | None
    keyword_hit_ratio: float | None


def test_build_turn_result_maps_feedback_metrics() -> None:
    query = _Query(prompt="q", expected_keywords=("alpha",), required_ratio=1.0)

    def _parse(lines: list[str]) -> dict[str, object]:
        if lines == ["feedback"]:
            return {
                "feedback": {
                    "recall_feedback_bias_before": "0.2",
                    "recall_feedback_bias_after": "0.5",
                }
            }
        return {
            "plan": {"k1": "7", "k2": "3", "lambda": "0.6"},
            "decision": {"event": "agent.memory.recall.injected", "query_tokens": "44"},
            "bot_line": "alpha",
            "embedding_timeout_fallback": False,
            "embedding_cooldown_fallback": False,
            "embedding_unavailable_fallback": False,
            "tool_error": False,
        }

    def _float_token(tokens: dict[str, object], key: str) -> float | None:
        raw = tokens.get(key)
        return float(raw) if raw is not None else None

    def _int_token(tokens: dict[str, object], key: str) -> int | None:
        raw = tokens.get(key)
        return int(raw) if raw is not None else None

    result = _execution_module.build_turn_result(
        mode="adaptive",
        iteration=1,
        scenario_id="s1",
        query_index=1,
        query=query,
        lines=["turn"],
        feedback_direction="up",
        feedback_lines=["feedback"],
        parse_turn_signals_fn=_parse,
        keyword_hit_ratio_fn=lambda _line, _keywords: 1.0,
        token_as_int_fn=_int_token,
        token_as_float_fn=_float_token,
        trim_text_fn=lambda value: value,
        turn_result_cls=SimpleNamespace,
    )

    assert result.keyword_success is True
    assert result.decision == "injected"
    assert result.query_tokens == 44
    assert result.feedback_bias_before == 0.2
    assert result.feedback_bias_after == 0.5


def test_run_mode_executes_feedback_path_for_adaptive_mode() -> None:
    config = SimpleNamespace(
        iterations=1,
        skip_reset=False,
        feedback_policy="deadband",
        feedback_down_threshold=0.34,
    )
    scenarios = (
        _Scenario(
            scenario_id="s1",
            setup_prompts=("seed",),
            queries=(
                _Query(prompt="q1", expected_keywords=("alpha",), required_ratio=1.0),
            ),
            reset_before=True,
            reset_after=True,
        ),
    )
    call_log: list[str] = []

    def _run_reset(_cfg: object) -> None:
        call_log.append("reset")

    def _run_non_command(_cfg: object, prompt: str) -> list[str]:
        call_log.append(f"prompt:{prompt}")
        return ["turn"]

    def _build_turn_result(**kwargs: object) -> _Turn:
        call_log.append("build")
        return _Turn(keyword_success=True, keyword_hit_ratio=1.0)

    def _run_feedback(_cfg: object, direction: str) -> list[str]:
        call_log.append(f"feedback:{direction}")
        return ["feedback"]

    turns = _execution_module.run_mode(
        config,
        scenarios,
        "adaptive",
        run_reset_fn=_run_reset,
        run_non_command_turn_fn=_run_non_command,
        build_turn_result_fn=_build_turn_result,
        select_feedback_direction_fn=lambda **_kwargs: "up",
        run_feedback_fn=_run_feedback,
    )

    assert len(turns) == 1
    assert call_log == [
        "reset",
        "prompt:seed",
        "prompt:q1",
        "build",
        "feedback:up",
        "build",
        "reset",
    ]


def test_run_probe_maps_admin_required_error(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    config = SimpleNamespace(
        blackbox_script=tmp_path / "blackbox.py",
        chat_id=1,
        user_id=2,
        thread_id=None,
        runtime_partition_mode=None,
        fail_on_tool_error=False,
        username="tester",
        max_wait=20,
        max_idle_secs=10,
        log_file=tmp_path / "runtime.log",
    )

    def _raise_called_process_error(*_args: object, **_kwargs: object) -> None:
        raise subprocess.CalledProcessError(returncode=1, cmd=["probe"])

    monkeypatch.setattr(
        _execution_module.subprocess, "run", _raise_called_process_error
    )

    with pytest.raises(RuntimeError, match="admin_required"):
        _execution_module.run_probe(
            config,
            prompt="/session json",
            expect_event="telegram.command.session_status_json.replied",
            count_lines_fn=lambda _path: 0,
            read_new_lines_fn=lambda _path, _cursor: (
                1,
                [
                    '2026-01-01T00:00:00Z WARN event="telegram.command.control_admin_required.replied"'
                ],
            ),
            strip_ansi_fn=lambda line: line,
            has_event_fn=lambda _lines, _event: True,
            control_admin_required_event="telegram.command.control_admin_required.replied",
            forbidden_log_pattern="tools/call: Tool runtime error",
        )


def test_parse_turn_signals_forwards_constants() -> None:
    captured: dict[str, object] = {}

    def _parser(lines: list[str], **kwargs: object) -> dict[str, object]:
        captured["lines"] = lines
        captured.update(kwargs)
        return {"ok": True}

    result = _execution_module.parse_turn_signals(
        ["line"],
        parse_turn_signals_fn=_parser,
        forbidden_log_pattern="forbid",
        bot_marker="bot",
        recall_plan_event="plan",
        recall_injected_event="inj",
        recall_skipped_event="skip",
        recall_feedback_event="feedback",
        embedding_timeout_fallback_event="timeout",
        embedding_cooldown_fallback_event="cooldown",
        embedding_unavailable_fallback_event="unavailable",
    )

    assert result == {"ok": True}
    assert captured["lines"] == ["line"]
    assert captured["forbidden_log_pattern"] == "forbid"
    assert captured["recall_plan_event"] == "plan"
