"""Knowledge retrieval black-box benchmark tests."""

from __future__ import annotations

import json
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path

from wendao_knowledge_retrieval_benchmark import build_benchmark_report
from wendao_knowledge_retrieval_benchmark.cli import main
from wendao_knowledge_retrieval_benchmark.reporting import render_markdown


def test_benchmark_report_compares_flat_topk_and_reasoning_tree(
    tmp_path: Path,
) -> None:
    receipt_path = tmp_path / "receipt.json"
    write_receipt(receipt_path)

    report = build_benchmark_report(receipt_path)
    repository = report.repositories[0]
    scores = {score.profile_id: score for score in repository.profile_scores}

    assert report.schema == "xiuxian_wendao.knowledge_retrieval_blackbox_benchmark.v1"
    assert repository.repo_id == "knowledge-repo"
    assert repository.recommended_profile_id == "graph-first-reasoning-tree"
    assert repository.scenario_recommendations[0].scenario_id == "scenario-a"
    assert (
        repository.scenario_recommendations[0].recommended_profile_id
        == "graph-first-reasoning-tree"
    )
    assert repository.scenario_recommendations[0].reason == "evidence_coverage_gain"
    assert scores["flat-topk"].exposed_item_count == 5
    assert scores["flat-topk"].disclosure_step_count == 0
    assert scores["flat-topk"].observed_evidence_kind_count == 1
    assert scores["flat-topk"].required_evidence_kind_count == 2
    assert scores["flat-topk"].evidence_coverage_bps == 5_000
    assert scores["flat-topk"].context_reduction_bps == 0
    assert scores["flat-topk"].scenario_scores[0].scenario_id == "scenario-a"
    assert scores["flat-topk"].scenario_scores[0].missing_evidence_kinds == [
        "relation_path"
    ]
    assert scores["graph-first-reasoning-tree"].exposed_item_count == 3
    assert scores["graph-first-reasoning-tree"].disclosure_step_count == 3
    assert scores["graph-first-reasoning-tree"].max_disclosure_depth == 2
    assert scores["graph-first-reasoning-tree"].observed_evidence_kind_count == 2
    assert scores["graph-first-reasoning-tree"].evidence_coverage_bps == 10_000
    assert scores["graph-first-reasoning-tree"].context_reduction_bps > 0
    assert (
        scores["graph-first-reasoning-tree"].scenario_scores[0].missing_evidence_kinds
        == []
    )
    assert scores["intent-tree-v1"].exposed_item_count == 9
    assert scores["intent-tree-v1"].disclosure_step_count == 3
    assert scores["intent-tree-v1"].max_disclosure_depth == 2
    assert scores["intent-tree-v1"].observed_evidence_kind_count == 2
    assert scores["intent-tree-v1"].evidence_coverage_bps == 10_000
    assert scores["intent-tree-v1"].context_reduction_bps == 0
    assert (
        scores["intent-tree-v1"].exposed_path_char_count
        > scores["graph-first-reasoning-tree"].exposed_path_char_count
    )
    assert (
        scores["graph-first-reasoning-tree"].promotion_score_bps
        > scores["flat-topk"].promotion_score_bps
    )


def test_benchmark_rejects_wrong_source_schema(tmp_path: Path) -> None:
    receipt = compact_receipt()
    receipt["schema"] = "unknown"
    receipt_path = tmp_path / "receipt.json"
    write_receipt(receipt_path, receipt)

    try:
        build_benchmark_report(receipt_path)
    except ValueError as error:
        assert "unsupported source receipt schema" in str(error)
    else:
        raise AssertionError("wrong receipt schema should fail")


