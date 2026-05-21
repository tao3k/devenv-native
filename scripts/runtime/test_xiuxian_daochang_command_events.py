#!/usr/bin/env python3
"""Run strict Telegram command black-box probes against local webhook runtime."""

from __future__ import annotations

import importlib
import sys
import time
from functools import partial
from pathlib import Path
from typing import Any

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

_CONSTANTS = importlib.import_module("command_events_constants")
FORBIDDEN_LOG_PATTERN = _CONSTANTS.FORBIDDEN_LOG_PATTERN
MATRIX_TRANSIENT_EXIT_CODES = _CONSTANTS.MATRIX_TRANSIENT_EXIT_CODES
SUITES = _CONSTANTS.SUITES
TARGET_SESSION_SCOPE_PLACEHOLDER = _CONSTANTS.TARGET_SESSION_SCOPE_PLACEHOLDER
load_module_bindings = importlib.import_module(
    "command_events_module_bindings"
).load_module_bindings

_MODULES = load_module_bindings(__file__)
_REPORT = _MODULES.report_module
_RUNTIME_CONTEXT = _MODULES.runtime_context_module
_ENTRY_BINDINGS = _MODULES.entry_bindings_module

build_report = _REPORT.build_report
write_outputs = _REPORT.write_outputs
ProbeCase = _MODULES.models_module.ProbeCase
ProbeAttemptRecord = _MODULES.models_module.ProbeAttemptRecord


def parse_args() -> Any:
    return _MODULES.config_module.parse_args(suites=SUITES)


parse_optional_int_env = _RUNTIME_CONTEXT.parse_optional_int_env
dedup_ints = _RUNTIME_CONTEXT.dedup_ints
runtime_log_file = _RUNTIME_CONTEXT.runtime_log_file

read_log_tail_lines = partial(
    _RUNTIME_CONTEXT.read_log_tail_lines,
    read_log_tail_lines_fn=_MODULES.shared_read_log_tail_lines,
    tail_bytes=_RUNTIME_CONTEXT.RUNTIME_LOG_TAIL_BYTES,
)

resolve_runtime_partition_mode = partial(
    _RUNTIME_CONTEXT.resolve_runtime_partition_mode,
    normalize_telegram_session_partition_mode_fn=_MODULES.normalize_telegram_session_partition_mode,
    session_partition_mode_from_runtime_log_fn=_MODULES.session_partition_mode_from_runtime_log,
    telegram_session_partition_mode_fn=_MODULES.telegram_session_partition_mode,
)

telegram_webhook_secret_token = _MODULES.telegram_webhook_secret_token

infer_group_thread_id_from_runtime_log = partial(
    _RUNTIME_CONTEXT.infer_group_thread_id_from_runtime_log,
    read_log_tail_lines_fn=_MODULES.shared_read_log_tail_lines,
)

apply_runtime_partition_defaults = _RUNTIME_CONTEXT.apply_runtime_partition_defaults
first_group_chat_id = _RUNTIME_CONTEXT.first_group_chat_id

profile_chat_ids_as_strings = partial(
    _RUNTIME_CONTEXT.profile_chat_ids_as_strings,
    group_profile_chat_ids_fn=_MODULES.group_profile_chat_ids,
)

resolve_allow_chat_ids = partial(
    _RUNTIME_CONTEXT.resolve_allow_chat_ids,
    group_profile_chat_ids_fn=_MODULES.group_profile_chat_ids,
)

resolve_group_chat_id = partial(
    _RUNTIME_CONTEXT.resolve_group_chat_id,
    group_profile_int_fn=_MODULES.group_profile_int,
)

resolve_admin_matrix_chat_ids = partial(
    _RUNTIME_CONTEXT.resolve_admin_matrix_chat_ids,
    group_profile_chat_ids_fn=_MODULES.group_profile_chat_ids,
)

resolve_topic_thread_pair = _RUNTIME_CONTEXT.resolve_topic_thread_pair

build_cases = partial(
    _MODULES.case_catalog_module.build_cases,
    make_probe_case=ProbeCase,
    target_session_scope_placeholder=TARGET_SESSION_SCOPE_PLACEHOLDER,
)

