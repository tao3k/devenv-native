"""Report rendering for Wendao knowledge retrieval black-box benchmarks."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .models import (
        BenchmarkReport,
        ProfileScenarioScore,
        ProfileScore,
        ScenarioProfileRecommendation,
    )


def render_markdown(report: BenchmarkReport) -> str:
    lines = [
        "# Wendao Knowledge Retrieval Black-Box Benchmark",
        "",
        f"- Schema: `{report.schema}`",
        f"- Source receipt schema: `{report.source_receipt_schema}`",
        "",
    ]
    for repository in report.repositories:
        lines.extend(
            [
                f"## Repository `{repository.repo_id}`",
                "",
                f"- Source total ms: `{repository.source_total_ms}`",
                f"- Recommended profile: `{repository.recommended_profile_id or 'none'}`",
                "",
                "| Profile | Passed | Recall@10 | MRR | Evidence | Coverage | Context cut | Exposed chars | Steps | Max depth | Query ms | Agent F/G/W | Julia C/D/Q/F/R | Flow C/T/F | Flow Loop/LLM | Score |",
                "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
            ]
        )
        if repository.profile_scores:
            lines.extend(_profile_row(score) for score in repository.profile_scores)
            scenario_scores = [
                scenario_score
                for score in repository.profile_scores
                for scenario_score in score.scenario_scores
            ]
            if scenario_scores:
                lines.extend(
                    [
                        "",
                        "### Scenario Recommendations",
                        "",
                        "| Scenario | Recommended | Reason | Score | Candidates |",
                        "| --- | --- | --- | ---: | ---: |",
                    ]
                )
                lines.extend(
                    _scenario_recommendation_row(recommendation)
                    for recommendation in repository.scenario_recommendations
                )
                lines.extend(
                    [
                        "",
                        "### Scenario Diagnostics",
                        "",
                        "| Profile | Scenario | Evidence | Missing | Context cut | Chars | Steps | Depth | Flow topology | Flow C/T/F | Flow Loop/LLM |",
                        "| --- | --- | ---: | --- | ---: | ---: | ---: | ---: | --- | ---: | ---: |",
                    ]
                )
                lines.extend(_scenario_row(score) for score in scenario_scores)
        else:
            lines.append("")
            lines.append(
                "No knowledge scenarios were available for profile comparison."
            )
        lines.append("")
    return "\n".join(lines).rstrip() + "\n"


def _profile_row(score: ProfileScore) -> str:
    return (
        f"| `{score.profile_id}` "
        f"| {score.passed_scenario_count}/{score.scenario_count} "
        f"| {score.mean_recall_at_10_bps} "
        f"| {score.mean_reciprocal_rank_bps} "
        f"| {score.observed_evidence_kind_count}/{score.required_evidence_kind_count} "
        f"| {score.evidence_coverage_bps} "
        f"| {score.context_reduction_bps} "
        f"| {score.exposed_path_char_count} "
        f"| {score.disclosure_step_count} "
        f"| {score.max_disclosure_depth} "
        f"| {score.total_query_ms} "
        f"| {_subagent_fanout_summary(score)} "
        f"| {_julia_schedule_summary(score)} "
        f"| {_strategy_flow_summary(score)} "
        f"| {_strategy_flow_topology_summary(score)} "
        f"| {score.promotion_score_bps} |"
    )


def _scenario_row(score: ProfileScenarioScore) -> str:
    missing = ", ".join(score.missing_evidence_kinds) or "none"
    return (
        f"| `{score.profile_id}` "
        f"| `{score.scenario_id}` "
        f"| {len(score.observed_evidence_kinds)}/{len(score.required_evidence_kinds)} "
        f"| {missing} "
        f"| {score.context_reduction_bps} "
        f"| {score.exposed_path_char_count} "
        f"| {score.disclosure_step_count} "
        f"| {score.max_disclosure_depth} "
        f"| {_scenario_strategy_flow_topology_summary(score)} "
        f"| {_scenario_strategy_flow_summary(score)} "
        f"| {_scenario_strategy_flow_loop_summary(score)} |"
    )


def _scenario_recommendation_row(recommendation: ScenarioProfileRecommendation) -> str:
    return (
        f"| `{recommendation.scenario_id}` "
        f"| `{recommendation.recommended_profile_id or 'none'}` "
        f"| `{recommendation.reason}` "
        f"| {recommendation.selected_score_bps} "
        f"| {recommendation.candidate_count} |"
    )


def _julia_schedule_summary(score: ProfileScore) -> str:
    if score.julia_candidate_node_count == 0:
        return "none"
    return (
        f"{score.julia_candidate_node_count}/"
        f"{score.julia_dispatch_node_count}/"
        f"{score.julia_queue_node_count}/"
        f"{score.julia_fallback_node_count}/"
        f"{score.julia_reject_node_count}"
    )


def _subagent_fanout_summary(score: ProfileScore) -> str:
    if score.subagent_fanout_node_count == 0:
        return "none"
    return (
        f"{score.subagent_fanout_node_count}/"
        f"{score.subagent_fanout_group_count}/"
        f"{score.subagent_max_parallel_width}"
    )


def _strategy_flow_summary(score: ProfileScore) -> str:
    if score.strategy_flow_candidate_node_count == 0:
        return "none"
    return (
        f"{score.strategy_flow_candidate_node_count}/"
        f"{score.strategy_flow_transition_node_count}/"
        f"{score.strategy_flow_frontier_node_count}"
    )


def _strategy_flow_topology_summary(score: ProfileScore) -> str:
    if score.strategy_flow_candidate_node_count == 0:
        return "none"
    return (
        f"{score.strategy_flow_loop_budget}/"
        f"{score.strategy_flow_cycle_candidate_node_count}/"
        f"{score.strategy_flow_llm_judgement_node_count}"
    )


def _scenario_strategy_flow_topology_summary(score: ProfileScenarioScore) -> str:
    if score.strategy_flow_candidate_node_count == 0:
        return "none"
    complexity = score.strategy_flow_intent_complexity_class or "unknown"
    initial = score.strategy_flow_initial_topology or "unknown"
    refinement = score.strategy_flow_refinement_topology or "unknown"
    return (
        f"{complexity}:{initial}->{refinement}:{score.strategy_flow_max_planned_depth}"
    )


def _scenario_strategy_flow_summary(score: ProfileScenarioScore) -> str:
    if score.strategy_flow_candidate_node_count == 0:
        return "none"
    return (
        f"{score.strategy_flow_candidate_node_count}/"
        f"{score.strategy_flow_transition_node_count}/"
        f"{score.strategy_flow_frontier_node_count}"
    )


def _scenario_strategy_flow_loop_summary(score: ProfileScenarioScore) -> str:
    if score.strategy_flow_candidate_node_count == 0:
        return "none"
    return (
        f"{score.strategy_flow_loop_budget}/"
        f"{score.strategy_flow_cycle_candidate_node_count}/"
        f"{score.strategy_flow_llm_judgement_node_count}"
    )