def test_benchmark_does_not_recommend_profiles_without_knowledge_scenarios(
    tmp_path: Path,
) -> None:
    receipt = compact_receipt()
    repository = receipt["repositories"][0]
    repository["knowledge_scenarios"] = []
    receipt_path = tmp_path / "receipt.json"
    write_receipt(receipt_path, receipt)

    report = build_benchmark_report(receipt_path)
    repository_report = report.repositories[0]

    assert repository_report.profile_scores == []
    assert repository_report.recommended_profile_id is None
    assert repository_report.scenario_recommendations == []
    assert "Recommended profile: `none`" in render_markdown(report)
    assert "No knowledge scenarios were available" in render_markdown(report)


def test_scenario_recommendation_keeps_flat_topk_for_simple_exact_lookup(
    tmp_path: Path,
) -> None:
    receipt = compact_receipt()
    repository = receipt["repositories"][0]
    repository["query_receipts"][0]["observed_paths"] = ["docs/a.md"]
    repository["query_receipts"][1]["observed_paths"] = []
    scenario = repository["knowledge_scenarios"][0]
    scenario["intent_frame"] = {
        "task_kind": "known_item_lookup",
        "anchor_terms": ["agent host"],
        "required_evidence_kinds": ["source_path"],
        "relation_hypotheses": [],
        "authority_policy": [],
        "max_disclosure_depth": 1,
        "verifier_required": True,
    }
    scenario["reasoning_tree"] = {
        "passed": True,
        "disclosure_step_count": 1,
        "max_disclosure_depth": 1,
        "steps": [
            {
                "step_kind": "source_evidence",
                "evidence_id": "source:docs/a.md",
                "path": "docs/a.md",
                "disclosure_depth": 1,
            }
        ],
    }
    receipt_path = tmp_path / "receipt.json"
    write_receipt(receipt_path, receipt)

    report = build_benchmark_report(receipt_path)
    repository = report.repositories[0]
    recommendation = repository.scenario_recommendations[0]

    assert repository.recommended_profile_id == "flat-topk"
    assert recommendation.recommended_profile_id == "flat-topk"
    assert recommendation.reason == "flat_topk_exact_small_context"


