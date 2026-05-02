"""transport_runtime test slice 4."""

from __future__ import annotations

from .support import (
    AnalyzerConfig,
    WendaoTransportClient,
    WendaoTransportConfig,
    WendaoTransportEndpoint,
    _assert_repo_search_metadata_call,
    _RepoSearchScoreAnalyzer,
    _run_rust_search_plane_seed_binary,
    _score_rows,
    _scripted_repo_search_query_client,
    _spawn_wendao_search_flight_server,
    _terminate_process,
    analyze_query,
    pytest,
    repo_search_metadata,
    repo_search_query,
    repo_search_request,
    run_query_analysis,
    socket,
    summarize_query,
    summarize_query_results,
    summarize_query_route,
    summarize_repo_query_text_results,
)


def test_summarize_repo_query_text_results_returns_top_row_snapshot() -> None:
    request = repo_search_request("alpha", limit=2, path_prefixes=("src/",))
    client = _scripted_repo_search_query_client(_score_rows())

    summary = summarize_repo_query_text_results(
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


@pytest.mark.integration
def test_run_query_analysis_reads_repo_search_rows_via_wendao_search_flight_server(
    tmp_path,
) -> None:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        host, port = sock.getsockname()

    _run_rust_search_plane_seed_binary(str(tmp_path))
    process = _spawn_wendao_search_flight_server(host, port, str(tmp_path))
    try:
        client = WendaoTransportClient(
            WendaoTransportConfig(
                endpoint=WendaoTransportEndpoint(host=host, port=port),
                schema_version="v2",
                request_timeout_seconds=10.0,
            )
        )
        query = repo_search_query()
        run = run_query_analysis(
            client,
            query,
            analyzer=_RepoSearchScoreAnalyzer(),
            extra_metadata=repo_search_metadata(
                repo_search_request("alpha", limit=3, path_prefixes=("src/",))
            ),
        )

        assert run.query == query
        assert run.rows
        assert all((row.path or "").startswith("src/") for row in run.rows)
        assert [row.rank for row in run.rows] == list(range(1, len(run.rows) + 1))
    finally:
        _terminate_process(process)


@pytest.mark.integration
def test_analyze_query_reads_repo_search_rows_via_wendao_search_flight_server(
    tmp_path,
) -> None:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        host, port = sock.getsockname()

    _run_rust_search_plane_seed_binary(str(tmp_path))
    process = _spawn_wendao_search_flight_server(host, port, str(tmp_path))
    try:
        client = WendaoTransportClient(
            WendaoTransportConfig(
                endpoint=WendaoTransportEndpoint(host=host, port=port),
                schema_version="v2",
                request_timeout_seconds=10.0,
            )
        )
        request = repo_search_request("alpha", limit=3, path_prefixes=("src/",))
        ranked = analyze_query(
            client,
            repo_search_query(),
            analyzer=_RepoSearchScoreAnalyzer(),
            extra_metadata=repo_search_metadata(request),
        )

        assert ranked
        assert all(str(row["path"]).startswith("src/") for row in ranked)
        assert [int(row["rank"]) for row in ranked] == list(range(1, len(ranked) + 1))
        assert [float(row["score"]) for row in ranked] == sorted(
            [float(row["score"]) for row in ranked],
            reverse=True,
        )
    finally:
        _terminate_process(process)


@pytest.mark.integration
def test_summarize_query_reads_repo_search_rows_via_wendao_search_flight_server(
    tmp_path,
) -> None:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        host, port = sock.getsockname()

    _run_rust_search_plane_seed_binary(str(tmp_path))
    process = _spawn_wendao_search_flight_server(host, port, str(tmp_path))
    try:
        client = WendaoTransportClient(
            WendaoTransportConfig(
                endpoint=WendaoTransportEndpoint(host=host, port=port),
                schema_version="v2",
                request_timeout_seconds=10.0,
            )
        )
        summary = summarize_query(
            run_query_analysis(
                client,
                repo_search_query(),
                analyzer=_RepoSearchScoreAnalyzer(),
                extra_metadata=repo_search_metadata(
                    repo_search_request("alpha", limit=3, path_prefixes=("src/",))
                ),
            )
        )

        assert summary.row_count >= 1
        assert summary.top_path is not None
        assert summary.top_path.startswith("src/")
        assert summary.top_rank == 1
    finally:
        _terminate_process(process)


@pytest.mark.integration
def test_summarize_query_route_reads_repo_search_rows_via_wendao_search_flight_server(
    tmp_path,
) -> None:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        host, port = sock.getsockname()

    _run_rust_search_plane_seed_binary(str(tmp_path))
    process = _spawn_wendao_search_flight_server(host, port, str(tmp_path))
    try:
        client = WendaoTransportClient(
            WendaoTransportConfig(
                endpoint=WendaoTransportEndpoint(host=host, port=port),
                schema_version="v2",
                request_timeout_seconds=10.0,
            )
        )
        summary = summarize_query_route(
            client,
            repo_search_query(),
            analyzer=_RepoSearchScoreAnalyzer(),
            extra_metadata=repo_search_metadata(
                repo_search_request("alpha", limit=3, path_prefixes=("src/",))
            ),
        )

        assert summary.row_count >= 1
        assert summary.top_path is not None
        assert summary.top_path.startswith("src/")
        assert summary.top_rank == 1
    finally:
        _terminate_process(process)


@pytest.mark.integration
def test_summarize_query_results_reads_repo_search_rows_via_wendao_search_flight_server(
    tmp_path,
) -> None:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        host, port = sock.getsockname()

    _run_rust_search_plane_seed_binary(str(tmp_path))
    process = _spawn_wendao_search_flight_server(host, port, str(tmp_path))
    try:
        client = WendaoTransportClient(
            WendaoTransportConfig(
                endpoint=WendaoTransportEndpoint(host=host, port=port),
                schema_version="v2",
                request_timeout_seconds=10.0,
            )
        )
        summary = summarize_query_results(
            client,
            repo_search_query(),
            analyzer=_RepoSearchScoreAnalyzer(),
            extra_metadata=repo_search_metadata(
                repo_search_request("alpha", limit=3, path_prefixes=("src/",))
            ),
        )

        assert summary.row_count >= 1
        assert summary.top_path is not None
        assert summary.top_path.startswith("src/")
        assert summary.top_rank == 1
    finally:
        _terminate_process(process)
