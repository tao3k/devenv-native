from __future__ import annotations

import pyarrow as pa

from xiuxian_wendao_analyzer import (
    AnalyzerConfig,
    analyze_result_rows,
    analyze_rows,
    analyze_table,
    analyze_table_results,
    build_analyzer,
    run_rows_analysis,
    run_table_analysis,
    summarize_result_rows,
    summarize_rows,
    summarize_rows_analysis,
    summarize_table,
    summarize_table_analysis,
)


class _DocIdAnalyzer:
    def analyze_rows(self, rows: list[dict[str, object]]) -> list[dict[str, object]]:
        ranked = sorted(rows, key=lambda row: str(row["doc_id"]))
        return [
            {"doc_id": str(row["doc_id"]), "rank": index + 1}
            for index, row in enumerate(ranked)
        ]


def _score_rows() -> list[dict[str, object]]:
    return [
        {"path": "src/main.rs", "score": 0.3},
        {"path": "src/lib.rs", "score": 0.9},
    ]


def test_analyze_rows_defaults_to_score_rank() -> None:
    ranked = analyze_rows(_score_rows())

    assert [row["path"] for row in ranked] == ["src/lib.rs", "src/main.rs"]
    assert [row["rank"] for row in ranked] == [1, 2]


def test_build_analyzer_returns_score_rank_strategy() -> None:
    ranked = build_analyzer(AnalyzerConfig()).analyze_rows(_score_rows())

    assert [row["path"] for row in ranked] == ["src/lib.rs", "src/main.rs"]
    assert [row["rank"] for row in ranked] == [1, 2]


def test_analyze_table_uses_provided_analyzer() -> None:
    table = pa.Table.from_pylist(
        [
            {"doc_id": "doc-b"},
            {"doc_id": "doc-a"},
        ]
    )

    ranked = analyze_table(table, analyzer=_DocIdAnalyzer())

    assert ranked == [
        {"doc_id": "doc-a", "rank": 1},
        {"doc_id": "doc-b", "rank": 2},
    ]


def test_analyze_result_rows_returns_typed_results() -> None:
    ranked = analyze_result_rows(_score_rows())

    assert len(ranked) == 2
    assert [row.path for row in ranked] == ["src/lib.rs", "src/main.rs"]
    assert [row.rank for row in ranked] == [1, 2]
    assert [row.score for row in ranked] == [0.9, 0.3]


def test_analyze_table_results_returns_typed_results() -> None:
    ranked = analyze_table_results(pa.Table.from_pylist(_score_rows()))

    assert len(ranked) == 2
    assert [row.path for row in ranked] == ["src/lib.rs", "src/main.rs"]
    assert [row.rank for row in ranked] == [1, 2]


def test_run_rows_analysis_preserves_input_and_typed_output() -> None:
    run = run_rows_analysis(_score_rows())

    assert [row["path"] for row in run.rows_in] == ["src/main.rs", "src/lib.rs"]
    assert [row.path for row in run.rows_out] == ["src/lib.rs", "src/main.rs"]
    assert [row.rank for row in run.rows_out] == [1, 2]


def test_run_table_analysis_preserves_input_and_typed_output() -> None:
    table = pa.Table.from_pylist(_score_rows())

    run = run_table_analysis(table)

    assert run.table_in == table
    assert [row.path for row in run.rows_out] == ["src/lib.rs", "src/main.rs"]
    assert [row.rank for row in run.rows_out] == [1, 2]


def test_summarize_result_rows_uses_first_ranked_row() -> None:
    summary = summarize_result_rows(analyze_result_rows(_score_rows()))

    assert summary.row_count == 2
    assert summary.top_path == "src/lib.rs"
    assert summary.top_rank == 1
    assert summary.top_score == 0.9
    assert summary.top_final_score is None


def test_summarize_rows_returns_top_row_snapshot() -> None:
    summary = summarize_rows(_score_rows())

    assert summary.row_count == 2
    assert summary.top_path == "src/lib.rs"
    assert summary.top_rank == 1


def test_summarize_table_returns_top_row_snapshot() -> None:
    summary = summarize_table(pa.Table.from_pylist(_score_rows()))

    assert summary.row_count == 2
    assert summary.top_path == "src/lib.rs"
    assert summary.top_rank == 1


def test_summarize_rows_analysis_returns_top_row_snapshot() -> None:
    summary = summarize_rows_analysis(run_rows_analysis(_score_rows()))

    assert summary.row_count == 2
    assert summary.top_path == "src/lib.rs"
    assert summary.top_rank == 1


def test_summarize_table_analysis_returns_top_row_snapshot() -> None:
    summary = summarize_table_analysis(
        run_table_analysis(pa.Table.from_pylist(_score_rows()))
    )

    assert summary.row_count == 2
    assert summary.top_path == "src/lib.rs"
    assert summary.top_rank == 1
