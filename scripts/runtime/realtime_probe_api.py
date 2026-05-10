#!/usr/bin/env python3
"""Realtime Telegram/Discord probe APIs for lightweight channel validation."""

from __future__ import annotations

import os
import time
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any

from agent_channel_blackbox_parsing_tokens import extract_event_token, strip_ansi
from concurrent_sessions_runtime_http import (
    build_payload as build_telegram_payload,
)
from concurrent_sessions_runtime_http import (
    post_webhook as post_telegram_webhook,
)
from config_resolver_profiles import group_profile_int
from config_resolver_telegram_webhook import (
    default_telegram_webhook_url,
    telegram_webhook_secret_token,
)
from discord_acl_events_config_urls import (
    default_ingress_url as default_discord_ingress_url,
)
from discord_ingress_stress_runtime_http import (
    build_ingress_payload as build_discord_payload,
)
from discord_ingress_stress_runtime_http import (
    post_ingress_event as post_discord_ingress_event,
)
from log_io import LogCursor, init_log_cursor, read_new_log_lines_with_cursor
from read_setting import read_setting

_TELEGRAM_ERROR_PATTERNS = (
    "telegram sendmessage failed",
    "failed to send",
    "foreground message handling failed",
    "tools/call: tool runtime error",
)
_DISCORD_ERROR_PATTERNS = (
    "discord failed to send command reply",
    "foreground message handling failed",
    "tools/call: tool runtime error",
)


@dataclass(frozen=True)
class TelegramRealtimeProbeConfig:
    webhook_url: str
    log_file: Path
    chat_id: int
    user_id: int
    prompt: str
    secret_token: str | None = None
    username: str | None = None
    thread_id: int | None = None
    max_wait_secs: float = 45.0
    max_idle_secs: float = 12.0
    expected_events: tuple[str, ...] = ()
    follow_logs: bool = False


@dataclass(frozen=True)
class DiscordRealtimeProbeConfig:
    ingress_url: str
    log_file: Path
    channel_id: str
    user_id: str
    prompt: str
    secret_token: str | None = None
    username: str | None = None
    guild_id: str | None = None
    role_ids: tuple[str, ...] = ()
    max_wait_secs: float = 45.0
    max_idle_secs: float = 12.0
    expected_events: tuple[str, ...] = ()
    follow_logs: bool = False


@dataclass(frozen=True)
class RealtimeProbeResult:
    provider: str
    ok: bool
    post_status: int
    post_body_preview: str
    elapsed_secs: float
    inbound_seen: bool
    reply_seen: bool
    bot_seen: bool
    missing_expected_events: tuple[str, ...]
    event_counts: dict[str, int]
    inbound_line: str | None
    reply_line: str | None
    bot_line: str | None
    error: str | None
    log_tail: list[str] = field(default_factory=list)

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass
class _ProbeState:
    inbound_seen: bool = False
    reply_seen: bool = False
    bot_seen: bool = False
    inbound_line: str | None = None
    reply_line: str | None = None
    bot_line: str | None = None
    error: str | None = None


def resolve_telegram_probe_chat_id(
    value: int | None, repo_root: Path | None = None
) -> int | None:
    """Resolve Telegram chat id from explicit value, env, then group profile."""
    if value is not None:
        return int(value)
    explicit = os.environ.get("OMNI_TEST_CHAT_ID", "").strip()
    if explicit:
        return int(explicit)
    return group_profile_int("OMNI_TEST_CHAT_ID", repo_root)


def resolve_telegram_probe_user_id(
    value: int | None, repo_root: Path | None = None
) -> int | None:
    """Resolve Telegram user id from explicit value, env, then group profile."""
    if value is not None:
        return int(value)
    explicit = os.environ.get("OMNI_TEST_USER_ID", "").strip()
    if explicit:
        return int(explicit)
    return group_profile_int("OMNI_TEST_USER_ID", repo_root)


def resolve_telegram_probe_url(value: str | None, repo_root: Path | None = None) -> str:
    """Resolve Telegram webhook URL."""
    explicit = (value or "").strip()
    if explicit:
        return explicit
    return default_telegram_webhook_url(repo_root)


def resolve_discord_probe_url(value: str | None) -> str:
    """Resolve Discord ingress URL."""
    explicit = (value or "").strip()
    if explicit:
        return explicit
    return default_discord_ingress_url()