def test_backend_frontier_profile_scores_rust_pruning_contract(
    tmp_path: Path,
) -> None:
    receipt = compact_receipt()
    scenario = receipt["repositories"][0]["knowledge_scenarios"][0]
    scenario["intent_frame"]["required_evidence_kinds"].append("authority_order")
    scenario["authority"] = {
        "passed": True,
        "preferred_path": "docs/a.md",
        "observed_top_path": "docs/a.md",
        "competing_paths": [],
    }
    scenario["backend_frontier"] = {
        "strategy": "rust_controlled_backend_frontier_v1",
        "control_plane_owner": "rust",
        "graph_backend": "rust-baseline",
        "graph_backend_live": False,
        "julia_schedule_basis": "static_warm_profile_projection_v1",
        "node_count": 5,
        "kept_node_count": 3,
        "pruned_node_count": 1,
        "expand_node_count": 1,
        "subagent_judgement_node_count": 1,
        "subagent_fanout_group_count": 1,
        "subagent_fanout_node_count": 1,
        "subagent_max_parallel_width": 1,
        "subagent_context_budget_chars": 960,
        "julia_candidate_node_count": 3,
        "julia_dispatch_node_count": 2,
        "julia_queue_node_count": 1,
        "julia_fallback_node_count": 0,
        "julia_reject_node_count": 0,
        "strategy_flow_projection_basis": "rust_receipt_projection_v1",
        "strategy_flow_candidate_node_count": 5,
        "strategy_flow_transition_node_count": 5,
        "strategy_flow_frontier_node_count": 3,
        "strategy_flow_context_budget_chars": 993,
        "strategy_flow_intent_complexity_class": "guarded_multi_hop",
        "strategy_flow_initial_topology": "acyclic_evidence_dag",
        "strategy_flow_refinement_topology": "cyclic_refinement_allowed",
        "strategy_flow_max_planned_depth": 3,
        "strategy_flow_loop_budget": 1,
        "strategy_flow_cycle_candidate_node_count": 1,
        "strategy_flow_llm_judgement_node_count": 1,
        "selected_beam_width": 4,
        "nodes": [
            frontier_node(
                "anchor",
                "anchor_query",
                "anchor_query",
                "expand",
                context_cost=12,
                subagent_fanout_group_id="subagent:scenario-a:depth:0",
                subagent_judgement_kind="branch_expand_candidate",
                subagent_priority_score_bps=8123,
                subagent_context_budget_chars=960,
                julia_algorithm_id="relationship_search.hnsw_semantic_fanout",
                julia_profile_id="wendao_graph_link_evidence",
                julia_schedule_action="dispatch",
                strategy_flow_frontier_rank=1,
                strategy_flow_context_budget_chars=960,
            ),
            frontier_node(
                "relation",
                "semantic_relation",
                "relation_path",
                "keep",
                context_cost=18,
                julia_algorithm_id="relationship_search.ppr_like_relatedness",
                julia_profile_id="wendao_graph_link_evidence",
                julia_schedule_action="dispatch",
                strategy_flow_frontier_rank=2,
                strategy_flow_context_budget_chars=18,
            ),
            frontier_node(
                "source",
                "source_evidence",
                "source_path",
                "keep",
                context_cost=15,
                julia_algorithm_id="relationship_search.graph_search_ranking",
                julia_profile_id="wendao_graph_page_index_reasoning",
                julia_schedule_action="queue",
                strategy_flow_frontier_rank=3,
                strategy_flow_context_budget_chars=15,
            ),
            frontier_node(
                "authority",
                "authority_order",
                "authority_order",
                "keep",
                context_cost=100,
            ),
            frontier_node(
                "noise",
                "source_evidence",
                "source_path",
                "prune",
                context_cost=200,
            ),
        ],
    }
    receipt_path = tmp_path / "receipt.json"
    write_receipt(receipt_path, receipt)

    report = build_benchmark_report(receipt_path)
    repository = report.repositories[0]
    scores = {score.profile_id: score for score in repository.profile_scores}

    assert "backend-frontier-pruning-v1" in scores
    frontier = scores["backend-frontier-pruning-v1"]
    strategy_flow = scores["search-strategy-flow-projection-v1"]
    assert frontier.exposed_item_count == 4
    assert frontier.exposed_path_char_count == 145
    assert frontier.observed_evidence_kind_count == 3
    assert frontier.subagent_fanout_group_count == 1
    assert frontier.subagent_fanout_node_count == 1
    assert frontier.subagent_max_parallel_width == 1
    assert frontier.subagent_context_budget_chars == 960
    assert frontier.julia_schedule_bases == ["static_warm_profile_projection_v1"]
    assert frontier.julia_algorithm_count == 3
    assert frontier.julia_profile_count == 2
    assert frontier.julia_candidate_node_count == 3
    assert frontier.julia_scheduled_node_count == 3
    assert frontier.julia_dispatch_node_count == 2
    assert frontier.julia_queue_node_count == 1
    assert frontier.julia_fallback_node_count == 0
    assert frontier.julia_reject_node_count == 0
    assert frontier.strategy_flow_projection_bases == ["rust_receipt_projection_v1"]
    assert frontier.strategy_flow_candidate_node_count == 5
    assert frontier.strategy_flow_transition_node_count == 5
    assert frontier.strategy_flow_frontier_node_count == 3
    assert frontier.strategy_flow_context_budget_chars == 993
    assert frontier.strategy_flow_complexity_classes == ["guarded_multi_hop"]
    assert frontier.strategy_flow_initial_topologies == ["acyclic_evidence_dag"]
    assert frontier.strategy_flow_refinement_topologies == ["cyclic_refinement_allowed"]
    assert frontier.strategy_flow_loop_budget == 1
    assert frontier.strategy_flow_cycle_candidate_node_count == 1
    assert frontier.strategy_flow_llm_judgement_node_count == 1
    assert strategy_flow.exposed_item_count == 3
    assert strategy_flow.exposed_path_char_count == 45
    assert strategy_flow.observed_evidence_kind_count == 3
    assert strategy_flow.strategy_flow_projection_bases == [
        "rust_receipt_projection_v1"
    ]
    assert strategy_flow.strategy_flow_candidate_node_count == 5
    assert strategy_flow.strategy_flow_transition_node_count == 5
    assert strategy_flow.strategy_flow_frontier_node_count == 3
    assert strategy_flow.strategy_flow_context_budget_chars == 993
    assert strategy_flow.strategy_flow_complexity_classes == ["guarded_multi_hop"]
    assert strategy_flow.strategy_flow_initial_topologies == ["acyclic_evidence_dag"]
    assert strategy_flow.strategy_flow_refinement_topologies == [
        "cyclic_refinement_allowed"
    ]
    assert strategy_flow.strategy_flow_loop_budget == 1
    assert strategy_flow.strategy_flow_cycle_candidate_node_count == 1
    assert strategy_flow.strategy_flow_llm_judgement_node_count == 1
    assert (
        strategy_flow.scenario_scores[0].strategy_flow_refinement_topology
        == "cyclic_refinement_allowed"
    )
    assert (
        strategy_flow.scenario_scores[0].strategy_flow_intent_complexity_class
        == "guarded_multi_hop"
    )
    assert (
        strategy_flow.scenario_scores[0].strategy_flow_initial_topology
        == "acyclic_evidence_dag"
    )
    assert strategy_flow.scenario_scores[0].strategy_flow_max_planned_depth == 3
    assert strategy_flow.scenario_scores[0].strategy_flow_candidate_node_count == 5
    assert strategy_flow.scenario_scores[0].strategy_flow_transition_node_count == 5
    assert strategy_flow.scenario_scores[0].strategy_flow_frontier_node_count == 3
    assert strategy_flow.scenario_scores[0].strategy_flow_loop_budget == 1
    assert (
        strategy_flow.scenario_scores[0].strategy_flow_cycle_candidate_node_count == 1
    )
    assert strategy_flow.scenario_scores[0].strategy_flow_llm_judgement_node_count == 1
    assert repository.recommended_profile_id == "search-strategy-flow-projection-v1"
    assert (
        repository.scenario_recommendations[0].recommended_profile_id
        == "search-strategy-flow-projection-v1"
    )


