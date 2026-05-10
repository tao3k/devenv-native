#!/usr/bin/env python3
"""Unit tests for memory SLO runtime helpers."""

from __future__ import annotations

import importlib
import sys
from pathlib import Path

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

models_module = importlib.import_module("memory_slo_models")
runtime_module = importlib.import_module("memory_slo_runtime")


def _cfg(tmp_path: Path) -> object:
    return models_module.SloConfig(
        project_root=tmp_path,
        evolution_report_json=tmp_path / "evolution.json",
        benchmark_report_json=tmp_path / "benchmark.json",
        session_matrix_report_json=tmp_path / "matrix.json",
        runtime_log_file=tmp_path / "runtime.log",
        output_json=tmp_path / "slo.json",
        output_markdown=tmp_path / "slo.md",
        min_planned_hits=10,
        min_successful_corrections=3,
        min_recall_credit_events=1,
        min_quality_score=90.0,
        required_benchmark_modes=("baseline", "adaptive"),
        min_query_turns=1,
        max_mode_tool_error_turns=0,
        max_total_tool_error_turns=0,
        min_session_steps=1,
        max_session_failed_steps=0,
        enable_stream_gate=True,
        min_stream_ack_ratio=0.8,
        min_stream_published_events=1,
        max_stream_read_failed=0,
    )


def test_evaluate_benchmark_reports_mode_failures(tmp_path: Path) -> None:
    cfg = _cfg(tmp_path)
    result = runtime_module.evaluate_benchmark(
        cfg,
        {
            "mode_summaries": {
                "baseline": {"query_turns": 2, "tool_error_turns": 1},
                "adaptive": {"query_turns": 2, "tool_error_turns": 0},
            }
        },
    )
    assert result["passed"] is False
    assert any(
        "benchmark.baseline.tool_error_turns=1 > 0" in item
        for item in result["failures"]
    )


def test_evaluate_stream_health_counts_ack_and_failures(tmp_path: Path) -> None:
    cfg = _cfg(tmp_path)
    runtime_log = tmp_path / "runtime.log"
    runtime_log.write_text(
        "\n".join(
            [
                'DEBUG event="session.stream_event.published"',
                'DEBUG event="session.stream_event.published"',
                'DEBUG event="agent.memory.stream_consumer.event_processed"',
                'DEBUG event="agent.memory.stream_consumer.read_failed"',
            ]
        )
        + "\n",
        encoding="utf-8",
    )
    result = runtime_module.evaluate_stream_health(cfg, runtime_log)
    assert result["passed"] is False
    assert result["summary"]["ack_ratio"] == 0.5
    assert result["summary"]["read_failed_events"] == 1
