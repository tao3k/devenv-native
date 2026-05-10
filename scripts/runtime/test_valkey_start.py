from __future__ import annotations

import os
import subprocess
from pathlib import Path


def _write_executable(path: Path, content: str) -> None:
    path.write_text(content, encoding="utf-8")
    path.chmod(0o755)


def _install_fake_valkey_tools(tmp_path: Path) -> tuple[Path, Path, Path]:
    fake_bin = tmp_path / "bin"
    fake_bin.mkdir(parents=True, exist_ok=True)
    state_file = tmp_path / "valkey-state"
    server_args = tmp_path / "valkey-server-args.txt"

    _write_executable(
        fake_bin / "valkey-server",
        """#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$@" > "$VALKEY_SERVER_ARGS_FILE"
pidfile=""
logfile=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --pidfile)
      pidfile="$2"
      shift 2
      ;;
    --logfile)
      logfile="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done
if [ -n "$pidfile" ]; then
  mkdir -p "$(dirname "$pidfile")"
  printf '12345\n' > "$pidfile"
fi
if [ -n "$logfile" ]; then
  mkdir -p "$(dirname "$logfile")"
  : > "$logfile"
fi
touch "$VALKEY_STATE_FILE"
""",
    )
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

    return fake_bin, state_file, server_args


def test_valkey_start_uses_cache_for_rdb_and_runtime_for_pid_log(
    tmp_path: Path,
) -> None:
    project_root = Path(__file__).resolve().parents[2]
    script_path = Path(__file__).resolve().with_name("valkey-start.sh")
    fake_bin, state_file, server_args = _install_fake_valkey_tools(tmp_path)
    pidfile = tmp_path / ".run" / "valkey" / "valkey-6387.pid"

    env = os.environ.copy()
    env["PATH"] = f"{fake_bin}:{env['PATH']}"
    env["PRJ_RUNTIME_DIR"] = str(tmp_path / ".run")
    env["PRJ_CACHE_HOME"] = str(tmp_path / ".cache")
    env["VALKEY_HOST"] = "127.0.0.1"
    env["VALKEY_DB"] = "0"
    env["VALKEY_SERVER_ARGS_FILE"] = str(server_args)
    env["VALKEY_STATE_FILE"] = str(state_file)
    env["VALKEY_PIDFILE"] = str(pidfile)

    result = subprocess.run(
        ["bash", str(script_path), "6387"],
        cwd=project_root,
        env=env,
        capture_output=True,
        text=True,
        check=True,
    )

    args = server_args.read_text(encoding="utf-8").splitlines()
    assert "--dir" in args
    dir_index = args.index("--dir")
    assert args[dir_index + 1] == str(tmp_path / ".cache" / "valkey")
    pid_index = args.index("--pidfile")
    assert args[pid_index + 1] == str(pidfile)
    log_index = args.index("--logfile")
    assert args[log_index + 1] == str(tmp_path / ".run" / "valkey" / "valkey-6387.log")
    assert f"Valkey started. pidfile={pidfile}" in result.stdout


def test_valkey_start_rejects_reachable_listener_without_matching_pidfile(
    tmp_path: Path,
) -> None:
    project_root = Path(__file__).resolve().parents[2]
    script_path = Path(__file__).resolve().with_name("valkey-start.sh")
    fake_bin, state_file, server_args = _install_fake_valkey_tools(tmp_path)
    pidfile = tmp_path / ".run" / "valkey" / "valkey-6387.pid"
    pidfile.parent.mkdir(parents=True, exist_ok=True)
    pidfile.write_text("12345\n", encoding="utf-8")
    state_file.touch()

    env = os.environ.copy()
    env["PATH"] = f"{fake_bin}:{env['PATH']}"
    env["PRJ_RUNTIME_DIR"] = str(tmp_path / ".run")
    env["PRJ_CACHE_HOME"] = str(tmp_path / ".cache")
    env["VALKEY_HOST"] = "127.0.0.1"
    env["VALKEY_DB"] = "0"
    env["VALKEY_STATE_FILE"] = str(state_file)
    env["VALKEY_PIDFILE"] = str(pidfile)
    env["VALKEY_INFO_PROCESS_ID"] = "99999"

    result = subprocess.run(
        ["bash", str(script_path), "6387"],
        cwd=project_root,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 1
    assert "does not match the listener" in result.stderr
    assert not server_args.exists()


def test_valkey_start_accepts_running_listener_with_shared_pidfile(
    tmp_path: Path,
) -> None:
    project_root = Path(__file__).resolve().parents[2]
    script_path = Path(__file__).resolve().with_name("valkey-start.sh")
    fake_bin, state_file, server_args = _install_fake_valkey_tools(tmp_path)
    shared_pidfile = tmp_path / ".run" / "valkey" / "valkey.pid"
    shared_pidfile.parent.mkdir(parents=True, exist_ok=True)
    shared_pidfile.write_text("12345\n", encoding="utf-8")
    state_file.touch()

    env = os.environ.copy()
    env["PATH"] = f"{fake_bin}:{env['PATH']}"
    env["PRJ_RUNTIME_DIR"] = str(tmp_path / ".run")
    env["PRJ_CACHE_HOME"] = str(tmp_path / ".cache")
    env["VALKEY_HOST"] = "127.0.0.1"
    env["VALKEY_DB"] = "0"
    env["VALKEY_STATE_FILE"] = str(state_file)
    env["VALKEY_INFO_PROCESS_ID"] = "12345"

    result = subprocess.run(
        ["bash", str(script_path), "6387"],
        cwd=project_root,
        env=env,
        capture_output=True,
        text=True,
        check=True,
    )

    assert "Valkey already running on 6387 (pid 12345)." in result.stdout
    assert not server_args.exists()
