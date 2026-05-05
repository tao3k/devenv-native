"""Run the host-backed repo-search beta smoke workflow against local Rust binaries."""

from __future__ import annotations

import argparse
import os
import shutil
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path


def _package_root() -> Path:
    return Path(__file__).resolve().parents[1]


def _project_root() -> Path:
    env_root = os.environ.get("PRJ_ROOT")
    if env_root:
        return Path(env_root)
    return Path(__file__).resolve().parents[4]


def _default_search_server_binary() -> Path:
    return (
        _project_root()
        / ".cache"
        / "pyflight-f56-target"
        / "debug"
        / "wendao_search_flight_server"
    )


def _default_seed_binary() -> Path:
    return (
        _project_root()
        / ".cache"
        / "pyflight-f56-target"
        / "debug"
        / "wendao_search_seed_sample"
    )


def _parse_host_backed_smoke_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run the full host-backed repo-search beta smoke path.",
    )
    parser.add_argument("--build", action="store_true")
    parser.add_argument("--mode", choices=("built_in", "custom"), default="built_in")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8815)
    parser.add_argument("--query-text", default="alpha")
    parser.add_argument("--path-prefix", action="append", default=["src/"])
    parser.add_argument("--schema-version", default="v2")
    parser.add_argument("--repo-id", default="alpha/repo")
    parser.add_argument("--result-limit", type=int, default=3)
    parser.add_argument("--workspace-root")
    parser.add_argument("--keep-workspace", action="store_true")
    return parser.parse_args()


def _run_checked(
    command: list[str], *, cwd: Path, env: dict[str, str] | None = None
) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        command,
        cwd=cwd,
        check=False,
        capture_output=True,
        text=True,
        env=env,
    )
    if result.returncode != 0:
        raise RuntimeError(
            "command failed:\n"
            f"command: {' '.join(command)}\n"
            f"stdout:\n{result.stdout}\n"
            f"stderr:\n{result.stderr}"
        )
    return result


def _ensure_binaries(*, build: bool) -> tuple[Path, Path]:
    server_binary = Path(
        os.environ.get(
            "WENDAO_SEARCH_SERVER_BINARY", str(_default_search_server_binary())
        )
    )
    seed_binary = Path(
        os.environ.get("WENDAO_SEARCH_SEED_BINARY", str(_default_seed_binary()))
    )

    if build:
        _run_checked(
            [
                "direnv",
                "exec",
                ".",
                "cargo",
                "build",
                "-p",
                "xiuxian-wendao",
                "--features",
                "julia",
                "--bin",
                "wendao_search_flight_server",
                "--bin",
                "wendao_search_seed_sample",
            ],
            cwd=_project_root(),
        )

    if not server_binary.exists():
        raise RuntimeError(
            f"missing search server binary: {server_binary}\n"
            "Run with --build or set WENDAO_SEARCH_SERVER_BINARY."
        )
    if not seed_binary.exists():
        raise RuntimeError(
            f"missing seed binary: {seed_binary}\n"
            "Run with --build or set WENDAO_SEARCH_SEED_BINARY."
        )
    return server_binary, seed_binary


def _choose_port(host: str, requested_port: int) -> int:
    if requested_port != 0:
        return requested_port
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind((host, 0))
        return int(sock.getsockname()[1])


def _seed_workspace(seed_binary: Path, workspace_root: Path, repo_id: str) -> None:
    _run_checked([str(seed_binary), repo_id, str(workspace_root)], cwd=_project_root())


def _spawn_search_server(
    server_binary: Path,
    *,
    host: str,
    port: int,
    schema_version: str,
    repo_id: str,
    workspace_root: Path,
    result_limit: int,
) -> subprocess.Popen[str]:
    process = subprocess.Popen(
        [
            str(server_binary),
            f"{host}:{port}",
            f"--schema-version={schema_version}",
            repo_id,
            str(workspace_root),
            str(result_limit),
        ],
        cwd=workspace_root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env={**os.environ, "PRJ_ROOT": str(_project_root())},
    )
    deadline = time.time() + 120
    while time.time() < deadline:
        line = process.stdout.readline() if process.stdout is not None else ""
        if line.startswith("READY http://"):
            time.sleep(1.0)
            return process
        if process.poll() is not None:
            stderr = process.stderr.read() if process.stderr is not None else ""
            raise RuntimeError(f"search server exited before readiness:\n{stderr}")
    raise RuntimeError("timed out waiting for search server readiness")


def _terminate_process(process: subprocess.Popen[str]) -> None:
    process.terminate()
    try:
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=10)


def _run_repo_search_example(
    *,
    mode: str,
    host: str,
    port: int,
    query_text: str,
    path_prefixes: list[str],
    schema_version: str,
) -> subprocess.CompletedProcess[str]:
    example_path = (
        "examples/repo_search_workflow.py"
        if mode == "built_in"
        else "examples/custom_repo_analyzer_workflow.py"
    )
    command = [
        "uv",
        "run",
        "python",
        example_path,
        "--host",
        host,
        "--port",
        str(port),
        "--query-text",
        query_text,
        "--schema-version",
        schema_version,
    ]
    for prefix in path_prefixes:
        command.extend(["--path-prefix", prefix])
    return _run_checked(command, cwd=_package_root())


def _emit(label: str, value: object) -> None:
    sys.stdout.write(f"{label}= {value}\n")


def _run_host_backed_smoke() -> None:
    args = _parse_host_backed_smoke_args()
    host = args.host
    port = _choose_port(host, args.port)
    server_binary, seed_binary = _ensure_binaries(build=args.build)

    temporary_workspace = args.workspace_root is None
    workspace_root = (
        Path(tempfile.mkdtemp(prefix="wendao-analyzer-beta-smoke-"))
        if temporary_workspace
        else Path(args.workspace_root)
    )
    workspace_root.mkdir(parents=True, exist_ok=True)

    process: subprocess.Popen[str] | None = None
    try:
        _seed_workspace(seed_binary, workspace_root, args.repo_id)
        process = _spawn_search_server(
            server_binary,
            host=host,
            port=port,
            schema_version=args.schema_version,
            repo_id=args.repo_id,
            workspace_root=workspace_root,
            result_limit=args.result_limit,
        )
        result = _run_repo_search_example(
            mode=args.mode,
            host=host,
            port=port,
            query_text=args.query_text,
            path_prefixes=list(args.path_prefix),
            schema_version=args.schema_version,
        )
        _emit("mode", args.mode)
        _emit("workspace_root", workspace_root)
        _emit("keep_workspace", args.keep_workspace)
        _emit("host", host)
        _emit("port", port)
        sys.stdout.write(result.stdout)
    finally:
        if process is not None:
            _terminate_process(process)
        if temporary_workspace and not args.keep_workspace and workspace_root.exists():
            shutil.rmtree(workspace_root)


if __name__ == "__main__":
    _run_host_backed_smoke()
