"""Typed analyzer result models."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping

    from wendao_core_lib import (
        WendaoFlightRouteQuery,
        WendaoRepoSearchRequest,
    )


@dataclass(frozen=True, slots=True)
class AnalyzerResultRow:
    """Typed view over one analyzer output row."""

    payload: Mapping[str, object]
    rank: int | None
    doc_id: str | None
    path: str | None
    score: float | None
    vector_score: float | None
    semantic_score: float | None
    final_score: float | None

    @classmethod
    def from_mapping(cls, row: Mapping[str, object]) -> AnalyzerResultRow:
        """Build one typed analyzer result row from a generic mapping."""

        rank = row.get("rank")
        doc_id = row.get("doc_id")
        path = row.get("path")
        score = row.get("score")
        vector_score = row.get("vector_score")
        semantic_score = row.get("semantic_score")
        final_score = row.get("final_score")
        return cls(
            payload=dict(row),
            rank=int(rank) if rank is not None else None,
            doc_id=str(doc_id) if doc_id is not None else None,
            path=str(path) if path is not None else None,
            score=float(score) if score is not None else None,
            vector_score=float(vector_score) if vector_score is not None else None,
            semantic_score=(
                float(semantic_score) if semantic_score is not None else None
            ),
            final_score=float(final_score) if final_score is not None else None,
        )


def parse_analyzer_result_rows(
    rows: list[Mapping[str, object]],
) -> list[AnalyzerResultRow]:
    """Parse one list of generic analyzer rows into typed analyzer result rows."""

    return [AnalyzerResultRow.from_mapping(row) for row in rows]


@dataclass(frozen=True, slots=True)
class RowsAnalysisRun:
    """Typed analyzer-owned local row-list pipeline result."""

    rows_in: tuple[Mapping[str, object], ...]
    rows_out: tuple[AnalyzerResultRow, ...]


@dataclass(frozen=True, slots=True)
class TableAnalysisRun:
    """Typed analyzer-owned local Arrow table pipeline result."""

    table_in: object
    rows_out: tuple[AnalyzerResultRow, ...]


@dataclass(frozen=True, slots=True)
class RepoAnalysisRun:
    """Typed analyzer-owned repo-search pipeline result."""

    request: WendaoRepoSearchRequest
    rows: tuple[AnalyzerResultRow, ...]


@dataclass(frozen=True, slots=True)
class QueryAnalysisRun:
    """Typed analyzer-owned generic query pipeline result."""

    query: WendaoFlightRouteQuery
    rows: tuple[AnalyzerResultRow, ...]


@dataclass(frozen=True, slots=True)
class AnalysisSummary:
    """Lightweight summary over one analyzer result set."""

    row_count: int
    top_rank: int | None
    top_doc_id: str | None
    top_path: str | None
    top_score: float | None
    top_final_score: float | None


__all__ = [
    "AnalysisSummary",
    "AnalyzerResultRow",
    "QueryAnalysisRun",
    "RepoAnalysisRun",
    "RowsAnalysisRun",
    "TableAnalysisRun",
    "parse_analyzer_result_rows",
]
