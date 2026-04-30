"""Local process lifecycle helpers."""

from __future__ import annotations

from .common import (
    Path,
    os,
    signal,
    subprocess,
)


def start_logged_process(
    command: list[str],
    *,
    log_dir: Path,
    name: str,
    env: dict[str, str] | None = None,
) -> subprocess.Popen[str]:
    log_dir.mkdir(parents=True, exist_ok=True)
    stdout_path = log_dir / f"{name}.stdout.log"
    stderr_path = log_dir / f"{name}.stderr.log"
    stdout_file = stdout_path.open("w", encoding="utf-8")
    stderr_file = stderr_path.open("w", encoding="utf-8")
    try:
        process = subprocess.Popen(
            command,
            stdout=stdout_file,
            stderr=stderr_file,
            text=True,
            env=env,
            start_new_session=True,
        )
    finally:
        stdout_file.close()
        stderr_file.close()
    process.wendao_stdout_log = stdout_path
    process.wendao_stderr_log = stderr_path
    return process


def terminate_server(server: subprocess.Popen[str] | None) -> None:
    if server is None:
        return
    if server.poll() is not None:
        return
    terminate_process_group(server, signal.SIGTERM)
    try:
        server.wait(timeout=10)
    except subprocess.TimeoutExpired:
        terminate_process_group(server, signal.SIGKILL)
        server.wait(timeout=10)


def terminate_process_group(server: subprocess.Popen[str], sig: signal.Signals) -> None:
    try:
        os.killpg(server.pid, sig)
    except ProcessLookupError:
        pass
    except OSError:
        if sig == signal.SIGTERM:
            server.terminate()
        else:
            server.kill()
