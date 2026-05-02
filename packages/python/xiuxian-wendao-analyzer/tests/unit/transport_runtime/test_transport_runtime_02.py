"""transport_runtime test slice 2."""

from __future__ import annotations

from .support import (
    AnalyzerConfig,
    WendaoFlightRouteQuery,
    _assert_repo_search_metadata_call,
    _assert_single_query_call,
    _RepoSearchScoreAnalyzer,
    _score_rows,
    _scripted_query_client,
    _scripted_repo_search_query_client,
    analyze_query_results,
    analyze_repo_query_text,
    analyze_repo_query_text_results,
    analyze_repo_search,
    analyze_repo_search_results,
    repo_search_request,
)


def test_analyze_query_results_returns_typed_rows() -> None:
    query = WendaoFlightRouteQuery(route="/analysis/test")
    client = _scripted_query_client(query, _score_rows())

    ranked = analyze_query_results(client, query)

    assert len(ranked) == 2
    assert [row.path for row in ranked] == ["src/lib.rs", "src/main.rs"]
    assert [row.rank for row in ranked] == [1, 2]
    _assert_single_query_call(client, query)


def test_analyze_repo_search_uses_typed_request_metadata() -> None:
    request = repo_search_request("alpha", limit=2, path_prefixes=("src/",))
    client = _scripted_repo_search_query_client(_score_rows())

    ranked = analyze_repo_search(client, request, analyzer=_RepoSearchScoreAnalyzer())

    assert ranked == [
        {"path": "src/lib.rs", "score": 0.9, "rank": 1},
        {"path": "src/main.rs", "score": 0.3, "rank": 2},
    ]
    _assert_repo_search_metadata_call(client, request)


def test_analyze_repo_search_supports_score_rank_config() -> None:
    request = repo_search_request("alpha", limit=2, path_prefixes=("src/",))
    client = _scripted_repo_search_query_client(_score_rows())

    ranked = analyze_repo_search(client, request, config=AnalyzerConfig())

    assert [row["path"] for row in ranked] == ["src/lib.rs", "src/main.rs"]
    assert [row["rank"] for row in ranked] == [1, 2]
    _assert_repo_search_metadata_call(client, request)


def test_analyze_repo_search_results_returns_typed_rows() -> None:
    request = repo_search_request("alpha", limit=2, path_prefixes=("src/",))
    client = _scripted_repo_search_query_client(_score_rows())

    ranked = analyze_repo_search_results(client, request, config=AnalyzerConfig())

    assert [row.path for row in ranked] == ["src/lib.rs", "src/main.rs"]
    assert [row.rank for row in ranked] == [1, 2]
    _assert_repo_search_metadata_call(client, request)


def test_analyze_repo_query_text_builds_request_and_applies_score_rank() -> None:
    request = repo_search_request("alpha", limit=2, path_prefixes=("src/",))
    client = _scripted_repo_search_query_client(_score_rows())

    ranked = analyze_repo_query_text(
        client,
        "alpha",
        limit=2,
        path_prefixes=("src/",),
        config=AnalyzerConfig(),
    )

    assert [row["path"] for row in ranked] == ["src/lib.rs", "src/main.rs"]
    assert [row["rank"] for row in ranked] == [1, 2]
    _assert_repo_search_metadata_call(client, request)


def test_analyze_repo_query_text_results_return_typed_rows() -> None:
    request = repo_search_request("alpha", limit=2, path_prefixes=("src/",))
    client = _scripted_repo_search_query_client(_score_rows())

    ranked = analyze_repo_query_text_results(
        client,
        "alpha",
        limit=2,
        path_prefixes=("src/",),
        config=AnalyzerConfig(),
    )

    assert [row.path for row in ranked] == ["src/lib.rs", "src/main.rs"]
    assert [row.rank for row in ranked] == [1, 2]
    _assert_repo_search_metadata_call(client, request)