run_case = partial(
    _MODULES.runtime_bindings_module.run_case,
    probe_runtime_module=_MODULES.probe_runtime_module,
    forbidden_log_pattern=FORBIDDEN_LOG_PATTERN,
)

is_transient_matrix_failure = partial(
    _MODULES.probe_runtime_module.is_transient_matrix_failure,
    transient_exit_codes=MATRIX_TRANSIENT_EXIT_CODES,
)


def _run_case_dynamic(**kwargs: Any) -> int:
    return run_case(**kwargs)


def _monotonic_dynamic() -> float:
    return time.monotonic()


def _sleep_dynamic(seconds: float) -> None:
    time.sleep(seconds)


run_case_with_retry = partial(
    _ENTRY_BINDINGS.run_case_with_retry,
    runtime_bindings_module=_MODULES.runtime_bindings_module,
    probe_runtime_module=_MODULES.probe_runtime_module,
    resolve_runtime_partition_mode_fn=resolve_runtime_partition_mode,
    apply_runtime_partition_defaults_fn=apply_runtime_partition_defaults,
    run_case_fn=_run_case_dynamic,
    transient_exit_codes=MATRIX_TRANSIENT_EXIT_CODES,
    probe_attempt_record_cls=ProbeAttemptRecord,
    monotonic_fn=_monotonic_dynamic,
    sleep_fn=_sleep_dynamic,
)

select_cases = _MODULES.case_catalog_module.select_cases

build_admin_list_isolation_case = partial(
    _MODULES.admin_isolation_module.build_admin_list_isolation_case,
    make_probe_case=ProbeCase,
)

build_admin_list_topic_isolation_case = partial(
    _MODULES.admin_isolation_module.build_admin_list_topic_isolation_case,
    make_probe_case=ProbeCase,
)


def _run_case_with_retry_dynamic(**kwargs: Any) -> int:
    return run_case_with_retry(**kwargs)


run_admin_isolation_assertions = partial(
    _ENTRY_BINDINGS.run_admin_isolation_assertions,
    runtime_bindings_module=_MODULES.runtime_bindings_module,
    admin_isolation_module=_MODULES.admin_isolation_module,
    build_cases_fn=build_cases,
    run_case_with_retry_fn=_run_case_with_retry_dynamic,
    probe_case_cls=ProbeCase,
)

run_admin_topic_isolation_assertions = partial(
    _ENTRY_BINDINGS.run_admin_topic_isolation_assertions,
    runtime_bindings_module=_MODULES.runtime_bindings_module,
    admin_isolation_module=_MODULES.admin_isolation_module,
    build_cases_fn=build_cases,
    run_case_with_retry_fn=_run_case_with_retry_dynamic,
    probe_case_cls=ProbeCase,
)


def main() -> int:
    return _MODULES.orchestrator_module.run_command_events(
        parse_args(),
        script_file=__file__,
        parse_optional_int_env_fn=parse_optional_int_env,
        group_profile_int_fn=_MODULES.group_profile_int,
        resolve_allow_chat_ids_fn=resolve_allow_chat_ids,
        resolve_group_chat_id_fn=resolve_group_chat_id,
        resolve_topic_thread_pair_fn=resolve_topic_thread_pair,
        resolve_runtime_partition_mode_fn=resolve_runtime_partition_mode,
        infer_group_thread_id_from_runtime_log_fn=infer_group_thread_id_from_runtime_log,
        build_cases_fn=build_cases,
        select_cases_fn=select_cases,
        resolve_admin_matrix_chat_ids_fn=resolve_admin_matrix_chat_ids,
        run_case_with_retry_fn=run_case_with_retry,
        run_admin_isolation_assertions_fn=run_admin_isolation_assertions,
        run_admin_topic_isolation_assertions_fn=run_admin_topic_isolation_assertions,
        build_report_fn=build_report,
        write_outputs_fn=write_outputs,
        telegram_webhook_secret_token_fn=telegram_webhook_secret_token,
        matrix_transient_exit_codes=MATRIX_TRANSIENT_EXIT_CODES,
    )


if __name__ == "__main__":
    raise SystemExit(main())
