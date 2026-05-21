#!/usr/bin/env python3
"""Unit tests for command-events probe runtime helpers."""

from __future__ import annotations

import importlib
import sys
from dataclasses import dataclass
from pathlib import Path

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

runtime_module = importlib.import_module("command_events_probe_runtime")


@dataclass(frozen=True)
class _Case:
    case_id: str
    prompt: str
    event_name: str
    suites: tuple[str, ...]
    extra_args: tuple[str, ...] = ()
    user_id: int | None = None
    chat_id: int | None = None
    thread_id: int | None = None
    max_wait_secs: int | None = None
    max_idle_secs: int | None = None


@dataclass(frozen=True)
class _Attempt:
    mode: str
    case_id: str
    prompt: str
    event_name: str
    suites: tuple[str, ...]
    chat_id: int | None
    user_id: int | None
    thread_id: int | None
    attempt: int
    max_attempts: int
    returncode: int
    passed: bool
    duration_ms: int
    retry_scheduled: bool


def test_run_case_with_retry_retries_transient_and_records_attempts(
    tmp_path: Path,
) -> None:
    statuses = [3, 0]
    sleeps: list[float] = []
    attempts: list[_Attempt] = []
    case = _Case(
        case_id="session_admin_list_json",
        prompt="/session admin list json",
        event_name="telegram.command.session_admin_json.replied",
        suites=("admin",),
        chat_id=-5101776367,
    )

    def _run_case_fn(**_kwargs):
        return statuses.pop(0)

    status = runtime_module.run_case_with_retry(
        blackbox_script=tmp_path / "agent_channel_blackbox.py",
        case=case,
        username="",
        allow_chat_ids=("-5101776367",),
        max_wait=25,
        max_idle_secs=25,
        secret_token="",
        retries=2,
        backoff_secs=1.0,
        attempt_records=attempts,
        mode_label="admin_matrix",
        runtime_partition_mode=None,
        resolve_runtime_partition_mode_fn=lambda: "chat_user",
        apply_runtime_partition_defaults_fn=lambda c, _mode: c,
        run_case_fn=_run_case_fn,
        is_transient_matrix_failure_fn=runtime_module.is_transient_matrix_failure,
        transient_exit_codes=frozenset({2, 3, 4, 6, 7}),
        probe_attempt_record_cls=_Attempt,
        monotonic_fn=lambda: 1000.0,
        sleep_fn=lambda sec: sleeps.append(sec),
    )

    assert status == 0
    assert sleeps == [1.0]
    assert len(attempts) == 2
    assert attempts[0].retry_scheduled is True
    assert attempts[1].passed is True


def test_run_case_uses_per_case_timeout_overrides(tmp_path: Path) -> None:
    captured_cmd: list[str] = []
    case = _Case(
        case_id="agenda_view_markdown",
        prompt="/agenda",
        event_name="telegram.zhixing.sync.completed",
        suites=("core",),
        max_wait_secs=120,
        max_idle_secs=60,
    )

    def _subprocess_run(cmd: list[str], check: bool) -> object:
        del check
        captured_cmd.extend(cmd)
        return type("Completed", (), {"returncode": 0})()

    status = runtime_module.run_case(
        blackbox_script=tmp_path / "agent_channel_blackbox.py",
        case=case,
        username="",
        allow_chat_ids=(),
        max_wait=25,
        max_idle_secs=25,
        secret_token="",
        runtime_partition_mode=None,
        forbidden_log_pattern="tools/call: Tool runtime error",
        python_executable="python3",
        subprocess_run_fn=_subprocess_run,
    )

    assert status == 0
    assert "--max-wait" in captured_cmd
    assert "--max-idle-secs" in captured_cmd
    assert captured_cmd[captured_cmd.index("--max-wait") + 1] == "120"
    assert captured_cmd[captured_cmd.index("--max-idle-secs") + 1] == "60"
