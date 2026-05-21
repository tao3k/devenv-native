#!/usr/bin/env python3
"""Datamodels for memory/session SLO aggregation."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


@dataclass(frozen=True)
class SloConfig:
    """Configuration thresholds and input/output paths for SLO report generation."""

    project_root: Path
    evolution_report_json: Path
    benchmark_report_json: Path
    session_matrix_report_json: Path
    runtime_log_file: Path | None
    output_json: Path
    output_markdown: Path
    min_planned_hits: int
    min_successful_corrections: int
    min_recall_credit_events: int
    min_quality_score: float
    required_benchmark_modes: tuple[str, ...]
    min_query_turns: int
    max_mode_tool_error_turns: int
    max_total_tool_error_turns: int
    min_session_steps: int
    max_session_failed_steps: int
    enable_stream_gate: bool
    min_stream_ack_ratio: float
    min_stream_published_events: int
    max_stream_read_failed: int
