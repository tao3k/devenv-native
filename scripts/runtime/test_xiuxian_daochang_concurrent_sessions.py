#!/usr/bin/env python3
"""
Concurrent multi-session black-box probe for Telegram webhook runtime.

This probe sends the same command concurrently to two distinct session identities
(same chat, different user ids by default; or different chats with same user) and verifies:
  - dedup accepted events for both update ids
  - parsed inbound session_key for both sessions
  - command reply event for both sessions
  - no duplicate_detected for these new update ids
"""

from __future__ import annotations

import importlib
import os
import re
import sys
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import argparse

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

load_sibling_module = importlib.import_module("module_loader").load_sibling_module

_resolver_module = load_sibling_module(
    module_name="config_resolver",
    file_name="config_resolver.py",
    caller_file=__file__,
    error_context="resolver module",
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
telegram_webhook_secret_token = _resolver_module.telegram_webhook_secret_token
username_from_runtime_log = _resolver_module.username_from_runtime_log
username_from_settings = _resolver_module.username_from_settings

_log_io_module = load_sibling_module(
    module_name="log_io",
    file_name="log_io.py",
    caller_file=__file__,
    error_context="shared log I/O helpers",
)
_session_keys_module = load_sibling_module(
    module_name="telegram_session_keys",
    file_name="telegram_session_keys.py",
    caller_file=__file__,
    error_context="telegram session key helpers",
)
_models_module = load_sibling_module(
    module_name="concurrent_sessions_models",
    file_name="concurrent_sessions_models.py",
    caller_file=__file__,
    error_context="concurrent sessions datamodels",
)
_config_module = load_sibling_module(
    module_name="concurrent_sessions_config",
    file_name="concurrent_sessions_config.py",
    caller_file=__file__,
    error_context="concurrent sessions config helpers",
)
_runtime_module = load_sibling_module(
    module_name="concurrent_sessions_runtime",
    file_name="concurrent_sessions_runtime.py",
    caller_file=__file__,
    error_context="concurrent sessions runtime helpers",
)

_SharedLogCursor = _log_io_module.LogCursor
_shared_init_log_cursor = _log_io_module.init_log_cursor
_shared_read_new_log_lines_with_cursor = _log_io_module.read_new_log_lines_with_cursor

ProbeConfig = _models_module.ProbeConfig
Observation = _models_module.Observation

ANSI_ESCAPE_RE = re.compile(r"\x1b\[[0-9;]*m")
SESSION_KEY_RE = re.compile(
    r"\bsession_key(?:\s*=\s*|\x1b\[2m=\x1b\[0m)(?:\"|')?([-\d:]+)(?:\"|')?"
)


def strip_ansi(value: str) -> str:
    return _runtime_module.strip_ansi(value, ansi_escape_re=ANSI_ESCAPE_RE)


def resolve_runtime_partition_mode(
    log_file: Path, override: str | None = None
) -> str | None:
    return _config_module.resolve_runtime_partition_mode(
        log_file,
        override=override,
        normalize_partition_mode_fn=normalize_telegram_session_partition_mode,
        session_partition_mode_from_runtime_log_fn=session_partition_mode_from_runtime_log,
        telegram_session_partition_mode_fn=telegram_session_partition_mode,
    )


def expected_session_keys(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    session_partition: str | None = None,
) -> tuple[str, ...]:
    return _config_module.expected_session_keys(
        chat_id,
        user_id,
        thread_id,
        session_partition,
        expected_session_keys_fn=_session_keys_module.expected_session_keys,
        normalize_partition_mode_fn=normalize_telegram_session_partition_mode,
    )


def expected_session_key(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    session_partition: str | None = None,
) -> str:
    return _config_module.expected_session_key(
        chat_id,
        user_id,
        thread_id,
        session_partition,
        expected_session_key_fn=_session_keys_module.expected_session_key,
        normalize_partition_mode_fn=normalize_telegram_session_partition_mode,
    )


def parse_args() -> argparse.Namespace:
    webhook_url_default = (
        os.environ.get("OMNI_WEBHOOK_URL") or default_telegram_webhook_url()
    )
    return _config_module.parse_args(webhook_url_default=webhook_url_default)


def build_config(args: argparse.Namespace) -> ProbeConfig:
    return _config_module.build_config(
        args,
        group_profile_int_fn=group_profile_int,
        session_ids_from_runtime_log_fn=session_ids_from_runtime_log,
        username_from_settings_fn=username_from_settings,
        username_from_runtime_log_fn=username_from_runtime_log,
        telegram_webhook_secret_token_fn=telegram_webhook_secret_token,
        expected_session_keys_fn=expected_session_keys,
        resolve_runtime_partition_mode_fn=resolve_runtime_partition_mode,
        probe_config_cls=ProbeConfig,
    )


def count_lines(path: Path) -> int:
    return _runtime_module.count_lines(path, init_log_cursor_fn=_shared_init_log_cursor)


def read_new_lines(path: Path, cursor: int) -> tuple[int, list[str]]:
    return _runtime_module.read_new_lines(
        path,
        cursor,
        log_cursor_cls=_SharedLogCursor,
        read_new_log_lines_with_cursor_fn=_shared_read_new_log_lines_with_cursor,
    )


def build_payload(
    *,
    update_id: int,
    chat_id: int,
    user_id: int,
    username: str | None,
    prompt: str,
    thread_id: int | None,
) -> bytes:
    return _runtime_module.build_payload(
        update_id=update_id,
        chat_id=chat_id,
        user_id=user_id,
        username=username,
        prompt=prompt,
        thread_id=thread_id,
    )


def post_webhook(url: str, payload: bytes, secret_token: str | None) -> tuple[int, str]:
    return _runtime_module.post_webhook(url, payload, secret_token)


def collect_observation(
    lines: list[str],
    *,
    update_a: int,
    update_b: int,
    key_a_candidates: tuple[str, ...],
    key_b_candidates: tuple[str, ...],
    forbid_log_regexes: tuple[str, ...],
) -> Observation:
    return _runtime_module.collect_observation(
        lines,
        update_a=update_a,
        update_b=update_b,
        key_a_candidates=key_a_candidates,
        key_b_candidates=key_b_candidates,
        forbid_log_regexes=forbid_log_regexes,
        strip_ansi_fn=strip_ansi,
        session_key_re=SESSION_KEY_RE,
        observation_cls=Observation,
    )


def run_probe(cfg: ProbeConfig) -> int:
    return _runtime_module.run_probe(
        cfg,
        count_lines_fn=count_lines,
        read_new_lines_fn=read_new_lines,
        expected_session_keys_fn=expected_session_keys,
        build_payload_fn=build_payload,
        post_webhook_fn=post_webhook,
        collect_observation_fn=collect_observation,
        observation_cls=Observation,
    )


def main() -> int:
    try:
        cfg = build_config(parse_args())
    except ValueError as error:
        print(f"Error: {error}", file=sys.stderr)
        return 2
    return run_probe(cfg)


if __name__ == "__main__":
    raise SystemExit(main())
