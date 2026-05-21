from __future__ import annotations

import importlib.util
from dataclasses import dataclass
from pathlib import Path


def _load_module():
    module_name = "command_events_admin_isolation_test_module"
    script_path = (
        Path(__file__).resolve().with_name("command_events_admin_isolation.py")
    )
    spec = importlib.util.spec_from_file_location(module_name, script_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"failed to load module from {script_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


@dataclass(frozen=True)
class _Case:
    case_id: str
    prompt: str
    event_name: str
    suites: tuple[str, ...]
    user_id: int | None = None
    chat_id: int | None = None
    thread_id: int | None = None
    extra_args: tuple[str, ...] = ()


def _make_probe_case(**kwargs):
    return _Case(**kwargs)


def _build_cases(
    _admin_user_id: int | None, chat_id: int | None, _thread_id: int | None
):
    return (
        _Case(
            case_id="session_admin_add",
            prompt="/session admin add 1001",
            event_name="telegram.command.session_admin.replied",
            suites=("admin",),
            chat_id=chat_id,
        ),
        _Case(
            case_id="session_admin_clear",
            prompt="/session admin clear",
            event_name="telegram.command.session_admin.replied",
            suites=("admin",),
            chat_id=chat_id,
        ),
    )


def test_build_admin_list_case_helpers() -> None:
    module = _load_module()
    case_a = module.build_admin_list_isolation_case(
        make_probe_case=_make_probe_case,
        chat_id=-1001,
        admin_user_id=42,
        thread_id=7,
        expected_override_count=1,
    )
    assert case_a.case_id == "session_admin_list_json_isolation_-1001_1"
    assert "json_override_admin_count=1" in case_a.extra_args

    case_b = module.build_admin_list_topic_isolation_case(
        make_probe_case=_make_probe_case,
        chat_id=-1001,
        admin_user_id=42,
        thread_id=9,
        expected_override_count=0,
    )
    assert case_b.case_id == "session_admin_list_json_topic_isolation_-1001_9_0"
    assert "json_override_admin_count=0" in case_b.extra_args


def test_run_admin_isolation_assertions_executes_expected_calls() -> None:
    module = _load_module()
    calls: list[dict[str, object]] = []

    def _run_case_with_retry_fn(**kwargs):
        calls.append(kwargs)
        return 0

    status = module.run_admin_isolation_assertions(
        blackbox_script=Path("/tmp/blackbox.py"),
        matrix_chat_ids=(-1001, -1002),
        admin_user_id=42,
        group_thread_id=None,
        username="tester",
        allow_chat_ids=("-1001", "-1002"),
        max_wait=25,
        max_idle_secs=25,
        secret_token="secret",
        retries=1,
        backoff_secs=1.0,
        attempt_records=[],
        runtime_partition_mode="chat_user",
        build_cases_fn=_build_cases,
        run_case_with_retry_fn=_run_case_with_retry_fn,
        make_probe_case=_make_probe_case,
    )

    assert status == 0
    assert len(calls) == 14
    assert calls[0]["mode_label"] == "admin_matrix_isolation_baseline"


def test_run_admin_topic_isolation_assertions_executes_expected_calls() -> None:
    module = _load_module()
    calls: list[dict[str, object]] = []

    def _run_case_with_retry_fn(**kwargs):
        calls.append(kwargs)
        return 0

    status = module.run_admin_topic_isolation_assertions(
        blackbox_script=Path("/tmp/blackbox.py"),
        group_chat_id=-1001,
        admin_user_id=42,
        thread_a=10,
        thread_b=11,
        username="tester",
        allow_chat_ids=("-1001",),
        max_wait=25,
        max_idle_secs=25,
        secret_token="secret",
        retries=1,
        backoff_secs=1.0,
        attempt_records=[],
        runtime_partition_mode="chat_thread_user",
        build_cases_fn=_build_cases,
        run_case_with_retry_fn=_run_case_with_retry_fn,
        make_probe_case=_make_probe_case,
    )

    assert status == 0
    assert len(calls) == 14
    assert calls[0]["mode_label"] == "admin_topic_isolation_baseline"
