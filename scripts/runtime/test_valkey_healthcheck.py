from __future__ import annotations

import os
import subprocess
from pathlib import Path


def _write_executable(path: Path, content: str) -> None:
    path.write_text(content, encoding="utf-8")
    path.chmod(0o755)


def _install_fake_valkey_cli(tmp_path: Path) -> Path:
    fake_bin = tmp_path / "bin"
    fake_bin.mkdir(parents=True, exist_ok=True)

    _write_executable(
        fake_bin / "valkey-cli",
        """#!/usr/bin/env bash
set -euo pipefail
if [ "${3:-}" = "info" ] && [ "${4:-}" = "server" ]; then
  if [ -n "${VALKEY_INFO_PROCESS_ID:-}" ]; then
    printf 'process_id:%s\n' "$VALKEY_INFO_PROCESS_ID"
    exit 0
  fi
  if [ -n "${VALKEY_PIDFILE:-}" ] && [ -f "$VALKEY_PIDFILE" ]; then
    printf 'process_id:%s\n' "$(cat "$VALKEY_PIDFILE")"
    exit 0
  fi
fi
if [ "${3:-}" = "ping" ] && [ -f "$VALKEY_STATE_FILE" ]; then
  printf 'PONG\n'
  exit 0
fi
exit 1
""",
    )

    return fake_bin


def test_valkey_healthcheck_reports_owned_listener(tmp_path: Path) -> None:
    project_root = Path(__file__).resolve().parents[2]
    script_path = Path(__file__).resolve().with_name("valkey-healthcheck.sh")
    fake_bin = _install_fake_valkey_cli(tmp_path)
    pidfile = tmp_path / "run" / "valkey-6387.pid"
    pidfile.parent.mkdir(parents=True, exist_ok=True)
    pidfile.write_text("12345\n", encoding="utf-8")
    state_file = tmp_path / "valkey-state"
    state_file.touch()

    env = os.environ.copy()
    env["PATH"] = f"{fake_bin}:{env['PATH']}"
    env["VALKEY_PORT"] = "6387"
    env["VALKEY_HOST"] = "127.0.0.1"
    env["VALKEY_DB"] = "0"
    env["VALKEY_RUNTIME_DIR"] = str(tmp_path / "run")
    env["VALKEY_PIDFILE"] = str(pidfile)
    env["VALKEY_STATE_FILE"] = str(state_file)

    result = subprocess.run(
        ["bash", str(script_path)],
        cwd=project_root,
        env=env,
        capture_output=True,
        text=True,
        check=True,
    )

    assert result.stdout.strip() == "PONG"


def test_valkey_healthcheck_rejects_reachable_listener_without_matching_pidfile(
    tmp_path: Path,
) -> None:
    project_root = Path(__file__).resolve().parents[2]
    script_path = Path(__file__).resolve().with_name("valkey-healthcheck.sh")
    fake_bin = _install_fake_valkey_cli(tmp_path)
    pidfile = tmp_path / "run" / "valkey-6387.pid"
    pidfile.parent.mkdir(parents=True, exist_ok=True)
    pidfile.write_text("12345\n", encoding="utf-8")
    state_file = tmp_path / "valkey-state"
    state_file.touch()

    env = os.environ.copy()
    env["PATH"] = f"{fake_bin}:{env['PATH']}"
    env["VALKEY_PORT"] = "6387"
    env["VALKEY_HOST"] = "127.0.0.1"
    env["VALKEY_DB"] = "0"
    env["VALKEY_RUNTIME_DIR"] = str(tmp_path / "run")
    env["VALKEY_PIDFILE"] = str(pidfile)
    env["VALKEY_STATE_FILE"] = str(state_file)
    env["VALKEY_INFO_PROCESS_ID"] = "99999"

    result = subprocess.run(
        ["bash", str(script_path)],
        cwd=project_root,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 1
    assert "does not match the listener" in result.stderr


def test_valkey_healthcheck_accepts_shared_pidfile(tmp_path: Path) -> None:
    project_root = Path(__file__).resolve().parents[2]
    script_path = Path(__file__).resolve().with_name("valkey-healthcheck.sh")
    fake_bin = _install_fake_valkey_cli(tmp_path)
    shared_pidfile = tmp_path / "run" / "valkey.pid"
    shared_pidfile.parent.mkdir(parents=True, exist_ok=True)
    shared_pidfile.write_text("12345\n", encoding="utf-8")
    state_file = tmp_path / "valkey-state"
    state_file.touch()

    env = os.environ.copy()
    env["PATH"] = f"{fake_bin}:{env['PATH']}"
    env["VALKEY_PORT"] = "6387"
    env["VALKEY_HOST"] = "127.0.0.1"
    env["VALKEY_DB"] = "0"
    env["VALKEY_RUNTIME_DIR"] = str(tmp_path / "run")
    env["VALKEY_STATE_FILE"] = str(state_file)
    env["VALKEY_INFO_PROCESS_ID"] = "12345"

    result = subprocess.run(
        ["bash", str(script_path)],
        cwd=project_root,
        env=env,
        capture_output=True,
        text=True,
        check=True,
    )

    assert result.stdout.strip() == "PONG"
