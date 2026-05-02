"""Query-text repo-search analysis helpers."""

from __future__ import annotations

from typing import TYPE_CHECKING

from wendao_core_lib import repo_search_request

from .models import (
    AnalysisSummary,
    AnalyzerResultRow,
    RepoAnalysisRun,
    parse_analyzer_result_rows,
)
from .runtime_local import WendaoAnalyzerRuntimeClient, summarize_result_rows
from .runtime_repo_request import (
    analyze_repo_search,
    analyze_repo_search_results,
    summarize_repo_analysis,
)

if TYPE_CHECKING:
    from .config import AnalyzerConfig
    from .strategies import AnalyzerStrategyProtocol


def analyze_repo_query_text(
    client: WendaoAnalyzerRuntimeClient,
    query_text: str,
    *,
    limit: int = 10,
    language_filters: tuple[str, ...] | list[str] = (),
    path_prefixes: tuple[str, ...] | list[str] = (),
    title_filters: tuple[str, ...] | list[str] = (),
    tag_filters: tuple[str, ...] | list[str] = (),
    filename_filters: tuple[str, ...] | list[str] = (),
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
    **connect_kwargs: object,
) -> list[dict[str, object]]:
    """Build one repo-search request from query text and analyze the result."""

    return analyze_repo_search(
        client,
        repo_search_request(
            query_text,
            limit=limit,
            language_filters=tuple(language_filters),
            path_prefixes=tuple(path_prefixes),
            title_filters=tuple(title_filters),
            tag_filters=tuple(tag_filters),
            filename_filters=tuple(filename_filters),
        ),
        analyzer=analyzer,
        config=config,
        **connect_kwargs,
    )


def analyze_repo_query_text_results(
    client: WendaoAnalyzerRuntimeClient,
    query_text: str,
    *,
    limit: int = 10,
    language_filters: tuple[str, ...] | list[str] = (),
    path_prefixes: tuple[str, ...] | list[str] = (),
    title_filters: tuple[str, ...] | list[str] = (),
    tag_filters: tuple[str, ...] | list[str] = (),
    filename_filters: tuple[str, ...] | list[str] = (),
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
    **connect_kwargs: object,
) -> list[AnalyzerResultRow]:
    """Build one repo-search request from query text and return typed results."""

    return parse_analyzer_result_rows(
        analyze_repo_query_text(
            client,
            query_text,
            limit=limit,
            language_filters=language_filters,
            path_prefixes=path_prefixes,
            title_filters=title_filters,
            tag_filters=tag_filters,
            filename_filters=filename_filters,
            analyzer=analyzer,
            config=config,
            **connect_kwargs,
        )
    )


def summarize_repo_query_text_results(
    client: WendaoAnalyzerRuntimeClient,
    query_text: str,
    *,
    limit: int = 10,
    language_filters: tuple[str, ...] | list[str] = (),
    path_prefixes: tuple[str, ...] | list[str] = (),
    title_filters: tuple[str, ...] | list[str] = (),
    tag_filters: tuple[str, ...] | list[str] = (),
    filename_filters: tuple[str, ...] | list[str] = (),
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
    **connect_kwargs: object,
) -> AnalysisSummary:
    """Build one repo-search request from query text, return typed results, and summarize them."""

    return summarize_result_rows(
        analyze_repo_query_text_results(
            client,
            query_text,
            limit=limit,
            language_filters=language_filters,
            path_prefixes=path_prefixes,
            title_filters=title_filters,
            tag_filters=tag_filters,
            filename_filters=filename_filters,
            analyzer=analyzer,
            config=config,
            **connect_kwargs,
        )
    )


def run_repo_analysis(
    client: WendaoAnalyzerRuntimeClient,
    query_text: str,
    *,
    limit: int = 10,
    language_filters: tuple[str, ...] | list[str] = (),
    path_prefixes: tuple[str, ...] | list[str] = (),
    title_filters: tuple[str, ...] | list[str] = (),
    tag_filters: tuple[str, ...] | list[str] = (),
    filename_filters: tuple[str, ...] | list[str] = (),
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
    **connect_kwargs: object,
) -> RepoAnalysisRun:
    """Run one analyzer-owned repo-search pipeline and return request plus results."""

    request = repo_search_request(
        query_text,
        limit=limit,
        language_filters=tuple(language_filters),
        path_prefixes=tuple(path_prefixes),
        title_filters=tuple(title_filters),
        tag_filters=tuple(tag_filters),
        filename_filters=tuple(filename_filters),
    )
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


def summarize_repo_query_text(
    client: WendaoAnalyzerRuntimeClient,
    query_text: str,
    *,
    limit: int = 10,
    language_filters: tuple[str, ...] | list[str] = (),
    path_prefixes: tuple[str, ...] | list[str] = (),
    title_filters: tuple[str, ...] | list[str] = (),
    tag_filters: tuple[str, ...] | list[str] = (),
    filename_filters: tuple[str, ...] | list[str] = (),
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
    **connect_kwargs: object,
) -> AnalysisSummary:
    """Run and summarize one repo-search analysis from high-level query text."""

    return summarize_repo_analysis(
        run_repo_analysis(
            client,
            query_text,
            limit=limit,
            language_filters=language_filters,
            path_prefixes=path_prefixes,
            title_filters=title_filters,
            tag_filters=tag_filters,
            filename_filters=filename_filters,
            analyzer=analyzer,
            config=config,
            **connect_kwargs,
        )
    )
