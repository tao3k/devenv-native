#!/usr/bin/env python3
"""Stat helpers for tool-list observability runtime."""

from __future__ import annotations


def percentile(sorted_values: list[float], p: float) -> float:
    """Compute index-based percentile for sorted values."""
    if not sorted_values:
        return 0.0
    idx = max(0, min(len(sorted_values) - 1, int(len(sorted_values) * p) - 1))
    return sorted_values[idx]


def normalize_base_url(base_url: str) -> str:
    """Normalize base URL by trimming trailing slash."""
    return base_url.rstrip("/")
