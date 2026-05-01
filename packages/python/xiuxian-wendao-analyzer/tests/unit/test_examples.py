from __future__ import annotations

import os
import socket
import subprocess
import time
from pathlib import Path

import pytest


def _package_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _project_root() -> Path:
    project_root = os.environ.get("PRJ_ROOT")
    if not project_root:
        pytest.skip("set PRJ_ROOT before running analyzer example integration tests")
    return Path(project_root)


def _wendao_search_flight_server_binary() -> Path:
    return (
        _project_root()
        / ".cache"
        / "pyflight-f56-target"
        / "debug"
        / "wendao_search_flight_server"
    )


def _wendao_search_seed_binary() -> Path:
    return (
        _project_root()
        / ".cache"
        / "pyflight-f56-target"
        / "debug"
        / "wendao_search_seed_sample"
    )


def _require_host_backed_repo_beta_binaries() -> None:
    search_binary = Path(
        os.environ.get(
            "WENDAO_SEARCH_SERVER_BINARY", str(_wendao_search_flight_server_binary())
        )
    )
    seed_binary = Path(
        os.environ.get("WENDAO_SEARCH_SEED_BINARY", str(_wendao_search_seed_binary()))
    )
    if not search_binary.exists():
        pytest.skip(
            f"build {search_binary} before running analyzer example integration tests"
        )
    if not seed_binary.exists():
        pytest.skip(
            f"build {seed_binary} before running analyzer example integration tests"
        )


def _run_rust_search_plane_seed_binary(
    project_root: Path, *, repo_id: str = "alpha/repo"
) -> None:
    binary = Path(
        os.environ.get("WENDAO_SEARCH_SEED_BINARY", str(_wendao_search_seed_binary()))
    )
    if not binary.exists():
        pytest.skip(f"build {binary} before running analyzer example integration tests")

    result = subprocess.run(
        [str(binary), repo_id, str(project_root)],
        cwd=_project_root(),
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise AssertionError(
            "Wendao search-plane seed binary failed:\n"
            f"stdout:\n{result.stdout}\n"
            f"stderr:\n{result.stderr}"
        )


def _spawn_wendao_search_flight_server(
    host: str, port: int, project_root: Path
) -> subprocess.Popen[str]:
    binary = Path(
        os.environ.get(
            "WENDAO_SEARCH_SERVER_BINARY", str(_wendao_search_flight_server_binary())
        )
    )
    if not binary.exists():
        pytest.skip(f"build {binary} before running analyzer example integration tests")

    process = subprocess.Popen(
        [
            str(binary),
            f"{host}:{port}",
            "--schema-version=v2",
            "alpha/repo",
            str(project_root),
            "3",
        ],
        cwd=project_root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env={**os.environ, "PRJ_ROOT": str(project_root)},
    )
    deadline = time.time() + 120
    while time.time() < deadline:
        line = process.stdout.readline() if process.stdout is not None else ""
        if line.startswith("READY http://"):
            time.sleep(1.0)
            return process
        if process.poll() is not None:
            stderr = process.stderr.read() if process.stderr is not None else ""
            raise AssertionError(
                f"Wendao search Flight server exited before readiness:\n{stderr}"
            )

    raise AssertionError("timed out waiting for Wendao search Flight server readiness")


def _terminate_process(process: subprocess.Popen[str]) -> None:
    process.terminate()
    try:
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=10)


def _run_example_via_uv(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["uv", "run", "python", *args],
        cwd=_package_root(),
        check=True,
        capture_output=True,
        text=True,
    )


def test_shipped_example_set_matches_current_beta_freeze() -> None:
    example_names = {
        path.name
        for path in (_package_root() / "examples").glob("*.py")
        if path.is_file()
    }

    assert example_names == {
        "attachment_pdf_analyzer_workflow.py",
        "custom_repo_analyzer_workflow.py",
        "document_extraction_workflow.py",
        "host_backed_repo_search_beta_smoke.py",
        "repo_search_workflow.py",
        "scripted_repo_search_workflow.py",
    }


def test_scripted_repo_search_example_runs() -> None:
    result = _run_example_via_uv("examples/scripted_repo_search_workflow.py")

    assert "query_text= alpha" in result.stdout
    assert "rows= 3" in result.stdout
    assert "top_path= src/alpha.py" in result.stdout
    assert "top_rank= 1" in result.stdout
    assert "recorded_calls= 1" in result.stdout
    assert "recorded_route= /search/repos/main" in result.stdout


def test_attachment_pdf_analyzer_example_runs_scripted() -> None:
    result = _run_example_via_uv("examples/attachment_pdf_analyzer_workflow.py")

    assert "mode= scripted" in result.stdout
    assert "query_text= architecture" in result.stdout
    assert "rows= 2" in result.stdout
    assert "top_path= assets/design-review.pdf" in result.stdout
    assert "top_rank= 1" in result.stdout
    assert "top_attachment_name= design-review.pdf" in result.stdout
    assert "top_source_title= Architecture Notes" in result.stdout
    assert "recorded_calls= 1" in result.stdout
    assert "recorded_route= /search/attachments" in result.stdout


def test_document_extraction_example_runs_fixture_mode() -> None:
    result = _run_example_via_uv("examples/document_extraction_workflow.py")

    assert "mode= fixture" in result.stdout
    assert "known_docling_source= True" in result.stdout
    assert "supported_formats= PDF,DOCX,XLSX,PPTX" in result.stdout
    assert "common_suffixes= .pdf,.docx,.xlsx,.pptx" in result.stdout
    assert "rows= 1" in result.stdout
    assert (
        "sourcePath,resourceType,resourcePath,pageIndex,caption,content,mimeType,status,elementId"
        in result.stdout
    )
    assert "top_status= ok" in result.stdout
    assert "top_resource_type= document" in result.stdout
    assert "top_mime_type= text/markdown" in result.stdout
    assert "top_content= # Parsed fixture" in result.stdout


def test_repo_search_example_exposes_help() -> None:
    result = _run_example_via_uv("examples/repo_search_workflow.py", "--help")

    assert "Run a host-backed repo-search analyzer workflow." in result.stdout
    assert "--query-text" in result.stdout
    assert "--path-prefix" in result.stdout


def test_custom_repo_search_example_exposes_help() -> None:
    result = _run_example_via_uv("examples/custom_repo_analyzer_workflow.py", "--help")

    assert (
        "Run a host-backed repo-search workflow with a custom Python analyzer."
        in result.stdout
    )
    assert "--query-text" in result.stdout
    assert "--path-prefix" in result.stdout


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
