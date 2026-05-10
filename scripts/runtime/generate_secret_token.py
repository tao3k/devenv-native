#!/usr/bin/env python3
"""Generate URL-safe secret tokens for local webhook development."""

from __future__ import annotations

import argparse
import secrets


def generate_secret_token(length: int = 32) -> str:
    return secrets.token_urlsafe(max(1, int(length)))


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate URL-safe secret token")
    parser.add_argument(
        "--length",
        type=int,
        default=32,
        help="token byte length passed to secrets.token_urlsafe",
    )
    args = parser.parse_args()
    print(generate_secret_token(args.length), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
