#!/usr/bin/env python3
"""Read telegram.* values via xiuxian.toml resolver."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

from config_resolver_core import (  # noqa: E402
    read_telegram_key_from_toml,
    repo_root_from,
    settings_candidates,
)


def read_telegram_setting(key: str) -> str:
    repo_root = repo_root_from(Path(__file__).resolve())
    for candidate_path in settings_candidates(repo_root):
        configured = read_telegram_key_from_toml(candidate_path, key)
        if configured is None:
            continue
        return str(configured).strip()
    return ""


def main() -> int:
    parser = argparse.ArgumentParser(description="Read telegram setting value")
    parser.add_argument(
        "--key", required=True, help="telegram setting key (without prefix)"
    )
    args = parser.parse_args()
    print(read_telegram_setting(str(args.key)), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
