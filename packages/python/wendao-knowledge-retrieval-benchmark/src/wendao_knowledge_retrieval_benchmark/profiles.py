"""Profile scoring for black-box Wendao knowledge retrieval benchmark rows."""

from __future__ import annotations

from typing import Any

from .models import ProfileScore
from .receipt import query_receipts_by_id

FLAT_TOPK_PROFILE_ID = "flat-topk"
GRAPH_FIRST_PROFILE_ID = "graph-first-reasoning-tree"


def score_flat_topk(repository: dict[str, Any]) -> ProfileScore:
    scenarios = _scenarios(repository)
    queries_by_id = query_receipts_by_id(repository)
    exposed_paths: list[str] = []
    total_query_ms = 0

    for scenario in scenarios:
        for query_id in _scenario_query_ids(scenario):
            query = queries_by_id.get(query_id)
            if query is None:
                continue
            total_query_ms += int(query.get("query_ms", 0))
            exposed_paths.extend(str(path) for path in query.get("observed_paths", []))

    return _profile_score(
        profile_id=FLAT_TOPK_PROFILE_ID,
        scenarios=scenarios,
        total_query_ms=total_query_ms,
        exposed_item_count=len(exposed_paths),
        exposed_path_char_count=sum(len(path) for path in exposed_paths),
        disclosure_step_count=0,
        max_disclosure_depth=0,
    )


def score_graph_first_reasoning_tree(repository: dict[str, Any]) -> ProfileScore:
    scenarios = _scenarios(repository)
    total_query_ms = 0
    exposed_item_count = 0
    exposed_path_char_count = 0
    disclosure_step_count = 0
    max_disclosure_depth = 0

    for scenario in scenarios:
        for evidence in scenario.get("query_evidence", []):
            total_query_ms += int(evidence.get("query_ms", 0))
        tree = scenario.get("reasoning_tree", {})
        steps = tree.get("steps", [])
        exposed_item_count += len(steps)
        disclosure_step_count += int(tree.get("disclosure_step_count", len(steps)))
        max_disclosure_depth = max(
            max_disclosure_depth,
            int(tree.get("max_disclosure_depth", 0)),
        )
        exposed_path_char_count += sum(
            _reasoning_step_char_cost(step) for step in steps
        )

    return _profile_score(
        profile_id=GRAPH_FIRST_PROFILE_ID,
        scenarios=scenarios,
        total_query_ms=total_query_ms,
        exposed_item_count=exposed_item_count,
        exposed_path_char_count=exposed_path_char_count,
        disclosure_step_count=disclosure_step_count,
        max_disclosure_depth=max_disclosure_depth,
    )


def recommended_profile(scores: list[ProfileScore]) -> str | None:
    if not scores:
        return None
    return max(
        scores,
        key=lambda score: (
            score.promotion_score_bps,
            score.passed_scenario_count,
            score.mean_recall_at_10_bps,
            score.mean_reciprocal_rank_bps,
            -score.exposed_path_char_count,
            -score.total_query_ms,
        ),
    ).profile_id


def _profile_score(
    *,
    profile_id: str,
    scenarios: list[dict[str, Any]],
    total_query_ms: int,
    exposed_item_count: int,
    exposed_path_char_count: int,
    disclosure_step_count: int,
    max_disclosure_depth: int,
) -> ProfileScore:
    scenario_count = len(scenarios)
    passed_scenario_count = sum(
        1 for scenario in scenarios if scenario.get("passed") is True
    )
    failed_scenario_count = scenario_count - passed_scenario_count
    mean_recall_at_1_bps = _mean_bps(scenarios, "required_path_recall_at_1_bps")
    mean_recall_at_3_bps = _mean_bps(scenarios, "required_path_recall_at_3_bps")
    mean_recall_at_5_bps = _mean_bps(scenarios, "required_path_recall_at_5_bps")
    mean_recall_at_10_bps = _mean_bps(scenarios, "required_path_recall_at_10_bps")
    mean_reciprocal_rank_bps = _mean_bps(
        scenarios,
        "mean_required_path_reciprocal_rank_bps",
    )
    pass_rate_bps = _ratio_bps(passed_scenario_count, scenario_count)
    cost_penalty_bps = min(2_000, exposed_path_char_count // 100)
    latency_penalty_bps = min(1_000, total_query_ms // 10)
    promotion_score_bps = max(
        0,
        ((pass_rate_bps + mean_recall_at_10_bps + mean_reciprocal_rank_bps) // 3)
        - cost_penalty_bps
        - latency_penalty_bps,
    )

    return ProfileScore(
        profile_id=profile_id,
        scenario_count=scenario_count,
        passed_scenario_count=passed_scenario_count,
        failed_scenario_count=failed_scenario_count,
        mean_recall_at_1_bps=mean_recall_at_1_bps,
        mean_recall_at_3_bps=mean_recall_at_3_bps,
        mean_recall_at_5_bps=mean_recall_at_5_bps,
        mean_recall_at_10_bps=mean_recall_at_10_bps,
        mean_reciprocal_rank_bps=mean_reciprocal_rank_bps,
        total_query_ms=total_query_ms,
        exposed_item_count=exposed_item_count,
        exposed_path_char_count=exposed_path_char_count,
        disclosure_step_count=disclosure_step_count,
        max_disclosure_depth=max_disclosure_depth,
        promotion_score_bps=promotion_score_bps,
    )


def _scenarios(repository: dict[str, Any]) -> list[dict[str, Any]]:
    return [
        scenario
        for scenario in repository.get("knowledge_scenarios", [])
        if isinstance(scenario, dict)
    ]


def _scenario_query_ids(scenario: dict[str, Any]) -> list[str]:
    query_ids = {str(query_id) for query_id in scenario.get("linked_query_ids", [])}
    for variant in scenario.get("query_variants", []):
        query_id = variant.get("query_id") if isinstance(variant, dict) else None
        if query_id:
            query_ids.add(str(query_id))
    return sorted(query_ids)


def _reasoning_step_char_cost(step: dict[str, Any]) -> int:
    values = [
        step.get("evidence_id"),
        step.get("query_id"),
        step.get("path"),
        step.get("semantic_object_id"),
    ]
    relation = step.get("relation")
    if isinstance(relation, dict):
        values.extend(
            [relation.get("source"), relation.get("kind"), relation.get("target")]
        )
    return sum(len(str(value)) for value in values if value)


def _mean_bps(scenarios: list[dict[str, Any]], key: str) -> int:
    if not scenarios:
        return 10_000
    return sum(int(scenario.get(key, 0)) for scenario in scenarios) // len(scenarios)


def _ratio_bps(value: int, total: int) -> int:
    if total == 0:
        return 10_000
    return (value * 10_000) // total
