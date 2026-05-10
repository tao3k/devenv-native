#!/usr/bin/env python3
"""Unit tests for complex scenario execution helpers."""

from __future__ import annotations

import importlib
import sys
from dataclasses import dataclass
from pathlib import Path

import pytest

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

_execution_module = importlib.import_module("complex_scenarios_execution")
extract_bot_excerpt = _execution_module.extract_bot_excerpt
merge_quality_requirements = _execution_module.merge_quality_requirements
parse_requirement = _execution_module.parse_requirement
select_scenarios = _execution_module.select_scenarios


@dataclass(frozen=True)
class _Requirement:
    steps: int
    dependency_edges: int
    critical_path_len: int
    parallel_waves: int


@dataclass(frozen=True)
class _QualityRequirement:
    min_error_signals: int
    min_negative_feedback_events: int
    min_correction_checks: int
    min_successful_corrections: int
    min_planned_hits: int
    min_natural_language_steps: int
    min_recall_credit_events: int
    min_decay_events: int


@dataclass(frozen=True)
class _Scenario:
    scenario_id: str


def test_parse_requirement_builds_requirement_class() -> None:
    result = parse_requirement(
        {
            "steps": 3,
            "dependency_edges": 2,
            "critical_path_len": 2,
            "parallel_waves": 1,
        },
        requirement_cls=_Requirement,
    )
    assert result == _Requirement(
        steps=3, dependency_edges=2, critical_path_len=2, parallel_waves=1
    )


def test_merge_quality_requirements_uses_maxima() -> None:
    global_req = _QualityRequirement(1, 2, 3, 4, 5, 6, 7, 8)
    scenario_req = _QualityRequirement(3, 1, 5, 2, 10, 4, 9, 6)
    merged = merge_quality_requirements(
        global_req,
        scenario_req,
        quality_requirement_cls=_QualityRequirement,
    )
    assert merged == _QualityRequirement(3, 2, 5, 4, 10, 6, 9, 8)


def test_select_scenarios_filters_by_id() -> None:
    scenarios = (_Scenario("a"), _Scenario("b"))
    selected = select_scenarios(scenarios, "b")
    assert selected == (_Scenario("b"),)


def test_select_scenarios_raises_for_missing_id() -> None:
    with pytest.raises(ValueError, match="scenario not found"):
        select_scenarios((_Scenario("a"),), "missing")


def test_extract_bot_excerpt_prefers_explicit_section() -> None:
    stdout = "\n".join(
        [
            "header",
            "Observed outbound bot log:",
            "  bot response line",
            "trailer",
        ]
    )
    assert extract_bot_excerpt(stdout) == "bot response line"
