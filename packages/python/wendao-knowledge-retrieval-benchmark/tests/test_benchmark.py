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
    assert scores["flat-topk"].exposed_item_count == 5
    assert scores["flat-topk"].disclosure_step_count == 0
    assert scores["graph-first-reasoning-tree"].exposed_item_count == 3
    assert scores["graph-first-reasoning-tree"].disclosure_step_count == 3
    assert scores["graph-first-reasoning-tree"].max_disclosure_depth == 2
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
    assert "Recommended profile: `none`" in render_markdown(report)
    assert "No knowledge scenarios were available" in render_markdown(report)


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
    markdown = output_markdown.read_text(encoding="utf-8")
    assert "Wendao Knowledge Retrieval Black-Box Benchmark" in markdown
    assert "`flat-topk`" in markdown
    assert "`graph-first-reasoning-tree`" in markdown


def write_receipt(path: Path, payload: dict[str, Any] | None = None) -> None:
    path.write_text(
        json.dumps(payload or compact_receipt(), indent=2, sort_keys=True),
        encoding="utf-8",
    )


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
