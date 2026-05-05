"""Local row and table analyzer helpers."""

from __future__ import annotations

from typing import TYPE_CHECKING, Protocol, cast

from .config import AnalyzerConfig
from .models import (
    AnalysisSummary,
    AnalyzerResultRow,
    RowsAnalysisRun,
    TableAnalysisRun,
    parse_analyzer_result_rows,
)
from .strategies import AnalyzerStrategyProtocol, build_analyzer

if TYPE_CHECKING:
    import pyarrow as pa

    from wendao_core_lib import WendaoFlightRouteQuery


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
    return resolved_analyzer.analyze_rows(
        cast("list[dict[str, object]]", table.to_pylist())
    )


def analyze_table_results(
    table: pa.Table,
    *,
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
) -> list[AnalyzerResultRow]:
    """Analyze one Arrow table and return typed analyzer result objects."""

    return parse_analyzer_result_rows(
        analyze_table(table, analyzer=analyzer, config=config)
    )


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

    return parse_analyzer_result_rows(
        analyze_rows(rows, analyzer=analyzer, config=config)
    )


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

    return summarize_result_rows(
        analyze_result_rows(rows, analyzer=analyzer, config=config)
    )


def summarize_table(
    table: pa.Table,
    *,
    analyzer: AnalyzerStrategyProtocol | None = None,
    config: AnalyzerConfig | None = None,
) -> AnalysisSummary:
    """Analyze and summarize one Arrow table payload."""

    return summarize_result_rows(
        analyze_table_results(table, analyzer=analyzer, config=config)
    )


def summarize_rows_analysis(run: RowsAnalysisRun) -> AnalysisSummary:
    """Summarize one local row-list analysis pipeline result."""

    return summarize_result_rows(list(run.rows_out))


def summarize_table_analysis(run: TableAnalysisRun) -> AnalysisSummary:
    """Summarize one local Arrow table analysis pipeline result."""

    return summarize_result_rows(list(run.rows_out))
