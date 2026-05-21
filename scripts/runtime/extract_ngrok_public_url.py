#!/usr/bin/env python3
"""Extract first ngrok tunnel public_url from ngrok local API JSON on stdin."""

from __future__ import annotations

import json
import sys
from typing import Any


def extract_public_url(payload: dict[str, Any]) -> str:
    tunnels = payload.get("tunnels", [])
    if not isinstance(tunnels, list):
        return ""
    for tunnel in tunnels:
        if not isinstance(tunnel, dict):
            continue
        public_url = str(tunnel.get("public_url", "")).strip()
        if public_url:
            return public_url
    return ""


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except Exception:
        return 0
    if not isinstance(payload, dict):
        return 0
    print(extract_public_url(payload), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
