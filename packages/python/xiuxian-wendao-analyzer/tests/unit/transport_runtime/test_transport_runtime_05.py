"""transport_runtime test slice 5."""

from __future__ import annotations

from .support import (
    AnalyzerConfig,
    WendaoTransportClient,
    WendaoTransportConfig,
    WendaoTransportEndpoint,
    _RepoSearchScoreAnalyzer,
    _run_rust_search_plane_seed_binary,
    _spawn_wendao_search_flight_server,
    _terminate_process,
    analyze_repo_search,
    pytest,
    repo_search_request,
    socket,
    summarize_repo_search_results,
)


@pytest.mark.integration
def test_analyze_repo_search_reads_rows_via_wendao_search_flight_server(
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
        ranked = analyze_repo_search(
            client,
            repo_search_request("alpha", limit=3, path_prefixes=("src/",)),
            analyzer=_RepoSearchScoreAnalyzer(),
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
def test_summarize_repo_search_results_reads_rows_via_wendao_search_flight_server(
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
        summary = summarize_repo_search_results(
            client,
            repo_search_request("alpha", limit=3, path_prefixes=("src/",)),
            config=AnalyzerConfig(),
        )

        assert summary.row_count >= 1
        assert summary.top_path is not None
        assert summary.top_path.startswith("src/")
        assert summary.top_rank == 1
    finally:
        _terminate_process(process)
