#!/usr/bin/env python3
"""Shared Telegram session-key derivation helpers for channel probes."""

from __future__ import annotations

import re
from typing import Any

TELEGRAM_SESSION_SCOPE_PREFIX = "telegram:"
DISCORD_SESSION_SCOPE_PREFIX = "discord:"


def expected_session_keys(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    session_partition: str | None,
    *,
    normalize_partition_fn: Any,
) -> tuple[str, ...]:
    """Return accepted Telegram session-key forms for target identity."""
    partition = normalize_partition_fn(session_partition)
    if partition == "chat":
        return (str(chat_id),)
    if partition == "chat_user":
        return (f"{chat_id}:{user_id}",)
    if partition == "user":
        return (str(user_id),)
    if partition == "chat_thread_user":
        if thread_id is None:
            return (f"{chat_id}:0:{user_id}", f"{chat_id}:{user_id}")
        return (f"{chat_id}:{thread_id}:{user_id}",)
    if thread_id is None:
        return (f"{chat_id}:{user_id}", f"{chat_id}:0:{user_id}")
    return (f"{chat_id}:{thread_id}:{user_id}",)


def expected_session_key(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    session_partition: str | None,
    *,
    normalize_partition_fn: Any,
) -> str:
    """Return canonical first expected session key."""
    return expected_session_keys(
        chat_id,
        user_id,
        thread_id,
        session_partition,
        normalize_partition_fn=normalize_partition_fn,
    )[0]


def expected_session_scope_values(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    session_partition: str | None,
    *,
    normalize_partition_fn: Any,
    scope_prefixes: tuple[str, ...],
) -> tuple[str, ...]:
    """Build expected session-scope values with configured prefixes."""
    values: list[str] = []
    for prefix in scope_prefixes:
        for session_key in expected_session_keys(
            chat_id,
            user_id,
            thread_id,
            session_partition,
            normalize_partition_fn=normalize_partition_fn,
        ):
            value = f"{prefix}{session_key}"
            if value not in values:
                values.append(value)
    return tuple(values)


def expected_session_scope_prefixes(
    expect_events: tuple[str, ...],
    *,
    telegram_prefix: str = TELEGRAM_SESSION_SCOPE_PREFIX,
    discord_prefix: str = DISCORD_SESSION_SCOPE_PREFIX,
) -> tuple[str, ...]:
    """Infer session-scope prefixes from expected event families."""
    has_telegram = any(event.startswith("telegram.") for event in expect_events)
    has_discord = any(event.startswith("discord.") for event in expect_events)
    if has_telegram and not has_discord:
        return (telegram_prefix,)
    if has_discord and not has_telegram:
        return (discord_prefix,)
    return (telegram_prefix, discord_prefix)


def expected_session_log_regex(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    partition_mode: str | None,
    *,
    normalize_partition_fn: Any,
) -> str:
    """Build regex that matches any accepted session_key token."""
    escaped = [
        re.escape(key)
        for key in expected_session_keys(
            chat_id,
            user_id,
            thread_id,
            partition_mode,
            normalize_partition_fn=normalize_partition_fn,
        )
    ]
    body = escaped[0] if len(escaped) == 1 else f"(?:{'|'.join(escaped)})"
    return rf'session_key="?{body}"?'
