"""Run a host-backed repo-search workflow with a custom score-rank analyzer."""

from __future__ import annotations

import argparse
import sys

from wendao_core_lib import (
    WendaoTransportClient,
    WendaoTransportConfig,
    WendaoTransportEndpoint,
)
from xiuxian_wendao_analyzer import run_repo_analysis, summarize_repo_analysis


class _HostBackedCustomScoreAnalyzer:
    def analyze_rows(self, rows: list[dict[str, object]]) -> list[dict[str, object]]:
        ranked = sorted(rows, key=lambda row: float(row["score"]), reverse=True)
        return [
            {
                "path": str(row["path"]),
                "score": float(row["score"]),
                "rank": index + 1,
            }
            for index, row in enumerate(ranked)
        ]


def _parse_custom_repo_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run a host-backed repo-search workflow with a custom Python analyzer.",
    )
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, required=True)
    parser.add_argument("--query-text", default="alpha")
    parser.add_argument("--limit", type=int, default=3)
    parser.add_argument("--path-prefix", action="append", default=["src/"])
    parser.add_argument("--schema-version", default="v2")
    return parser.parse_args()


def _emit(label: str, value: object) -> None:
    sys.stdout.write(f"{label}= {value}\n")


def _run_custom_repo_workflow() -> None:
    args = _parse_custom_repo_args()
    client = WendaoTransportClient(
        WendaoTransportConfig(
            endpoint=WendaoTransportEndpoint(host=args.host, port=args.port),
            schema_version=args.schema_version,
            request_timeout_seconds=10.0,
        )
    )
    run = run_repo_analysis(
        client,
        args.query_text,
        limit=args.limit,
        path_prefixes=tuple(args.path_prefix),
        analyzer=_HostBackedCustomScoreAnalyzer(),
    )
    summary = summarize_repo_analysis(run)

    _emit("query_text", run.request.query_text)
    _emit("rows", len(run.rows))
    _emit("top_path", summary.top_path)
    _emit("top_rank", summary.top_rank)


if __name__ == "__main__":
    _run_custom_repo_workflow()
