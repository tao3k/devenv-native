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
    --daemonize)
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
exit 0
""",
    )
    _write_executable(
        fake_bin / "valkey-cli",
        """#!/usr/bin/env bash
set -euo pipefail
resolve_pidfile() {
  local pidfile="${VALKEY_PIDFILE:-}"
  if [ -n "${VALKEY_SERVER_ARGS_FILE:-}" ] && [ -f "$VALKEY_SERVER_ARGS_FILE" ]; then
    pidfile="$(
      awk '
        prev == "--pidfile" { print; exit }
        { prev = $0 }
      ' "$VALKEY_SERVER_ARGS_FILE"
    )"
  fi
  printf '%s' "$pidfile"
}
if [ "${3:-}" = "info" ] && [ "${4:-}" = "server" ]; then
  pidfile="$(resolve_pidfile)"
  if [ -n "$pidfile" ] && [ -f "$pidfile" ]; then
    printf 'process_id:%s\n' "$(cat "$pidfile")"
    exit 0
  fi
  if [ -n "${VALKEY_INFO_PROCESS_ID:-}" ]; then
    printf 'process_id:%s\n' "$VALKEY_INFO_PROCESS_ID"
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


def test_valkey_launch_daemonized_uses_env_and_waits_for_health(tmp_path: Path) -> None:
    project_root = Path(__file__).resolve().parents[2]
    script_path = Path(__file__).resolve().with_name("valkey-launch.sh")
    fake_bin, state_file, server_args = _install_fake_valkey_tools(tmp_path)
    runtime_dir = tmp_path / ".run" / "valkey"
    data_dir = tmp_path / ".data" / "valkey"
    pidfile = runtime_dir / "valkey-6387.pid"
    logfile = runtime_dir / "valkey-6387.log"

    env = os.environ.copy()
    env["PATH"] = f"{fake_bin}:{env['PATH']}"
    env["VALKEY_PORT"] = "6387"
    env["VALKEY_BIND"] = "127.0.0.1"
    env["VALKEY_DB"] = "0"
    env["VALKEY_RUNTIME_DIR"] = str(runtime_dir)
    env["VALKEY_DATA_DIR"] = str(data_dir)
    env["VALKEY_PIDFILE"] = str(pidfile)
    env["VALKEY_LOGFILE"] = str(logfile)
    env["VALKEY_PROTECTED_MODE"] = "no"
    env["VALKEY_DAEMONIZE"] = "yes"
    env["VALKEY_TCP_BACKLOG"] = "128"
    env["VALKEY_STARTUP_INITIAL_DELAY_SECONDS"] = "0"
    env["VALKEY_STARTUP_PERIOD_SECONDS"] = "0"
    env["VALKEY_STARTUP_FAILURE_THRESHOLD"] = "3"
    env["VALKEY_STATE_FILE"] = str(state_file)
    env["VALKEY_INFO_PROCESS_ID"] = "12345"
    env["VALKEY_PIDFILE"] = str(pidfile)
    env["VALKEY_HOST"] = "127.0.0.1"
    env["VALKEY_SERVER_ARGS_FILE"] = str(server_args)

    result = subprocess.run(
        ["bash", str(script_path)],
        cwd=project_root,
        env=env,
        capture_output=True,
        text=True,
        check=True,
    )

    args = server_args.read_text(encoding="utf-8").splitlines()
    assert args[0] == "--port"
    assert args[1] == "6387"
    assert args[args.index("--bind") + 1] == "127.0.0.1"
    assert args[args.index("--dir") + 1] == str(data_dir)
    assert args[args.index("--pidfile") + 1] == str(pidfile)
    assert args[args.index("--logfile") + 1] == str(logfile)
    assert args[args.index("--protected-mode") + 1] == "no"
    assert "--daemonize" in args
    assert args[args.index("--daemonize") + 1] == "yes"
    assert "Valkey started." in result.stdout
    assert pidfile.exists()
    assert logfile.exists()