def test_cli_writes_json_and_markdown_outputs(tmp_path: Path) -> None:
    receipt_path = tmp_path / "receipt.json"
    output_json = tmp_path / "report.json"
    output_markdown = tmp_path / "report.md"
    write_receipt(receipt_path)

    status = main(
        [
            "--receipt",
            str(receipt_path),
            "--output-json",
            str(output_json),
            "--output-markdown",
            str(output_markdown),
        ]
    )

    assert status == 0
    payload = json.loads(output_json.read_text(encoding="utf-8"))
    assert payload["repositories"][0]["recommended_profile_id"] == (
        "graph-first-reasoning-tree"
    )
    assert (
        payload["repositories"][0]["scenario_recommendations"][0][
            "recommended_profile_id"
        ]
        == "graph-first-reasoning-tree"
    )
    scenario_scores = payload["repositories"][0]["profile_scores"][0]["scenario_scores"]
    assert scenario_scores[0]["scenario_id"] == "scenario-a"
    assert scenario_scores[0]["missing_evidence_kinds"] == ["relation_path"]
    assert (
        payload["repositories"][0]["profile_scores"][0]["julia_candidate_node_count"]
        == 0
    )
    assert (
        payload["repositories"][0]["profile_scores"][0]["subagent_fanout_node_count"]
        == 0
    )
    markdown = output_markdown.read_text(encoding="utf-8")
    assert "Wendao Knowledge Retrieval Black-Box Benchmark" in markdown
    assert "`flat-topk`" in markdown
    assert "`graph-first-reasoning-tree`" in markdown
    assert "`intent-tree-v1`" in markdown
    assert "Evidence" in markdown
    assert "Context cut" in markdown
    assert "Agent F/G/W" in markdown
    assert "Julia C/D/Q/F/R" in markdown
    assert "Flow C/T/F" in markdown
    assert "Flow Loop/LLM" in markdown
    assert "Flow topology" in markdown
    assert "Scenario Recommendations" in markdown
    assert "Scenario Diagnostics" in markdown


