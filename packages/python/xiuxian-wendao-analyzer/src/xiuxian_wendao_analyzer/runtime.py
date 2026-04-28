"""Runtime helpers for local analyzer execution."""

from __future__ import annotations

from typing import TYPE_CHECKING, Protocol, cast

from wendao_core_lib import (
    WendaoFlightRouteQuery,
    WendaoRepoSearchRequest,
    repo_search_metadata,
    repo_search_query,
    repo_search_request,
)

from .config import AnalyzerConfig
from .models import (
    AnalysisSummary,
    AnalyzerResultRow,
    QueryAnalysisRun,
    RepoAnalysisRun,
    RowsAnalysisRun,
    TableAnalysisRun,
    parse_analyzer_result_rows,
)
from .strategies import AnalyzerStrategyProtocol, build_analyzer

if TYPE_CHECKING:
    import pyarrow as pa


class WendaoAnalyzerRuntimeClient(Protocol):
    """Minimal host-backed client contract consumed by analyzer runtime helpers."""

    def read_query_table(
        self,
        query: WendaoFlightRouteQuery,
        **connect_kwargs: object,
    ) -> pa.Table: ...


def analyze_table(
    table: pa.Table,
    *,
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
) -> list[dict[str, object]]:
    """Analyze one Arrow table through the configured analyzer strategy."""

    resolved_analyzer = (
        analyzer if analyzer is not None else build_analyzer(config or AnalyzerConfig())
    )
    return resolved_analyzer.analyze_rows(cast("list[dict[str, object]]", table.to_pylist()))


def run_table_analysis(
    table: pa.Table,
    *,
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
) -> TableAnalysisRun:
    """Run one local Arrow table analysis pipeline and return input plus results."""

    return TableAnalysisRun(
        table_in=table,
        rows_out=tuple(analyze_table_results(table, analyzer=analyzer, config=config)),
    )


def analyze_rows(
    rows: list[dict[str, object]],
    *,
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
) -> list[dict[str, object]]:
    """Analyze one list-of-dicts payload through the configured strategy."""

    resolved_analyzer = (
        analyzer if analyzer is not None else build_analyzer(config or AnalyzerConfig())
    )
    return resolved_analyzer.analyze_rows(rows)


def run_rows_analysis(
    rows: list[dict[str, object]],
    *,
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
) -> RowsAnalysisRun:
    """Run one local row-list analysis pipeline and return input plus results."""

    return RowsAnalysisRun(
        rows_in=tuple(dict(row) for row in rows),
        rows_out=tuple(analyze_result_rows(rows, analyzer=analyzer, config=config)),
    )


def analyze_result_rows(
    rows: list[dict[str, object]],
    *,
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
) -> list[AnalyzerResultRow]:
    """Analyze rows and return typed analyzer result objects."""

    return parse_analyzer_result_rows(analyze_rows(rows, analyzer=analyzer, config=config))


def summarize_result_rows(rows: list[AnalyzerResultRow]) -> AnalysisSummary:
    """Summarize one typed analyzer result set."""

    top_row = rows[0] if rows else None
    return AnalysisSummary(
        row_count=len(rows),
        top_rank=top_row.rank if top_row is not None else None,
        top_doc_id=top_row.doc_id if top_row is not None else None,
        top_path=top_row.path if top_row is not None else None,
        top_score=top_row.score if top_row is not None else None,
        top_final_score=top_row.final_score if top_row is not None else None,
    )


def summarize_rows(
    rows: list[dict[str, object]],
    *,
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
) -> AnalysisSummary:
    """Analyze and summarize one list-of-dicts payload."""

    return summarize_result_rows(analyze_result_rows(rows, analyzer=analyzer, config=config))


def summarize_table(
    table: pa.Table,
    *,
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
) -> AnalysisSummary:
    """Analyze and summarize one Arrow table payload."""

    return summarize_result_rows(analyze_table_results(table, analyzer=analyzer, config=config))


def summarize_rows_analysis(run: RowsAnalysisRun) -> AnalysisSummary:
    """Summarize one local row-list analysis pipeline result."""

    return summarize_result_rows(list(run.rows_out))


def summarize_table_analysis(run: TableAnalysisRun) -> AnalysisSummary:
    """Summarize one local Arrow table analysis pipeline result."""

    return summarize_result_rows(list(run.rows_out))


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


def analyze_table_results(
    table: pa.Table,
    *,
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
) -> list[AnalyzerResultRow]:
    """Analyze one Arrow table and return typed analyzer result objects."""

    return parse_analyzer_result_rows(analyze_table(table, analyzer=analyzer, config=config))


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


__all__ = [
    "analyze_query",
    "analyze_query_results",
    "analyze_repo_query_text",
    "analyze_repo_query_text_results",
    "analyze_repo_search",
    "analyze_repo_search_results",
    "analyze_result_rows",
    "analyze_rows",
    "analyze_table",
    "analyze_table_results",
    "run_query_analysis",
    "run_repo_analysis",
    "run_repo_search_analysis",
    "run_rows_analysis",
    "run_table_analysis",
    "summarize_query",
    "summarize_query_results",
    "summarize_query_route",
    "summarize_repo_analysis",
    "summarize_repo_query_text",
    "summarize_repo_query_text_results",
    "summarize_repo_search",
    "summarize_repo_search_results",
    "summarize_result_rows",
    "summarize_rows",
    "summarize_rows_analysis",
    "summarize_table",
    "summarize_table_analysis",
]
