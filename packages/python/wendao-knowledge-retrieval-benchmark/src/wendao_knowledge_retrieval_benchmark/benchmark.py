"""Build black-box benchmark reports from real-repo precision receipts."""

from __future__ import annotations

from typing import TYPE_CHECKING

from .models import REPORT_SCHEMA, BenchmarkReport, RepositoryBenchmark
from .profiles import (
    recommended_profile,
    score_flat_topk,
    score_graph_first_reasoning_tree,
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
            profile_scores = [
                score_flat_topk(repository),
                score_graph_first_reasoning_tree(repository),
            ]
        repositories.append(
            RepositoryBenchmark(
                repo_id=str(repository.get("repo_id", "")),
                source_total_ms=int(repository.get("total_ms", 0)),
                profile_scores=profile_scores,
                recommended_profile_id=recommended_profile(profile_scores),
            )
        )
    return BenchmarkReport(
        schema=REPORT_SCHEMA,
        source_receipt_schema=str(receipt.get("schema", "")),
        repositories=repositories,
    )
