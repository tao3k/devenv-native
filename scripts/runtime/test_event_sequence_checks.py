#!/usr/bin/env python3
"""Unit tests for event-sequence core checks."""

from __future__ import annotations

import importlib

module = importlib.import_module("event_sequence_checks")


def test_run_checks_returns_zero_for_minimal_happy_path() -> None:
    lines = [
        'event="telegram.dedup.evaluated"',
        'event="telegram.dedup.update_accepted"',
        'event="session.gate.backend.initialized" backend="valkey"',
        'event="session.gate.lease.acquired"',
        'event="session.window_slots.appended"',
        'event="session.gate.lease.released"',
        'event="agent.memory.backend.initialized" backend="valkey"',
        'event="agent.memory.state_load_succeeded"',
        'event="agent.memory.recall.planned"',
        'event="agent.memory.recall.injected"',
        'event="agent.memory.state_save_succeeded"',
    ]
    stripped_lines = list(lines)
    assert (
        module.run_checks(
            lines,
            stripped_lines,
            strict=False,
            require_memory=True,
            expect_memory_backend="valkey",
        )
        == 0
    )


def test_run_checks_fails_when_required_memory_missing() -> None:
    lines = ['event="telegram.dedup.evaluated"']
    stripped_lines = list(lines)
    assert (
        module.run_checks(
            lines,
            stripped_lines,
            strict=False,
            require_memory=True,
            expect_memory_backend="",
        )
        == 1
    )
