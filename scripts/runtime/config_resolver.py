#!/usr/bin/env python3
"""
Shared config helpers for retained local runtime scripts.

Resolution priority:
1) Explicit CLI / env values
2) User config:     $PRJ_CONFIG_HOME/xiuxian-artisan-workshop/xiuxian.toml
3) System config:   <repo>/packages/rust/crates/xiuxian-wendao/resources/config/xiuxian.toml
4) Runtime log inference (when available)
"""

from __future__ import annotations

import sys
from pathlib import Path

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

from config_resolver_profiles import (  # noqa: E402
    env_or_dotenv_value,
    group_profile_chat_ids,
    group_profile_int,
    group_profile_value,
)
from config_resolver_runtime import (  # noqa: E402
    normalize_telegram_session_partition_mode,
    session_ids_from_runtime_log,
    session_partition_mode_from_runtime_log,
    username_from_runtime_log,
)
from config_resolver_telegram import (  # noqa: E402
    allowed_users_from_settings,
    default_telegram_webhook_url,
    telegram_session_partition_mode,
    telegram_webhook_bind,
    telegram_webhook_port,
    telegram_webhook_secret_token,
    username_from_settings,
)

__all__ = [
    "allowed_users_from_settings",
    "default_telegram_webhook_url",
    "env_or_dotenv_value",
    "group_profile_chat_ids",
    "group_profile_int",
    "group_profile_value",
    "normalize_telegram_session_partition_mode",
    "session_ids_from_runtime_log",
    "session_partition_mode_from_runtime_log",
    "telegram_session_partition_mode",
    "telegram_webhook_bind",
    "telegram_webhook_port",
    "telegram_webhook_secret_token",
    "username_from_runtime_log",
    "username_from_settings",
]
