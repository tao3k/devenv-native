#!/usr/bin/env python3
"""CLI parsing helpers for memory CI finalization script."""

from __future__ import annotations

import argparse


def parse_args() -> argparse.Namespace:
    """Parse CLI args for memory CI finalization."""
    parser = argparse.ArgumentParser(
        description="Finalize memory CI gate run artifacts."
    )
    parser.add_argument("--reports-dir", required=True)
    parser.add_argument("--profile", required=True, choices=("quick", "nightly"))
    parser.add_argument("--start-stamp", required=True, type=int)
    parser.add_argument("--exit-code", required=True, type=int)
    parser.add_argument("--latest-failure-json", required=True)
    parser.add_argument("--latest-failure-md", required=True)
    parser.add_argument("--latest-run-json", required=True)
    parser.add_argument("--log-file", required=True)
    parser.add_argument("--finish-stamp", required=True, type=int)
    return parser.parse_args()
