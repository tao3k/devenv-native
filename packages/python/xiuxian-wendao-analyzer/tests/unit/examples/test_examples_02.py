"""examples test slice 2."""

from __future__ import annotations

from .support import (
    _require_host_backed_repo_beta_binaries,
    _run_example_via_uv,
    _run_rust_search_plane_seed_binary,
    _spawn_wendao_search_flight_server,
    _terminate_process,
    pytest,
    socket,
)


def test_attachment_pdf_analyzer_example_exposes_help() -> None:
    result = _run_example_via_uv(
        "examples/attachment_pdf_analyzer_workflow.py", "--help"
    )

    assert "attachment_pdf_analyzer_workflow.py" in result.stdout
    assert "--mode {scripted,endpoint}" in result.stdout
    assert "--ext-filter" in result.stdout
    assert "--kind-filter" in result.stdout


def test_document_extraction_example_exposes_help() -> None:
    result = _run_example_via_uv("examples/document_extraction_workflow.py", "--help")

    assert "Docling-backed multi-format document extraction workflow" in result.stdout
    assert "--mode {fixture,docling}" in result.stdout
    assert "--source" in result.stdout
    assert "--error-row" in result.stdout


def test_host_backed_beta_smoke_example_exposes_help() -> None:
    result = _run_example_via_uv(
        "examples/host_backed_repo_search_beta_smoke.py", "--help"
    )

    assert "Run the full host-backed repo-search beta smoke path." in result.stdout
    assert "--build" in result.stdout
    assert "--mode {built_in,custom}" in result.stdout
    assert "--keep-workspace" in result.stdout
    assert "--workspace-root" in result.stdout
    assert "--repo-id" in result.stdout


@pytest.mark.integration
def test_repo_search_example_runs_via_runtime_search_server(tmp_path) -> None:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        host, port = sock.getsockname()

    _run_rust_search_plane_seed_binary(tmp_path)
    process = _spawn_wendao_search_flight_server(host, port, tmp_path)
    try:
        result = _run_example_via_uv(
            "examples/repo_search_workflow.py",
            "--host",
            host,
            "--port",
            str(port),
        )

        assert "query_text= alpha" in result.stdout
        assert "rows=" in result.stdout
        assert "top_path= src/" in result.stdout
        assert "top_rank= 1" in result.stdout
    finally:
        _terminate_process(process)


@pytest.mark.integration
def test_custom_repo_search_example_runs_via_runtime_search_server(tmp_path) -> None:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        host, port = sock.getsockname()

    _run_rust_search_plane_seed_binary(tmp_path)
    process = _spawn_wendao_search_flight_server(host, port, tmp_path)
    try:
        result = _run_example_via_uv(
            "examples/custom_repo_analyzer_workflow.py",
            "--host",
            host,
            "--port",
            str(port),
        )

        assert "query_text= alpha" in result.stdout
        assert "rows=" in result.stdout
        assert "top_path= src/" in result.stdout
        assert "top_rank= 1" in result.stdout
    finally:
        _terminate_process(process)


def test_host_backed_repo_search_beta_smoke_example_runs() -> None:
    _require_host_backed_repo_beta_binaries()

    result = _run_example_via_uv(
        "examples/host_backed_repo_search_beta_smoke.py",
        "--mode",
        "custom",
        "--port",
        "0",
    )

    assert "mode= custom" in result.stdout
    assert "keep_workspace= False" in result.stdout
    assert "query_text= alpha" in result.stdout
    assert "top_rank= 1" in result.stdout
