"""transport_runtime test slice 1."""

from __future__ import annotations

from .support import (
    WendaoFlightRouteQuery,
    _assert_single_query_call,
    _DocIdAnalyzer,
    _score_rows,
    _scripted_query_client,
    analyze_query,
    run_query_analysis,
    summarize_query,
    summarize_query_results,
    summarize_query_route,
)


def test_analyze_query_uses_transport_client_table_fetch() -> None:
    query = WendaoFlightRouteQuery(route="/analysis/test")
    client = _scripted_query_client(query, _score_rows())

    ranked = analyze_query(client, query, tls_root_certs=b"roots")

    assert [row["path"] for row in ranked] == ["src/lib.rs", "src/main.rs"]
    assert [row["rank"] for row in ranked] == [1, 2]
    _assert_single_query_call(
        client,
        query,
        connect_kwargs={"tls_root_certs": b"roots"},
    )


def test_analyze_query_uses_explicit_analyzer() -> None:
    query = WendaoFlightRouteQuery(route="/analysis/doc-id")
    client = _scripted_query_client(
        query,
        [
            {"doc_id": "doc-b"},
            {"doc_id": "doc-a"},
        ],
    )

    ranked = analyze_query(client, query, analyzer=_DocIdAnalyzer())

    assert ranked == [
        {"doc_id": "doc-a", "rank": 1},
        {"doc_id": "doc-b", "rank": 2},
    ]
    _assert_single_query_call(client, query)


def test_run_query_analysis_preserves_query_and_typed_rows() -> None:
    query = WendaoFlightRouteQuery(route="/analysis/test")
    client = _scripted_query_client(query, _score_rows())

    run = run_query_analysis(client, query)

    assert run.query == query
    assert [row.path for row in run.rows] == ["src/lib.rs", "src/main.rs"]
    assert [row.rank for row in run.rows] == [1, 2]
    _assert_single_query_call(client, query)


def test_summarize_query_returns_top_row_snapshot() -> None:
    query = WendaoFlightRouteQuery(route="/analysis/test")
    client = _scripted_query_client(query, _score_rows())

    summary = summarize_query(run_query_analysis(client, query))

    assert summary.row_count == 2
    assert summary.top_path == "src/lib.rs"
    assert summary.top_rank == 1
    _assert_single_query_call(client, query)


def test_summarize_query_route_returns_top_row_snapshot() -> None:
    query = WendaoFlightRouteQuery(route="/analysis/test")
    client = _scripted_query_client(query, _score_rows())

    summary = summarize_query_route(client, query)

    assert summary.row_count == 2
    assert summary.top_path == "src/lib.rs"
    assert summary.top_rank == 1
    _assert_single_query_call(client, query)


def test_summarize_query_results_returns_top_row_snapshot() -> None:
    query = WendaoFlightRouteQuery(route="/analysis/test")
    client = _scripted_query_client(query, _score_rows())

    summary = summarize_query_results(client, query)

    assert summary.row_count == 2
    assert summary.top_path == "src/lib.rs"
    assert summary.top_rank == 1
    _assert_single_query_call(client, query)
