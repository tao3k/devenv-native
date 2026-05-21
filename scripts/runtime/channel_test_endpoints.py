#!/usr/bin/env python3
"""Shared endpoint builders for channel test modules."""

from __future__ import annotations

import os

DEFAULT_LOCAL_HOST = (
    os.environ.get("XIUXIAN_WENDAO_LOCAL_HOST", "localhost").strip() or "localhost"
)


def http_url(port: int, path: str = "", *, host: str = DEFAULT_LOCAL_HOST) -> str:
    """Build a loopback HTTP URL for test fixtures."""
    normalized_path = path if path.startswith("/") or not path else f"/{path}"
    return f"http://{host}:{port}{normalized_path}"


def webhook_url(port: int = 18081, *, host: str = DEFAULT_LOCAL_HOST) -> str:
    """Build the default Telegram webhook URL used by channel tests."""
    return http_url(port, "/telegram/webhook", host=host)


def discord_ingress_url(port: int = 18082, *, host: str = DEFAULT_LOCAL_HOST) -> str:
    """Build the default Discord ingress URL used by channel tests."""
    return http_url(port, "/discord/ingress", host=host)


def redis_url(port: int = 6379, db: int = 0, *, host: str = DEFAULT_LOCAL_HOST) -> str:
    """Build a Valkey URL for test fixtures."""
    return f"redis://{host}:{port}/{db}"
