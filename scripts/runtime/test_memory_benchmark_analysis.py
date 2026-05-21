from __future__ import annotations

import importlib.util
from dataclasses import dataclass
from pathlib import Path


def _load_module():
    module_name = "memory_benchmark_analysis_test_module"
    script_path = Path(__file__).resolve().with_name("memory_benchmark_analysis.py")
    spec = importlib.util.spec_from_file_location(module_name, script_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"failed to load module from {script_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


@dataclass(frozen=True)
class _Turn:
    keyword_hit_ratio: float | None
    keyword_success: bool | None
    query_tokens: int | None
    recalled_selected: int | None
    recalled_injected: int | None
    context_chars_injected: int | None
    pipeline_duration_ms: int | None
    best_score: float | None
    weakest_score: float | None
    k1: int | None
    k2: int | None
    lambda_value: float | None
    min_score: float | None
    budget_pressure: float | None
    window_pressure: float | None
    recall_feedback_bias: float | None
    feedback_bias_before: float | None
    feedback_bias_after: float | None
    feedback_direction: str | None
    tool_error_detected: bool
    embedding_timeout_fallback_seen: bool
    embedding_cooldown_fallback_seen: bool
    embedding_unavailable_fallback_seen: bool
    decision: str | None


@dataclass(frozen=True)
class _Summary:
    success_rate: float
    avg_keyword_hit_ratio: float
    injected_rate: float
    avg_pipeline_duration_ms: float
    avg_recalled_selected: float
    avg_recalled_injected: float
    avg_best_score: float
    avg_recall_feedback_bias: float
    tool_error_turns: int
    embedding_timeout_fallback_turns: int
    embedding_cooldown_fallback_turns: int
    embedding_unavailable_fallback_turns: int
    embedding_fallback_turns_total: int


def test_keyword_hit_ratio_and_feedback_direction() -> None:
    module = _load_module()
    assert module.keyword_hit_ratio("A B C", ("a", "x")) == 0.5
    assert module.keyword_hit_ratio(None, ("a",)) == 0.0
    assert module.keyword_hit_ratio("x", tuple()) is None

    assert (
        module.select_feedback_direction(
            keyword_hit_ratio=0.9,
            keyword_success=True,
            policy="strict",
            down_threshold=0.34,
        )
        == "up"
    )
    assert (
        module.select_feedback_direction(
            keyword_hit_ratio=0.2,
            keyword_success=False,
            policy="deadband",
            down_threshold=0.34,
        )
        == "down"
    )
    assert (
        module.select_feedback_direction(
            keyword_hit_ratio=0.5,
            keyword_success=False,
            policy="deadband",
            down_threshold=0.34,
        )
        is None
    )


def test_summarize_mode_data_and_compare_mode_summaries() -> None:
    module = _load_module()
    turns = [
        _Turn(
            keyword_hit_ratio=1.0,
            keyword_success=True,
            query_tokens=100,
            recalled_selected=3,
            recalled_injected=2,
            context_chars_injected=50,
            pipeline_duration_ms=20,
            best_score=0.8,
            weakest_score=0.2,
            k1=8,
            k2=4,
            lambda_value=0.4,
            min_score=0.1,
            budget_pressure=0.2,
            window_pressure=0.3,
            recall_feedback_bias=0.1,
            feedback_bias_before=0.0,
            feedback_bias_after=0.1,
            feedback_direction="up",
            tool_error_detected=False,
            embedding_timeout_fallback_seen=True,
            embedding_cooldown_fallback_seen=False,
            embedding_unavailable_fallback_seen=False,
            decision="injected",
        ),
        _Turn(
            keyword_hit_ratio=0.0,
            keyword_success=False,
            query_tokens=120,
            recalled_selected=1,
            recalled_injected=0,
            context_chars_injected=10,
            pipeline_duration_ms=30,
            best_score=0.2,
            weakest_score=0.1,
            k1=8,
            k2=4,
            lambda_value=0.4,
            min_score=0.1,
            budget_pressure=0.4,
            window_pressure=0.6,
            recall_feedback_bias=0.0,
            feedback_bias_before=0.1,
            feedback_bias_after=0.0,
            feedback_direction="down",
            tool_error_detected=True,
            embedding_timeout_fallback_seen=False,
            embedding_cooldown_fallback_seen=True,
            embedding_unavailable_fallback_seen=False,
            decision="skipped",
        ),
    ]
    summary_payload = module.summarize_mode_data(
        mode="baseline",
        iterations=1,
        scenario_count=1,
        turns=turns,
    )
    assert summary_payload["mode"] == "baseline"
    assert summary_payload["query_turns"] == 2
    assert summary_payload["scored_turns"] == 2
    assert summary_payload["success_count"] == 1
    assert summary_payload["tool_error_turns"] == 1
    assert summary_payload["embedding_fallback_turns_total"] == 2
    assert summary_payload["feedback_up_count"] == 1
    assert summary_payload["feedback_down_count"] == 1

    baseline = _Summary(
        success_rate=0.5,
        avg_keyword_hit_ratio=0.5,
        injected_rate=0.5,
        avg_pipeline_duration_ms=25.0,
        avg_recalled_selected=2.0,
        avg_recalled_injected=1.0,
        avg_best_score=0.5,
        avg_recall_feedback_bias=0.05,
        tool_error_turns=1,
        embedding_timeout_fallback_turns=1,
        embedding_cooldown_fallback_turns=1,
        embedding_unavailable_fallback_turns=0,
        embedding_fallback_turns_total=2,
    )
    adaptive = _Summary(
        success_rate=0.75,
        avg_keyword_hit_ratio=0.6,
        injected_rate=0.7,
        avg_pipeline_duration_ms=20.0,
        avg_recalled_selected=2.5,
        avg_recalled_injected=1.2,
        avg_best_score=0.55,
        avg_recall_feedback_bias=0.08,
        tool_error_turns=0,
        embedding_timeout_fallback_turns=0,
        embedding_cooldown_fallback_turns=1,
        embedding_unavailable_fallback_turns=0,
        embedding_fallback_turns_total=1,
    )
    delta = module.compare_mode_summaries(baseline, adaptive)
    assert delta["success_rate_delta"] == 0.25
    assert delta["tool_error_turns_delta"] == -1.0
