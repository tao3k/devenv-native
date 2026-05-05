"""Shared helpers for examples tests."""

from __future__ import annotations

import os
import socket
import subprocess
import time
from pathlib import Path

import pytest


def _package_root() -> Path:
    return Path(__file__).resolve().parents[3]


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


__all__ = [
    "Path",
    "_package_root",
    "_project_root",
    "_require_host_backed_repo_beta_binaries",
    "_run_example_via_uv",
    "_run_rust_search_plane_seed_binary",
    "_spawn_wendao_search_flight_server",
    "_terminate_process",
    "_wendao_search_flight_server_binary",
    "_wendao_search_seed_binary",
    "os",
    "pytest",
    "socket",
    "subprocess",
    "time",
]
