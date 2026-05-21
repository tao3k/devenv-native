#!/usr/bin/env python3
"""Config builder helpers for memory/session SLO report."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

from memory_slo_config_parser import parse_required_modes
from memory_slo_config_paths import resolve_path

if TYPE_CHECKING:
    import argparse

    from memory_slo_models import SloConfig


def build_config(
    args: argparse.Namespace,
    *,
    config_cls: type[SloConfig],
) -> SloConfig:
    """Build validated SLO config from parsed arguments."""
    project_root = resolve_path(args.project_root, Path.cwd())
    if args.min_planned_hits < 0:
        raise ValueError("--min-planned-hits must be >= 0.")
    if args.min_successful_corrections < 0:
        raise ValueError("--min-successful-corrections must be >= 0.")
    if args.min_recall_credit_events < 0:
        raise ValueError("--min-recall-credit-events must be >= 0.")
    if args.min_quality_score < 0:
        raise ValueError("--min-quality-score must be >= 0.")
    if args.min_query_turns < 0:
        raise ValueError("--min-query-turns must be >= 0.")
    if args.max_mode_tool_error_turns < 0:
        raise ValueError("--max-mode-tool-error-turns must be >= 0.")
    if args.max_total_tool_error_turns < 0:
        raise ValueError("--max-total-tool-error-turns must be >= 0.")
    if args.min_session_steps < 0:
        raise ValueError("--min-session-steps must be >= 0.")
    if args.max_session_failed_steps < 0:
        raise ValueError("--max-session-failed-steps must be >= 0.")
    if args.min_stream_ack_ratio < 0 or args.min_stream_ack_ratio > 1:
        raise ValueError("--min-stream-ack-ratio must be in [0, 1].")
    if args.min_stream_published_events < 0:
        raise ValueError("--min-stream-published-events must be >= 0.")
    if args.max_stream_read_failed < 0:
        raise ValueError("--max-stream-read-failed must be >= 0.")

    runtime_log_file = None
    if args.runtime_log_file.strip():
        runtime_log_file = resolve_path(args.runtime_log_file, project_root)

    return config_cls(
        project_root=project_root,
        evolution_report_json=resolve_path(args.evolution_report_json, project_root),
        benchmark_report_json=resolve_path(args.benchmark_report_json, project_root),
        session_matrix_report_json=resolve_path(
            args.session_matrix_report_json, project_root
        ),
        runtime_log_file=runtime_log_file,
        output_json=resolve_path(args.output_json, project_root),
        output_markdown=resolve_path(args.output_markdown, project_root),
        min_planned_hits=int(args.min_planned_hits),
        min_successful_corrections=int(args.min_successful_corrections),
        min_recall_credit_events=int(args.min_recall_credit_events),
        min_quality_score=float(args.min_quality_score),
        required_benchmark_modes=parse_required_modes(args.required_benchmark_modes),
        min_query_turns=int(args.min_query_turns),
        max_mode_tool_error_turns=int(args.max_mode_tool_error_turns),
        max_total_tool_error_turns=int(args.max_total_tool_error_turns),
        min_session_steps=int(args.min_session_steps),
        max_session_failed_steps=int(args.max_session_failed_steps),
        enable_stream_gate=bool(args.enable_stream_gate),
        min_stream_ack_ratio=float(args.min_stream_ack_ratio),
        min_stream_published_events=int(args.min_stream_published_events),
        max_stream_read_failed=int(args.max_stream_read_failed),
    )
