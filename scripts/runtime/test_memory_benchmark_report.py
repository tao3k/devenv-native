from __future__ import annotations

import importlib.util
from dataclasses import dataclass
from pathlib import Path


def _load_module():
    module_name = "memory_benchmark_report_test_module"
    script_path = Path(__file__).resolve().with_name("memory_benchmark_report.py")
    spec = importlib.util.spec_from_file_location(module_name, script_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"failed to load module from {script_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


@dataclass(frozen=True)
class _Scenario:
    scenario_id: str
    description: str
    setup_prompts: tuple[str, ...]
    queries: tuple[str, ...]


@dataclass(frozen=True)
class _Summary:
    query_turns: int = 4
    scored_turns: int = 4
    success_rate: float = 0.75
    avg_keyword_hit_ratio: float = 0.8
    injected_rate: float = 0.5
    avg_pipeline_duration_ms: float = 12.3
    avg_k1: float = 8.0
    avg_k2: float = 3.0
    avg_lambda: float = 0.4
    avg_recall_feedback_bias: float = 0.1
    tool_error_turns: int = 0
    embedding_fallback_turns_total: int = 1


@dataclass(frozen=True)
class _Config:
    dataset_path: Path
    log_file: Path
    chat_id: int
    user_id: int
    thread_id: int | None
    runtime_partition_mode: str | None
    modes: tuple[str, ...]
    iterations: int
    feedback_policy: str
    feedback_down_threshold: float


def test_build_markdown_report_renders_sections_and_delta_table() -> None:
    module = _load_module()
    cfg = _Config(
        dataset_path=Path("/tmp/dataset.json"),
        log_file=Path("/tmp/runtime.log"),
        chat_id=1,
        user_id=2,
        thread_id=None,
        runtime_partition_mode="chat_user",
        modes=("baseline", "adaptive"),
        iterations=2,
        feedback_policy="deadband",
        feedback_down_threshold=0.34,
    )
    scenarios = (
        _Scenario("s1", "Scenario One", ("setup",), ("q1", "q2")),
        _Scenario("s2", "Scenario Two", tuple(), ("q1",)),
    )
    mode_summaries = {"baseline": _Summary(), "adaptive": _Summary(success_rate=0.8)}
    comparison = {"success_rate_delta": 0.05}

    markdown = module.build_markdown_report(
        config=cfg,
        scenarios=scenarios,
        started_at="2026-02-20T00:00:00Z",
        finished_at="2026-02-20T00:00:10Z",
        mode_summaries=mode_summaries,
        comparison=comparison,
    )

    assert "# Omni-Agent Memory A/B Benchmark" in markdown
    assert "## Scenario Set" in markdown
    assert "`s1`" in markdown
    assert "## Adaptive Delta vs Baseline" in markdown
    assert "success_rate_delta" in markdown
