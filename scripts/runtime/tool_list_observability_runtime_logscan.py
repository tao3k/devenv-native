#!/usr/bin/env python3
"""Log scan helpers for tool-list observability runtime."""

from __future__ import annotations

import re
from typing import Any

_TOOLS_STATS_LINE_RE = re.compile(
    r"requests=(?P<requests>\d+)\s+hit_rate=(?P<hit_rate>[0-9.]+)%\s+"
    r"cache_hits=(?P<cache_hits>\d+)\s+cache_misses=(?P<cache_misses>\d+)\s+"
    r"build_count=(?P<build_count>\d+)\s+build_failures=(?P<build_failures>\d+)\s+"
    r"build_avg_ms=(?P<build_avg_ms>[0-9.]+)\s+build_max_ms=(?P<build_max_ms>[0-9.]+)"
)


def scan_log_file(log_file: Any, *, iter_log_lines_fn: Any) -> dict[str, Any]:
    """Scan runtime logs for Dynamic Loader and tools/list stats lines."""
    if not log_file.exists():
        return {"exists": False}

    dynamic_loader_count = 0
    tools_list_stats_count = 0
    tools_list_served_debug_count = 0
    last_dynamic_loader_line: str | None = None
    last_tools_list_stats_line: str | None = None
    tools_stats_lines: list[str] = []

    for line in iter_log_lines_fn(log_file, errors="replace"):
        if "Dynamic Loader" in line:
            dynamic_loader_count += 1
            last_dynamic_loader_line = line
        if "[tool-runtime] tools/list stats" in line:
            tools_list_stats_count += 1
            last_tools_list_stats_line = line
            tools_stats_lines.append(line)
        if "tools/list served" in line:
            tools_list_served_debug_count += 1

    parsed_stats: dict[str, Any] | None = None
    if tools_stats_lines:
        match = _TOOLS_STATS_LINE_RE.search(tools_stats_lines[-1])
        if match:
            parsed_stats = {
                "requests": int(match.group("requests")),
                "hit_rate_pct": float(match.group("hit_rate")),
                "cache_hits": int(match.group("cache_hits")),
                "cache_misses": int(match.group("cache_misses")),
                "build_count": int(match.group("build_count")),
                "build_failures": int(match.group("build_failures")),
                "build_avg_ms": float(match.group("build_avg_ms")),
                "build_max_ms": float(match.group("build_max_ms")),
            }

    return {
        "exists": True,
        "path": str(log_file),
        "dynamic_loader_count": dynamic_loader_count,
        "tools_list_stats_count": tools_list_stats_count,
        "tools_list_served_debug_count": tools_list_served_debug_count,
        "last_dynamic_loader_line": last_dynamic_loader_line,
        "last_tools_list_stats_line": last_tools_list_stats_line,
        "parsed_last_tools_list_stats": parsed_stats,
    }
