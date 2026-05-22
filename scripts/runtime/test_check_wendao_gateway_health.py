from __future__ import annotations

import importlib.util
import os
import subprocess
import sys
from email.message import Message
from pathlib import Path


class _FakeResponse:
    def __init__(
        self,
        status: int,
        headers: dict[str, str] | None = None,
        body: bytes | None = None,
    ) -> None:
        self.status = status
        self.headers = Message()
        self._body = body or b'{"service":"wendao-gateway","ready":true}'
        for key, value in (headers or {}).items():
            self.headers[key] = value

    def read(self) -> bytes:
        return self._body

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb) -> bool:
        return False


def _load_module():
    script_path = Path(__file__).resolve().with_name("check_wendao_gateway_health.py")
    module_name = "test_check_wendao_gateway_health_module"
    spec = importlib.util.spec_from_file_location(module_name, script_path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module


def _write_flight_ready_log(logfile: Path) -> None:
    logfile.write_text(
        (
            "INFO wendao::execute::gateway::command: "
            "  - POST /arrow.flight.protocol.FlightService/{*grpc_method}  - "
            "Arrow Flight business plane\n"
        ),
        encoding="utf-8",
    )


def test_is_gateway_healthy_accepts_matching_health_payload(tmp_path) -> None:
    module = _load_module()
    pidfile = tmp_path / "wendao.pid"
    logfile = tmp_path / "wendao-gateway.stderr.log"
    pidfile.write_text("4321\n", encoding="utf-8")
    _write_flight_ready_log(logfile)

    def _fake_open(request_or_url, timeout: float):
        assert request_or_url == "http://127.0.0.1:9517/api/health"
        assert timeout == 2.0
        return _FakeResponse(200, {"x-wendao-process-id": "4321"})

    healthy, message = module.is_gateway_healthy(
        host="127.0.0.1",
        port=9517,
        pidfile=pidfile,
        logfile=logfile,
        timeout_secs=2.0,
        opener=_fake_open,
        pid_exists=lambda pid: pid == 4321,
        process_command_for_pid=(
            lambda pid: "target/debug/wendao --conf /tmp/wendao.toml gateway start"
        ),
    )

    assert healthy is True
    assert message == "healthy"


def test_is_gateway_healthy_accepts_health_payload_flight_plane_without_log(
    tmp_path,
) -> None:
    module = _load_module()
    pidfile = tmp_path / "wendao.pid"
    logfile = tmp_path / "wendao-gateway.stderr.log"
    pidfile.write_text("4321\n", encoding="utf-8")
    logfile.write_text("", encoding="utf-8")

    def _fake_open(request_or_url, timeout: float):
        assert request_or_url == "http://127.0.0.1:9517/api/health"
        assert timeout == 2.0
        return _FakeResponse(
            200,
            {"x-wendao-process-id": "4321"},
            b'{"service":"wendao-gateway","ready":true,'
            b'"planes":{"flight":"mounted","http":"ready"}}',
        )

    healthy, message = module.is_gateway_healthy(
        host="127.0.0.1",
        port=9517,
        pidfile=pidfile,
        logfile=logfile,
        timeout_secs=2.0,
        opener=_fake_open,
        pid_exists=lambda pid: pid == 4321,
        process_command_for_pid=(
            lambda pid: "target/debug/wendao --conf /tmp/wendao.toml gateway start"
        ),
    )

    assert healthy is True
    assert message == "healthy"


def test_is_gateway_healthy_accepts_reported_process_id_when_pidfile_is_stale(
    tmp_path,
) -> None:
    module = _load_module()
    pidfile = tmp_path / "wendao.pid"
    logfile = tmp_path / "wendao-gateway.stderr.log"
    pidfile.write_text("4321\n", encoding="utf-8")
    _write_flight_ready_log(logfile)

    def _fake_open(request_or_url, timeout: float):
        assert request_or_url == "http://127.0.0.1:9517/api/health"
        assert timeout == 2.0
        return _FakeResponse(200, {"x-wendao-process-id": "9999"})

    healthy, message = module.is_gateway_healthy(
        host="127.0.0.1",
        port=9517,
        pidfile=pidfile,
        logfile=logfile,
        timeout_secs=2.0,
        opener=_fake_open,
        pid_exists=lambda pid: True,
        process_command_for_pid=(
            lambda pid: "target/debug/wendao --conf /tmp/wendao.toml gateway start"
        ),
    )

    assert healthy is True
    assert message == "healthy"


def test_is_gateway_healthy_rejects_not_ready_payload(tmp_path) -> None:
    module = _load_module()
    pidfile = tmp_path / "wendao.pid"
    logfile = tmp_path / "wendao-gateway.stderr.log"
    pidfile.write_text("4321\n", encoding="utf-8")
    _write_flight_ready_log(logfile)

    def _fake_open(request_or_url, timeout: float):
        assert request_or_url == "http://127.0.0.1:9517/api/health"
        assert timeout == 2.0
        return _FakeResponse(
            200,
            {"x-wendao-process-id": "4321"},
            b'{"service":"wendao-gateway","ready":false}',
        )

    healthy, message = module.is_gateway_healthy(
        host="127.0.0.1",
        port=9517,
        pidfile=pidfile,
        logfile=logfile,
        timeout_secs=2.0,
        opener=_fake_open,
        pid_exists=lambda pid: True,
        process_command_for_pid=(
            lambda pid: "target/debug/wendao --conf /tmp/wendao.toml gateway start"
        ),
    )

    assert healthy is False
    assert "ready=true" in message


def test_is_gateway_healthy_rejects_invalid_payload(tmp_path) -> None:
    module = _load_module()
    pidfile = tmp_path / "wendao.pid"
    logfile = tmp_path / "wendao-gateway.stderr.log"
    pidfile.write_text("4321\n", encoding="utf-8")
    _write_flight_ready_log(logfile)

    def _fake_open(request_or_url, timeout: float):
        assert request_or_url == "http://127.0.0.1:9517/api/health"
        assert timeout == 2.0
        return _FakeResponse(
            200,
            {"x-wendao-process-id": "4321"},
            b"not-json",
        )

    healthy, message = module.is_gateway_healthy(
        host="127.0.0.1",
        port=9517,
        pidfile=pidfile,
        logfile=logfile,
        timeout_secs=2.0,
        opener=_fake_open,
        pid_exists=lambda pid: True,
        process_command_for_pid=(
            lambda pid: "target/debug/wendao --conf /tmp/wendao.toml gateway start"
        ),
    )

    assert healthy is False
    assert "invalid json" in message


def test_is_gateway_healthy_accepts_live_health_when_pidfile_missing(tmp_path) -> None:
    module = _load_module()
    pidfile = tmp_path / "missing.pid"
    logfile = tmp_path / "wendao-gateway.stderr.log"
    _write_flight_ready_log(logfile)

    def _fake_open(request_or_url, timeout: float):
        assert request_or_url == "http://127.0.0.1:9517/api/health"
        assert timeout == 2.0
        return _FakeResponse(200, {"x-wendao-process-id": "4805"})

    healthy, message = module.is_gateway_healthy(
        host="127.0.0.1",
        port=9517,
        pidfile=pidfile,
        logfile=logfile,
        timeout_secs=2.0,
        opener=_fake_open,
        pid_exists=lambda pid: pid == 4805,
        process_command_for_pid=(
            lambda pid: "target/debug/wendao --conf /tmp/wendao.toml gateway start"
        ),
    )

    assert healthy is True
    assert message == "healthy"


def test_is_gateway_healthy_rejects_unexpected_process_when_pidfile_missing(
    tmp_path,
) -> None:
    module = _load_module()
    pidfile = tmp_path / "missing.pid"
    logfile = tmp_path / "wendao-gateway.stderr.log"
    _write_flight_ready_log(logfile)

    def _fake_open(request_or_url, timeout: float):
        assert request_or_url == "http://127.0.0.1:9517/api/health"
        assert timeout == 2.0
        return _FakeResponse(200, {"x-wendao-process-id": "4805"})

    healthy, message = module.is_gateway_healthy(
        host="127.0.0.1",
        port=9517,
        pidfile=pidfile,
        logfile=logfile,
        timeout_secs=2.0,
        opener=_fake_open,
        pid_exists=lambda pid: pid == 4805,
        process_command_for_pid=lambda pid: "python -m http.server 9517",
    )

    assert healthy is False
    assert "unexpected" in message


def test_is_gateway_healthy_rejects_missing_flight_ready_marker(tmp_path) -> None:
    module = _load_module()
    pidfile = tmp_path / "wendao.pid"
    logfile = tmp_path / "wendao-gateway.stderr.log"
    pidfile.write_text("4321\n", encoding="utf-8")
    logfile.write_text("no flight marker yet\n", encoding="utf-8")

    def _fake_open(request_or_url, timeout: float):
        assert request_or_url == "http://127.0.0.1:9517/api/health"
        assert timeout == 2.0
        return _FakeResponse(200, {"x-wendao-process-id": "4321"})

    healthy, message = module.is_gateway_healthy(
        host="127.0.0.1",
        port=9517,
        pidfile=pidfile,
        logfile=logfile,
        timeout_secs=2.0,
        opener=_fake_open,
        pid_exists=lambda pid: pid == 4321,
        process_command_for_pid=(
            lambda pid: "target/debug/wendao --conf /tmp/wendao.toml gateway start"
        ),
    )

    assert healthy is False
    assert "Flight business plane" in message
    assert "health payload" in message


def test_gateway_healthcheck_shell_prefers_pyo3_python(tmp_path) -> None:
    project_root = Path(__file__).resolve().parents[2]
    script_path = project_root / "scripts" / "runtime" / "wendao-gateway-healthcheck.sh"
    fake_bin = tmp_path / "bin"
    fake_bin.mkdir()
    bad_python3 = fake_bin / "python3"
    bad_python3.write_text("#!/usr/bin/env bash\nexit 23\n", encoding="utf-8")
    bad_python3.chmod(0o755)

    fake_python = fake_bin / "python-good"
    fake_log = tmp_path / "python.log"
    fake_python.write_text(
        "#!/usr/bin/env bash\n"
        "set -euo pipefail\n"
        'printf \'%s\\n\' "$@" >> "$FAKE_PYTHON_LOG"\n'
        'case "$1" in\n'
        "  *resolve_wendao_gateway_port.py)\n"
        "    printf '9517'\n"
        "    ;;\n"
        "  *check_wendao_gateway_health.py)\n"
        "    ;;\n"
        "  *)\n"
        "    exit 99\n"
        "    ;;\n"
        "esac\n",
        encoding="utf-8",
    )
    fake_python.chmod(0o755)

    env = dict(os.environ)
    env["PATH"] = f"{fake_bin}:/usr/bin:/bin"
    env["PYO3_PYTHON"] = str(fake_python)
    env["FAKE_PYTHON_LOG"] = str(fake_log)
    env["WENDAO_GATEWAY_CONFIG"] = "wendao.toml"
    env["WENDAO_GATEWAY_PIDFILE"] = str(tmp_path / "wendao.pid")
    env["WENDAO_GATEWAY_STDERR_LOG"] = str(tmp_path / "wendao-gateway.stderr.log")

    result = subprocess.run(
        ["bash", str(script_path)],
        cwd=project_root,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 0, result.stderr
    calls = fake_log.read_text(encoding="utf-8").splitlines()
    assert any("resolve_wendao_gateway_port.py" in line for line in calls)
    assert any("check_wendao_gateway_health.py" in line for line in calls)
    assert any("--logfile" in line for line in calls)
