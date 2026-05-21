#!/usr/bin/env python3
"""Top-level probe flow for tool-list observability runtime."""

from __future__ import annotations

from dataclasses import asdict
from typing import Any

import httpx
from tool_list_observability_runtime_stats import normalize_base_url


async def run_probe(
    args: Any,
    *,
    sequential_stats_cls: Any,
    benchmark_stats_cls: Any,
    iter_log_lines_fn: Any,
    call_tools_list_fn: Any,
    run_sequential_profile_fn: Any,
    run_benchmark_fn: Any,
    scan_log_file_fn: Any,
) -> dict[str, Any]:
    """Execute full observability probe and return JSON-ready summary."""
    base_url = normalize_base_url(args.base_url)
    health_url = f"{base_url}/health"
    rpc_url = f"{base_url}/"
    embed_url = f"{base_url}/embed/batch"

    timeout = httpx.Timeout(args.timeout_secs)
    async with httpx.AsyncClient(timeout=timeout) as client:
        health_resp = await client.get(health_url)
        health_resp.raise_for_status()

        first_latency_ms, payload_bytes, tool_count = await call_tools_list_fn(
            client, rpc_url, 1
        )
        sequential = await run_sequential_profile_fn(
            client,
            rpc_url,
            sample_count=args.sequential_samples,
            sleep_ms=args.sequential_sleep_ms,
            start_id=1000,
            call_tools_list_fn=call_tools_list_fn,
            sequential_stats_cls=sequential_stats_cls,
        )

        small = await run_benchmark_fn(
            client,
            rpc_url,
            total=args.bench_small_total,
            concurrency=args.bench_small_concurrency,
            start_id=10_000,
            call_tools_list_fn=call_tools_list_fn,
            benchmark_stats_cls=benchmark_stats_cls,
        )
        large = await run_benchmark_fn(
            client,
            rpc_url,
            total=args.bench_large_total,
            concurrency=args.bench_large_concurrency,
            start_id=20_000,
            call_tools_list_fn=call_tools_list_fn,
            benchmark_stats_cls=benchmark_stats_cls,
        )

        embed_resp = await client.post(
            embed_url, json={"texts": ["obs-probe-a", "obs-probe-b"]}
        )
        embed_resp.raise_for_status()
        embed_payload = embed_resp.json()
        vectors = embed_payload.get("vectors")
        if (
            not isinstance(vectors, list)
            or not vectors
            or not isinstance(vectors[0], list)
        ):
            raise RuntimeError("embed/batch returned invalid vectors payload")

    log_scan = (
        scan_log_file_fn(args.log_file, iter_log_lines_fn=iter_log_lines_fn)
        if args.log_file
        else None
    )

    return {
        "base_url": base_url,
        "health_ok": True,
        "tools_list": {
            "tool_count": tool_count,
            "payload_bytes": payload_bytes,
            "first_call_ms": round(first_latency_ms, 2),
        },
        "embed_batch": {
            "vector_count": len(vectors),
            "vector_dim": len(vectors[0]),
        },
        "sequential_profile": asdict(sequential),
        "benchmarks": {
            "small": asdict(small),
            "large": asdict(large),
        },
        "log_scan": log_scan,
    }
