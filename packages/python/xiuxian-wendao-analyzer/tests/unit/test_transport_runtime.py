from __future__ import annotations

import os
import socket
import subprocess
import time
from typing import TYPE_CHECKING

import pytest

from wendao_arrow_interface import (
    WendaoArrowCall,
    WendaoArrowResult,
    WendaoArrowScriptedClient,
)
from wendao_core_lib import (
    WendaoFlightRouteQuery,
    WendaoTransportClient,
    WendaoTransportConfig,
    WendaoTransportEndpoint,
    repo_search_metadata,
    repo_search_query,
    repo_search_request,
)
from xiuxian_wendao_analyzer import (
    AnalyzerConfig,
    analyze_query,
    analyze_query_results,
    analyze_repo_query_text,
    analyze_repo_query_text_results,
    analyze_repo_search,
    analyze_repo_search_results,
    run_query_analysis,
    run_repo_analysis,
    run_repo_search_analysis,
    summarize_query,
    summarize_query_results,
    summarize_query_route,
    summarize_repo_analysis,
    summarize_repo_query_text,
    summarize_repo_query_text_results,
    summarize_repo_search,
    summarize_repo_search_results,
)

if TYPE_CHECKING:
    import pyarrow as pa


def _score_rows() -> list[dict[str, object]]:
    return [
        {"path": "src/main.rs", "score": 0.3},
        {"path": "src/lib.rs", "score": 0.9},
    ]


def _result_table(route: str, rows: list[dict[str, object]]) -> pa.Table:
    return WendaoArrowResult.from_query_rows(rows, route=route).table


def _scripted_query_client(
    query: WendaoFlightRouteQuery,
    rows: list[dict[str, object]],
) -> WendaoArrowScriptedClient:
    return WendaoArrowScriptedClient.for_query_route(
        query.normalized_route(),
        _result_table(query.normalized_route(), rows),
    )


def _scripted_repo_search_query_client(
    rows: list[dict[str, object]],
) -> WendaoArrowScriptedClient:
    return _scripted_query_client(repo_search_query(), rows)


def _assert_single_query_call(
    client: WendaoArrowScriptedClient,
    query: WendaoFlightRouteQuery,
    *,
    extra_metadata: dict[str, str] | None = None,
    connect_kwargs: dict[str, object] | None = None,
) -> None:
    assert client.calls == [
        WendaoArrowCall(
            operation="query",
            route=query.normalized_route(),
            query=query,
            extra_metadata=extra_metadata or {},
            connect_kwargs=connect_kwargs or {},
        )
    ]


def _assert_repo_search_metadata_call(
    client: WendaoArrowScriptedClient,
    request,
) -> None:
    _assert_single_query_call(
        client,
        repo_search_query(),
        extra_metadata=repo_search_metadata(request),
    )


class _RepoSearchScoreAnalyzer:
    def analyze_rows(self, rows: list[dict[str, object]]) -> list[dict[str, object]]:
        ranked = sorted(rows, key=lambda row: float(row["score"]), reverse=True)
        return [
            {
                "path": str(row["path"]),
                "score": float(row["score"]),
                "rank": index + 1,
            }
            for index, row in enumerate(ranked)
        ]


class _DocIdAnalyzer:
    def analyze_rows(self, rows: list[dict[str, object]]) -> list[dict[str, object]]:
        ranked = sorted(rows, key=lambda row: str(row["doc_id"]))
        return [
            {"doc_id": str(row["doc_id"]), "rank": index + 1}
            for index, row in enumerate(ranked)
        ]


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


def _project_root() -> str:
    project_root = os.environ.get("PRJ_ROOT")
    if not project_root:
        pytest.skip("set PRJ_ROOT before running analyzer real-host integration tests")
    return project_root


def _wendao_search_flight_server_binary() -> str:
    return os.path.join(
        _project_root(),
        ".cache",
        "pyflight-f56-target",
        "debug",
        "wendao_search_flight_server",
    )


def _wendao_search_seed_binary() -> str:
    return os.path.join(
        _project_root(),
        ".cache",
        "pyflight-f56-target",
        "debug",
        "wendao_search_seed_sample",
    )


def _run_rust_search_plane_seed_binary(
    project_root: str, *, repo_id: str = "alpha/repo"
) -> None:
    binary = os.environ.get("WENDAO_SEARCH_SEED_BINARY", _wendao_search_seed_binary())
    if not os.path.exists(binary):
        pytest.skip(
            f"build {binary} before running analyzer real-host integration tests"
        )

    result = subprocess.run(
        [binary, repo_id, project_root],
        cwd=_project_root(),
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise AssertionError(
            "Wendao search-plane seed binary failed:\n"
            f"stdout:\n{result.stdout}\n"
            f"stderr:\n{result.stderr}"
        )


def _spawn_wendao_search_flight_server(
    host: str, port: int, project_root: str
) -> subprocess.Popen[str]:
    binary = os.environ.get(
        "WENDAO_SEARCH_SERVER_BINARY", _wendao_search_flight_server_binary()
    )
    if not os.path.exists(binary):
        pytest.skip(
            f"build {binary} before running analyzer real-host integration tests"
        )

    process = subprocess.Popen(
        [
            binary,
            f"{host}:{port}",
            "--schema-version=v2",
            "alpha/repo",
            project_root,
            "3",
        ],
        cwd=project_root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env={**os.environ, "PRJ_ROOT": project_root},
    )
    ready_line = ""
    deadline = time.time() + 120
    while time.time() < deadline:
        line = process.stdout.readline() if process.stdout is not None else ""
        if line.startswith("READY http://"):
            ready_line = line.strip()
            break
        if process.poll() is not None:
            stderr = process.stderr.read() if process.stderr is not None else ""
            raise AssertionError(
                f"Wendao search Flight server exited before readiness:\n{stderr}"
            )
    if not ready_line:
        raise AssertionError(
            "timed out waiting for Wendao search Flight server readiness"
        )
    time.sleep(1.0)
    return process


def _terminate_process(process: subprocess.Popen[str]) -> None:
    process.terminate()
    try:
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=10)


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
