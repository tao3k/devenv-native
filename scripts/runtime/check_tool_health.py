#!/usr/bin/env python3
"""Check tool runtime health endpoint and exit 0 only when status is healthy/ok."""

from __future__ import annotations

import argparse
import json
import urllib.error
import urllib.request
from collections.abc import Callable
from typing import Any

UrlOpen = Callable[..., Any]


def is_tool_healthy(
    host: str,
    port: int,
    timeout_secs: float = 2.0,
    *,
    opener: UrlOpen = urllib.request.urlopen,
) -> bool:
    url = f"http://{host}:{port}/health"
    try:
        with opener(url, timeout=timeout_secs) as response:
            if int(getattr(response, "status", 0)) != 200:
                return False
            payload = json.loads(response.read().decode("utf-8"))
    except (
        urllib.error.URLError,
        urllib.error.HTTPError,
        TimeoutError,
        ValueError,
        OSError,
    ):
        return False

    if not isinstance(payload, dict):
        return False
    status = str(payload.get("status", "")).lower()
    return status in {"healthy", "ok"}


def main() -> int:
    parser = argparse.ArgumentParser(description="Check tool runtime /health endpoint")
    parser.add_argument("--host", required=True, help="Tool runtime host")
    parser.add_argument("--port", type=int, required=True, help="Tool runtime port")
    parser.add_argument(
        "--timeout-secs", type=float, default=2.0, help="HTTP timeout seconds"
    )
    args = parser.parse_args()

    return 0 if is_tool_healthy(args.host, args.port, args.timeout_secs) else 1


if __name__ == "__main__":
    raise SystemExit(main())
