"""Runtime environment and process readiness helpers."""

from __future__ import annotations

from .common import (
    Path,
    os,
    socket,
    subprocess,
    sys,
    time,
)


def resolve_project_root() -> Path:
    return Path(os.environ.get("PRJ_ROOT", Path.cwd())).resolve()


def rust_process_env() -> dict[str, str]:
    env = dict(os.environ)
    if sys.platform == "darwin" and ("SDKROOT" not in env or "LIBRARY_PATH" not in env):
        try:
            sdk_path = subprocess.run(
                ["xcrun", "--show-sdk-path"],
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()
        except (OSError, subprocess.CalledProcessError):
            sdk_path = ""
        if sdk_path:
            env.setdefault("SDKROOT", sdk_path)
            env.setdefault("LIBRARY_PATH", str(Path(sdk_path) / "usr/lib"))
    return env


def wait_for_port(
    host: str,
    port: int,
    server: subprocess.Popen[str],
    *,
    timeout_seconds: float,
) -> None:
    deadline = time.monotonic() + timeout_seconds
    while time.monotonic() < deadline:
        if server.poll() is not None:
            raise RuntimeError(
                "document extract service exited before listening:\n"
                + process_log_tail(server)
            )
        try:
            with socket.create_connection((host, port), timeout=1):
                return
        except OSError:
            time.sleep(0.2)
    raise TimeoutError(
        f"document extract service did not listen on {host}:{port} "
        f"within {timeout_seconds:.1f}s\n{process_log_tail(server)}"
    )


def wait_for_document_extract_flight_endpoint(
    host: str,
    port: int,
    server: subprocess.Popen[str],
    *,
    timeout_seconds: float,
) -> None:
    deadline = time.monotonic() + timeout_seconds
    location = f"grpc://{host}:{port}"
    while time.monotonic() < deadline:
        if server.poll() is not None:
            raise RuntimeError(
                "document extract service exited before Flight readiness:\n"
                + process_log_tail(server)
            )
        try:
            import pyarrow.flight as flight

            client = flight.FlightClient(location)
            descriptor = flight.FlightDescriptor.for_path(
                "analysis", "document-extract"
            )
            client.get_flight_info(descriptor)
            return
        except Exception:
            time.sleep(0.2)
    raise TimeoutError(
        f"document extract Flight endpoint did not become ready on {location} "
        f"within {timeout_seconds:.1f}s\n{process_log_tail(server)}"
    )


def process_log_tail(server: subprocess.Popen[str]) -> str:
    stderr_log = getattr(server, "wendao_stderr_log", None)
    stdout_log = getattr(server, "wendao_stdout_log", None)
    parts = []
    if stderr_log is not None:
        parts.append(f"stderr log: {stderr_log}\n{tail_file(Path(stderr_log))}")
    elif server.stderr is not None:
        parts.append(server.stderr.read())
    if stdout_log is not None:
        parts.append(f"stdout log: {stdout_log}\n{tail_file(Path(stdout_log))}")
    elif server.stdout is not None:
        parts.append(server.stdout.read())
    return "\n".join(part for part in parts if part).strip()


def tail_file(path: Path, limit: int = 4000) -> str:
    if not path.exists():
        return ""
    text = path.read_text(encoding="utf-8", errors="replace")
    return text[-limit:]
