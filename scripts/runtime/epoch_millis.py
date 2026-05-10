#!/usr/bin/env python3
"""Emit current epoch milliseconds."""

from __future__ import annotations

import time


def epoch_millis() -> int:
    return int(time.time() * 1000)


def main() -> int:
    print(epoch_millis(), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
