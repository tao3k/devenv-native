#!/usr/bin/env python3
"""
Deterministic webhook dedup black-box probe.

Posts the same Telegram update_id twice and verifies:
  - first post accepted (`telegram.dedup.update_accepted`)
  - second post dropped as duplicate (`telegram.dedup.duplicate_detected`)
"""

from __future__ import annotations

import importlib
import os
import sys
from functools import partial
from pathlib import Path

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

load_module_bindings = importlib.import_module(
    "dedup_probe_module_bindings"
).load_module_bindings
_config_module = importlib.import_module("dedup_probe_config")
_io_module = importlib.import_module("dedup_probe_io")
_models_module = importlib.import_module("dedup_probe_models")

_MODULES = load_module_bindings(__file__)
_FLOW = _MODULES.flow_module

ProbeConfig = _models_module.ProbeConfig
_SharedLogCursor = _MODULES.shared_log_cursor_cls
_shared_init_log_cursor = _MODULES.shared_init_log_cursor
_shared_read_new_log_lines_with_cursor = _MODULES.shared_read_new_log_lines_with_cursor

parse_args = partial(
    _config_module.parse_args,
    webhook_url_default=os.environ.get("OMNI_WEBHOOK_URL")
    or _MODULES.default_telegram_webhook_url(),
)
session_ids_from_runtime_log = _MODULES.session_ids_from_runtime_log
username_from_settings = _MODULES.username_from_settings
username_from_runtime_log = _MODULES.username_from_runtime_log
telegram_webhook_secret_token = _MODULES.telegram_webhook_secret_token


def build_config(args) -> ProbeConfig:
    return _config_module.build_config(
        args,
        probe_config_cls=ProbeConfig,
        session_ids_from_runtime_log_fn=session_ids_from_runtime_log,
        username_from_settings_fn=username_from_settings,
        username_from_runtime_log_fn=username_from_runtime_log,
        telegram_webhook_secret_token_fn=telegram_webhook_secret_token,
    )


def count_lines(path: Path) -> int:
    return _io_module.count_lines(path, init_log_cursor_fn=_shared_init_log_cursor)


def read_new_lines(path: Path, cursor: int) -> tuple[int, list[str]]:
    return _io_module.read_new_lines(
        path,
        cursor,
        read_new_log_lines_with_cursor_fn=_shared_read_new_log_lines_with_cursor,
        log_cursor_cls=_SharedLogCursor,
    )


post_webhook_update = _io_module.post_webhook_update
build_payload = _io_module.build_payload
strip_ansi = _io_module.strip_ansi


def collect_stats(lines: list[str], update_id: int) -> dict[str, int]:
    return _FLOW.collect_stats(lines, update_id, strip_ansi_fn=strip_ansi)


def print_relevant_tail(lines: list[str], update_id: int) -> None:
    _FLOW.print_relevant_tail(lines, update_id, strip_ansi_fn=strip_ansi)


def main() -> int:
    try:
        cfg = build_config(parse_args())
    except ValueError as error:
        print(f"Error: {error}", file=sys.stderr)
        return 2

    return _FLOW.run_probe(
        cfg,
        count_lines_fn=count_lines,
        build_payload_fn=build_payload,
        post_webhook_update_fn=post_webhook_update,
        read_new_lines_fn=read_new_lines,
        collect_stats_fn=collect_stats,
        print_relevant_tail_fn=print_relevant_tail,
    )


if __name__ == "__main__":
    raise SystemExit(main())
