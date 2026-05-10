#!/usr/bin/env python3
"""
Probe tools/list observability and performance for a running tool runtime server.

What this script checks in one run:
1. /health is reachable
2. tools/list returns valid payload + tool count + payload size
3. embed/batch returns vectors and dimension
4. tools/list latency profile (sequential sample + concurrent benchmarks)
5. Optional log scan for Dynamic Loader + tools/list stats lines
"""

from __future__ import annotations

import argparse
import asyncio
import importlib
import json
import sys
from pathlib import Path
from typing import Any

from log_io import iter_log_lines
from resolve_tool_endpoint import resolve_tool_endpoint

_models_module = importlib.import_module("tool_list_observability_models")
_runtime_module = importlib.import_module("tool_list_observability_runtime")

SequentialStats = _models_module.SequentialStats
BenchmarkStats = _models_module.BenchmarkStats

_percentile = _runtime_module.percentile
_normalize_base_url = _runtime_module.normalize_base_url
_call_tools_list = _runtime_module.call_tools_list


async def _run_sequential_profile(
    client: Any,
    rpc_url: str,
    *,
    sample_count: int,
    sleep_ms: int,
    start_id: int,
) -> SequentialStats:
    return await _runtime_module.run_sequential_profile(
        client,
        rpc_url,
        sample_count=sample_count,
        sleep_ms=sleep_ms,
        start_id=start_id,
        call_tools_list_fn=_call_tools_list,
        sequential_stats_cls=SequentialStats,
    )


async def _run_benchmark(
    client: Any,
    rpc_url: str,
    *,
    total: int,
    concurrency: int,
    start_id: int,
) -> BenchmarkStats:
    return await _runtime_module.run_benchmark(
        client,
        rpc_url,
        total=total,
        concurrency=concurrency,
        start_id=start_id,
        call_tools_list_fn=_call_tools_list,
        benchmark_stats_cls=BenchmarkStats,
    )


def _scan_log_file(log_file: Path) -> dict[str, Any]:
    return _runtime_module.scan_log_file(log_file, iter_log_lines_fn=iter_log_lines)


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "One-shot tools/list observability + benchmark probe for a running tool runtime server."
        )
    )
    parser.add_argument("--base-url", default="")
    parser.add_argument("--timeout-secs", type=float, default=30.0)
    parser.add_argument("--sequential-samples", type=int, default=20)
    parser.add_argument("--sequential-sleep-ms", type=int, default=50)
    parser.add_argument("--bench-small-total", type=int, default=200)
    parser.add_argument("--bench-small-concurrency", type=int, default=40)
    parser.add_argument("--bench-large-total", type=int, default=1000)
    parser.add_argument("--bench-large-concurrency", type=int, default=100)
    parser.add_argument(
        "--log-file",
        type=Path,
        default=None,
        help="Optional runtime log file path for Dynamic Loader/tools-list stats checks.",
    )
    parser.add_argument(
        "--allow-request-errors",
        action="store_true",
        help="Do not fail process exit on benchmark request errors.",
    )
    parser.add_argument(
        "--json-out",
        type=Path,
        default=None,
        help="Optional path to write full JSON summary.",
    )
    return parser.parse_args()


async def _run_probe(args: argparse.Namespace) -> dict[str, Any]:
    return await _runtime_module.run_probe(
        args,
        sequential_stats_cls=SequentialStats,
        benchmark_stats_cls=BenchmarkStats,
        iter_log_lines_fn=iter_log_lines,
        call_tools_list_fn=_call_tools_list,
        run_sequential_profile_fn=_runtime_module.run_sequential_profile,
        run_benchmark_fn=_runtime_module.run_benchmark,
        scan_log_file_fn=_runtime_module.scan_log_file,
    )


def main() -> int:
    args = _parse_args()
    args.base_url = args.base_url.strip() or str(resolve_tool_endpoint()["base_url"])
    try:
        summary = asyncio.run(_run_probe(args))
    except Exception as exc:
        print(f"probe_failed: {exc}", file=sys.stderr)
        return 1

    small_errors = int(summary["benchmarks"]["small"]["errors"])
    large_errors = int(summary["benchmarks"]["large"]["errors"])
    error_total = small_errors + large_errors

    print("=== Tool-list observability probe ===")
    print(f"base_url: {summary['base_url']}")
    print(
        "tools/list: "
        f"count={summary['tools_list']['tool_count']} "
        f"payload_bytes={summary['tools_list']['payload_bytes']} "
        f"first_call_ms={summary['tools_list']['first_call_ms']}"
    )
    print(
        "embed/batch: "
        f"vectors={summary['embed_batch']['vector_count']} "
        f"dim={summary['embed_batch']['vector_dim']}"
    )
    print(f"sequential: {summary['sequential_profile']}")
    print(f"bench_small: {summary['benchmarks']['small']}")
    print(f"bench_large: {summary['benchmarks']['large']}")
    if summary["log_scan"] is not None:
        print(f"log_scan: {summary['log_scan']}")

    if args.json_out is not None:
        args.json_out.parent.mkdir(parents=True, exist_ok=True)
        args.json_out.write_text(
            json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
        )
        print(f"json_out: {args.json_out}")

    print("--- summary_json ---")
    print(json.dumps(summary, ensure_ascii=False))

    if error_total > 0 and not args.allow_request_errors:
        print(
            f"probe_failed: benchmark request errors detected (errors={error_total})",
            file=sys.stderr,
        )
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
