from __future__ import annotations

import importlib.util
import os
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path


def _load_module():
    module_name = "complex_scenarios_report_test_module"
    script_path = Path(__file__).resolve().with_name("complex_scenarios_report.py")
    spec = importlib.util.spec_from_file_location(module_name, script_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"failed to load module from {script_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _webhook_url() -> str:
    host = (
        os.environ.get("XIUXIAN_WENDAO_LOCAL_HOST", "localhost").strip() or "localhost"
    )
    return f"http://{host}:18081/telegram/webhook"


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
class _Complexity:
    step_count: int
    dependency_edges: int
    critical_path_len: int
    parallel_waves: int
    complexity_score: float


@dataclass(frozen=True)
class _Quality:
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
class _Step:
    step_id: str
    prompt: str
    session_key: str
    wave_index: int
    event: str
    passed: bool
    skipped: bool
    duration_ms: int
    bot_excerpt: str | None
    stderr_tail: str
    stdout_tail: str
    memory_planned_seen: bool = True
    memory_injected_seen: bool = True
    memory_skipped_seen: bool = False
    memory_feedback_updated_seen: bool = False
    memory_recall_credit_seen: bool = False
    memory_decay_seen: bool = False
    memory_recall_credit_count: int = 0
    memory_decay_count: int = 0
    tool_waiting_seen: bool = False
    tool_event_counts: dict[str, int] | None = None
    memory_planned_bias: float | None = None
    memory_decision: str | None = None
    feedback_command_bias_delta: float | None = None
    feedback_heuristic_bias_delta: float | None = None
    tool_last_event: str | None = None


@dataclass(frozen=True)
class _Session:
    alias: str
    chat_id: int
    user_id: int
    thread_id: int | None
    chat_title: str


@dataclass(frozen=True)
class _ScenarioResult:
    scenario_id: str
    description: str
    requirement: _Requirement
    complexity: _Complexity
    complexity_passed: bool
    complexity_failures: tuple[str, ...]
    quality_requirement: _QualityRequirement
    quality: _Quality
    quality_passed: bool
    quality_failures: tuple[str, ...]
    duration_ms: int
    passed: bool
    steps: tuple[_Step, ...]


@dataclass(frozen=True)
class _Config:
    dataset_path: Path
    scenario_id: str | None
    blackbox_script: Path
    webhook_url: str
    log_file: Path
    max_wait: int
    max_idle_secs: int
    max_parallel: int
    execute_wave_parallel: bool
    runtime_partition_mode: str | None
    username: str
    forbid_log_regexes: tuple[str, ...]
    global_requirement: _Requirement
    global_quality_requirement: _QualityRequirement
    sessions: tuple[_Session, ...]


def test_build_report_render_and_write_outputs(tmp_path: Path) -> None:
    module = _load_module()
    cfg = _Config(
        dataset_path=tmp_path / "dataset.json",
        scenario_id=None,
        blackbox_script=tmp_path / "agent_channel_blackbox.py",
        webhook_url=_webhook_url(),
        log_file=tmp_path / "runtime.log",
        max_wait=30,
        max_idle_secs=25,
        max_parallel=2,
        execute_wave_parallel=True,
        runtime_partition_mode="chat_user",
        username="tester",
        forbid_log_regexes=("tools/call: Tool runtime error",),
        global_requirement=_Requirement(1, 0, 1, 1),
        global_quality_requirement=_QualityRequirement(0, 0, 0, 0, 0, 0, 0, 0),
        sessions=(_Session("s1", -1001, 42, None, "group"),),
    )
    result = _ScenarioResult(
        scenario_id="scenario-1",
        description="test scenario",
        requirement=_Requirement(1, 0, 1, 1),
        complexity=_Complexity(1, 0, 1, 1, 0.5),
        complexity_passed=True,
        complexity_failures=tuple(),
        quality_requirement=_QualityRequirement(0, 0, 0, 0, 0, 0, 0, 0),
        quality=_Quality(0, 0, 0, 0, 1, 1, 0, 0, 1.0),
        quality_passed=True,
        quality_failures=tuple(),
        duration_ms=100,
        passed=True,
        steps=(
            _Step(
                step_id="s1-step-1",
                prompt="hello",
                session_key="s1",
                wave_index=0,
                event="telegram.command.session_status_json.replied",
                passed=True,
                skipped=False,
                duration_ms=20,
                bot_excerpt="ok",
                stderr_tail="",
                stdout_tail="",
            ),
        ),
    )

    report = module.build_report(
        cfg=cfg,
        scenario_results=(result,),
        started_mono=0.0,
        started_dt=datetime.now(UTC),
    )
    assert report["overall_passed"] is True
    assert report["summary"]["passed"] == 1

    markdown = module.render_markdown(report)
    assert "Agent Channel Complex Scenario Report" in markdown
    assert "scenario-1" in markdown

    output_json = tmp_path / "report.json"
    output_markdown = tmp_path / "report.md"
    module.write_outputs(report, output_json, output_markdown)
    assert output_json.exists()
    assert output_markdown.exists()
