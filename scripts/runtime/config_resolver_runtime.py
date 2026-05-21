#!/usr/bin/env python3
"""Runtime-log driven resolvers for session and username inference."""

from __future__ import annotations

import re
from typing import TYPE_CHECKING

from log_io import read_log_tail_lines

if TYPE_CHECKING:
    from pathlib import Path

ANSI_ESCAPE_RE = re.compile(r"\x1b\[[0-9;]*m")
SESSION_KEY_RE = re.compile(
    r"\bsession_key\s*(?:=|:)\s*(?:\"|')?([-\d]+(?::[-\d]+){0,2})(?:\"|')?"
)
PARTITION_MODE_RE = re.compile(
    r"\b(?:json_partition_mode|current_mode|requested_partition_mode)\s*=\s*"
    r"(?:\"|')?([A-Za-z0-9_-]+)(?:\"|')?"
)
USERNAME_TOKEN_RE = re.compile(r"\busername\s*=\s*(?:\"([^\"]*)\"|'([^']*)'|([^\s]+))")
CHAT_ID_RE = re.compile(r"\bchat_id:\s*Some\(([-\d]+)\)")
CHAT_TYPE_RE = re.compile(r'\bchat_type:\s*Some\("([^"]+)"\)')
THREAD_ID_RE = re.compile(r"\bmessage_thread_id:\s*Some\(([-\d]+)\)")

RUNTIME_LOG_TAIL_BYTES = 256 * 1024


def read_runtime_log_tail_lines(
    path: Path, tail_bytes: int = RUNTIME_LOG_TAIL_BYTES
) -> list[str]:
    """Read only the log tail to avoid loading large runtime logs into memory."""
    return read_log_tail_lines(path, tail_bytes=tail_bytes)


def normalize_telegram_session_partition_mode(raw: str | None) -> str | None:
    """Normalize partition mode aliases to canonical values."""
    if raw is None:
        return None
    token = raw.strip().lower()
    if not token:
        return None
    token = token.replace("-", "_")
    if token in {"chat", "channel"}:
        return "chat"
    if token in {"chat_user", "chatuser"}:
        return "chat_user"
    if token in {"user", "user_only", "useronly"}:
        return "user"
    if token in {"chat_thread_user", "chatthreaduser", "topic_user", "topicuser"}:
        return "chat_thread_user"
    return None


def session_ids_from_runtime_log(
    log_file: Path,
) -> tuple[int | None, int | None, int | None]:
    """Infer `(chat_id, user_id, thread_id)` from recent Telegram runtime logs."""
    if not log_file.exists():
        return None, None, None

    lines = read_runtime_log_tail_lines(log_file)
    for raw_line in reversed(lines):
        line = ANSI_ESCAPE_RE.sub("", raw_line)
        if "Parsed message, forwarding to agent" not in line:
            continue
        inferred = _session_ids_from_runtime_line(line)
        if inferred != (None, None, None):
            return inferred

    last_session_key: str | None = None
    for raw_line in lines:
        line = ANSI_ESCAPE_RE.sub("", raw_line)
        match = SESSION_KEY_RE.search(line)
        if match:
            last_session_key = match.group(1)

    if not last_session_key:
        return None, None, None

    return _session_ids_from_session_key(last_session_key)


def _session_ids_from_runtime_line(
    line: str,
) -> tuple[int | None, int | None, int | None]:
    session_key_match = SESSION_KEY_RE.search(line)
    if not session_key_match:
        return None, None, None

    session_key = session_key_match.group(1)
    chat_id_match = CHAT_ID_RE.search(line)
    chat_type_match = CHAT_TYPE_RE.search(line)
    thread_id_match = THREAD_ID_RE.search(line)

    thread_id = int(thread_id_match.group(1)) if thread_id_match else None
    parts = session_key.split(":")
    if len(parts) == 3:
        return int(parts[0]), int(parts[2]), int(parts[1])
    if len(parts) == 2:
        return int(parts[0]), int(parts[1]), thread_id
    if len(parts) == 1 and chat_type_match and chat_type_match.group(1) == "private":
        chat_id = int(chat_id_match.group(1)) if chat_id_match else int(parts[0])
        return chat_id, chat_id, thread_id
    return None, None, None


def _session_ids_from_session_key(
    session_key: str,
) -> tuple[int | None, int | None, int | None]:
    parts = session_key.split(":")
    if len(parts) == 2:
        return int(parts[0]), int(parts[1]), None
    if len(parts) == 3:
        return int(parts[0]), int(parts[2]), int(parts[1])
    return None, None, None


def session_partition_mode_from_runtime_log(log_file: Path) -> str | None:
    """Infer partition mode from runtime logs and session key shape."""
    if not log_file.exists():
        return None

    lines = read_runtime_log_tail_lines(log_file)
    for raw_line in reversed(lines):
        line = ANSI_ESCAPE_RE.sub("", raw_line)
        if "Parsed message, forwarding to agent" in line:
            key_match = SESSION_KEY_RE.search(line)
            if key_match:
                parts = key_match.group(1).split(":")
                if len(parts) == 3:
                    return "chat_thread_user"
                if len(parts) == 2:
                    return "chat_user"
                if len(parts) == 1:
                    return "chat"
        mode_match = PARTITION_MODE_RE.search(line)
        if mode_match:
            normalized = normalize_telegram_session_partition_mode(mode_match.group(1))
            if normalized:
                return normalized

    _, _, inferred_thread = session_ids_from_runtime_log(log_file)
    if inferred_thread is not None:
        return "chat_thread_user"
    return None


def username_from_runtime_log(log_file: Path) -> str | None:
    """Infer username from runtime logs by scanning latest username token."""
    if not log_file.exists():
        return None

    for raw_line in reversed(read_runtime_log_tail_lines(log_file)):
        line = ANSI_ESCAPE_RE.sub("", raw_line)
        match = USERNAME_TOKEN_RE.search(line)
        if not match:
            continue
        value = (match.group(1) or match.group(2) or match.group(3) or "").strip()
        if value and value not in {"*", "''", '""'}:
            return value
    return None
