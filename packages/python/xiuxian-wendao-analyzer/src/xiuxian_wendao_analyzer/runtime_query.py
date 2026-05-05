"""Host-backed query analysis helpers."""

from __future__ import annotations

from typing import TYPE_CHECKING

from .models import (
    AnalysisSummary,
    AnalyzerResultRow,
    QueryAnalysisRun,
    parse_analyzer_result_rows,
)
from .runtime_local import (
    WendaoAnalyzerRuntimeClient,
    analyze_table,
    summarize_result_rows,
)

if TYPE_CHECKING:
    from wendao_core_lib import WendaoFlightRouteQuery

    from .config import AnalyzerConfig
    from .strategies import AnalyzerStrategyProtocol


def summarize_query(run: QueryAnalysisRun) -> AnalysisSummary:
    """Summarize one generic host-backed query analysis pipeline result."""

    return summarize_result_rows(list(run.rows))


def summarize_query_route(
    client: WendaoAnalyzerRuntimeClient,
    query: WendaoFlightRouteQuery,
    *,
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
    **connect_kwargs: object,
) -> AnalysisSummary:
    """Fetch, analyze, and summarize one generic host-backed query."""

    return summarize_query(
        run_query_analysis(
            client,
            query,
            analyzer=analyzer,
            config=config,
            **connect_kwargs,
        )
    )


def analyze_query(
    client: WendaoAnalyzerRuntimeClient,
    query: WendaoFlightRouteQuery,
    *,
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
    **connect_kwargs: object,
) -> list[dict[str, object]]:
    """Fetch one Arrow table through the Wendao transport client and analyze it."""

    table = client.read_query_table(query, **connect_kwargs)
    return analyze_table(table, analyzer=analyzer, config=config)


def run_query_analysis(
    client: WendaoAnalyzerRuntimeClient,
    query: WendaoFlightRouteQuery,
    *,
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
    **connect_kwargs: object,
) -> QueryAnalysisRun:
    """Run one generic host-backed query analysis pipeline and return query plus results."""

    return QueryAnalysisRun(
        query=query,
        rows=tuple(
            analyze_query_results(
                client,
                query,
                analyzer=analyzer,
                config=config,
                **connect_kwargs,
            )
        ),
    )


def analyze_query_results(
    client: WendaoAnalyzerRuntimeClient,
    query: WendaoFlightRouteQuery,
    *,
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
    **connect_kwargs: object,
) -> list[AnalyzerResultRow]:
    """Fetch one query and return typed analyzer result objects."""

    return parse_analyzer_result_rows(
        analyze_query(
            client,
            query,
            analyzer=analyzer,
            config=config,
            **connect_kwargs,
        )
    )


def summarize_query_results(
    client: WendaoAnalyzerRuntimeClient,
    query: WendaoFlightRouteQuery,
    *,
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
    **connect_kwargs: object,
) -> AnalysisSummary:
    """Fetch one query, return typed results, and summarize them."""

    return summarize_result_rows(
        analyze_query_results(
            client,
            query,
            analyzer=analyzer,
            config=config,
            **connect_kwargs,
        )
    )
