from __future__ import annotations

import argparse

from wendao_core_lib import (
    WendaoTransportClient,
    WendaoTransportConfig,
    WendaoTransportEndpoint,
)
from xiuxian_wendao_analyzer import (
    AnalyzerConfig,
    run_repo_analysis,
    summarize_repo_analysis,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run a host-backed repo-search analyzer workflow.",
    )
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, required=True)
    parser.add_argument("--query-text", default="alpha")
    parser.add_argument("--limit", type=int, default=3)
    parser.add_argument("--path-prefix", action="append", default=["src/"])
    parser.add_argument("--schema-version", default="v2")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
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
        config=AnalyzerConfig(strategy="score_rank"),
    )
    summary = summarize_repo_analysis(run)

    print("query_text=", run.request.query_text)
    print("rows=", len(run.rows))
    print("top_path=", summary.top_path)
    print("top_rank=", summary.top_rank)


if __name__ == "__main__":
    main()
