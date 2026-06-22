#!/usr/bin/env python3
"""Check Wendao gateway readiness with health and Flight business-plane signals."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
import urllib.error
import urllib.request
from collections.abc import Callable
from pathlib import Path
from typing import Any

GATEWAY_PROCESS_ID_HEADER = "x-wendao-process-id"
EXPECTED_HEALTH_STATUS = 200
EXPECTED_HEALTH_SERVICE = "wendao-gateway"

Opener = Callable[[Any, float], Any]
ProcessExists = Callable[[int], bool]
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


def _normalize_process_id(raw_value: str | None) -> str:
    return (raw_value or "").strip()


def process_exists(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


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


def is_wendao_gateway_command(command: str) -> bool:
    return "target/debug/wendao" in command and " gateway start" in command


def line_reports_flight_business_plane_ready(line: str) -> bool:
    return (
        "arrow.flight.protocol.FlightService" in line
        and "Arrow Flight business plane" in line
        and "POST " in line
    )


def log_reports_flight_business_plane_ready(logfile: Path) -> bool:
    try:
        with logfile.open("r", encoding="utf-8") as handle:
            for line in handle:
                if line_reports_flight_business_plane_ready(line):
                    return True
    except OSError:
        return False
    return False


def _health_payload_reports_flight_ready(payload: dict[str, Any]) -> bool:
    planes = payload.get("planes")
    return isinstance(planes, dict) and planes.get("flight") == "mounted"


def _parse_health_payload(raw_payload: bytes) -> tuple[bool, str, str, bool]:
    try:
        payload = json.loads(raw_payload.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        return False, f"health endpoint returned invalid json: {error}", "", False

    if not isinstance(payload, dict):
        return False, "health endpoint returned a non-object payload", "", False

    if payload.get("service") != EXPECTED_HEALTH_SERVICE:
        return (
            False,
            "health endpoint reported an unexpected service "
            f"({payload.get('service')!r} != {EXPECTED_HEALTH_SERVICE!r})",
            "",
            False,
        )

    if payload.get("ready") is not True:
        return (
            False,
            f"health endpoint did not report ready=true: {payload!r}",
            "",
            False,
        )

    process_id = _normalize_process_id(str(payload.get("processId", "")))
    return True, "ok", process_id, _health_payload_reports_flight_ready(payload)


def is_gateway_healthy(
    *,
    host: str,
    port: int,
    pidfile: Path,
    logfile: Path,
    timeout_secs: float,
    opener: Opener = urllib.request.urlopen,
    pid_exists: ProcessExists = process_exists,
    process_command_for_pid: ProcessCommand = process_command,
) -> tuple[bool, str]:
    try:
        expected_pid = read_expected_pid(pidfile)
    except (OSError, ValueError):
        expected_pid = None

    health_url = f"http://{host}:{port}/api/health"
    try:
        with opener(health_url, timeout=timeout_secs) as response:
            health_status = getattr(response, "status", None)
            reported_pid = _normalize_process_id(
                response.headers.get(GATEWAY_PROCESS_ID_HEADER)
            )
            raw_payload = response.read()
    except urllib.error.HTTPError as error:
        return False, f"health endpoint returned HTTP {error.code}: {health_url}"
    except urllib.error.URLError as error:
        return False, f"health endpoint unreachable: {health_url} ({error.reason})"

    if health_status != EXPECTED_HEALTH_STATUS:
        return False, f"health endpoint returned HTTP {health_status}: {health_url}"

    (
        payload_ok,
        payload_message,
        payload_process_id,
        payload_reports_flight_ready,
    ) = _parse_health_payload(raw_payload)
    if not payload_ok:
        return False, payload_message

    actual_pid = reported_pid or payload_process_id

    if actual_pid:
        try:
            candidate_pid = int(actual_pid)
        except ValueError:
            return (
                False,
                f"health endpoint reported an invalid process id: {actual_pid!r}",
            )
    elif expected_pid is not None:
        candidate_pid = expected_pid
    else:
        return (
            False,
            "health endpoint did not report a process id and pidfile is unavailable",
        )

    if not pid_exists(candidate_pid):
        return False, f"wendao-gateway process is not alive: {candidate_pid}"

    command = process_command_for_pid(candidate_pid)
    if command and not is_wendao_gateway_command(command):
        return False, f"wendao-gateway process {candidate_pid} is unexpected: {command}"

    if (
        not payload_reports_flight_ready
        and not log_reports_flight_business_plane_ready(logfile)
    ):
        return (
            False,
            "gateway health payload and log have not reported the "
            f"Flight business plane yet: {logfile}",
        )

    return True, "healthy"


def is_transient_healthcheck_failure(message: str) -> bool:
    if "health endpoint unreachable" not in message:
        return False
    transient_markers = (
        "Can't assign requested address",
        "Connection refused",
        "Operation timed out",
        "timed out",
        "Temporary failure",
    )
    return any(marker in message for marker in transient_markers)


def is_gateway_healthy_with_retries(
    *,
    host: str,
    port: int,
    pidfile: Path,
    logfile: Path,
    timeout_secs: float,
    attempts: int,
    retry_delay_secs: float,
    opener: Opener = urllib.request.urlopen,
    pid_exists: ProcessExists = process_exists,
    process_command_for_pid: ProcessCommand = process_command,
    sleeper: Callable[[float], None] = time.sleep,
) -> tuple[bool, str]:
    normalized_attempts = max(1, attempts)
    for attempt_index in range(normalized_attempts):
        healthy, message = is_gateway_healthy(
            host=host,
            port=port,
            pidfile=pidfile,
            logfile=logfile,
            timeout_secs=timeout_secs,
            opener=opener,
            pid_exists=pid_exists,
            process_command_for_pid=process_command_for_pid,
        )
        if (
            healthy
            or attempt_index + 1 >= normalized_attempts
            or not is_transient_healthcheck_failure(message)
        ):
            return healthy, message
        sleeper(max(0.0, retry_delay_secs))

    return False, "gateway healthcheck retry loop ended unexpectedly"


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Check Wendao gateway readiness against health and Flight routes"
    )
    parser.add_argument("--host", default="127.0.0.1", help="Gateway host")
    parser.add_argument("--port", type=int, required=True, help="Gateway port")
    parser.add_argument(
        "--pidfile",
        type=Path,
        required=True,
        help="Pidfile written by the Wendao gateway launcher",
    )
    parser.add_argument(
        "--logfile",
        type=Path,
        required=True,
        help="Gateway stderr log file written by the process launcher",
    )
    parser.add_argument(
        "--timeout-secs",
        type=float,
        default=2.0,
        help="Per-request timeout in seconds",
    )
    parser.add_argument(
        "--attempts",
        type=int,
        default=1,
        help="Health request attempts inside one process-compose probe",
    )
    parser.add_argument(
        "--retry-delay-secs",
        type=float,
        default=0.2,
        help="Delay between transient health request failures",
    )
    args = parser.parse_args()

    healthy, message = is_gateway_healthy_with_retries(
        host=args.host,
        port=args.port,
        pidfile=args.pidfile,
        logfile=args.logfile,
        timeout_secs=args.timeout_secs,
        attempts=args.attempts,
        retry_delay_secs=args.retry_delay_secs,
    )
    if healthy:
        print(message)
        return 0
    print(f"Error: {message}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
