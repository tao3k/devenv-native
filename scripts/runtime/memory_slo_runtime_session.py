#!/usr/bin/env python3
"""Session-matrix evaluation for memory/session SLO aggregation."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from memory_slo_models import SloConfig


def evaluate_session_matrix(cfg: SloConfig, report: dict[str, Any]) -> dict[str, Any]:
    """Evaluate session matrix report coverage/failures."""
    failures: list[str] = []
    summary_obj = report.get("summary")
    summary = summary_obj if isinstance(summary_obj, dict) else {}
    total = int(summary.get("total", 0))
    failed = int(summary.get("failed", 0))
    overall_passed = bool(report.get("overall_passed", False))
    if not overall_passed:
        failures.append("session_matrix.overall_passed=false")
    if total < cfg.min_session_steps:
        failures.append(
            f"session_matrix.summary.total={total} < {cfg.min_session_steps}"
        )
    if failed > cfg.max_session_failed_steps:
        failures.append(
            f"session_matrix.summary.failed={failed} > {cfg.max_session_failed_steps}"
        )
    return {
        "passed": len(failures) == 0,
        "failures": failures,
        "summary": {
            "total_steps": total,
            "failed_steps": failed,
            "overall_passed": overall_passed,
        },
    }
