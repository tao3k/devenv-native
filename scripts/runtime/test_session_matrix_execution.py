#!/usr/bin/env python3
"""Unit tests for session matrix execution helpers."""

from __future__ import annotations

import importlib
import os
import sys
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

_execution_module = importlib.import_module("session_matrix_execution")
run_command_with_restart_retry = _execution_module.run_command_with_restart_retry
run_matrix = _execution_module.run_matrix


def _local_host() -> str:
    return (
        os.environ.get("XIUXIAN_WENDAO_LOCAL_HOST", "localhost").strip() or "localhost"
    )


@dataclass(frozen=True)
class _Cfg:
    session_partition: str | None
    chat_id: int
    chat_b: int
    chat_c: int
    user_a: int
    user_b: int
    user_c: int
    thread_a: int | None
    thread_b: int | None
    thread_c: int | None


@dataclass(frozen=True)
class _Step:
    name: str


@dataclass(frozen=True)
class _Result:
    name: str
    passed: bool


def _build_report(
    _cfg: _Cfg, results: list[_Result], _started_dt: datetime, _started_mono: float
) -> dict[str, object]:
    passed = sum(1 for step in results if step.passed)
    return {
        "started_at": datetime.now(UTC).isoformat(),
        "summary": {
            "total": len(results),
            "passed": passed,
            "failed": len(results) - passed,
        },
        "steps": [{"name": step.name, "passed": step.passed} for step in results],
    }


def test_run_command_with_restart_retry_retries_once_on_known_restart_noise() -> None:
    calls = 0

    def _run_command(_cmd: list[str]) -> tuple[int, int, str, str]:
        nonlocal calls
        calls += 1
        if calls == 1:
            return 1, 7, f"Telegram webhook listening on {_local_host()}", ""
        return 0, 4, "ok", ""

    returncode, duration_ms, stdout, stderr = run_command_with_restart_retry(
        ["dummy"],
        run_command_fn=_run_command,
    )
    assert calls == 2
    assert returncode == 0
    assert duration_ms == 11
    assert "[matrix-retry] detected webhook restart noise" in stdout
    assert stderr == ""


def test_run_matrix_uses_cross_chat_baseline_for_chat_partition() -> None:
    cfg = _Cfg(
        session_partition="chat",
        chat_id=1,
        chat_b=2,
        chat_c=3,
        user_a=10,
        user_b=11,
        user_c=12,
        thread_a=None,
        thread_b=None,
        thread_c=None,
    )
    concurrent_calls: list[str] = []

    def _run_concurrent(_script_dir: Path, _cfg: _Cfg, **kwargs: object) -> _Result:
        concurrent_calls.append(str(kwargs["name"]))
        return _Result(name=str(kwargs["name"]), passed=True)

    passed, report = run_matrix(
        cfg,
        script_dir=Path("."),
        build_report_fn=_build_report,
        build_matrix_steps_fn=lambda _cfg: (_Step("status"),),
        run_concurrent_step_fn=_run_concurrent,
        run_blackbox_step_fn=lambda _script_dir, _cfg, step: _Result(step.name, True),
        run_mixed_concurrency_batch_fn=lambda _script_dir, _cfg: [
            _Result("mixed", True)
        ],
    )

    assert passed is True
    assert concurrent_calls == ["concurrent_baseline_cross_chat"]
    assert report["summary"]["total"] == 3


def test_run_matrix_adds_cross_group_probe_for_non_chat_partition() -> None:
    cfg = _Cfg(
        session_partition="chat_user",
        chat_id=1,
        chat_b=2,
        chat_c=3,
        user_a=10,
        user_b=11,
        user_c=12,
        thread_a=None,
        thread_b=None,
        thread_c=None,
    )
    concurrent_calls: list[str] = []

    def _run_concurrent(_script_dir: Path, _cfg: _Cfg, **kwargs: object) -> _Result:
        concurrent_calls.append(str(kwargs["name"]))
        return _Result(name=str(kwargs["name"]), passed=True)

    passed, _report = run_matrix(
        cfg,
        script_dir=Path("."),
        build_report_fn=_build_report,
        build_matrix_steps_fn=lambda _cfg: (),
        run_concurrent_step_fn=_run_concurrent,
        run_blackbox_step_fn=lambda _script_dir, _cfg, _step: _Result("blackbox", True),
        run_mixed_concurrency_batch_fn=lambda _script_dir, _cfg: [],
    )

    assert passed is True
    assert concurrent_calls == [
        "concurrent_baseline_same_chat",
        "concurrent_cross_group",
    ]


def test_run_matrix_stops_on_first_failure() -> None:
    cfg = _Cfg(
        session_partition="chat_user",
        chat_id=1,
        chat_b=2,
        chat_c=3,
        user_a=10,
        user_b=11,
        user_c=12,
        thread_a=None,
        thread_b=None,
        thread_c=None,
    )
    matrix_steps_called = False

    def _build_steps(_cfg: _Cfg) -> tuple[_Step, ...]:
        nonlocal matrix_steps_called
        matrix_steps_called = True
        return (_Step("never"),)

    passed, report = run_matrix(
        cfg,
        script_dir=Path("."),
        build_report_fn=_build_report,
        build_matrix_steps_fn=_build_steps,
        run_concurrent_step_fn=lambda _script_dir, _cfg, **kwargs: _Result(
            name=str(kwargs["name"]), passed=False
        ),
        run_blackbox_step_fn=lambda _script_dir, _cfg, _step: _Result("blackbox", True),
        run_mixed_concurrency_batch_fn=lambda _script_dir, _cfg: [
            _Result("mixed", True)
        ],
    )

    assert passed is False
    assert matrix_steps_called is False
    assert report["summary"]["total"] == 1
