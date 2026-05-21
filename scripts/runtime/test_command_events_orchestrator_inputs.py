from __future__ import annotations

import importlib
from argparse import Namespace

module = importlib.import_module("command_events_orchestrator_inputs")


def test_validate_basic_args_rejects_non_positive_max_wait() -> None:
    args = Namespace(
        max_wait=0, max_idle_secs=1, matrix_retries=0, matrix_backoff_secs=0.0
    )
    assert module.validate_basic_args(args) == 2


def test_resolve_admin_user_id_prefers_explicit_arg() -> None:
    args = Namespace(admin_user_id=42)
    value, error = module.resolve_admin_user_id(
        args,
        parse_optional_int_env_fn=lambda _name: None,
        group_profile_int_fn=lambda _name: None,
    )
    assert error is None
    assert value == 42


def test_resolve_admin_user_id_returns_error_on_env_parse_failure() -> None:
    args = Namespace(admin_user_id=None)

    def _raise(_name: str) -> int | None:
        raise ValueError("bad env")

    value, error = module.resolve_admin_user_id(
        args,
        parse_optional_int_env_fn=_raise,
        group_profile_int_fn=lambda _name: None,
    )
    assert value is None
    assert error == 2


def test_resolve_topic_thread_inputs_updates_args_with_pair() -> None:
    args = Namespace(group_thread_id=10, group_thread_id_b=11)
    group_thread_id, group_thread_id_b, pair, error = (
        module.resolve_topic_thread_inputs(
            args,
            parse_optional_int_env_fn=lambda _name: None,
            resolve_topic_thread_pair_fn=lambda **_kwargs: (10, 11),
        )
    )
    assert error is None
    assert pair == (10, 11)
    assert group_thread_id == 10
    assert group_thread_id_b == 11
    assert args.group_thread_id == 10
    assert args.group_thread_id_b == 11