def write_receipt(path: Path, payload: dict[str, Any] | None = None) -> None:
    path.write_text(
        json.dumps(payload or compact_receipt(), indent=2, sort_keys=True),
        encoding="utf-8",
    )


def frontier_node(
    node_id: str,
    step_kind: str,
    evidence_kind: str,
    backend_action: str,
    *,
    context_cost: int,
    subagent_fanout_group_id: str | None = None,
    subagent_judgement_kind: str | None = None,
    subagent_priority_score_bps: int | None = None,
    subagent_context_budget_chars: int | None = None,
    julia_algorithm_id: str | None = None,
    julia_profile_id: str | None = None,
    julia_schedule_action: str | None = None,
    strategy_flow_frontier_rank: int | None = None,
    strategy_flow_context_budget_chars: int | None = None,
) -> dict[str, Any]:
    return {
        "node_id": node_id,
        "parent_node_id": None,
        "reasoning_step_index": None,
        "step_kind": step_kind,
        "evidence_kind": evidence_kind,
        "evidence_id": node_id,
        "query_id": None,
        "path": "docs/a.md" if evidence_kind == "source_path" else None,
        "relation": (
            {"source": "a", "kind": "governs", "target": "b"}
            if evidence_kind == "relation_path"
            else None
        ),
        "semantic_object_id": None,
        "disclosure_depth": 1,
        "parallel_group": "scenario:scenario-a:depth:1",
        "graph_batch_key": f"multi_hop_relation:{step_kind}",
        "graph_score_bps": 9_000,
        "authority_score_bps": 7_000,
        "coverage_score_bps": 10_000,
        "context_cost": context_cost,
        "backend_action": backend_action,
        "requires_subagent_judgement": backend_action == "expand",
        "subagent_prompt_hint": None,
        "subagent_fanout_group_id": subagent_fanout_group_id,
        "subagent_judgement_kind": subagent_judgement_kind,
        "subagent_priority_score_bps": subagent_priority_score_bps,
        "subagent_context_budget_chars": subagent_context_budget_chars,
        "julia_algorithm_id": julia_algorithm_id,
        "julia_profile_id": julia_profile_id,
        "julia_capability": (
            "graph_evidence_compute" if julia_algorithm_id is not None else None
        ),
        "julia_schedule_action": julia_schedule_action,
        "julia_schedule_reason": (
            "julia_advantage" if julia_schedule_action is not None else None
        ),
        "julia_schedule_confidence_score": (
            200 if julia_schedule_action is not None else None
        ),
        "julia_selected_batch_size": 1 if julia_schedule_action is not None else None,
        "strategy_flow_candidate_id": f"strategy-flow:candidate:{node_id}",
        "strategy_flow_transition_id": f"strategy-flow:transition:{node_id}:{backend_action}",
        "strategy_flow_action": backend_action,
        "strategy_flow_score_bps": 8_000 if backend_action != "prune" else 0,
        "strategy_flow_frontier_rank": strategy_flow_frontier_rank,
        "strategy_flow_context_budget_chars": strategy_flow_context_budget_chars,
        "strategy_flow_step_role": strategy_flow_step_role(evidence_kind),
        "strategy_flow_iteration_policy": strategy_flow_iteration_policy(
            evidence_kind,
            backend_action,
        ),
        "strategy_flow_loop_candidate": (
            backend_action != "prune"
            and evidence_kind in {"relation_path", "page_index_seed"}
        ),
        "strategy_flow_requires_llm_judgement": backend_action == "expand",
    }


