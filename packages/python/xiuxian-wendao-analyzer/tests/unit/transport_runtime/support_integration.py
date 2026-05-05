"""Integration helpers for transport runtime tests."""

from __future__ import annotations

import os
import subprocess
import time

import pytest


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
