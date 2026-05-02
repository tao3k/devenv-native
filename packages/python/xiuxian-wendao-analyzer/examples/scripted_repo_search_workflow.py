"""Run a scripted repo-search analyzer workflow without a live endpoint."""

from __future__ import annotations

import sys

from wendao_arrow_interface import WendaoArrowScriptedClient, WendaoArrowSession
from xiuxian_wendao_analyzer import run_repo_analysis, summarize_repo_analysis


class _ScriptedRepoScoreAnalyzer:
    def analyze_rows(self, rows: list[dict[str, object]]) -> list[dict[str, object]]:
        ranked = sorted(rows, key=lambda row: float(row["score"]), reverse=True)
        return [
            {
                "doc_id": str(row["doc_id"]),
                "path": str(row["path"]),
                "score": float(row["score"]),
                "rank": index + 1,
            }
            for index, row in enumerate(ranked)
        ]


def _build_session() -> WendaoArrowSession:
    return WendaoArrowSession.for_repo_search_testing(
        [
            {"doc_id": "doc-alpha", "path": "src/alpha.py", "score": 0.91},
            {"doc_id": "doc-beta", "path": "docs/alpha.md", "score": 0.44},
            {"doc_id": "doc-gamma", "path": "src/beta.py", "score": 0.72},
        ]
    )


def _emit(label: str, value: object) -> None:
    sys.stdout.write(f"{label}= {value}\n")


def _run_scripted_repo_search_workflow() -> None:
    session = _build_session()
    if not isinstance(session.client, WendaoArrowScriptedClient):
        raise TypeError(
            "scripted example expects WendaoArrowSession.for_repo_search_testing()"
        )

    run = run_repo_analysis(
        session.client,
        "alpha",
        limit=3,
        analyzer=_ScriptedRepoScoreAnalyzer(),
    )
    summary = summarize_repo_analysis(run)
    recorded_call = session.client.calls[0]

    _emit("query_text", run.request.query_text)
    _emit("rows", len(run.rows))
    _emit("top_path", summary.top_path)
    _emit("top_rank", summary.top_rank)
    _emit("recorded_calls", len(session.client.calls))
    _emit("recorded_route", recorded_call.route)


if __name__ == "__main__":
    _run_scripted_repo_search_workflow()
