"""Report rendering for Wendao knowledge retrieval black-box benchmarks."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .models import BenchmarkReport, ProfileScore


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
                "| Profile | Passed | Recall@10 | MRR | Exposed chars | Steps | Max depth | Query ms | Score |",
                "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
            ]
        )
        if repository.profile_scores:
            lines.extend(_profile_row(score) for score in repository.profile_scores)
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
        f"| {score.exposed_path_char_count} "
        f"| {score.disclosure_step_count} "
        f"| {score.max_disclosure_depth} "
        f"| {score.total_query_ms} "
        f"| {score.promotion_score_bps} |"
    )