def strategy_flow_step_role(evidence_kind: str) -> str:
    match evidence_kind:
        case "anchor_query":
            return "intent_anchor"
        case "relation_path":
            return "relation_refinement"
        case "page_index_seed":
            return "page_index_grounding"
        case "source_path":
            return "source_materialization"
        case "authority_order" | "negative_guard":
            return "validation_guard"
        case _:
            return "unknown"


def strategy_flow_iteration_policy(evidence_kind: str, backend_action: str) -> str:
    if backend_action == "prune":
        return "closed"
    match evidence_kind:
        case "anchor_query":
            return "expand_once"
        case "relation_path" | "page_index_seed":
            return "can_revisit"
        case "source_path":
            return "terminal_materialization"
        case "authority_order" | "negative_guard":
            return "guard_only"
        case _:
            return "single_pass"


def compact_receipt() -> dict[str, Any]:
    return {
        "schema": "xiuxian_wendao.real_repo_search_precision.v1",
        "repositories": [
            {
                "repo_id": "knowledge-repo",
                "total_ms": 42,
                "query_receipts": [
                    {
                        "query_id": "query-a",
                        "query_ms": 7,
                        "observed_paths": [
                            "docs/long/context/a.md",
                            "docs/long/context/b.md",
                            "docs/long/context/c.md",
                        ],
                    },
                    {
                        "query_id": "query-b",
                        "query_ms": 5,
                        "observed_paths": [
                            "docs/long/context/d.md",
                            "docs/long/context/e.md",
                        ],
                    },
                ],
                "knowledge_scenarios": [
                    {
                        "scenario_id": "scenario-a",
                        "passed": True,
                        "linked_query_ids": ["query-a"],
                        "query_variants": [{"query_id": "query-b"}],
                        "intent_frame": {
                            "task_kind": "multi_hop_relation",
                            "anchor_terms": ["explain", "governs"],
                            "required_evidence_kinds": [
                                "source_path",
                                "relation_path",
                            ],
                            "relation_hypotheses": [
                                {
                                    "source": "a",
                                    "kind": "governs",
                                    "target": "b",
                                }
                            ],
                            "authority_policy": ["semantic_ssot_before_package_docs"],
                            "max_disclosure_depth": 2,
                            "verifier_required": True,
                        },
                        "query_evidence": [
                            {
                                "query_id": "query-a",
                                "query_ms": 7,
                            },
                            {
                                "query_id": "query-b",
                                "query_ms": 5,
                            },
                        ],
                        "required_path_recall_at_1_bps": 10_000,
                        "required_path_recall_at_3_bps": 10_000,
                        "required_path_recall_at_5_bps": 10_000,
                        "required_path_recall_at_10_bps": 10_000,
                        "mean_required_path_reciprocal_rank_bps": 10_000,
                        "reasoning_tree": {
                            "passed": True,
                            "disclosure_step_count": 3,
                            "max_disclosure_depth": 2,
                            "steps": [
                                {
                                    "step_kind": "anchor_query",
                                    "evidence_id": "anchor:query-a",
                                    "query_id": "query-a",
                                    "disclosure_depth": 0,
                                },
                                {
                                    "step_kind": "semantic_relation",
                                    "evidence_id": "relation:a:governs:b",
                                    "relation": {
                                        "source": "a",
                                        "kind": "governs",
                                        "target": "b",
                                    },
                                    "disclosure_depth": 1,
                                },
                                {
                                    "step_kind": "source_evidence",
                                    "evidence_id": "source:docs/a.md",
                                    "path": "docs/a.md",
                                    "disclosure_depth": 2,
                                },
                            ],
                        },
                    }
                ],
            }
        ],
    }
