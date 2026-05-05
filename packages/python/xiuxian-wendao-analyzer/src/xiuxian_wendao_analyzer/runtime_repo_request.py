"""Typed repo-search request analysis helpers."""

from __future__ import annotations

from typing import TYPE_CHECKING

from wendao_core_lib import (
    WendaoRepoSearchRequest,
    repo_search_metadata,
    repo_search_query,
)

from .models import (
    AnalysisSummary,
    AnalyzerResultRow,
    RepoAnalysisRun,
    parse_analyzer_result_rows,
)
from .runtime_local import WendaoAnalyzerRuntimeClient, summarize_result_rows
from .runtime_query import analyze_query

if TYPE_CHECKING:
    from .config import AnalyzerConfig
    from .strategies import AnalyzerStrategyProtocol


def analyze_repo_search(
    client: WendaoAnalyzerRuntimeClient,
    request: WendaoRepoSearchRequest,
    *,
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
    **connect_kwargs: object,
) -> list[dict[str, object]]:
    """Fetch one typed repo-search request and analyze the returned Arrow table."""

    return analyze_query(
        client,
        repo_search_query(),
        analyzer=analyzer,
        config=config,
        extra_metadata=repo_search_metadata(request),
        **connect_kwargs,
    )


def analyze_repo_search_results(
    client: WendaoAnalyzerRuntimeClient,
    request: WendaoRepoSearchRequest,
    *,
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
    **connect_kwargs: object,
) -> list[AnalyzerResultRow]:
    """Analyze one typed repo-search request and return typed analyzer result objects."""

    return parse_analyzer_result_rows(
        analyze_repo_search(
            client,
            request,
            analyzer=analyzer,
            config=config,
            **connect_kwargs,
        )
    )


def summarize_repo_search_results(
    client: WendaoAnalyzerRuntimeClient,
    request: WendaoRepoSearchRequest,
    *,
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
    **connect_kwargs: object,
) -> AnalysisSummary:
    """Fetch one typed repo-search request, return typed results, and summarize them."""

    return summarize_result_rows(
        analyze_repo_search_results(
            client,
            request,
            analyzer=analyzer,
            config=config,
            **connect_kwargs,
        )
    )


def run_repo_search_analysis(
    client: WendaoAnalyzerRuntimeClient,
    request: WendaoRepoSearchRequest,
    *,
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
    **connect_kwargs: object,
) -> RepoAnalysisRun:
    """Run one typed repo-search analysis pipeline and return request plus results."""

    return RepoAnalysisRun(
        request=request,
        rows=tuple(
            analyze_repo_search_results(
                client,
                request,
                analyzer=analyzer,
                config=config,
                **connect_kwargs,
            )
        ),
    )


def summarize_repo_analysis(run: RepoAnalysisRun) -> AnalysisSummary:
    """Summarize one repo-search analysis pipeline result."""

    return summarize_result_rows(list(run.rows))


def summarize_repo_search(
    client: WendaoAnalyzerRuntimeClient,
    request: WendaoRepoSearchRequest,
    *,
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
    **connect_kwargs: object,
) -> AnalysisSummary:
    """Fetch, analyze, and summarize one typed repo-search request."""

    return summarize_repo_analysis(
        run_repo_search_analysis(
            client,
            request,
            analyzer=analyzer,
            config=config,
            **connect_kwargs,
        )
    )