def resolve_discord_probe_secret(value: str | None) -> str | None:
    """Resolve Discord ingress secret token from explicit value/env/settings."""
    explicit = (value or "").strip()
    if explicit:
        return explicit
    for key in (
        "XIUXIAN_DAOCHANG_DISCORD_INGRESS_SECRET_TOKEN",
        "DISCORD_INGRESS_SECRET_TOKEN",
        "DISCORD_INGRESS_SECRET",
    ):
        token = os.environ.get(key, "").strip()
        if token:
            return token
    setting_value = read_setting("discord.ingress_secret_token").strip()
    return setting_value or None


def resolve_discord_probe_channel_id(value: str | None) -> str | None:
    """Resolve Discord channel id from explicit value or env."""
    explicit = (value or "").strip()
    if explicit:
        return explicit
    env_value = os.environ.get("OMNI_TEST_DISCORD_CHANNEL_ID", "").strip()
    return env_value or None


def resolve_discord_probe_user_id(value: str | None) -> str | None:
    """Resolve Discord user id from explicit value or env."""
    explicit = (value or "").strip()
    if explicit:
        return explicit
    env_value = os.environ.get("OMNI_TEST_DISCORD_USER_ID", "").strip()
    return env_value or None


def _count_lines(path: Path) -> int:
    return int(init_log_cursor(path, kind="offset").value)


def _read_new_lines(path: Path, cursor: int) -> tuple[int, list[str]]:
    next_cursor, lines = read_new_log_lines_with_cursor(
        path, LogCursor(kind="offset", value=cursor)
    )
    return int(next_cursor.value), list(lines)


def _trim_tail(lines: list[str], max_lines: int = 180) -> list[str]:
    if len(lines) <= max_lines:
        return lines
    return lines[-max_lines:]


def _preview(text: str, max_chars: int = 360) -> str:
    if len(text) <= max_chars:
        return text
    return text[: max_chars - 3] + "..."


def _update_event_counts(event_counts: dict[str, int], event: str | None) -> None:
    if not event:
        return
    event_counts[event] = int(event_counts.get(event, 0)) + 1


def _poll_runtime_log(
    *,
    provider: str,
    log_file: Path,
    start_cursor: int,
    max_wait_secs: float,
    max_idle_secs: float,
    expected_events: tuple[str, ...],
    state: _ProbeState,
    process_line_fn: Any,
    error_patterns: tuple[str, ...],
    follow_logs: bool,
    count_lines_fn: Any,
    read_new_lines_fn: Any,
    sleep_fn: Any,
    monotonic_fn: Any,
) -> tuple[float, tuple[str, ...], dict[str, int], list[str]]:
    _ = count_lines_fn  # retained for API symmetry/injection compatibility
    started = monotonic_fn()
    deadline = started + max_wait_secs
    last_activity = started
    cursor = start_cursor
    event_counts: dict[str, int] = {}
    pending_expected = {token for token in expected_events if token}
    log_tail: list[str] = []

    while monotonic_fn() <= deadline:
        cursor, chunk = read_new_lines_fn(log_file, cursor)
        if chunk:
            last_activity = monotonic_fn()
            for raw_line in chunk:
                line = strip_ansi(raw_line.rstrip("\n"))
                log_tail.append(line)
                event = extract_event_token(line)
                _update_event_counts(event_counts, event)
                if event in pending_expected:
                    pending_expected.discard(event)

                process_line_fn(line, event, state)
                lowered = line.lower()
                if any(pattern in lowered for pattern in error_patterns):
                    state.error = f"{provider} runtime error pattern observed"
                    break
                if follow_logs:
                    print(f"[{provider}] {line}")
            if state.error:
                break

            if (
                provider == "telegram"
                and state.inbound_seen
                and (state.reply_seen or state.bot_seen)
                and not pending_expected
            ):
                break
            if (
                provider == "discord"
                and state.inbound_seen
                and (state.reply_seen or state.bot_seen)
                and not pending_expected
            ):
                break
        else:
            idle_secs = monotonic_fn() - last_activity
            if idle_secs >= max_idle_secs:
                break
            sleep_fn(0.25)

    return (
        monotonic_fn() - started,
        tuple(sorted(pending_expected)),
        event_counts,
        _trim_tail(log_tail),
    )


