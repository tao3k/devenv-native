"""Shared helpers for transport_runtime tests."""

from __future__ import annotations

import os
import socket
import subprocess
import time
from typing import TYPE_CHECKING

import pytest

from wendao_arrow_interface import (
    WendaoArrowCall,
    WendaoArrowResult,
    WendaoArrowScriptedClient,
)
from wendao_core_lib import (
    WendaoFlightRouteQuery,
    WendaoTransportClient,
    WendaoTransportConfig,
    WendaoTransportEndpoint,
    repo_search_metadata,
    repo_search_query,
    repo_search_request,
)
from xiuxian_wendao_analyzer import (
    AnalyzerConfig,
    analyze_query,
    analyze_query_results,
    analyze_repo_query_text,
    analyze_repo_query_text_results,
    analyze_repo_search,
    analyze_repo_search_results,
    run_query_analysis,
    run_repo_analysis,
    run_repo_search_analysis,
    summarize_query,
    summarize_query_results,
    summarize_query_route,
    summarize_repo_analysis,
    summarize_repo_query_text,
    summarize_repo_query_text_results,
    summarize_repo_search,
    summarize_repo_search_results,
)

from .support_integration import (
    _project_root,
    _run_rust_search_plane_seed_binary,
    _spawn_wendao_search_flight_server,
    _terminate_process,
    _wendao_search_flight_server_binary,
    _wendao_search_seed_binary,
)

if TYPE_CHECKING:
    import pyarrow as pa


def _score_rows() -> list[dict[str, object]]:
    return [
        {"path": "src/main.rs", "score": 0.3},
        {"path": "src/lib.rs", "score": 0.9},
    ]


def _result_table(route: str, rows: list[dict[str, object]]) -> pa.Table:
    return WendaoArrowResult.from_query_rows(rows, route=route).table


def _scripted_query_client(
    query: WendaoFlightRouteQuery,
    rows: list[dict[str, object]],
) -> WendaoArrowScriptedClient:
    return WendaoArrowScriptedClient.for_query_route(
        query.normalized_route(),
        _result_table(query.normalized_route(), rows),
    )


def _scripted_repo_search_query_client(
    rows: list[dict[str, object]],
) -> WendaoArrowScriptedClient:
    return _scripted_query_client(repo_search_query(), rows)


def _assert_single_query_call(
    client: WendaoArrowScriptedClient,
    query: WendaoFlightRouteQuery,
    *,
    extra_metadata: dict[str, str] | None = None,
    connect_kwargs: dict[str, object] | None = None,
) -> None:
    assert client.calls == [
        WendaoArrowCall(
            operation="query",
            route=query.normalized_route(),
            query=query,
            extra_metadata=extra_metadata or {},
            connect_kwargs=connect_kwargs or {},
        )
    ]


def _assert_repo_search_metadata_call(
    client: WendaoArrowScriptedClient,
    request,
) -> None:
    _assert_single_query_call(
        client,
        repo_search_query(),
        extra_metadata=repo_search_metadata(request),
    )


class _RepoSearchScoreAnalyzer:
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


class _DocIdAnalyzer:
    def analyze_rows(self, rows: list[dict[str, object]]) -> list[dict[str, object]]:
        ranked = sorted(rows, key=lambda row: str(row["doc_id"]))
        return [
            {"doc_id": str(row["doc_id"]), "rank": index + 1}
            for index, row in enumerate(ranked)
        ]


__all__ = [
    "TYPE_CHECKING",
    "AnalyzerConfig",
    "WendaoArrowCall",
    "WendaoArrowResult",
    "WendaoArrowScriptedClient",
    "WendaoFlightRouteQuery",
    "WendaoTransportClient",
    "WendaoTransportConfig",
    "WendaoTransportEndpoint",
    "_DocIdAnalyzer",
    "_RepoSearchScoreAnalyzer",
    "_assert_repo_search_metadata_call",
    "_assert_single_query_call",
    "_project_root",
    "_result_table",
    "_run_rust_search_plane_seed_binary",
    "_score_rows",
    "_scripted_query_client",
    "_scripted_repo_search_query_client",
    "_spawn_wendao_search_flight_server",
    "_terminate_process",
    "_wendao_search_flight_server_binary",
    "_wendao_search_seed_binary",
    "analyze_query",
    "analyze_query_results",
    "analyze_repo_query_text",
    "analyze_repo_query_text_results",
    "analyze_repo_search",
    "analyze_repo_search_results",
    "os",
    "pytest",
    "repo_search_metadata",
    "repo_search_query",
    "repo_search_request",
    "run_query_analysis",
    "run_repo_analysis",
    "run_repo_search_analysis",
    "socket",
    "subprocess",
    "summarize_query",
    "summarize_query_results",
    "summarize_query_route",
    "summarize_repo_analysis",
    "summarize_repo_query_text",
    "summarize_repo_query_text_results",
    "summarize_repo_search",
    "summarize_repo_search_results",
    "time",
]
