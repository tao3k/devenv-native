#!/usr/bin/env python3
"""Datamodels for tool-list observability probe."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class SequentialStats:
    """Summary statistics for sequential tools/list sampling."""

    count: int
    first_ms: float
    second_ms: float
    min_ms: float
    median_ms: float
    max_ms: float


@dataclass
class BenchmarkStats:
    """Summary statistics for concurrent tools/list benchmark."""

    total: int
    concurrency: int
    errors: int
    elapsed_s: float
    rps: float
    p50_ms: float
    p95_ms: float
    p99_ms: float
