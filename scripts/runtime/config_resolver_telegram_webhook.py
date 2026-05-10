#!/usr/bin/env python3
"""Webhook secret/bind/port/url resolution helpers for Telegram channel."""

from __future__ import annotations

import os
import re
from pathlib import Path

from config_resolver_core import (
    read_telegram_key_from_toml,
    repo_root_from,
    settings_candidates,
)
from config_resolver_profiles import env_or_dotenv_value
from resolve_tool_endpoint import resolve_tool_endpoint

WEBHOOK_BIND_PORT_RE = re.compile(r":(\d{1,5})$")
DEFAULT_TELEGRAM_WEBHOOK_PORT = 18081
NAMESPACED_WEBHOOK_BIND_ENV = "XIUXIAN_WENDAO_TELEGRAM_WEBHOOK_BIND"
LEGACY_WEBHOOK_BIND_ENV = "WEBHOOK_BIND"
NAMESPACED_WEBHOOK_PORT_ENV = "XIUXIAN_WENDAO_TELEGRAM_WEBHOOK_PORT"
LEGACY_WEBHOOK_PORT_ENV = "WEBHOOK_PORT"


def telegram_webhook_secret_token(repo_root: Path | None = None) -> str | None:
    """Resolve webhook secret from env/.env, then settings."""
    root = repo_root or repo_root_from(Path(__file__).resolve())
    secret = env_or_dotenv_value("TELEGRAM_WEBHOOK_SECRET", root)
    if secret:
        return secret

    for settings_path in settings_candidates(root):
        configured = read_telegram_key_from_toml(settings_path, "webhook_secret_token")
        if configured is None:
            continue
        normalized = configured.strip()
        if normalized and normalized not in {"null", "None", "~"}:
            return normalized
    return None


def _parse_port(raw: str, *, source: str) -> int:
    token = raw.strip()
    if not token:
        raise ValueError(f"{source} webhook port cannot be empty.")
    try:
        port = int(token)
    except ValueError as error:
        raise ValueError(
            f"{source} webhook port must be an integer, got '{raw}'."
        ) from error
    if port <= 0 or port > 65535:
        raise ValueError(f"{source} webhook port out of range: {port}.")
    return port


def _port_from_bind(bind: str) -> int | None:
    match = WEBHOOK_BIND_PORT_RE.search(bind.strip())
    if not match:
        return None
    raw_port = match.group(1)
    try:
        return _parse_port(raw_port, source="settings")
    except ValueError:
        return None


def telegram_webhook_bind(repo_root: Path | None = None) -> str | None:
    """Resolve webhook bind from env or settings."""
    for env_name in (NAMESPACED_WEBHOOK_BIND_ENV, LEGACY_WEBHOOK_BIND_ENV):
        explicit_bind = os.environ.get(env_name, "").strip()
        if explicit_bind:
            return explicit_bind

    root = repo_root or repo_root_from(Path(__file__).resolve())
    for settings_path in settings_candidates(root):
        configured = read_telegram_key_from_toml(settings_path, "webhook_bind")
        if configured is None:
            continue
        normalized = configured.strip()
        if normalized and normalized not in {"null", "None", "~"}:
            return normalized
    return None


def telegram_webhook_port(repo_root: Path | None = None) -> int:
    """Resolve webhook port from WEBHOOK_PORT, bind, or default."""
    for env_name in (NAMESPACED_WEBHOOK_PORT_ENV, LEGACY_WEBHOOK_PORT_ENV):
        explicit_port = os.environ.get(env_name, "").strip()
        if explicit_port:
            return _parse_port(explicit_port, source=env_name)

    bind = telegram_webhook_bind(repo_root)
    if bind:
        parsed = _port_from_bind(bind)
        if parsed is not None:
            return parsed
    return DEFAULT_TELEGRAM_WEBHOOK_PORT


def default_telegram_webhook_url(repo_root: Path | None = None) -> str:
    """Build local webhook URL from resolved webhook port."""
    bind = telegram_webhook_bind(repo_root)
    host = ""
    if bind:
        host = bind.strip().split(":", 1)[0].strip("[]").strip()
    if not host:
        host = str(resolve_tool_endpoint()["host"])
    return f"http://{host}:{telegram_webhook_port(repo_root)}/telegram/webhook"
