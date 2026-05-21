#!/usr/bin/env python3
"""Unit tests for Discord ingress stress summary helpers."""

from __future__ import annotations

import importlib
import sys
from pathlib import Path
from types import SimpleNamespace

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

module = importlib.import_module("discord_ingress_stress_runtime_summary")
endpoints = importlib.import_module("channel_test_endpoints")


def test_evaluate_quality_applies_failure_rate_and_p95_thresholds() -> None:
    cfg = SimpleNamespace(
        quality_max_failure_rate=0.05,
        quality_max_p95_ms=100.0,
        quality_min_rps=10.0,
    )
    measured = [
        SimpleNamespace(
            total_requests=100, failed_requests=8, p95_latency_ms=120.0, rps=20.0
        ),
    ]
    passed, failures = module.evaluate_quality(cfg, measured)
    assert passed is False
    assert any("failure_rate" in line for line in failures)
    assert any("max_round_p95_ms" in line for line in failures)


def test_build_report_populates_summary_and_rounds() -> None:
    cfg = SimpleNamespace(
        rounds=1,
        warmup_rounds=0,
        parallel=2,
        requests_per_worker=3,
        timeout_secs=0.5,
        cooldown_secs=0.0,
        ingress_url=endpoints.discord_ingress_url(),
        channel_id="2001",
        user_id="1001",
        guild_id="3001",
        username="alice",
        role_ids=("r1",),
        log_file="/tmp/runtime.log",
        quality_max_failure_rate=0.05,
        quality_max_p95_ms=100.0,
        quality_min_rps=10.0,
    )
    rounds = [
        SimpleNamespace(
            round_index=1,
            warmup=False,
            total_requests=6,
            success_requests=6,
            failed_requests=0,
            non_200_responses=0,
            responses_5xx=0,
            connection_errors=0,
            avg_latency_ms=10.0,
            p95_latency_ms=12.0,
            max_latency_ms=13.0,
            duration_ms=100,
            rps=60.0,
            log_parsed_messages=6,
            log_queue_wait_events=0,
            log_foreground_gate_wait_events=0,
            log_inbound_queue_unavailable_events=0,
        )
    ]
    report = module.build_report(
        cfg,
        started_at="2026-01-01T00:00:00Z",
        finished_at="2026-01-01T00:00:01Z",
        duration_ms=1000,
        rounds=rounds,
        measured=rounds,
        quality_passed=True,
        quality_failures=[],
    )
    assert report["summary"]["total_requests"] == 6
    assert report["summary"]["quality_passed"] is True
    assert report["rounds"][0]["round_index"] == 1
