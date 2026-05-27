#!/usr/bin/env python3
"""Check runtime web service readiness using pidfile ownership and root HTTP."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any, Callable

Opener = Callable[[Any, float], Any]
ProcessExists = Callable[[int], bool]
ListenerPid = Callable[[int], int | None]
ProcessCommand = Callable[[int], str]


def read_expected_pid(pidfile: Path) -> int:
    contents = pidfile.read_text(encoding="utf-8").strip()
    if not contents:
        raise ValueError(f"pidfile is empty: {pidfile}")
    try:
        return int(contents)
    except ValueError as error:
        raise ValueError(
            f"pidfile does not contain a valid process id: {pidfile}"
        ) from error


def process_exists(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def listening_process_id(port: int) -> int | None:
    try:
        result = subprocess.run(
            ["lsof", "-nP", f"-iTCP:{port}", "-sTCP:LISTEN", "-t"],
            check=False,
            capture_output=True,
            text=True,
        )
    except OSError:
        return None

    for line in result.stdout.splitlines():
        candidate = line.strip()
        if not candidate:
            continue
        try:
            return int(candidate)
        except ValueError:
            continue
    return None


def process_command(pid: int) -> str:
    try:
        result = subprocess.run(
            ["ps", "-p", str(pid), "-o", "command="],
            check=False,
            capture_output=True,
            text=True,
        )
    except OSError:
        return ""
    return result.stdout.strip()


def is_expected_web_app_command(command: str, service_name: str) -> bool:
    patterns = {
        "wendao.ai": (
            "rsbuild",
            "rsbuild.js",
            "rspack-node",
        ),
    }.get(
        service_name,
        (
            "rspack-node",
            "webpack-dev-server",
            "rspack dev",
        ),
    )
    return any(pattern in command for pattern in patterns)


def is_runtime_web_healthy(
    *,
    host: str,
    port: int,
    pidfile: Path,
    timeout_secs: float,
    service_name: str,
    opener: Opener = urllib.request.urlopen,
    pid_exists: ProcessExists = process_exists,
    listener_pid_for_port: ListenerPid = listening_process_id,
    process_command_for_pid: ProcessCommand = process_command,
) -> tuple[bool, str]:
    pid_issue: str | None = None
    try:
        expected_pid = read_expected_pid(pidfile)
    except OSError as error:
        expected_pid = None
        pid_issue = f"failed to read pidfile {pidfile}: {error}"
    except ValueError as error:
        expected_pid = None
        pid_issue = str(error)

    if expected_pid is not None and not pid_exists(expected_pid):
        pid_issue = f"{service_name} process from pidfile is not alive: {expected_pid}"

    listener_pid = listener_pid_for_port(port)
    if listener_pid is None:
        return False, pid_issue or f"{service_name} is not listening on {host}:{port}"

    listener_command = process_command_for_pid(listener_pid)
    if listener_command and not is_expected_web_app_command(listener_command, service_name):
        return (
            False,
            f"{service_name} listener on {host}:{port} is owned by unexpected process {listener_pid}: {listener_command}",
        )

    service_url = f"http://{host}:{port}/"
    try:
        with opener(service_url, timeout=timeout_secs) as response:
            status = getattr(response, "status", None)
            response.read(1)
    except urllib.error.HTTPError as error:
        return False, f"{service_name} returned HTTP {error.code}: {service_url}"
    except urllib.error.URLError as error:
        return False, f"{service_name} unreachable: {service_url} ({error.reason})"

    if status != 200:
        return False, f"{service_name} returned HTTP {status}: {service_url}"

    return True, "healthy"


def main(default_service_name: str = "runtime-web") -> int:
    parser = argparse.ArgumentParser(description="Check runtime web service readiness")
    parser.add_argument(
        "--service-name",
        default=default_service_name,
        help="Service label used in healthcheck diagnostics",
    )
    parser.add_argument("--host", default="127.0.0.1", help="Web service host")
    parser.add_argument("--port", type=int, required=True, help="Web service port")
    parser.add_argument("--pidfile", type=Path, required=True, help="Web service pidfile")
    parser.add_argument(
        "--timeout-secs",
        type=float,
        default=2.0,
        help="Per-request timeout in seconds",
    )
    args = parser.parse_args()

    healthy, message = is_runtime_web_healthy(
        host=args.host,
        port=args.port,
        pidfile=args.pidfile,
        timeout_secs=args.timeout_secs,
        service_name=args.service_name,
    )
    if healthy:
        print(message)
        return 0
    print(f"Error: {message}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
