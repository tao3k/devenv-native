from __future__ import annotations

import importlib.util
from dataclasses import dataclass
from pathlib import Path


def _load_module():
    module_name = "complex_scenarios_evaluation_test_module"
    script_path = Path(__file__).resolve().with_name("complex_scenarios_evaluation.py")
    spec = importlib.util.spec_from_file_location(module_name, script_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"failed to load module from {script_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


@dataclass(frozen=True)
class _Step:
    step_id: str
    order: int
    prompt: str
    tags: tuple[str, ...]
    depends_on: tuple[str, ...] = ()


@dataclass(frozen=True)
class _Scenario:
    scenario_id: str
    steps: tuple[_Step, ...]


@dataclass(frozen=True)
class _ComplexityProfile:
    step_count: int
    dependency_edges: int
    critical_path_len: int
    wave_count: int
    parallel_waves: int
    max_wave_width: int
    branch_nodes: int
    complexity_score: float


@dataclass(frozen=True)
class _ComplexityRequirement:
    steps: int
    dependency_edges: int
    critical_path_len: int
    parallel_waves: int


@dataclass(frozen=True)
class _Result:
    step_id: str
    passed: bool
    memory_planned_seen: bool
    memory_recall_credit_count: int
    memory_decay_count: int
    feedback_command_bias_delta: float | None


@dataclass(frozen=True)
class _QualityProfile:
    error_signal_steps: int
    negative_feedback_events: int
    correction_check_steps: int
    successful_corrections: int
    planned_hits: int
    natural_language_steps: int
    recall_credit_events: int
    decay_events: int
    quality_score: float


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


def test_complexity_profile_and_evaluation() -> None:
    module = _load_module()
    scenario = _Scenario(
        scenario_id="s",
        steps=(
            _Step(step_id="a", order=0, prompt="nl a", tags=("error_signal",)),
            _Step(step_id="b", order=1, prompt="nl b", tags=(), depends_on=("a",)),
            _Step(
                step_id="c",
                order=2,
                prompt="/cmd",
                tags=("correction_check",),
                depends_on=("a",),
            ),
        ),
    )
    waves = module.build_execution_waves(scenario)
    assert len(waves) == 2
    profile = module.compute_complexity_profile(
        scenario, complexity_profile_cls=_ComplexityProfile
    )
    assert profile.step_count == 3
    assert profile.dependency_edges == 2

    passed, failures = module.evaluate_complexity(
        profile,
        _ComplexityRequirement(
            steps=3, dependency_edges=2, critical_path_len=2, parallel_waves=1
        ),
    )
    assert passed
    assert failures == tuple()


def test_quality_profile_and_evaluation() -> None:
    module = _load_module()
    scenario = _Scenario(
        scenario_id="s",
        steps=(
            _Step(step_id="a", order=0, prompt="nl a", tags=("error_signal",)),
            _Step(step_id="b", order=1, prompt="nl b", tags=(), depends_on=("a",)),
            _Step(
                step_id="c",
                order=2,
                prompt="nl c",
                tags=("correction_check",),
                depends_on=("a",),
            ),
        ),
    )
    results = (
        _Result("a", True, True, 1, 0, -0.2),
        _Result("b", True, True, 0, 1, None),
        _Result("c", True, True, 0, 0, None),
    )
    profile = module.compute_quality_profile(
        scenario,
        results,
        quality_profile_cls=_QualityProfile,
    )
    assert profile.error_signal_steps == 1
    assert profile.negative_feedback_events == 1
    assert profile.successful_corrections == 1

    passed, failures = module.evaluate_quality(
        profile,
        _QualityRequirement(1, 1, 1, 1, 1, 2, 1, 1),
    )
    assert passed
    assert failures == tuple()
