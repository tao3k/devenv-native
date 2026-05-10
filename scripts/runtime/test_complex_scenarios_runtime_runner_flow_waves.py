#!/usr/bin/env python3
"""Unit tests for wave execution helper argument binding behavior."""

from __future__ import annotations

import importlib
import sys
from dataclasses import dataclass
from functools import partial
from pathlib import Path
from types import SimpleNamespace

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

module = importlib.import_module("complex_scenarios_runtime_runner_flow_waves")
execute_waves = module.execute_waves


@dataclass(frozen=True)
class _Step:
    step_id: str
    session_alias: str
    depends_on: tuple[str, ...]


@dataclass(frozen=True)
class _Session:
    alias: str


@dataclass(frozen=True)
class _Result:
    step_id: str
    passed: bool


def test_execute_waves_skipped_callback_works_with_partial_runtime_partition_binding() -> (
    None
):
    cfg = SimpleNamespace(
        execute_wave_parallel=False, max_parallel=1, runtime_partition_mode="global"
    )
    scenario = SimpleNamespace(scenario_id="s")
    sessions = {"a": _Session(alias="a")}

    fail_step = _Step(step_id="a_fail", session_alias="a", depends_on=())
    blocked_step = _Step(step_id="a_blocked", session_alias="a", depends_on=("a_fail",))
    waves = ((fail_step,), (blocked_step,))

    def run_step_fn(
        _cfg: object,
        _scenario_id: str,
        step: _Step,
        _session: _Session,
        _wave_index: int,
    ) -> _Result:
        return _Result(step_id=step.step_id, passed=False)

    observed_runtime_modes: list[str | None] = []

    def skipped_step_result(
        _scenario_id: str,
        step: _Step,
        _session: _Session,
        _wave_index: int,
        _reason: str,
        runtime_partition_mode: str | None = None,
    ) -> _Result:
        observed_runtime_modes.append(runtime_partition_mode)
        return _Result(step_id=step.step_id, passed=False)

    skipped_step_result_fn = partial(
        skipped_step_result, runtime_partition_mode="per_sender"
    )
    results = execute_waves(
        cfg,
        scenario,
        sessions=sessions,
        waves=waves,
        run_step_fn=run_step_fn,
        skipped_step_result_fn=skipped_step_result_fn,
    )

    assert [result.step_id for result in results] == ["a_fail", "a_blocked"]
    assert observed_runtime_modes == ["per_sender"]
