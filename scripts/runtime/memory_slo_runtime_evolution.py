#!/usr/bin/env python3
"""Evolution-check evaluation for memory/session SLO aggregation."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from memory_slo_models import SloConfig


def evaluate_evolution(cfg: SloConfig, report: dict[str, Any]) -> dict[str, Any]:
    """Evaluate evolution report against thresholds."""
    failures: list[str] = []
    scenarios_obj = report.get("scenarios")
    scenarios = scenarios_obj if isinstance(scenarios_obj, list) else []
    if not bool(report.get("overall_passed", False)):
        failures.append("evolution.overall_passed=false")
    if not scenarios:
        failures.append("evolution.scenarios is empty")
        return {
            "passed": False,
            "failures": failures,
            "summary": {
                "scenario_count": 0,
                "min_planned_hits": 0,
                "min_successful_corrections": 0,
                "min_recall_credit_events": 0,
                "min_quality_score": 0.0,
            },
        }

    min_planned_hits: int | None = None
    min_successful_corrections: int | None = None
    min_recall_credit_events: int | None = None
    min_quality_score: float | None = None

    for scenario in scenarios:
        if not isinstance(scenario, dict):
            failures.append("evolution.scenario payload is not an object")
            continue
        scenario_id = str(scenario.get("scenario_id", "unknown"))
        quality_obj = scenario.get("quality")
        quality = quality_obj if isinstance(quality_obj, dict) else {}
        planned_hits = int(quality.get("planned_hits", 0))
        successful_corrections = int(quality.get("successful_corrections", 0))
        recall_credit_events = int(quality.get("recall_credit_events", 0))
        quality_score = float(quality.get("quality_score", 0.0))
        if not bool(scenario.get("quality_passed", True)):
            failures.append(f"evolution.{scenario_id}.quality_passed=false")
        if planned_hits < cfg.min_planned_hits:
            failures.append(
                f"evolution.{scenario_id}.planned_hits={planned_hits} < {cfg.min_planned_hits}"
            )
        if successful_corrections < cfg.min_successful_corrections:
            failures.append(
                "evolution."
                f"{scenario_id}.successful_corrections={successful_corrections} "
                f"< {cfg.min_successful_corrections}"
            )
        if recall_credit_events < cfg.min_recall_credit_events:
            failures.append(
                "evolution."
                f"{scenario_id}.recall_credit_events={recall_credit_events} "
                f"< {cfg.min_recall_credit_events}"
            )
        if quality_score < cfg.min_quality_score:
            failures.append(
                f"evolution.{scenario_id}.quality_score={quality_score:.2f} < "
                f"{cfg.min_quality_score:.2f}"
            )
        min_planned_hits = (
            planned_hits
            if min_planned_hits is None
            else min(min_planned_hits, planned_hits)
        )
        min_successful_corrections = (
            successful_corrections
            if min_successful_corrections is None
            else min(min_successful_corrections, successful_corrections)
        )
        min_recall_credit_events = (
            recall_credit_events
            if min_recall_credit_events is None
            else min(min_recall_credit_events, recall_credit_events)
        )
        min_quality_score = (
            quality_score
            if min_quality_score is None
            else min(min_quality_score, quality_score)
        )

    return {
        "passed": len(failures) == 0,
        "failures": failures,
        "summary": {
            "scenario_count": len(scenarios),
            "min_planned_hits": int(min_planned_hits or 0),
            "min_successful_corrections": int(min_successful_corrections or 0),
            "min_recall_credit_events": int(min_recall_credit_events or 0),
            "min_quality_score": round(float(min_quality_score or 0.0), 2),
        },
    }
