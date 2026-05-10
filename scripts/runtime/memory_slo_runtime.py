#!/usr/bin/env python3
"""Runtime orchestration helpers for memory/session SLO aggregation."""

from __future__ import annotations

import json
import time
from datetime import UTC, datetime
from typing import TYPE_CHECKING, Any

from memory_slo_runtime_benchmark import evaluate_benchmark
from memory_slo_runtime_evolution import evaluate_evolution
from memory_slo_runtime_session import evaluate_session_matrix
from memory_slo_runtime_stream import evaluate_stream_health

if TYPE_CHECKING:
    from pathlib import Path

    from memory_slo_models import SloConfig


def load_json(path: Path) -> dict[str, Any]:
    """Load object JSON payload from disk."""
    if not path.exists():
        raise RuntimeError(f"missing report: {path}")
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise RuntimeError(f"report payload is not an object: {path}")
    return payload


def run_slo_report(cfg: SloConfig, *, load_json_fn: Any = load_json) -> dict[str, Any]:
    """Run all checks and assemble final SLO report payload."""
    started_ts = time.time()
    started_at = datetime.fromtimestamp(started_ts, tz=UTC).isoformat()

    evolution = evaluate_evolution(cfg, load_json_fn(cfg.evolution_report_json))
    benchmark = evaluate_benchmark(cfg, load_json_fn(cfg.benchmark_report_json))
    session_matrix = evaluate_session_matrix(
        cfg, load_json_fn(cfg.session_matrix_report_json)
    )
    stream = evaluate_stream_health(cfg, cfg.runtime_log_file)

    failures = [
        *evolution["failures"],
        *benchmark["failures"],
        *session_matrix["failures"],
        *stream["failures"],
    ]

    finished_ts = time.time()
    finished_at = datetime.fromtimestamp(finished_ts, tz=UTC).isoformat()
    return {
        "metadata": {
            "started_at_utc": started_at,
            "finished_at_utc": finished_at,
            "duration_secs": round(finished_ts - started_ts, 3),
        },
        "inputs": {
            "evolution_report_json": str(cfg.evolution_report_json),
            "benchmark_report_json": str(cfg.benchmark_report_json),
            "session_matrix_report_json": str(cfg.session_matrix_report_json),
            "runtime_log_file": (
                str(cfg.runtime_log_file) if cfg.runtime_log_file else None
            ),
        },
        "thresholds": {
            "min_planned_hits": cfg.min_planned_hits,
            "min_successful_corrections": cfg.min_successful_corrections,
            "min_recall_credit_events": cfg.min_recall_credit_events,
            "min_quality_score": cfg.min_quality_score,
            "required_benchmark_modes": list(cfg.required_benchmark_modes),
            "min_query_turns": cfg.min_query_turns,
            "max_mode_tool_error_turns": cfg.max_mode_tool_error_turns,
            "max_total_tool_error_turns": cfg.max_total_tool_error_turns,
            "min_session_steps": cfg.min_session_steps,
            "max_session_failed_steps": cfg.max_session_failed_steps,
            "enable_stream_gate": cfg.enable_stream_gate,
            "min_stream_ack_ratio": cfg.min_stream_ack_ratio,
            "min_stream_published_events": cfg.min_stream_published_events,
            "max_stream_read_failed": cfg.max_stream_read_failed,
        },
        "checks": {
            "evolution": evolution,
            "benchmark": benchmark,
            "session_matrix": session_matrix,
            "stream": stream,
        },
        "overall_passed": len(failures) == 0,
        "failure_count": len(failures),
        "failures": failures,
    }
