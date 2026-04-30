"""HTTP readiness and Rust job status sampling helpers."""

from __future__ import annotations

from .common import (
    Any,
    json,
    socket,
    subprocess,
    time,
    urllib,
)


def normalize_rest_endpoint(endpoint: str | None) -> str | None:
    if endpoint is None:
        return None
    endpoint = endpoint.strip().rstrip("/")
    return endpoint or None


def pick_free_port(host: str) -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        listener.bind((host, 0))
        return int(listener.getsockname()[1])


def wait_for_http_endpoint(
    url: str,
    server: subprocess.Popen[str],
    *,
    timeout_seconds: float,
) -> None:
    deadline = time.monotonic() + timeout_seconds
    while time.monotonic() < deadline:
        if server.poll() is not None:
            stderr = server.stderr.read() if server.stderr is not None else ""
            raise RuntimeError(
                f"server exited before HTTP endpoint was ready:\n{stderr}"
            )
        try:
            with urllib.request.urlopen(url, timeout=1.0) as response:
                if 200 <= response.status < 500:
                    return
        except (OSError, TimeoutError, urllib.error.URLError):
            time.sleep(0.2)
    raise TimeoutError(f"HTTP endpoint did not become ready: {url}")


def fetch_rust_jobs_status(
    endpoint: str | None,
    *,
    require_status: bool,
) -> dict[str, Any] | None:
    endpoint = normalize_rest_endpoint(endpoint)
    if endpoint is None:
        return None
    url = f"{endpoint}/api/document-extract-jobs"
    try:
        with urllib.request.urlopen(url, timeout=1.0) as response:
            payload = json.loads(response.read().decode("utf-8"))
    except (
        OSError,
        TimeoutError,
        urllib.error.URLError,
        json.JSONDecodeError,
    ) as error:
        if require_status:
            raise RuntimeError(
                f"failed to sample Rust document extract jobs status: {error}"
            ) from error
        return None
    payload["sampledAtMs"] = int(time.time() * 1000)
    return payload


def run_command_with_status_sampling(
    command: list[str],
    *,
    env: dict[str, str],
    rest_endpoint: str | None,
    sample_interval_ms: int,
    require_status: bool,
) -> list[dict[str, Any]]:
    endpoint = normalize_rest_endpoint(rest_endpoint)
    if endpoint is None:
        subprocess.run(command, check=True, env=env)
        return []

    samples: list[dict[str, Any]] = []
    before = fetch_rust_jobs_status(endpoint, require_status=require_status)
    if before is not None:
        samples.append(before)

    process = subprocess.Popen(command, env=env)
    interval = max(sample_interval_ms, 25) / 1000
    while process.poll() is None:
        sample = fetch_rust_jobs_status(endpoint, require_status=require_status)
        if sample is not None:
            samples.append(sample)
        time.sleep(interval)

    after = fetch_rust_jobs_status(endpoint, require_status=require_status)
    if after is not None:
        samples.append(after)

    if process.returncode != 0:
        raise subprocess.CalledProcessError(process.returncode, command)
    return samples
