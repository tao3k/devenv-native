"""Build black-box benchmark reports from real-repo precision receipts."""

from __future__ import annotations

from typing import TYPE_CHECKING

from .models import REPORT_SCHEMA, BenchmarkReport, RepositoryBenchmark
from .profiles import (
    recommended_repository_profile,
    repository_has_backend_frontier,
    repository_has_search_strategy_flow_projection,
    scenario_profile_recommendations,
    score_backend_frontier_pruning,
    score_flat_topk,
    score_graph_first_reasoning_tree,
    score_intent_tree,
    score_search_strategy_flow_projection,
)
from .receipt import load_receipt

if TYPE_CHECKING:
    from pathlib import Path


def build_benchmark_report(receipt_path: Path) -> BenchmarkReport:
    receipt = load_receipt(receipt_path)
    repositories = []
    for repository in receipt.get("repositories", []):
        profile_scores = []
        if repository.get("knowledge_scenarios"):
            flat_topk = score_flat_topk(repository)
            profile_scores = [
                flat_topk,
                score_graph_first_reasoning_tree(
                    repository,
                    baseline_exposed_path_char_count=flat_topk.exposed_path_char_count,
                ),
                score_intent_tree(
                    repository,
                    baseline_exposed_path_char_count=flat_topk.exposed_path_char_count,
                ),
            ]
            if repository_has_backend_frontier(repository):
                profile_scores.append(
                    score_backend_frontier_pruning(
                        repository,
                        baseline_exposed_path_char_count=flat_topk.exposed_path_char_count,
                    )
                )
            if repository_has_search_strategy_flow_projection(repository):
                profile_scores.append(
                    score_search_strategy_flow_projection(
                        repository,
                        baseline_exposed_path_char_count=flat_topk.exposed_path_char_count,
                    )
                )
        scenario_recommendations = scenario_profile_recommendations(profile_scores)
        repositories.append(
            RepositoryBenchmark(
                repo_id=str(repository.get("repo_id", "")),
                source_total_ms=int(repository.get("total_ms", 0)),
                profile_scores=profile_scores,
                recommended_profile_id=recommended_repository_profile(
                    profile_scores,
                    scenario_recommendations,
                ),
                scenario_recommendations=scenario_recommendations,
            )
        )
    return BenchmarkReport(
        schema=REPORT_SCHEMA,
        source_receipt_schema=str(receipt.get("schema", "")),
        repositories=repositories,
    )
