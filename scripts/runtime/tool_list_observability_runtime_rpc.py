#!/usr/bin/env python3
"""RPC call and benchmark helpers for tool-list observability."""

from __future__ import annotations

import asyncio
import statistics
import time
from typing import Any

from tool_list_observability_runtime_stats import percentile


async def call_tools_list(
    client: Any,
    rpc_url: str,
    request_id: int,
) -> tuple[float, int, int]:
    """Call JSON-RPC tools/list and return latency, payload size, tool count."""
    started = time.perf_counter()
    resp = await client.post(
        rpc_url,
        json={"jsonrpc": "2.0", "id": request_id, "method": "tools/list", "params": {}},
    )
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    resp.raise_for_status()
    payload = resp.json()
    if payload.get("error") is not None:
        raise RuntimeError(f"tools/list returned error: {payload['error']}")
    result = payload.get("result")
    if not isinstance(result, dict):
        raise RuntimeError("tools/list result is not an object")
    tools = result.get("tools")
    if not isinstance(tools, list):
        raise RuntimeError("tools/list result.tools is not a list")
    return elapsed_ms, len(resp.content), len(tools)


async def run_sequential_profile(
    client: Any,
    rpc_url: str,
    *,
    sample_count: int,
    sleep_ms: int,
    start_id: int,
    call_tools_list_fn: Any,
    sequential_stats_cls: Any,
) -> Any:
    """Run sequential tools/list sampling."""
    latencies: list[float] = []
    for i in range(sample_count):
        elapsed_ms, _, _ = await call_tools_list_fn(client, rpc_url, start_id + i)
        latencies.append(elapsed_ms)
        if sleep_ms > 0:
            await asyncio.sleep(sleep_ms / 1000.0)

    sorted_lat = sorted(latencies)
    second = sorted_lat[1] if len(sorted_lat) > 1 else sorted_lat[0]
    return sequential_stats_cls(
        count=len(sorted_lat),
        first_ms=round(sorted_lat[0], 2),
        second_ms=round(second, 2),
        min_ms=round(sorted_lat[0], 2),
        median_ms=round(statistics.median(sorted_lat), 2),
        max_ms=round(sorted_lat[-1], 2),
    )


async def run_benchmark(
    client: Any,
    rpc_url: str,
    *,
    total: int,
    concurrency: int,
    start_id: int,
    call_tools_list_fn: Any,
    benchmark_stats_cls: Any,
) -> Any:
    """Run concurrent tools/list benchmark."""
    semaphore = asyncio.Semaphore(concurrency)
    latencies: list[float] = []
    errors = 0

    async def one(idx: int) -> None:
        nonlocal errors
        async with semaphore:
            try:
                elapsed_ms, _, _ = await call_tools_list_fn(
                    client, rpc_url, start_id + idx
                )
            except Exception:
                errors += 1
                return
            latencies.append(elapsed_ms)

    started = time.perf_counter()
    await asyncio.gather(*(one(i) for i in range(total)))
    elapsed_s = time.perf_counter() - started
    sorted_lat = sorted(latencies)

    p50 = percentile(sorted_lat, 0.50)
    p95 = percentile(sorted_lat, 0.95)
    p99 = percentile(sorted_lat, 0.99)
    rps = (total / elapsed_s) if elapsed_s > 0 else 0.0

    return benchmark_stats_cls(
        total=total,
        concurrency=concurrency,
        errors=errors,
        elapsed_s=round(elapsed_s, 3),
        rps=round(rps, 2),
        p50_ms=round(p50, 2),
        p95_ms=round(p95, 2),
        p99_ms=round(p99, 2),
    )
