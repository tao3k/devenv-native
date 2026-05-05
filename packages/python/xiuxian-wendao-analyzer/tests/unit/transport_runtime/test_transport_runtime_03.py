"""transport_runtime test slice 3."""

from __future__ import annotations

from .support import (
    AnalyzerConfig,
    _assert_repo_search_metadata_call,
    _score_rows,
    _scripted_repo_search_query_client,
    repo_search_request,
    run_repo_analysis,
    run_repo_search_analysis,
    summarize_repo_analysis,
    summarize_repo_query_text,
    summarize_repo_search,
    summarize_repo_search_results,
)


def test_run_repo_analysis_returns_request_and_typed_rows() -> None:
    request = repo_search_request("alpha", limit=2, path_prefixes=("src/",))
    client = _scripted_repo_search_query_client(_score_rows())

    run = run_repo_analysis(
        client,
        "alpha",
        limit=2,
        path_prefixes=("src/",),
        config=AnalyzerConfig(),
    )

    assert run.request == request
    assert [row.path for row in run.rows] == ["src/lib.rs", "src/main.rs"]
    assert [row.rank for row in run.rows] == [1, 2]
    _assert_repo_search_metadata_call(client, request)


def test_run_repo_search_analysis_preserves_typed_request_and_rows() -> None:
    request = repo_search_request("alpha", limit=2, path_prefixes=("src/",))
    client = _scripted_repo_search_query_client(_score_rows())

    run = run_repo_search_analysis(client, request, config=AnalyzerConfig())

    assert run.request == request
    assert [row.path for row in run.rows] == ["src/lib.rs", "src/main.rs"]
    assert [row.rank for row in run.rows] == [1, 2]
    _assert_repo_search_metadata_call(client, request)


def test_summarize_repo_analysis_returns_top_row_snapshot() -> None:
    request = repo_search_request("alpha", limit=2, path_prefixes=("src/",))
    client = _scripted_repo_search_query_client(_score_rows())

    summary = summarize_repo_analysis(
        run_repo_analysis(
            client,
            "alpha",
            limit=2,
            path_prefixes=("src/",),
            config=AnalyzerConfig(),
        )
    )

    assert summary.row_count == 2
    assert summary.top_path == "src/lib.rs"
    assert summary.top_rank == 1
    _assert_repo_search_metadata_call(client, request)


def test_summarize_repo_search_returns_top_row_snapshot() -> None:
    request = repo_search_request("alpha", limit=2, path_prefixes=("src/",))
    client = _scripted_repo_search_query_client(_score_rows())

    summary = summarize_repo_search(client, request, config=AnalyzerConfig())

    assert summary.row_count == 2
    assert summary.top_path == "src/lib.rs"
    assert summary.top_rank == 1
    _assert_repo_search_metadata_call(client, request)


def test_summarize_repo_search_results_returns_top_row_snapshot() -> None:
    request = repo_search_request("alpha", limit=2, path_prefixes=("src/",))
    client = _scripted_repo_search_query_client(_score_rows())

    summary = summarize_repo_search_results(client, request, config=AnalyzerConfig())

    assert summary.row_count == 2
    assert summary.top_path == "src/lib.rs"
    assert summary.top_rank == 1
    _assert_repo_search_metadata_call(client, request)


def test_summarize_repo_query_text_returns_top_row_snapshot() -> None:
    request = repo_search_request("alpha", limit=2, path_prefixes=("src/",))
    client = _scripted_repo_search_query_client(_score_rows())

    summary = summarize_repo_query_text(
        client,
        "alpha",
        limit=2,
        path_prefixes=("src/",),
        config=AnalyzerConfig(),
    )

    assert summary.row_count == 2
    assert summary.top_path == "src/lib.rs"
    assert summary.top_rank == 1
    _assert_repo_search_metadata_call(client, request)
