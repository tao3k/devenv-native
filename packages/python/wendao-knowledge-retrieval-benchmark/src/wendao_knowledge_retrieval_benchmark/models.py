"""Data records for Wendao knowledge retrieval black-box benchmark reports."""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from typing import Any

REPORT_SCHEMA = "xiuxian_wendao.knowledge_retrieval_blackbox_benchmark.v1"


@dataclass(frozen=True)
class ProfileScenarioScore:
    profile_id: str
    scenario_id: str
    scenario_kind: str
    passed: bool
    required_evidence_kinds: list[str]
    observed_evidence_kinds: list[str]
    missing_evidence_kinds: list[str]
    evidence_coverage_bps: int
    exposed_item_count: int
    exposed_path_char_count: int
    disclosure_step_count: int
    max_disclosure_depth: int
    context_reduction_bps: int
    strategy_flow_intent_complexity_class: str | None = None
    strategy_flow_initial_topology: str | None = None
    strategy_flow_refinement_topology: str | None = None
    strategy_flow_max_planned_depth: int = 0
    strategy_flow_candidate_node_count: int = 0
    strategy_flow_transition_node_count: int = 0
    strategy_flow_frontier_node_count: int = 0
    strategy_flow_loop_budget: int = 0
    strategy_flow_cycle_candidate_node_count: int = 0
    strategy_flow_llm_judgement_node_count: int = 0

    def to_json(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(frozen=True)
class ProfileScore:
    profile_id: str
    scenario_count: int
    passed_scenario_count: int
    failed_scenario_count: int
    mean_recall_at_1_bps: int
    mean_recall_at_3_bps: int
    mean_recall_at_5_bps: int
    mean_recall_at_10_bps: int
    mean_reciprocal_rank_bps: int
    total_query_ms: int
    exposed_item_count: int
    exposed_path_char_count: int
    disclosure_step_count: int
    max_disclosure_depth: int
    required_evidence_kind_count: int
    observed_evidence_kind_count: int
    missing_evidence_kind_count: int
    evidence_coverage_bps: int
    context_reduction_bps: int
    scenario_scores: list[ProfileScenarioScore]
    promotion_score_bps: int
    subagent_fanout_group_count: int = 0
    subagent_fanout_node_count: int = 0
    subagent_max_parallel_width: int = 0
    subagent_context_budget_chars: int = 0
    julia_schedule_bases: list[str] = field(default_factory=list)
    julia_algorithm_count: int = 0
    julia_profile_count: int = 0
    julia_candidate_node_count: int = 0
    julia_scheduled_node_count: int = 0
    julia_dispatch_node_count: int = 0
    julia_queue_node_count: int = 0
    julia_fallback_node_count: int = 0
    julia_reject_node_count: int = 0
    strategy_flow_projection_bases: list[str] = field(default_factory=list)
    strategy_flow_candidate_node_count: int = 0
    strategy_flow_transition_node_count: int = 0
    strategy_flow_frontier_node_count: int = 0
    strategy_flow_context_budget_chars: int = 0
    strategy_flow_complexity_classes: list[str] = field(default_factory=list)
    strategy_flow_initial_topologies: list[str] = field(default_factory=list)
    strategy_flow_refinement_topologies: list[str] = field(default_factory=list)
    strategy_flow_loop_budget: int = 0
    strategy_flow_cycle_candidate_node_count: int = 0
    strategy_flow_llm_judgement_node_count: int = 0

    def to_json(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(frozen=True)
class ScenarioProfileRecommendation:
    scenario_id: str
    scenario_kind: str
    recommended_profile_id: str | None
    reason: str
    selected_score_bps: int
    candidate_count: int

    def to_json(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(frozen=True)
class RepositoryBenchmark:
    repo_id: str
    source_total_ms: int
    profile_scores: list[ProfileScore]
    recommended_profile_id: str | None
    scenario_recommendations: list[ScenarioProfileRecommendation]

    def to_json(self) -> dict[str, Any]:
        return {
            "repo_id": self.repo_id,
            "source_total_ms": self.source_total_ms,
            "profile_scores": [score.to_json() for score in self.profile_scores],
            "recommended_profile_id": self.recommended_profile_id,
            "scenario_recommendations": [
                recommendation.to_json()
                for recommendation in self.scenario_recommendations
            ],
        }


@dataclass(frozen=True)
class BenchmarkReport:
    schema: str
    source_receipt_schema: str
    repositories: list[RepositoryBenchmark]

    def to_json(self) -> dict[str, Any]:
        return {
            "schema": self.schema,
            "source_receipt_schema": self.source_receipt_schema,
            "repositories": [repository.to_json() for repository in self.repositories],
        }