def run_telegram_realtime_probe(
    config: TelegramRealtimeProbeConfig,
    *,
    count_lines_fn: Any = _count_lines,
    read_new_lines_fn: Any = _read_new_lines,
    post_webhook_fn: Any = post_telegram_webhook,
    sleep_fn: Any = time.sleep,
    monotonic_fn: Any = time.monotonic,
) -> RealtimeProbeResult:
    """Run one synthetic Telegram realtime probe against local webhook runtime."""
    log_file = Path(config.log_file)
    log_file.parent.mkdir(parents=True, exist_ok=True)
    cursor = count_lines_fn(log_file)
    update_id = int(time.time() * 1_000_000)
    trace = f"bootcamp-tg-{update_id}"
    prompt = f"{config.prompt.strip()} [{trace}]"
    payload = build_telegram_payload(
        update_id=update_id,
        chat_id=config.chat_id,
        user_id=config.user_id,
        username=config.username,
        prompt=prompt,
        thread_id=config.thread_id,
    )

    post_started = monotonic_fn()
    status, body = post_webhook_fn(config.webhook_url, payload, config.secret_token)
    _ = monotonic_fn() - post_started
    state = _ProbeState()

    def _process_line(line: str, event: str | None, mutable_state: _ProbeState) -> None:
        if (
            str(update_id) in line
            and "Webhook received Telegram update" in line
            and not mutable_state.inbound_seen
        ):
            mutable_state.inbound_seen = True
            mutable_state.inbound_line = line
        if str(update_id) in line and "telegram.dedup.update_accepted" in line:
            mutable_state.inbound_seen = True
            mutable_state.inbound_line = mutable_state.inbound_line or line
        if (
            event
            and event.startswith("telegram.command.")
            and event.endswith(".replied")
        ):
            mutable_state.reply_seen = True
            mutable_state.reply_line = mutable_state.reply_line or line
        if "→ Bot:" in line:
            mutable_state.bot_seen = True
            mutable_state.bot_line = mutable_state.bot_line or line

    elapsed_secs, missing_events, event_counts, log_tail = _poll_runtime_log(
        provider="telegram",
        log_file=log_file,
        start_cursor=cursor,
        max_wait_secs=config.max_wait_secs,
        max_idle_secs=config.max_idle_secs,
        expected_events=config.expected_events,
        state=state,
        process_line_fn=_process_line,
        error_patterns=_TELEGRAM_ERROR_PATTERNS,
        follow_logs=config.follow_logs,
        count_lines_fn=count_lines_fn,
        read_new_lines_fn=read_new_lines_fn,
        sleep_fn=sleep_fn,
        monotonic_fn=monotonic_fn,
    )

    post_ok = status == 200
    markers_ok = state.inbound_seen and (state.reply_seen or state.bot_seen)
    ok = post_ok and markers_ok and not missing_events and state.error is None
    error: str | None = state.error
    if error is None and not post_ok:
        error = f"telegram webhook returned HTTP {status}"
    if error is None and not state.inbound_seen:
        error = "telegram inbound marker not observed"
    if error is None and not (state.reply_seen or state.bot_seen):
        error = "telegram reply marker not observed"
    if error is None and missing_events:
        error = f"missing expected events: {', '.join(missing_events)}"

    return RealtimeProbeResult(
        provider="telegram",
        ok=ok,
        post_status=int(status),
        post_body_preview=_preview(body),
        elapsed_secs=round(float(elapsed_secs), 3),
        inbound_seen=state.inbound_seen,
        reply_seen=state.reply_seen,
        bot_seen=state.bot_seen,
        missing_expected_events=missing_events,
        event_counts=event_counts,
        inbound_line=state.inbound_line,
        reply_line=state.reply_line,
        bot_line=state.bot_line,
        error=error,
        log_tail=log_tail,
    )


