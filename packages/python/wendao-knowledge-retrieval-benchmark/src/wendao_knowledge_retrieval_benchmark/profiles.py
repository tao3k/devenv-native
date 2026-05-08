"""Profile scoring for black-box Wendao knowledge retrieval benchmark rows."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from collections.abc import Callable

from .models import ProfileScenarioScore, ProfileScore, ScenarioProfileRecommendation
from .receipt import query_receipts_by_id

FLAT_TOPK_PROFILE_ID = "flat-topk"
GRAPH_FIRST_PROFILE_ID = "graph-first-reasoning-tree"
INTENT_TREE_PROFILE_ID = "intent-tree-v1"
BACKEND_FRONTIER_PROFILE_ID = "backend-frontier-pruning-v1"
SEARCH_STRATEGY_FLOW_PROFILE_ID = "search-strategy-flow-projection-v1"


def score_flat_topk(repository: dict[str, Any]) -> ProfileScore:
    scenarios = _scenarios(repository)
    queries_by_id = query_receipts_by_id(repository)
    exposed_paths: list[str] = []
    total_query_ms = 0
    scenario_scores = []

    for scenario in scenarios:
        scenario_paths: list[str] = []
        for query_id in _scenario_query_ids(scenario):
            query = queries_by_id.get(query_id)
            if query is None:
                continue
            query_ms = int(query.get("query_ms", 0))
            paths = [str(path) for path in query.get("observed_paths", [])]
            total_query_ms += query_ms
            exposed_paths.extend(paths)
            scenario_paths.extend(paths)
        scenario_scores.append(
            _scenario_score(
                profile_id=FLAT_TOPK_PROFILE_ID,
                scenario=scenario,
                observed_evidence_kinds=_flat_topk_evidence_kinds(scenario),
                exposed_item_count=len(scenario_paths),
                exposed_path_char_count=sum(len(path) for path in scenario_paths),
                disclosure_step_count=0,
                max_disclosure_depth=0,
                baseline_exposed_path_char_count=0,
            )
        )

    evidence = _evidence_counts(scenarios, _flat_topk_evidence_kinds)
    return _profile_score(
        profile_id=FLAT_TOPK_PROFILE_ID,
        scenarios=scenarios,
        total_query_ms=total_query_ms,
        exposed_item_count=len(exposed_paths),
        exposed_path_char_count=sum(len(path) for path in exposed_paths),
        disclosure_step_count=0,
        max_disclosure_depth=0,
        evidence=evidence,
        context_reduction_bps=0,
        scenario_scores=scenario_scores,
    )


def score_graph_first_reasoning_tree(
    repository: dict[str, Any],
    *,
    baseline_exposed_path_char_count: int = 0,
) -> ProfileScore:
    scenarios = _scenarios(repository)
    total_query_ms = 0
    exposed_item_count = 0
    exposed_path_char_count = 0
    disclosure_step_count = 0
    max_disclosure_depth = 0
    scenario_scores = []

    for scenario in scenarios:
        for evidence in scenario.get("query_evidence", []):
            total_query_ms += int(evidence.get("query_ms", 0))
        tree = scenario.get("reasoning_tree", {})
        steps = tree.get("steps", [])
        scenario_exposed_path_char_count = sum(
            _reasoning_step_char_cost(step) for step in steps
        )
        exposed_item_count += len(steps)
        disclosure_step_count += int(tree.get("disclosure_step_count", len(steps)))
        max_disclosure_depth = max(
            max_disclosure_depth,
            int(tree.get("max_disclosure_depth", 0)),
        )
        exposed_path_char_count += scenario_exposed_path_char_count
        scenario_scores.append(
            _scenario_score(
                profile_id=GRAPH_FIRST_PROFILE_ID,
                scenario=scenario,
                observed_evidence_kinds=_observed_evidence_kinds(scenario),
                exposed_item_count=len(steps),
                exposed_path_char_count=scenario_exposed_path_char_count,
                disclosure_step_count=int(
                    tree.get("disclosure_step_count", len(steps))
                ),
                max_disclosure_depth=int(tree.get("max_disclosure_depth", 0)),
                baseline_exposed_path_char_count=_flat_topk_scenario_char_count(
                    repository,
                    scenario,
                ),
            )
        )

    evidence = _evidence_counts(scenarios, _observed_evidence_kinds)
    return _profile_score(
        profile_id=GRAPH_FIRST_PROFILE_ID,
        scenarios=scenarios,
        total_query_ms=total_query_ms,
        exposed_item_count=exposed_item_count,
        exposed_path_char_count=exposed_path_char_count,
        disclosure_step_count=disclosure_step_count,
        max_disclosure_depth=max_disclosure_depth,
        evidence=evidence,
        context_reduction_bps=_context_reduction_bps(
            baseline_exposed_path_char_count,
            exposed_path_char_count,
        ),
        scenario_scores=scenario_scores,
    )


def score_intent_tree(
    repository: dict[str, Any],
    *,
    baseline_exposed_path_char_count: int = 0,
) -> ProfileScore:
    scenarios = _scenarios(repository)
    total_query_ms = 0
    exposed_item_count = 0
    exposed_path_char_count = 0
    disclosure_step_count = 0
    max_disclosure_depth = 0
    evidence_penalty_bps = 0
    scenario_scores = []

    for scenario in scenarios:
        for evidence in scenario.get("query_evidence", []):
            total_query_ms += int(evidence.get("query_ms", 0))
        tree = scenario.get("reasoning_tree", {})
        steps = tree.get("steps", [])
        frame = scenario.get("intent_frame", {})
        scenario_exposed_path_char_count = sum(
            _reasoning_step_char_cost(step) for step in steps
        ) + _intent_frame_char_cost(frame)
        exposed_item_count += len(steps) + _intent_frame_item_count(frame)
        disclosure_step_count += int(tree.get("disclosure_step_count", len(steps)))
        max_disclosure_depth = max(
            max_disclosure_depth,
            int(tree.get("max_disclosure_depth", 0)),
            int(frame.get("max_disclosure_depth", 0)) if isinstance(frame, dict) else 0,
        )
        exposed_path_char_count += scenario_exposed_path_char_count
        evidence_penalty_bps += _intent_evidence_penalty_bps(frame, scenario)
        scenario_scores.append(
            _scenario_score(
                profile_id=INTENT_TREE_PROFILE_ID,
                scenario=scenario,
                observed_evidence_kinds=_observed_evidence_kinds(scenario),
                exposed_item_count=len(steps) + _intent_frame_item_count(frame),
                exposed_path_char_count=scenario_exposed_path_char_count,
                disclosure_step_count=int(
                    tree.get("disclosure_step_count", len(steps))
                ),
                max_disclosure_depth=max(
                    int(tree.get("max_disclosure_depth", 0)),
                    (
                        int(frame.get("max_disclosure_depth", 0))
                        if isinstance(frame, dict)
                        else 0
                    ),
                ),
                baseline_exposed_path_char_count=_flat_topk_scenario_char_count(
                    repository,
                    scenario,
                ),
            )
        )

    evidence = _evidence_counts(scenarios, _observed_evidence_kinds)
    return _profile_score(
        profile_id=INTENT_TREE_PROFILE_ID,
        scenarios=scenarios,
        total_query_ms=total_query_ms,
        exposed_item_count=exposed_item_count,
        exposed_path_char_count=exposed_path_char_count,
        disclosure_step_count=disclosure_step_count,
        max_disclosure_depth=max_disclosure_depth,
        evidence=evidence,
        context_reduction_bps=_context_reduction_bps(
            baseline_exposed_path_char_count,
            exposed_path_char_count,
        ),
        scenario_scores=scenario_scores,
        extra_penalty_bps=evidence_penalty_bps,
    )


def score_backend_frontier_pruning(
    repository: dict[str, Any],
    *,
    baseline_exposed_path_char_count: int = 0,
) -> ProfileScore:
    scenarios = _scenarios(repository)
    total_query_ms = 0
    exposed_item_count = 0
    exposed_path_char_count = 0
    disclosure_step_count = 0
    max_disclosure_depth = 0
    scenario_scores = []

    for scenario in scenarios:
        for evidence in scenario.get("query_evidence", []):
            total_query_ms += int(evidence.get("query_ms", 0))
        nodes = _selected_backend_frontier_nodes(scenario)
        scenario_context_cost = sum(_frontier_node_context_cost(node) for node in nodes)
        scenario_depth = max(
            (int(node.get("disclosure_depth", 0)) for node in nodes),
            default=0,
        )
        exposed_item_count += len(nodes)
        exposed_path_char_count += scenario_context_cost
        disclosure_step_count += len(nodes)
        max_disclosure_depth = max(max_disclosure_depth, scenario_depth)
        scenario_scores.append(
            _scenario_score(
                profile_id=BACKEND_FRONTIER_PROFILE_ID,
                scenario=scenario,
                observed_evidence_kinds=_backend_frontier_evidence_kinds(scenario),
                exposed_item_count=len(nodes),
                exposed_path_char_count=scenario_context_cost,
                disclosure_step_count=len(nodes),
                max_disclosure_depth=scenario_depth,
                baseline_exposed_path_char_count=_flat_topk_scenario_char_count(
                    repository,
                    scenario,
                ),
            )
        )

    evidence = _evidence_counts(scenarios, _backend_frontier_evidence_kinds)
    schedule = _backend_frontier_schedule_diagnostics(scenarios)
    subagent = _backend_frontier_subagent_diagnostics(scenarios)
    strategy_flow = _backend_frontier_strategy_flow_diagnostics(scenarios)
    return _profile_score(
        profile_id=BACKEND_FRONTIER_PROFILE_ID,
        scenarios=scenarios,
        total_query_ms=total_query_ms,
        exposed_item_count=exposed_item_count,
        exposed_path_char_count=exposed_path_char_count,
        disclosure_step_count=disclosure_step_count,
        max_disclosure_depth=max_disclosure_depth,
        evidence=evidence,
        context_reduction_bps=_context_reduction_bps(
            baseline_exposed_path_char_count,
            exposed_path_char_count,
        ),
        scenario_scores=scenario_scores,
        subagent_fanout_group_count=subagent["fanout_group_count"],
        subagent_fanout_node_count=subagent["fanout_node_count"],
        subagent_max_parallel_width=subagent["max_parallel_width"],
        subagent_context_budget_chars=subagent["context_budget_chars"],
        julia_schedule_bases=schedule["bases"],
        julia_algorithm_count=schedule["algorithm_count"],
        julia_profile_count=schedule["profile_count"],
        julia_candidate_node_count=schedule["candidate_node_count"],
        julia_scheduled_node_count=schedule["scheduled_node_count"],
        julia_dispatch_node_count=schedule["dispatch_node_count"],
        julia_queue_node_count=schedule["queue_node_count"],
        julia_fallback_node_count=schedule["fallback_node_count"],
        julia_reject_node_count=schedule["reject_node_count"],
        strategy_flow_projection_bases=strategy_flow["bases"],
        strategy_flow_candidate_node_count=strategy_flow["candidate_node_count"],
        strategy_flow_transition_node_count=strategy_flow["transition_node_count"],
        strategy_flow_frontier_node_count=strategy_flow["frontier_node_count"],
        strategy_flow_context_budget_chars=strategy_flow["context_budget_chars"],
        strategy_flow_complexity_classes=strategy_flow["complexity_classes"],
        strategy_flow_initial_topologies=strategy_flow["initial_topologies"],
        strategy_flow_refinement_topologies=strategy_flow["refinement_topologies"],
        strategy_flow_loop_budget=strategy_flow["loop_budget"],
        strategy_flow_cycle_candidate_node_count=strategy_flow[
            "cycle_candidate_node_count"
        ],
        strategy_flow_llm_judgement_node_count=strategy_flow[
            "llm_judgement_node_count"
        ],
    )


def repository_has_backend_frontier(repository: dict[str, Any]) -> bool:
    return any(
        isinstance(scenario.get("backend_frontier"), dict)
        for scenario in _scenarios(repository)
    )


def score_search_strategy_flow_projection(
    repository: dict[str, Any],
    *,
    baseline_exposed_path_char_count: int = 0,
) -> ProfileScore:
    scenarios = _scenarios(repository)
    total_query_ms = 0
    exposed_item_count = 0
    exposed_path_char_count = 0
    disclosure_step_count = 0
    max_disclosure_depth = 0
    scenario_scores = []

    for scenario in scenarios:
        for evidence in scenario.get("query_evidence", []):
            total_query_ms += int(evidence.get("query_ms", 0))
        nodes = _selected_strategy_flow_nodes(scenario)
        topology = _strategy_flow_scenario_topology(scenario)
        scenario_context_cost = sum(
            _strategy_flow_node_context_cost(node) for node in nodes
        )
        scenario_depth = max(
            (int(node.get("disclosure_depth", 0)) for node in nodes),
            default=0,
        )
        exposed_item_count += len(nodes)
        exposed_path_char_count += scenario_context_cost
        disclosure_step_count += len(nodes)
        max_disclosure_depth = max(max_disclosure_depth, scenario_depth)
        scenario_scores.append(
            _scenario_score(
                profile_id=SEARCH_STRATEGY_FLOW_PROFILE_ID,
                scenario=scenario,
                observed_evidence_kinds=_strategy_flow_evidence_kinds(scenario),
                exposed_item_count=len(nodes),
                exposed_path_char_count=scenario_context_cost,
                disclosure_step_count=len(nodes),
                max_disclosure_depth=scenario_depth,
                baseline_exposed_path_char_count=_flat_topk_scenario_char_count(
                    repository,
                    scenario,
                ),
                strategy_flow_intent_complexity_class=topology[
                    "intent_complexity_class"
                ],
                strategy_flow_initial_topology=topology["initial_topology"],
                strategy_flow_refinement_topology=topology["refinement_topology"],
                strategy_flow_max_planned_depth=topology["max_planned_depth"],
                strategy_flow_candidate_node_count=topology["candidate_node_count"],
                strategy_flow_transition_node_count=topology["transition_node_count"],
                strategy_flow_frontier_node_count=topology["frontier_node_count"],
                strategy_flow_loop_budget=topology["loop_budget"],
                strategy_flow_cycle_candidate_node_count=topology[
                    "cycle_candidate_node_count"
                ],
                strategy_flow_llm_judgement_node_count=topology[
                    "llm_judgement_node_count"
                ],
            )
        )

    evidence = _evidence_counts(scenarios, _strategy_flow_evidence_kinds)
    schedule = _backend_frontier_schedule_diagnostics(scenarios)
    subagent = _backend_frontier_subagent_diagnostics(scenarios)
    strategy_flow = _backend_frontier_strategy_flow_diagnostics(scenarios)
    return _profile_score(
        profile_id=SEARCH_STRATEGY_FLOW_PROFILE_ID,
        scenarios=scenarios,
        total_query_ms=total_query_ms,
        exposed_item_count=exposed_item_count,
        exposed_path_char_count=exposed_path_char_count,
        disclosure_step_count=disclosure_step_count,
        max_disclosure_depth=max_disclosure_depth,
        evidence=evidence,
        context_reduction_bps=_context_reduction_bps(
            baseline_exposed_path_char_count,
            exposed_path_char_count,
        ),
        scenario_scores=scenario_scores,
        subagent_fanout_group_count=subagent["fanout_group_count"],
        subagent_fanout_node_count=subagent["fanout_node_count"],
        subagent_max_parallel_width=subagent["max_parallel_width"],
        subagent_context_budget_chars=subagent["context_budget_chars"],
        julia_schedule_bases=schedule["bases"],
        julia_algorithm_count=schedule["algorithm_count"],
        julia_profile_count=schedule["profile_count"],
        julia_candidate_node_count=schedule["candidate_node_count"],
        julia_scheduled_node_count=schedule["scheduled_node_count"],
        julia_dispatch_node_count=schedule["dispatch_node_count"],
        julia_queue_node_count=schedule["queue_node_count"],
        julia_fallback_node_count=schedule["fallback_node_count"],
        julia_reject_node_count=schedule["reject_node_count"],
        strategy_flow_projection_bases=strategy_flow["bases"],
        strategy_flow_candidate_node_count=strategy_flow["candidate_node_count"],
        strategy_flow_transition_node_count=strategy_flow["transition_node_count"],
        strategy_flow_frontier_node_count=strategy_flow["frontier_node_count"],
        strategy_flow_context_budget_chars=strategy_flow["context_budget_chars"],
        strategy_flow_complexity_classes=strategy_flow["complexity_classes"],
        strategy_flow_initial_topologies=strategy_flow["initial_topologies"],
        strategy_flow_refinement_topologies=strategy_flow["refinement_topologies"],
        strategy_flow_loop_budget=strategy_flow["loop_budget"],
        strategy_flow_cycle_candidate_node_count=strategy_flow[
            "cycle_candidate_node_count"
        ],
        strategy_flow_llm_judgement_node_count=strategy_flow[
            "llm_judgement_node_count"
        ],
    )


def repository_has_search_strategy_flow_projection(repository: dict[str, Any]) -> bool:
    return any(
        _selected_strategy_flow_nodes(scenario) for scenario in _scenarios(repository)
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
            _profile_topology_preference(score),
            -score.exposed_path_char_count,
            -score.total_query_ms,
        ),
    ).profile_id


def recommended_repository_profile(
    scores: list[ProfileScore],
    recommendations: list[ScenarioProfileRecommendation],
) -> str | None:
    if not recommendations:
        return recommended_profile(scores)

    profile_scores = {score.profile_id: score for score in scores}
    counts: dict[str, int] = {}
    selected_scores: dict[str, int] = {}
    for recommendation in recommendations:
        profile_id = recommendation.recommended_profile_id
        if profile_id is None:
            continue
        counts[profile_id] = counts.get(profile_id, 0) + 1
        selected_scores[profile_id] = selected_scores.get(profile_id, 0) + (
            recommendation.selected_score_bps
        )
    if not counts:
        return recommended_profile(scores)

    def repository_key(profile_id: str) -> tuple[int, int, int, int, int]:
        score = profile_scores.get(profile_id)
        return (
            counts[profile_id],
            selected_scores[profile_id],
            score.promotion_score_bps if score is not None else 0,
            _profile_topology_preference(score) if score is not None else 0,
            -score.exposed_path_char_count if score is not None else 0,
        )

    return max(
        counts,
        key=repository_key,
    )


def scenario_profile_recommendations(
    scores: list[ProfileScore],
) -> list[ScenarioProfileRecommendation]:
    by_scenario: dict[str, list[ProfileScenarioScore]] = {}
    for score in scores:
        for scenario_score in score.scenario_scores:
            by_scenario.setdefault(scenario_score.scenario_id, []).append(
                scenario_score
            )

    recommendations = []
    for scenario_id, scenario_scores in sorted(by_scenario.items()):
        recommendations.append(
            _recommend_scenario_profile(scenario_id, scenario_scores)
        )
    return recommendations


def _recommend_scenario_profile(
    scenario_id: str,
    scores: list[ProfileScenarioScore],
) -> ScenarioProfileRecommendation:
    if not scores:
        return ScenarioProfileRecommendation(
            scenario_id=scenario_id,
            scenario_kind="",
            recommended_profile_id=None,
            reason="no_scenario_scores",
            selected_score_bps=0,
            candidate_count=0,
        )

    flat = next(
        (score for score in scores if score.profile_id == FLAT_TOPK_PROFILE_ID),
        None,
    )
    selected = max(scores, key=_scenario_selection_key)
    reason = "score_rank"

    if flat is not None and flat.passed and flat.evidence_coverage_bps == 10_000:
        non_flat = [
            score for score in scores if score.profile_id != FLAT_TOPK_PROFILE_ID
        ]
        best_non_flat = max(non_flat, key=_scenario_selection_key) if non_flat else None
        if best_non_flat is None or (
            best_non_flat.evidence_coverage_bps <= flat.evidence_coverage_bps
            and best_non_flat.context_reduction_bps < 3_000
        ):
            selected = flat
            reason = "flat_topk_exact_small_context"
        elif best_non_flat.context_reduction_bps >= 3_000:
            selected = best_non_flat
            reason = "context_reduction_gain"

    if flat is not None and selected.evidence_coverage_bps > flat.evidence_coverage_bps:
        reason = "evidence_coverage_gain"
    elif (
        selected.profile_id == SEARCH_STRATEGY_FLOW_PROFILE_ID
        and _scenario_topology_preference(selected) > 0
    ):
        reason = "strategy_flow_topology_gain"

    return ScenarioProfileRecommendation(
        scenario_id=scenario_id,
        scenario_kind=selected.scenario_kind,
        recommended_profile_id=selected.profile_id,
        reason=reason,
        selected_score_bps=_scenario_selection_score(selected),
        candidate_count=len(scores),
    )


def _scenario_selection_key(
    score: ProfileScenarioScore,
) -> tuple[int, int, int, int, int, int]:
    return (
        _scenario_selection_score(score),
        1 if score.passed else 0,
        score.evidence_coverage_bps,
        score.context_reduction_bps,
        _scenario_topology_preference(score),
        -score.exposed_path_char_count,
    )


def _scenario_selection_score(score: ProfileScenarioScore) -> int:
    pass_score = 10_000 if score.passed else 0
    context_reward_bps = min(score.context_reduction_bps, 3_000) // 2
    cost_penalty_bps = min(3_000, score.exposed_path_char_count // 25)
    step_penalty_bps = min(1_000, score.disclosure_step_count * 100)
    return max(
        0,
        pass_score
        + score.evidence_coverage_bps
        + context_reward_bps
        - cost_penalty_bps
        - step_penalty_bps,
    )


def _profile_topology_preference(score: ProfileScore) -> int:
    if score.profile_id != SEARCH_STRATEGY_FLOW_PROFILE_ID:
        return 0
    preference = 0
    if "cyclic_refinement_allowed" in score.strategy_flow_refinement_topologies:
        preference += 300
    if "iterative_graph_refinement" in score.strategy_flow_refinement_topologies:
        preference += 150
    preference += min(score.strategy_flow_loop_budget, 4) * 25
    preference += min(score.strategy_flow_cycle_candidate_node_count, 10) * 10
    preference += min(score.strategy_flow_llm_judgement_node_count, 20) * 5
    return preference


def _scenario_topology_preference(score: ProfileScenarioScore) -> int:
    if score.profile_id != SEARCH_STRATEGY_FLOW_PROFILE_ID:
        return 0
    preference = 0
    match score.strategy_flow_refinement_topology:
        case "cyclic_refinement_allowed":
            preference += 300
        case "iterative_graph_refinement":
            preference += 150
    preference += min(score.strategy_flow_loop_budget, 4) * 25
    preference += min(score.strategy_flow_cycle_candidate_node_count, 10) * 10
    preference += min(score.strategy_flow_llm_judgement_node_count, 20) * 5
    return preference


def _profile_score(
    *,
    profile_id: str,
    scenarios: list[dict[str, Any]],
    total_query_ms: int,
    exposed_item_count: int,
    exposed_path_char_count: int,
    disclosure_step_count: int,
    max_disclosure_depth: int,
    evidence: tuple[int, int, int, int],
    context_reduction_bps: int,
    scenario_scores: list[ProfileScenarioScore],
    extra_penalty_bps: int = 0,
    subagent_fanout_group_count: int = 0,
    subagent_fanout_node_count: int = 0,
    subagent_max_parallel_width: int = 0,
    subagent_context_budget_chars: int = 0,
    julia_schedule_bases: list[str] | None = None,
    julia_algorithm_count: int = 0,
    julia_profile_count: int = 0,
    julia_candidate_node_count: int = 0,
    julia_scheduled_node_count: int = 0,
    julia_dispatch_node_count: int = 0,
    julia_queue_node_count: int = 0,
    julia_fallback_node_count: int = 0,
    julia_reject_node_count: int = 0,
    strategy_flow_projection_bases: list[str] | None = None,
    strategy_flow_candidate_node_count: int = 0,
    strategy_flow_transition_node_count: int = 0,
    strategy_flow_frontier_node_count: int = 0,
    strategy_flow_context_budget_chars: int = 0,
    strategy_flow_complexity_classes: list[str] | None = None,
    strategy_flow_initial_topologies: list[str] | None = None,
    strategy_flow_refinement_topologies: list[str] | None = None,
    strategy_flow_loop_budget: int = 0,
    strategy_flow_cycle_candidate_node_count: int = 0,
    strategy_flow_llm_judgement_node_count: int = 0,
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
    (
        required_evidence_kind_count,
        observed_evidence_kind_count,
        missing_evidence_kind_count,
        evidence_coverage_bps,
    ) = evidence
    promotion_score_bps = max(
        0,
        ((pass_rate_bps + mean_recall_at_10_bps + mean_reciprocal_rank_bps) // 3)
        - cost_penalty_bps
        - latency_penalty_bps
        - extra_penalty_bps,
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
        required_evidence_kind_count=required_evidence_kind_count,
        observed_evidence_kind_count=observed_evidence_kind_count,
        missing_evidence_kind_count=missing_evidence_kind_count,
        evidence_coverage_bps=evidence_coverage_bps,
        context_reduction_bps=context_reduction_bps,
        scenario_scores=scenario_scores,
        promotion_score_bps=promotion_score_bps,
        subagent_fanout_group_count=subagent_fanout_group_count,
        subagent_fanout_node_count=subagent_fanout_node_count,
        subagent_max_parallel_width=subagent_max_parallel_width,
        subagent_context_budget_chars=subagent_context_budget_chars,
        julia_schedule_bases=julia_schedule_bases or [],
        julia_algorithm_count=julia_algorithm_count,
        julia_profile_count=julia_profile_count,
        julia_candidate_node_count=julia_candidate_node_count,
        julia_scheduled_node_count=julia_scheduled_node_count,
        julia_dispatch_node_count=julia_dispatch_node_count,
        julia_queue_node_count=julia_queue_node_count,
        julia_fallback_node_count=julia_fallback_node_count,
        julia_reject_node_count=julia_reject_node_count,
        strategy_flow_projection_bases=strategy_flow_projection_bases or [],
        strategy_flow_candidate_node_count=strategy_flow_candidate_node_count,
        strategy_flow_transition_node_count=strategy_flow_transition_node_count,
        strategy_flow_frontier_node_count=strategy_flow_frontier_node_count,
        strategy_flow_context_budget_chars=strategy_flow_context_budget_chars,
        strategy_flow_complexity_classes=strategy_flow_complexity_classes or [],
        strategy_flow_initial_topologies=strategy_flow_initial_topologies or [],
        strategy_flow_refinement_topologies=strategy_flow_refinement_topologies or [],
        strategy_flow_loop_budget=strategy_flow_loop_budget,
        strategy_flow_cycle_candidate_node_count=strategy_flow_cycle_candidate_node_count,
        strategy_flow_llm_judgement_node_count=strategy_flow_llm_judgement_node_count,
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


def _scenario_score(
    *,
    profile_id: str,
    scenario: dict[str, Any],
    observed_evidence_kinds: set[str],
    exposed_item_count: int,
    exposed_path_char_count: int,
    disclosure_step_count: int,
    max_disclosure_depth: int,
    baseline_exposed_path_char_count: int,
    strategy_flow_intent_complexity_class: str | None = None,
    strategy_flow_initial_topology: str | None = None,
    strategy_flow_refinement_topology: str | None = None,
    strategy_flow_max_planned_depth: int = 0,
    strategy_flow_candidate_node_count: int = 0,
    strategy_flow_transition_node_count: int = 0,
    strategy_flow_frontier_node_count: int = 0,
    strategy_flow_loop_budget: int = 0,
    strategy_flow_cycle_candidate_node_count: int = 0,
    strategy_flow_llm_judgement_node_count: int = 0,
) -> ProfileScenarioScore:
    required = _required_evidence_kinds(scenario)
    observed_required = required & observed_evidence_kinds
    missing = required - observed_evidence_kinds
    return ProfileScenarioScore(
        profile_id=profile_id,
        scenario_id=str(scenario.get("scenario_id", "")),
        scenario_kind=str(scenario.get("scenario_kind", "")),
        passed=scenario.get("passed") is True and not missing,
        required_evidence_kinds=sorted(required),
        observed_evidence_kinds=sorted(observed_required),
        missing_evidence_kinds=sorted(missing),
        evidence_coverage_bps=_ratio_bps(len(observed_required), len(required)),
        exposed_item_count=exposed_item_count,
        exposed_path_char_count=exposed_path_char_count,
        disclosure_step_count=disclosure_step_count,
        max_disclosure_depth=max_disclosure_depth,
        context_reduction_bps=_context_reduction_bps(
            baseline_exposed_path_char_count,
            exposed_path_char_count,
        ),
        strategy_flow_intent_complexity_class=strategy_flow_intent_complexity_class,
        strategy_flow_initial_topology=strategy_flow_initial_topology,
        strategy_flow_refinement_topology=strategy_flow_refinement_topology,
        strategy_flow_max_planned_depth=strategy_flow_max_planned_depth,
        strategy_flow_candidate_node_count=strategy_flow_candidate_node_count,
        strategy_flow_transition_node_count=strategy_flow_transition_node_count,
        strategy_flow_frontier_node_count=strategy_flow_frontier_node_count,
        strategy_flow_loop_budget=strategy_flow_loop_budget,
        strategy_flow_cycle_candidate_node_count=strategy_flow_cycle_candidate_node_count,
        strategy_flow_llm_judgement_node_count=strategy_flow_llm_judgement_node_count,
    )


def _flat_topk_scenario_char_count(
    repository: dict[str, Any],
    scenario: dict[str, Any],
) -> int:
    queries_by_id = query_receipts_by_id(repository)
    total = 0
    for query_id in _scenario_query_ids(scenario):
        query = queries_by_id.get(query_id)
        if query is None:
            continue
        total += sum(len(str(path)) for path in query.get("observed_paths", []))
    return total


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


def _intent_frame_item_count(frame: Any) -> int:
    if not isinstance(frame, dict):
        return 0
    return (
        len(_string_items(frame.get("anchor_terms")))
        + len(_string_items(frame.get("required_evidence_kinds")))
        + len(_dict_items(frame.get("relation_hypotheses")))
        + len(_string_items(frame.get("authority_policy")))
    )


def _intent_frame_char_cost(frame: Any) -> int:
    if not isinstance(frame, dict):
        return 0
    values = [
        frame.get("task_kind"),
        *_string_items(frame.get("anchor_terms")),
        *_string_items(frame.get("required_evidence_kinds")),
        *_string_items(frame.get("authority_policy")),
    ]
    relation_cost = sum(
        len(str(relation.get("source", "")))
        + len(str(relation.get("kind", "")))
        + len(str(relation.get("target", "")))
        for relation in _dict_items(frame.get("relation_hypotheses"))
    )
    return sum(len(str(value)) for value in values if value) + relation_cost


def _intent_evidence_penalty_bps(
    frame: Any,
    scenario: dict[str, Any],
) -> int:
    if not isinstance(frame, dict):
        return 1_000

    required = set(_string_items(frame.get("required_evidence_kinds")))
    if not required:
        return 500

    observed = _observed_evidence_kinds(scenario)
    missing_count = len(required - observed)
    if frame.get("verifier_required") is not True:
        missing_count += 1
    return missing_count * 250


def _evidence_counts(
    scenarios: list[dict[str, Any]],
    observed_kinds_for_scenario: Callable[[dict[str, Any]], set[str]],
) -> tuple[int, int, int, int]:
    required_count = 0
    observed_count = 0
    for scenario in scenarios:
        required = _required_evidence_kinds(scenario)
        observed = observed_kinds_for_scenario(scenario)
        required_count += len(required)
        observed_count += len(required & observed)
    missing_count = required_count - observed_count
    coverage_bps = _ratio_bps(observed_count, required_count)
    return required_count, observed_count, missing_count, coverage_bps


def _required_evidence_kinds(scenario: dict[str, Any]) -> set[str]:
    frame = scenario.get("intent_frame", {})
    if isinstance(frame, dict):
        return set(_string_items(frame.get("required_evidence_kinds")))
    return set()


def _flat_topk_evidence_kinds(scenario: dict[str, Any]) -> set[str]:
    required = _required_evidence_kinds(scenario)
    if "source_path" in required and (
        scenario.get("linked_query_ids") or scenario.get("query_evidence")
    ):
        return {"source_path"}
    return set()


def _context_reduction_bps(
    baseline_exposed_path_char_count: int,
    exposed_path_char_count: int,
) -> int:
    if baseline_exposed_path_char_count <= 0:
        return 0
    if exposed_path_char_count >= baseline_exposed_path_char_count:
        return 0
    reduced_chars = baseline_exposed_path_char_count - exposed_path_char_count
    return _ratio_bps(reduced_chars, baseline_exposed_path_char_count)


def _observed_evidence_kinds(scenario: dict[str, Any]) -> set[str]:
    observed: set[str] = set()
    tree = scenario.get("reasoning_tree", {})
    steps = tree.get("steps", []) if isinstance(tree, dict) else []
    for step in steps:
        if not isinstance(step, dict):
            continue
        match step.get("step_kind"):
            case "source_evidence":
                observed.add("source_path")
            case "semantic_object" | "page_index_seed":
                observed.add("semantic_object")
                observed.add("page_index_seed")
            case "semantic_relation":
                observed.add("relation_path")
            case "negative_guard":
                observed.add("negative_guard")
    if scenario.get("authority") is not None:
        observed.add("authority_order")
    if scenario.get("negative_guard") is not None:
        observed.add("negative_guard")
    return observed


def _selected_backend_frontier_nodes(scenario: dict[str, Any]) -> list[dict[str, Any]]:
    frontier = scenario.get("backend_frontier", {})
    nodes = frontier.get("nodes", []) if isinstance(frontier, dict) else []
    return [
        node
        for node in nodes
        if isinstance(node, dict) and node.get("backend_action") != "prune"
    ]


def _backend_frontier_evidence_kinds(scenario: dict[str, Any]) -> set[str]:
    observed: set[str] = set()
    for node in _selected_backend_frontier_nodes(scenario):
        match node.get("evidence_kind"):
            case "source_path":
                observed.add("source_path")
            case "relation_path":
                observed.add("relation_path")
            case "page_index_seed":
                observed.add("page_index_seed")
                observed.add("semantic_object")
            case "authority_order":
                observed.add("authority_order")
            case "negative_guard":
                observed.add("negative_guard")
    return observed


def _selected_strategy_flow_nodes(scenario: dict[str, Any]) -> list[dict[str, Any]]:
    nodes = [
        node
        for node in _selected_backend_frontier_nodes(scenario)
        if isinstance(node.get("strategy_flow_frontier_rank"), int)
    ]
    return sorted(
        nodes,
        key=lambda node: (
            int(node.get("strategy_flow_frontier_rank", 0)),
            str(node.get("node_id", "")),
        ),
    )


def _strategy_flow_evidence_kinds(scenario: dict[str, Any]) -> set[str]:
    observed: set[str] = set()
    for node in _selected_strategy_flow_nodes(scenario):
        match node.get("evidence_kind"):
            case "source_path":
                observed.add("source_path")
            case "relation_path":
                observed.add("relation_path")
            case "page_index_seed":
                observed.add("page_index_seed")
                observed.add("semantic_object")
            case "authority_order":
                observed.add("authority_order")
            case "negative_guard":
                observed.add("negative_guard")
    frontier = scenario.get("backend_frontier", {})
    nodes = frontier.get("nodes", []) if isinstance(frontier, dict) else []
    for node in nodes:
        if not isinstance(node, dict):
            continue
        if not isinstance(node.get("strategy_flow_candidate_id"), str):
            continue
        match node.get("evidence_kind"):
            case "authority_order":
                observed.add("authority_order")
            case "negative_guard":
                observed.add("negative_guard")
    return observed


def _strategy_flow_scenario_topology(
    scenario: dict[str, Any],
) -> dict[str, int | str | None]:
    frontier = scenario.get("backend_frontier", {})
    if not isinstance(frontier, dict):
        return {
            "intent_complexity_class": None,
            "initial_topology": None,
            "refinement_topology": None,
            "max_planned_depth": 0,
            "candidate_node_count": 0,
            "transition_node_count": 0,
            "frontier_node_count": 0,
            "loop_budget": 0,
            "cycle_candidate_node_count": 0,
            "llm_judgement_node_count": 0,
        }

    nodes = _dict_items(frontier.get("nodes", []))
    complexity_class = frontier.get("strategy_flow_intent_complexity_class")
    initial_topology = frontier.get("strategy_flow_initial_topology")
    refinement_topology = frontier.get("strategy_flow_refinement_topology")
    return {
        "intent_complexity_class": (
            complexity_class if isinstance(complexity_class, str) else None
        ),
        "initial_topology": (
            initial_topology if isinstance(initial_topology, str) else None
        ),
        "refinement_topology": (
            refinement_topology if isinstance(refinement_topology, str) else None
        ),
        "max_planned_depth": _non_negative_int(
            frontier.get("strategy_flow_max_planned_depth")
        ),
        "candidate_node_count": sum(
            1
            for node in nodes
            if isinstance(node.get("strategy_flow_candidate_id"), str)
        ),
        "transition_node_count": sum(
            1
            for node in nodes
            if isinstance(node.get("strategy_flow_transition_id"), str)
        ),
        "frontier_node_count": sum(
            1
            for node in nodes
            if isinstance(node.get("strategy_flow_frontier_rank"), int)
        ),
        "loop_budget": _non_negative_int(frontier.get("strategy_flow_loop_budget")),
        "cycle_candidate_node_count": _non_negative_int(
            frontier.get("strategy_flow_cycle_candidate_node_count")
        ),
        "llm_judgement_node_count": _non_negative_int(
            frontier.get("strategy_flow_llm_judgement_node_count")
        ),
    }


def _non_negative_int(value: Any) -> int:
    return (
        max(0, value) if isinstance(value, int) and not isinstance(value, bool) else 0
    )


def _frontier_node_context_cost(node: dict[str, Any]) -> int:
    if isinstance(node.get("context_cost"), int):
        return max(0, int(node["context_cost"]))
    values = [
        node.get("evidence_id"),
        node.get("query_id"),
        node.get("path"),
        node.get("semantic_object_id"),
    ]
    relation = node.get("relation")
    if isinstance(relation, dict):
        values.extend(
            [relation.get("source"), relation.get("kind"), relation.get("target")]
        )
    return sum(len(str(value)) for value in values if value)


def _strategy_flow_node_context_cost(node: dict[str, Any]) -> int:
    context_cost = _frontier_node_context_cost(node)
    if context_cost > 0:
        return context_cost
    if isinstance(node.get("strategy_flow_context_budget_chars"), int):
        return max(0, int(node["strategy_flow_context_budget_chars"]))
    return 0


def _backend_frontier_schedule_diagnostics(
    scenarios: list[dict[str, Any]],
) -> dict[str, Any]:
    bases: set[str] = set()
    algorithms: set[str] = set()
    profiles: set[str] = set()
    candidate_node_count = 0
    scheduled_node_count = 0
    dispatch_node_count = 0
    queue_node_count = 0
    fallback_node_count = 0
    reject_node_count = 0

    for scenario in scenarios:
        frontier = scenario.get("backend_frontier", {})
        if not isinstance(frontier, dict):
            continue
        basis = frontier.get("julia_schedule_basis")
        if isinstance(basis, str) and basis:
            bases.add(basis)
        nodes = frontier.get("nodes", [])
        if not isinstance(nodes, list):
            continue
        for node in nodes:
            if not isinstance(node, dict):
                continue
            algorithm_id = node.get("julia_algorithm_id")
            profile_id = node.get("julia_profile_id")
            action = node.get("julia_schedule_action")
            if isinstance(algorithm_id, str) and algorithm_id:
                algorithms.add(algorithm_id)
                candidate_node_count += 1
            if isinstance(profile_id, str) and profile_id:
                profiles.add(profile_id)
            if isinstance(action, str) and action:
                scheduled_node_count += 1
                match action:
                    case "dispatch":
                        dispatch_node_count += 1
                    case "queue":
                        queue_node_count += 1
                    case "fallback":
                        fallback_node_count += 1
                    case "reject":
                        reject_node_count += 1

    return {
        "bases": sorted(bases),
        "algorithm_count": len(algorithms),
        "profile_count": len(profiles),
        "candidate_node_count": candidate_node_count,
        "scheduled_node_count": scheduled_node_count,
        "dispatch_node_count": dispatch_node_count,
        "queue_node_count": queue_node_count,
        "fallback_node_count": fallback_node_count,
        "reject_node_count": reject_node_count,
    }


def _backend_frontier_subagent_diagnostics(
    scenarios: list[dict[str, Any]],
) -> dict[str, int]:
    fanout_groups: dict[str, int] = {}
    fanout_node_count = 0
    context_budget_chars = 0

    for scenario in scenarios:
        frontier = scenario.get("backend_frontier", {})
        if not isinstance(frontier, dict):
            continue
        nodes = frontier.get("nodes", [])
        if not isinstance(nodes, list):
            continue
        for node in nodes:
            if not isinstance(node, dict):
                continue
            group_id = node.get("subagent_fanout_group_id")
            if not isinstance(group_id, str) or not group_id:
                continue
            fanout_groups[group_id] = fanout_groups.get(group_id, 0) + 1
            fanout_node_count += 1
            budget = node.get("subagent_context_budget_chars")
            if isinstance(budget, int):
                context_budget_chars += max(0, budget)

    return {
        "fanout_group_count": len(fanout_groups),
        "fanout_node_count": fanout_node_count,
        "max_parallel_width": max(fanout_groups.values(), default=0),
        "context_budget_chars": context_budget_chars,
    }


def _backend_frontier_strategy_flow_diagnostics(
    scenarios: list[dict[str, Any]],
) -> dict[str, Any]:
    bases: set[str] = set()
    complexity_classes: set[str] = set()
    initial_topologies: set[str] = set()
    refinement_topologies: set[str] = set()
    candidate_node_count = 0
    transition_node_count = 0
    frontier_node_count = 0
    context_budget_chars = 0
    loop_budget = 0
    cycle_candidate_node_count = 0
    llm_judgement_node_count = 0

    for scenario in scenarios:
        frontier = scenario.get("backend_frontier", {})
        if not isinstance(frontier, dict):
            continue
        basis = frontier.get("strategy_flow_projection_basis")
        if isinstance(basis, str) and basis:
            bases.add(basis)
        complexity_class = frontier.get("strategy_flow_intent_complexity_class")
        if isinstance(complexity_class, str) and complexity_class:
            complexity_classes.add(complexity_class)
        initial_topology = frontier.get("strategy_flow_initial_topology")
        if isinstance(initial_topology, str) and initial_topology:
            initial_topologies.add(initial_topology)
        refinement_topology = frontier.get("strategy_flow_refinement_topology")
        if isinstance(refinement_topology, str) and refinement_topology:
            refinement_topologies.add(refinement_topology)
        budget = frontier.get("strategy_flow_loop_budget")
        if isinstance(budget, int):
            loop_budget += max(0, budget)
        cycle_count = frontier.get("strategy_flow_cycle_candidate_node_count")
        if isinstance(cycle_count, int):
            cycle_candidate_node_count += max(0, cycle_count)
        llm_count = frontier.get("strategy_flow_llm_judgement_node_count")
        if isinstance(llm_count, int):
            llm_judgement_node_count += max(0, llm_count)
        nodes = frontier.get("nodes", [])
        if not isinstance(nodes, list):
            continue
        for node in nodes:
            if not isinstance(node, dict):
                continue
            if isinstance(node.get("strategy_flow_candidate_id"), str):
                candidate_node_count += 1
            if isinstance(node.get("strategy_flow_transition_id"), str):
                transition_node_count += 1
            if isinstance(node.get("strategy_flow_frontier_rank"), int):
                frontier_node_count += 1
            budget = node.get("strategy_flow_context_budget_chars")
            if isinstance(budget, int):
                context_budget_chars += max(0, budget)

    return {
        "bases": sorted(bases),
        "complexity_classes": sorted(complexity_classes),
        "initial_topologies": sorted(initial_topologies),
        "refinement_topologies": sorted(refinement_topologies),
        "candidate_node_count": candidate_node_count,
        "transition_node_count": transition_node_count,
        "frontier_node_count": frontier_node_count,
        "context_budget_chars": context_budget_chars,
        "loop_budget": loop_budget,
        "cycle_candidate_node_count": cycle_candidate_node_count,
        "llm_judgement_node_count": llm_judgement_node_count,
    }


def _string_items(value: Any) -> list[str]:
    if not isinstance(value, list):
        return []
    return [str(item) for item in value if isinstance(item, str)]


def _dict_items(value: Any) -> list[dict[str, Any]]:
    if not isinstance(value, list):
        return []
    return [item for item in value if isinstance(item, dict)]


def _mean_bps(scenarios: list[dict[str, Any]], key: str) -> int:
    if not scenarios:
        return 10_000
    return sum(int(scenario.get(key, 0)) for scenario in scenarios) // len(scenarios)


def _ratio_bps(value: int, total: int) -> int:
    if total == 0:
        return 10_000
    return (value * 10_000) // total
