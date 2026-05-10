#!/usr/bin/env python3
"""Benchmark-check evaluation for memory/session SLO aggregation."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from memory_slo_models import SloConfig


def evaluate_benchmark(cfg: SloConfig, report: dict[str, Any]) -> dict[str, Any]:
    """Evaluate benchmark report against query/error thresholds."""
    failures: list[str] = []
    mode_summaries_obj = report.get("mode_summaries")
    mode_summaries = mode_summaries_obj if isinstance(mode_summaries_obj, dict) else {}
    if not mode_summaries:
        failures.append("benchmark.mode_summaries missing or empty")
    total_tool_error_turns = 0
    query_turns_by_mode: dict[str, int] = {}
    tool_errors_by_mode: dict[str, int] = {}

    for mode in cfg.required_benchmark_modes:
        summary_obj = mode_summaries.get(mode)
        if not isinstance(summary_obj, dict):
            failures.append(f"benchmark.mode={mode} missing")
            continue
        query_turns = int(summary_obj.get("query_turns", 0))
        tool_error_turns = int(summary_obj.get("tool_error_turns", 0))
        query_turns_by_mode[mode] = query_turns
        tool_errors_by_mode[mode] = tool_error_turns
        total_tool_error_turns += tool_error_turns
        if query_turns < cfg.min_query_turns:
            failures.append(
                f"benchmark.{mode}.query_turns={query_turns} < {cfg.min_query_turns}"
            )
        if tool_error_turns > cfg.max_mode_tool_error_turns:
            failures.append(
                "benchmark."
                f"{mode}.tool_error_turns={tool_error_turns} > {cfg.max_mode_tool_error_turns}"
            )

    if total_tool_error_turns > cfg.max_total_tool_error_turns:
        failures.append(
            f"benchmark.total_tool_error_turns={total_tool_error_turns} > "
            f"{cfg.max_total_tool_error_turns}"
        )

    return {
        "passed": len(failures) == 0,
        "failures": failures,
        "summary": {
            "required_modes": list(cfg.required_benchmark_modes),
            "query_turns_by_mode": query_turns_by_mode,
            "tool_error_turns_by_mode": tool_errors_by_mode,
            "total_tool_error_turns": total_tool_error_turns,
        },
    }
