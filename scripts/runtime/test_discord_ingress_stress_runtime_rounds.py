#!/usr/bin/env python3
"""Unit tests for Discord ingress stress round helpers."""

from __future__ import annotations

import importlib
import sys
from dataclasses import dataclass
from pathlib import Path
from types import SimpleNamespace

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

module = importlib.import_module("discord_ingress_stress_runtime_rounds")
endpoints = importlib.import_module("channel_test_endpoints")


def test_collect_log_stats_counts_expected_markers() -> None:
    stats = module.collect_log_stats(
        [
            'event="discord.ingress.inbound_queue_wait"',
            "discord ingress parsed message",
            'event="discord.foreground.gate_wait"',
            "discord inbound queue unavailable",
            "discord ingress parsed message",
        ]
    )
    assert stats["parsed_messages"] == 2
    assert stats["queue_wait_events"] == 1
    assert stats["foreground_gate_wait_events"] == 1
    assert stats["inbound_queue_unavailable_events"] == 1


def test_run_worker_aggregates_status_classes() -> None:
    cfg = SimpleNamespace(
        requests_per_worker=3,
        prompt="stress",
        ingress_url=endpoints.discord_ingress_url(),
        secret_token="secret",
        timeout_secs=0.2,
    )
    statuses = [(200, "ok", 10.0), (503, "upstream", 20.0), (0, "conn", 30.0)]

    def _next_event_id() -> str:
        return "evt"

    def _build_payload(_cfg: object, _event_id: str, _prompt: str) -> bytes:
        return b"{}"

    state = {"idx": 0}

    def _post_event(_url: str, _payload: bytes, _secret: str, _timeout: float):
        value = statuses[state["idx"]]
        state["idx"] += 1
        return value

    result = module.run_worker(
        cfg,
        round_index=1,
        worker_index=1,
        next_event_id_fn=_next_event_id,
        build_ingress_payload_fn=_build_payload,
        post_ingress_event_fn=_post_event,
    )
    assert result["total_requests"] == 3
    assert result["success_requests"] == 1
    assert result["failed_requests"] == 2
    assert result["non_200_responses"] == 1
    assert result["responses_5xx"] == 1
    assert result["connection_errors"] == 1
    assert result["latencies_ms"] == (10.0, 20.0, 30.0)


def test_run_round_aggregates_workers_and_log_counters(tmp_path: Path) -> None:
    cfg = SimpleNamespace(
        parallel=2,
        log_file=tmp_path / "runtime.log",
    )

    def _init_log_offset(_path: Path) -> int:
        return 0

    def _read_new_log_lines(_path: Path, _offset: int) -> tuple[int, list[str]]:
        return 1, [
            "discord ingress parsed message",
            'event="discord.ingress.inbound_queue_wait"',
        ]

    def _run_worker(
        _cfg: object, _round_index: int, worker_index: int
    ) -> dict[str, object]:
        if worker_index == 1:
            return {
                "total_requests": 2,
                "success_requests": 2,
                "failed_requests": 0,
                "non_200_responses": 0,
                "responses_5xx": 0,
                "connection_errors": 0,
                "latencies_ms": (10.0, 12.0),
            }
        return {
            "total_requests": 2,
            "success_requests": 1,
            "failed_requests": 1,
            "non_200_responses": 1,
            "responses_5xx": 1,
            "connection_errors": 0,
            "latencies_ms": (20.0, 25.0),
        }

    @dataclass(frozen=True)
    class _RoundResult:
        round_index: int
        warmup: bool
        total_requests: int
        success_requests: int
        failed_requests: int
        non_200_responses: int
        responses_5xx: int
        connection_errors: int
        avg_latency_ms: float
        p95_latency_ms: float
        max_latency_ms: float
        duration_ms: int
        rps: float
        log_parsed_messages: int
        log_queue_wait_events: int
        log_foreground_gate_wait_events: int
        log_inbound_queue_unavailable_events: int

    row = module.run_round(
        cfg,
        round_index=1,
        warmup=False,
        round_result_cls=_RoundResult,
        init_log_offset_fn=_init_log_offset,
        read_new_log_lines_fn=_read_new_log_lines,
        run_worker_fn=_run_worker,
        p95_fn=lambda values: sorted(values)[2],
    )
    assert row.total_requests == 4
    assert row.success_requests == 3
    assert row.failed_requests == 1
    assert row.non_200_responses == 1
    assert row.responses_5xx == 1
    assert row.log_parsed_messages == 1
    assert row.log_queue_wait_events == 1
