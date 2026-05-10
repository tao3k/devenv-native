from __future__ import annotations

import importlib
from argparse import Namespace
from dataclasses import dataclass

module = importlib.import_module("command_events_orchestrator_paths")


@dataclass(frozen=True)
class _Case:
    case_id: str
    suites: tuple[str, ...]
    chat_id: int | None


def _base_args(**overrides: object) -> Namespace:
    payload = {
        "max_wait": 25,
        "max_idle_secs": 25,
        "matrix_retries": 0,
        "matrix_backoff_secs": 0.0,
        "assert_admin_isolation": False,
        "assert_admin_topic_isolation": False,
        "admin_group_chat_id": (),
    }
    payload.update(overrides)
    return Namespace(**payload)


def test_run_default_mode_rejects_when_no_runnable_cases() -> None:
    args = _base_args()
    selected_cases = [
        _Case(case_id="session_admin_add", suites=("admin",), chat_id=None)
    ]
    exit_code = module.run_default_mode(
        args=args,
        selected_cases=selected_cases,
        selected_admin_cases=selected_cases,
        group_chat_id=None,
        topic_thread_pair=None,
        admin_user_id=42,
        username="tester",
        allow_chat_ids=("-1001",),
        secret_token="secret",
        blackbox_script="/tmp/blackbox.py",
        runtime_partition_mode="chat_user",
        attempts=[],
        run_case_with_retry_fn=lambda **_kwargs: 0,
        run_admin_topic_isolation_assertions_fn=lambda **_kwargs: 0,
    )
    assert exit_code == 2


def test_run_matrix_mode_requires_resolved_matrix_chats() -> None:
    args = _base_args(admin_group_chat_id=())
    selected_cases = [
        _Case(case_id="session_status_json", suites=("core",), chat_id=-1001)
    ]
    exit_code, chats = module.run_matrix_mode(
        args=args,
        selected_cases=selected_cases,
        group_chat_id=None,
        group_thread_id=None,
        topic_thread_pair=None,
        admin_user_id=42,
        username="tester",
        allow_chat_ids=("-1001",),
        secret_token="secret",
        blackbox_script="/tmp/blackbox.py",
        runtime_partition_mode="chat_user",
        attempts=[],
        build_cases_fn=lambda *_args, **_kwargs: (),
        run_case_with_retry_fn=lambda **_kwargs: 0,
        run_admin_isolation_assertions_fn=lambda **_kwargs: 0,
        run_admin_topic_isolation_assertions_fn=lambda **_kwargs: 0,
        resolve_admin_matrix_chat_ids_fn=lambda **_kwargs: (),
        matrix_transient_exit_codes=frozenset({2, 3}),
    )
    assert exit_code == 2
    assert chats == ()
