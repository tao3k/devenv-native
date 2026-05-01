from __future__ import annotations

from wendao_core_lib import WendaoFlightRouteQuery, repo_search_request
from xiuxian_wendao_analyzer import (
    AnalysisSummary,
    AnalyzerResultRow,
    QueryAnalysisRun,
    RepoAnalysisRun,
    RowsAnalysisRun,
    TableAnalysisRun,
    parse_analyzer_result_rows,
)


def test_analyzer_result_row_preserves_optional_score_fields() -> None:
    row = AnalyzerResultRow.from_mapping(
        {
            "doc_id": "doc-a",
            "vector_score": 0.1,
            "semantic_score": 1.0,
            "final_score": 0.685,
            "rank": 1,
        }
    )

    assert row.doc_id == "doc-a"
    assert row.rank == 1
    assert row.vector_score == 0.1
    assert row.semantic_score == 1.0
    assert row.final_score == 0.685
    assert row.path is None
    assert row.payload["doc_id"] == "doc-a"


def test_parse_analyzer_result_rows_parses_repo_search_style_payload() -> None:
    rows = parse_analyzer_result_rows(
        [
            {"path": "src/lib.rs", "score": 0.9, "rank": 1},
            {"path": "src/main.rs", "score": 0.3, "rank": 2},
        ]
    )

    assert [row.path for row in rows] == ["src/lib.rs", "src/main.rs"]
    assert [row.score for row in rows] == [0.9, 0.3]
    assert [row.rank for row in rows] == [1, 2]
    assert all(row.doc_id is None for row in rows)


def test_repo_analysis_run_preserves_request_and_rows() -> None:
    run = RepoAnalysisRun(
        request=repo_search_request("alpha", limit=2, path_prefixes=("src/",)),
        rows=tuple(
            parse_analyzer_result_rows(
                [
                    {"path": "src/lib.rs", "score": 0.9, "rank": 1},
                    {"path": "src/main.rs", "score": 0.3, "rank": 2},
                ]
            )
        ),
    )

    assert run.request.query_text == "alpha"
    assert run.request.path_prefixes == ("src/",)
    assert [row.path for row in run.rows] == ["src/lib.rs", "src/main.rs"]


def test_query_analysis_run_preserves_query_and_rows() -> None:
    run = QueryAnalysisRun(
        query=WendaoFlightRouteQuery(route="/repo-search/flight"),
        rows=tuple(
            parse_analyzer_result_rows(
                [
                    {"path": "src/lib.rs", "score": 0.9, "rank": 1},
                    {"path": "src/main.rs", "score": 0.3, "rank": 2},
                ]
            )
        ),
    )

    assert run.query.route == "/repo-search/flight"
    assert [row.path for row in run.rows] == ["src/lib.rs", "src/main.rs"]


def test_rows_analysis_run_preserves_input_and_rows() -> None:
    run = RowsAnalysisRun(
        rows_in=(
            {"path": "src/lib.rs", "score": 0.9},
            {"path": "src/main.rs", "score": 0.3},
        ),
        rows_out=tuple(
            parse_analyzer_result_rows(
                [
                    {"path": "src/lib.rs", "score": 0.9, "rank": 1},
                    {"path": "src/main.rs", "score": 0.3, "rank": 2},
                ]
            )
        ),
    )

    assert [row["path"] for row in run.rows_in] == ["src/lib.rs", "src/main.rs"]
    assert [row.path for row in run.rows_out] == ["src/lib.rs", "src/main.rs"]


def test_table_analysis_run_preserves_input_and_rows() -> None:
    run = TableAnalysisRun(
        table_in="placeholder-table",
        rows_out=tuple(
            parse_analyzer_result_rows(
                [
                    {"path": "src/lib.rs", "score": 0.9, "rank": 1},
                    {"path": "src/main.rs", "score": 0.3, "rank": 2},
                ]
            )
        ),
    )

    assert run.table_in == "placeholder-table"
    assert [row.path for row in run.rows_out] == ["src/lib.rs", "src/main.rs"]


def test_analysis_summary_holds_top_row_snapshot() -> None:
    summary = AnalysisSummary(
        row_count=2,
        top_rank=1,
        top_doc_id="doc-a",
        top_path=None,
        top_score=None,
        top_final_score=0.72,
    )

    assert summary.row_count == 2
    assert summary.top_doc_id == "doc-a"
    assert summary.top_final_score == 0.72
