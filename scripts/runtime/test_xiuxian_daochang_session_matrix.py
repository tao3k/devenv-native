#!/usr/bin/env python3
"""
Session matrix black-box probe for Telegram webhook runtime.

Step 3 validation target:
1) Baseline concurrent dual-session handling.
2) Cross-session reset/resume sequence.
3) Structured JSON/Markdown report for acceptance and debugging.
"""

from __future__ import annotations

import importlib
import os
import sys
from functools import partial
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import argparse

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

load_sibling_module = importlib.import_module("module_loader").load_sibling_module


def _load_sibling(module_name: str, file_name: str, error_context: str):
    return load_sibling_module(
        module_name=module_name,
        file_name=file_name,
        caller_file=__file__,
        error_context=error_context,
    )


_resolver_module = _load_sibling(
    "config_resolver", "config_resolver.py", "resolver module"
)
_report_module = _load_sibling(
    "session_matrix_report",
    "session_matrix_report.py",
    "session matrix report helpers",
)
_session_keys_module = _load_sibling(
    "telegram_session_keys",
    "telegram_session_keys.py",
    "telegram session key helpers",
)
_execution_module = _load_sibling(
    "session_matrix_execution",
    "session_matrix_execution.py",
    "session matrix execution helpers",
)
_config_module = _load_sibling(
    "session_matrix_config",
    "session_matrix_config.py",
    "session matrix config helpers",
)
_models_module = _load_sibling(
    "session_matrix_models",
    "session_matrix_models.py",
    "session matrix datamodels",
)
_steps_module = _load_sibling(
    "session_matrix_steps",
    "session_matrix_steps.py",
    "session matrix step templates",
)

default_telegram_webhook_url = _resolver_module.default_telegram_webhook_url
group_profile_int = _resolver_module.group_profile_int
normalize_telegram_session_partition_mode = (
    _resolver_module.normalize_telegram_session_partition_mode
)
session_ids_from_runtime_log = _resolver_module.session_ids_from_runtime_log
session_partition_mode_from_runtime_log = (
    _resolver_module.session_partition_mode_from_runtime_log
)
telegram_session_partition_mode = _resolver_module.telegram_session_partition_mode
username_from_runtime_log = _resolver_module.username_from_runtime_log
username_from_settings = _resolver_module.username_from_settings

build_report = _report_module.build_report
render_markdown = _report_module.render_markdown
write_outputs = _report_module.write_outputs

ProbeConfig = _models_module.ProbeConfig
MatrixStep = _models_module.MatrixStep
StepResult = _models_module.StepResult

RESTART_NOISE_MARKERS = _execution_module.RESTART_NOISE_MARKERS

parse_args = partial(
    _config_module.parse_args,
    webhook_url_default=os.environ.get("OMNI_WEBHOOK_URL")
    or default_telegram_webhook_url(),
)


def expected_session_keys(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    session_partition: str | None = None,
) -> tuple[str, ...]:
    return _session_keys_module.expected_session_keys(
        chat_id,
        user_id,
        thread_id,
        session_partition,
        normalize_partition_fn=normalize_telegram_session_partition_mode,
    )


def expected_session_key(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    session_partition: str | None = None,
) -> str:
    return _session_keys_module.expected_session_key(
        chat_id,
        user_id,
        thread_id,
        session_partition,
        normalize_partition_fn=normalize_telegram_session_partition_mode,
    )


def resolve_runtime_partition_mode(log_file: Path) -> str | None:
    return _config_module.resolve_runtime_partition_mode(
        log_file,
        normalize_telegram_session_partition_mode_fn=normalize_telegram_session_partition_mode,
        session_partition_mode_from_runtime_log_fn=session_partition_mode_from_runtime_log,
        telegram_session_partition_mode_fn=telegram_session_partition_mode,
    )


def session_context_result_fields(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    session_partition: str | None,
) -> tuple[str, ...]:
    return _config_module.session_context_result_fields(
        chat_id,
        user_id,
        thread_id,
        session_partition,
        expected_session_key_fn=expected_session_key,
    )


def session_memory_result_fields(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    session_partition: str | None,
) -> tuple[str, ...]:
    return _config_module.session_memory_result_fields(
        chat_id,
        user_id,
        thread_id,
        session_partition,
        expected_session_key_fn=expected_session_key,
    )


def build_config(args: argparse.Namespace) -> ProbeConfig:
    return _config_module.build_config(
        args,
        config_cls=ProbeConfig,
        resolve_runtime_partition_mode_fn=resolve_runtime_partition_mode,
        group_profile_int_fn=group_profile_int,
        session_ids_from_runtime_log_fn=session_ids_from_runtime_log,
        username_from_settings_fn=username_from_settings,
        username_from_runtime_log_fn=username_from_runtime_log,
        expected_session_key_fn=expected_session_key,
    )


build_matrix_steps = partial(
    _steps_module.build_matrix_steps,
    matrix_step_cls=MatrixStep,
    session_context_result_fields_fn=session_context_result_fields,
    session_memory_result_fields_fn=session_memory_result_fields,
)
_tail_text = _execution_module.tail_text
_run_command = _execution_module.run_command
should_retry_on_restart_noise = _execution_module.should_retry_on_restart_noise


def run_command_with_restart_retry(cmd: list[str]) -> tuple[int, int, str, str]:
    return _execution_module.run_command_with_restart_retry(
        cmd,
        run_command_fn=_run_command,
        should_retry_on_restart_noise_fn=should_retry_on_restart_noise,
    )


run_concurrent_step = partial(
    _execution_module.run_concurrent_step,
    expected_session_key_fn=expected_session_key,
    run_command_with_restart_retry_fn=run_command_with_restart_retry,
    tail_text_fn=_tail_text,
    step_result_cls=StepResult,
)
run_blackbox_step = partial(
    _execution_module.run_blackbox_step,
    expected_session_key_fn=expected_session_key,
    run_command_with_restart_retry_fn=run_command_with_restart_retry,
    tail_text_fn=_tail_text,
    step_result_cls=StepResult,
)
build_mixed_concurrency_steps = partial(
    _execution_module.build_mixed_concurrency_steps,
    matrix_step_cls=MatrixStep,
)
run_mixed_concurrency_batch = partial(
    _execution_module.run_mixed_concurrency_batch,
    run_blackbox_step_fn=run_blackbox_step,
    build_mixed_concurrency_steps_fn=build_mixed_concurrency_steps,
)


def run_matrix(cfg: ProbeConfig) -> tuple[bool, dict[str, object]]:
    return _execution_module.run_matrix(
        cfg,
        script_dir=Path(__file__).resolve().parent,
        build_report_fn=build_report,
        build_matrix_steps_fn=build_matrix_steps,
        run_concurrent_step_fn=run_concurrent_step,
        run_blackbox_step_fn=run_blackbox_step,
        run_mixed_concurrency_batch_fn=run_mixed_concurrency_batch,
    )


def main() -> int:
    try:
        cfg = build_config(parse_args())
    except ValueError as error:
        print(f"Error: {error}", file=sys.stderr)
        return 2

    passed, report = run_matrix(cfg)
    write_outputs(report, cfg.output_json, cfg.output_markdown)

    print("Session matrix completed.")
    print(f"  overall={'PASS' if passed else 'FAIL'}")
    print(f"  steps={report['summary']['passed']}/{report['summary']['total']}")
    print(f"  json_report={cfg.output_json}")
    print(f"  markdown_report={cfg.output_markdown}")

    if not passed:
        failed_steps = [step["name"] for step in report["steps"] if not step["passed"]]
        print(f"  failed_steps={failed_steps}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
