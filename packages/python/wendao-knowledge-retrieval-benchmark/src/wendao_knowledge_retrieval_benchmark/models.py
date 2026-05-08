"""Data records for Wendao knowledge retrieval black-box benchmark reports."""

from __future__ import annotations

from dataclasses import asdict, dataclass
from typing import Any

REPORT_SCHEMA = "xiuxian_wendao.knowledge_retrieval_blackbox_benchmark.v1"


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
    promotion_score_bps: int

    def to_json(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(frozen=True)
class RepositoryBenchmark:
    repo_id: str
    source_total_ms: int
    profile_scores: list[ProfileScore]
    recommended_profile_id: str | None

    def to_json(self) -> dict[str, Any]:
        return {
            "repo_id": self.repo_id,
            "source_total_ms": self.source_total_ms,
            "profile_scores": [score.to_json() for score in self.profile_scores],
            "recommended_profile_id": self.recommended_profile_id,
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