def run_discord_realtime_probe(
    config: DiscordRealtimeProbeConfig,
    *,
    count_lines_fn: Any = _count_lines,
    read_new_lines_fn: Any = _read_new_lines,
    post_ingress_fn: Any = post_discord_ingress_event,
    sleep_fn: Any = time.sleep,
    monotonic_fn: Any = time.monotonic,
) -> RealtimeProbeResult:
    """Run one synthetic Discord realtime probe against local ingress runtime."""
    log_file = Path(config.log_file)
    log_file.parent.mkdir(parents=True, exist_ok=True)
    cursor = count_lines_fn(log_file)
    event_id = f"{int(time.time() * 1000)}-{os.getpid()}"
    trace = f"bootcamp-ds-{event_id}"
    prompt = f"{config.prompt.strip()} [{trace}]"
    payload = build_discord_payload(config, event_id, prompt)

    post_started = monotonic_fn()
    status, body, _latency_ms = post_ingress_fn(
        config.ingress_url,
        payload,
        config.secret_token,
        min(15.0, max(1.0, float(config.max_wait_secs))),
    )
    _ = monotonic_fn() - post_started
    state = _ProbeState()

    def _process_line(line: str, event: str | None, mutable_state: _ProbeState) -> None:
        lowered = line.lower()
        if "discord ingress ignored event" in lowered:
            mutable_state.inbound_seen = True
            mutable_state.reply_seen = True
            mutable_state.inbound_line = mutable_state.inbound_line or line
            mutable_state.reply_line = mutable_state.reply_line or line
        if "discord ingress parsed message" in lowered or (
            event is not None and event.startswith("discord.ingress.")
        ):
            mutable_state.inbound_seen = True
            mutable_state.inbound_line = mutable_state.inbound_line or line
        if (
            event
            and event.startswith("discord.command.")
            and event.endswith(".replied")
        ):
            mutable_state.reply_seen = True
            mutable_state.reply_line = mutable_state.reply_line or line
        if "→ Bot:" in line:
            mutable_state.bot_seen = True
            mutable_state.bot_line = mutable_state.bot_line or line

    elapsed_secs, missing_events, event_counts, log_tail = _poll_runtime_log(
        provider="discord",
        log_file=log_file,
        start_cursor=cursor,
        max_wait_secs=config.max_wait_secs,
        max_idle_secs=config.max_idle_secs,
        expected_events=config.expected_events,
        state=state,
        process_line_fn=_process_line,
        error_patterns=_DISCORD_ERROR_PATTERNS,
        follow_logs=config.follow_logs,
        count_lines_fn=count_lines_fn,
        read_new_lines_fn=read_new_lines_fn,
        sleep_fn=sleep_fn,
        monotonic_fn=monotonic_fn,
    )

    post_ok = 200 <= int(status) < 300
    markers_ok = state.inbound_seen and (state.reply_seen or state.bot_seen)
    ok = post_ok and markers_ok and not missing_events and state.error is None
    error: str | None = state.error
    if error is None and not post_ok:
        error = f"discord ingress returned HTTP {status}"
    if error is None and not state.inbound_seen:
        error = "discord inbound marker not observed"
    if error is None and not (state.reply_seen or state.bot_seen):
        error = "discord reply marker not observed"
    if error is None and missing_events:
        error = f"missing expected events: {', '.join(missing_events)}"

    return RealtimeProbeResult(
        provider="discord",
        ok=ok,
        post_status=int(status),
        post_body_preview=_preview(body),
        elapsed_secs=round(float(elapsed_secs), 3),
        inbound_seen=state.inbound_seen,
        reply_seen=state.reply_seen,
        bot_seen=state.bot_seen,
        missing_expected_events=missing_events,
        event_counts=event_counts,
        inbound_line=state.inbound_line,
        reply_line=state.reply_line,
        bot_line=state.bot_line,
        error=error,
        log_tail=log_tail,
    )


def default_telegram_secret(repo_root: Path | None = None) -> str | None:
    """Resolve Telegram webhook secret from env/settings."""
    return telegram_webhook_secret_token(repo_root)


__all__ = [
    "DiscordRealtimeProbeConfig",
    "RealtimeProbeResult",
    "TelegramRealtimeProbeConfig",
    "default_telegram_secret",
    "resolve_discord_probe_channel_id",
    "resolve_discord_probe_secret",
    "resolve_discord_probe_url",
    "resolve_discord_probe_user_id",
    "resolve_telegram_probe_chat_id",
    "resolve_telegram_probe_url",
    "resolve_telegram_probe_user_id",
    "run_discord_realtime_probe",
    "run_telegram_realtime_probe",
]
