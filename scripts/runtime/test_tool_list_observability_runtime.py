#!/usr/bin/env python3
"""Unit tests for tool-list observability runtime helpers."""

from __future__ import annotations

import asyncio
import importlib
import sys
from pathlib import Path

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

runtime = importlib.import_module("tool_list_observability_runtime")
models = importlib.import_module("tool_list_observability_models")
endpoints = importlib.import_module("channel_test_endpoints")


def test_percentile_handles_empty_and_bounds() -> None:
    assert runtime.percentile([], 0.95) == 0.0
    values = [10.0, 20.0, 30.0, 40.0]
    assert runtime.percentile(values, 0.50) == 20.0
    assert runtime.percentile(values, 0.95) == 30.0
    assert runtime.percentile(values, 1.0) == 40.0


def test_normalize_base_url_trims_trailing_slash() -> None:
    base = endpoints.http_url(3002)
    assert runtime.normalize_base_url(f"{base}/") == base
    assert runtime.normalize_base_url(base) == base


def test_scan_log_file_parses_tools_list_stats(tmp_path) -> None:
    log_file = tmp_path / "runtime.log"
    log_file.write_text(
        "\n".join(
            [
                "2026-02-20 INFO Dynamic Loader initialized",
                "2026-02-20 INFO [tool-runtime] tools/list stats requests=10 hit_rate=60.0% cache_hits=6 cache_misses=4 build_count=4 build_failures=0 build_avg_ms=2.50 build_max_ms=6.25",
                "2026-02-20 DEBUG tools/list served request_id=42",
            ]
        )
        + "\n",
        encoding="utf-8",
    )

    def _iter_lines(path: Path, *, errors: str = "replace") -> list[str]:
        del errors
        return path.read_text(encoding="utf-8").splitlines()

    result = runtime.scan_log_file(log_file, iter_log_lines_fn=_iter_lines)
    assert result["exists"] is True
    assert result["dynamic_loader_count"] == 1
    assert result["tools_list_stats_count"] == 1
    assert result["tools_list_served_debug_count"] == 1
    parsed = result["parsed_last_tools_list_stats"]
    assert parsed == {
        "requests": 10,
        "hit_rate_pct": 60.0,
        "cache_hits": 6,
        "cache_misses": 4,
        "build_count": 4,
        "build_failures": 0,
        "build_avg_ms": 2.5,
        "build_max_ms": 6.25,
    }


def test_scan_log_file_reports_missing_file(tmp_path) -> None:
    missing = tmp_path / "missing.log"
    result = runtime.scan_log_file(
        missing, iter_log_lines_fn=lambda *_args, **_kwargs: []
    )
    assert result == {"exists": False}


def test_run_sequential_profile_collects_stats() -> None:
    async def _fake_call(_client, _rpc_url, request_id: int) -> tuple[float, int, int]:
        return float(request_id), 128, 7

    result = asyncio.run(
        runtime.run_sequential_profile(
            client=object(),
            rpc_url=f"{endpoints.http_url(3002)}/",
            sample_count=3,
            sleep_ms=0,
            start_id=100,
            call_tools_list_fn=_fake_call,
            sequential_stats_cls=models.SequentialStats,
        )
    )
    assert result == models.SequentialStats(
        count=3,
        first_ms=100.0,
        second_ms=101.0,
        min_ms=100.0,
        median_ms=101.0,
        max_ms=102.0,
    )


def test_run_benchmark_collects_errors_and_percentiles() -> None:
    async def _fake_call(_client, _rpc_url, request_id: int) -> tuple[float, int, int]:
        if request_id % 2 == 0:
            raise RuntimeError("boom")
        return 10.0 + float(request_id % 5), 64, 3

    result = asyncio.run(
        runtime.run_benchmark(
            client=object(),
            rpc_url=f"{endpoints.http_url(3002)}/",
            total=6,
            concurrency=3,
            start_id=1,
            call_tools_list_fn=_fake_call,
            benchmark_stats_cls=models.BenchmarkStats,
        )
    )
    assert result.total == 6
    assert result.concurrency == 3
    assert result.errors == 3
    assert result.p50_ms > 0.0
    assert result.p95_ms >= result.p50_ms