def test_valkey_launch_foreground_execs_valkey_server(tmp_path: Path) -> None:
    project_root = Path(__file__).resolve().parents[2]
    script_path = Path(__file__).resolve().with_name("valkey-launch.sh")
    fake_bin, state_file, server_args = _install_fake_valkey_tools(tmp_path)
    runtime_dir = tmp_path / ".run" / "valkey"
    data_dir = tmp_path / ".data" / "valkey"
    pidfile = runtime_dir / "valkey-6387.pid"

    env = os.environ.copy()
    env["PATH"] = f"{fake_bin}:{env['PATH']}"
    env["VALKEY_PORT"] = "6387"
    env["VALKEY_BIND"] = "127.0.0.1"
    env["VALKEY_DB"] = "0"
    env["VALKEY_RUNTIME_DIR"] = str(runtime_dir)
    env["VALKEY_DATA_DIR"] = str(data_dir)
    env["VALKEY_PIDFILE"] = str(pidfile)
    env["VALKEY_DAEMONIZE"] = "no"
    env["VALKEY_STATE_FILE"] = str(state_file)
    env["VALKEY_SERVER_ARGS_FILE"] = str(server_args)

    result = subprocess.run(
        ["bash", str(script_path)],
        cwd=project_root,
        env=env,
        capture_output=True,
        text=True,
        check=True,
    )

    args = server_args.read_text(encoding="utf-8").splitlines()
    assert args[0] == "--port"
    assert args[1] == "6387"
    assert "--daemonize" in args
    assert args[args.index("--daemonize") + 1] == "no"
    assert args[args.index("--dir") + 1] == str(data_dir)
    assert args[args.index("--pidfile") + 1] == str(pidfile)
    assert "--protected-mode" not in args
    assert result.stdout == ""
    assert pidfile.exists()


def test_valkey_launch_daemonized_resolves_relative_paths_against_project_root(
    tmp_path: Path,
) -> None:
    project_root = Path(__file__).resolve().parents[2]
    script_path = Path(__file__).resolve().with_name("valkey-launch.sh")
    fake_bin, state_file, server_args = _install_fake_valkey_tools(tmp_path)
    pidfile = ".run/valkey/valkey-6387.pid"
    logfile = ".run/valkey/valkey-6387.log"
    runtime_dir = ".run/valkey"
    data_dir = ".cache/valkey"

    env = os.environ.copy()
    env["PATH"] = f"{fake_bin}:{env['PATH']}"
    env["VALKEY_PORT"] = "6387"
    env["VALKEY_BIND"] = "127.0.0.1"
    env["VALKEY_DB"] = "0"
    env["VALKEY_RUNTIME_DIR"] = runtime_dir
    env["VALKEY_DATA_DIR"] = data_dir
    env["VALKEY_PIDFILE"] = pidfile
    env["VALKEY_LOGFILE"] = logfile
    env["VALKEY_PROTECTED_MODE"] = "no"
    env["VALKEY_DAEMONIZE"] = "yes"
    env["VALKEY_TCP_BACKLOG"] = "128"
    env["VALKEY_STARTUP_INITIAL_DELAY_SECONDS"] = "0"
    env["VALKEY_STARTUP_PERIOD_SECONDS"] = "0"
    env["VALKEY_STARTUP_FAILURE_THRESHOLD"] = "3"
    env["VALKEY_STATE_FILE"] = str(state_file)
    env["VALKEY_INFO_PROCESS_ID"] = "12345"
    env["VALKEY_SERVER_ARGS_FILE"] = str(server_args)

    result = subprocess.run(
        ["bash", str(script_path)],
        cwd=project_root,
        env=env,
        capture_output=True,
        text=True,
        check=True,
    )

    args = server_args.read_text(encoding="utf-8").splitlines()
    assert args[args.index("--dir") + 1] == str(project_root / data_dir)
    assert args[args.index("--pidfile") + 1] == str(project_root / pidfile)
    assert args[args.index("--logfile") + 1] == str(project_root / logfile)
    assert "--daemonize" in args
    assert args[args.index("--daemonize") + 1] == "yes"
    assert "Valkey started." in result.stdout
