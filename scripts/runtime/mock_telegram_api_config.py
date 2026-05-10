#!/usr/bin/env python3
"""Config helpers for mock Telegram API server."""

from __future__ import annotations

import argparse
from dataclasses import dataclass

from resolve_tool_endpoint import resolve_tool_endpoint


@dataclass(frozen=True)
class ServerConfig:
    host: str
    port: int


def parse_args() -> ServerConfig:
    """Parse CLI args for mock Telegram API server."""
    parser = argparse.ArgumentParser(
        description="Run a minimal Telegram Bot API mock server."
    )
    parser.add_argument(
        "--host",
        default=str(resolve_tool_endpoint()["host"]),
        help="Bind host (default: resolved local host).",
    )
    parser.add_argument(
        "--port", type=int, default=18080, help="Bind port (default: 18080)."
    )
    args = parser.parse_args()
    if args.port <= 0 or args.port > 65535:
        raise ValueError("--port must be in range 1..65535")
    return ServerConfig(host=args.host, port=args.port)
