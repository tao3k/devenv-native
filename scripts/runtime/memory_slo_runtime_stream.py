#!/usr/bin/env python3
"""Runtime-stream health evaluation for memory/session SLO aggregation."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path

    from memory_slo_models import SloConfig


def evaluate_stream_health(
    cfg: SloConfig, runtime_log_file: Path | None
) -> dict[str, Any]:
    """Evaluate optional stream-consumer health metrics from runtime logs."""
    if runtime_log_file is None:
        return {
            "enabled": False,
            "passed": True,
            "failures": [],
            "summary": {
                "published_events": 0,
                "processed_events": 0,
                "read_failed_events": 0,
                "ack_ratio": None,
                "runtime_log_file": None,
            },
        }

    if not runtime_log_file.exists():
        return {
            "enabled": bool(cfg.enable_stream_gate),
            "passed": not cfg.enable_stream_gate,
            "failures": (
                [f"stream.runtime_log_file_missing={runtime_log_file}"]
                if cfg.enable_stream_gate
                else []
            ),
            "summary": {
                "published_events": 0,
                "processed_events": 0,
                "read_failed_events": 0,
                "ack_ratio": None,
                "runtime_log_file": str(runtime_log_file),
            },
        }

    published_events = 0
    processed_events = 0
    read_failed_events = 0

    with runtime_log_file.open("r", encoding="utf-8", errors="ignore") as handle:
        for line in handle:
            if "session.stream_event.published" in line:
                published_events += 1
            if "agent.memory.stream_consumer.event_processed" in line:
                processed_events += 1
            if "agent.memory.stream_consumer.read_failed" in line:
                read_failed_events += 1

    ack_ratio: float | None = None
    if published_events > 0:
        ack_ratio = processed_events / published_events

    failures: list[str] = []
    if cfg.enable_stream_gate:
        if published_events < cfg.min_stream_published_events:
            failures.append(
                f"stream.published_events={published_events} < {cfg.min_stream_published_events}"
            )
        if ack_ratio is None:
            failures.append("stream.ack_ratio unavailable (published_events=0)")
        elif ack_ratio < cfg.min_stream_ack_ratio:
            failures.append(
                f"stream.ack_ratio={ack_ratio:.4f} < {cfg.min_stream_ack_ratio:.4f}"
            )
        if read_failed_events > cfg.max_stream_read_failed:
            failures.append(
                f"stream.read_failed_events={read_failed_events} > {cfg.max_stream_read_failed}"
            )

    return {
        "enabled": bool(cfg.enable_stream_gate),
        "passed": len(failures) == 0,
        "failures": failures,
        "summary": {
            "published_events": published_events,
            "processed_events": processed_events,
            "read_failed_events": read_failed_events,
            "ack_ratio": round(ack_ratio, 4) if ack_ratio is not None else None,
            "runtime_log_file": str(runtime_log_file),
        },
    }
