#!/usr/bin/env python3
"""Resolve tool-runtime port from unified resolver (xiuxian.toml)."""

from __future__ import annotations

from resolve_tool_endpoint import resolve_tool_endpoint


def resolve_tool_port() -> int | None:
    resolved = resolve_tool_endpoint()
    try:
        port = int(resolved["port"])
    except (KeyError, TypeError, ValueError):
        return None
    return port if 1 <= port <= 65535 else None


def main() -> int:
    port = resolve_tool_port()
    print("" if port is None else str(port), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
