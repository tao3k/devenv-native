#!/usr/bin/env python3
"""Output rendering helpers for memory/session SLO reports."""

from __future__ import annotations

import json
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


def render_markdown(report: dict[str, Any]) -> str:
    """Render compact markdown summary from SLO report payload."""
    checks = report["checks"]
    evolution = checks["evolution"]
    benchmark = checks["benchmark"]
    session_matrix = checks["session_matrix"]
    stream = checks["stream"]
    status = "PASS" if report["overall_passed"] else "FAIL"

    lines = [
        "# Omni-Agent Memory SLO Report",
        "",
        f"- Overall: `{status}`",
        f"- Failure count: `{report['failure_count']}`",
        "",
        "## Inputs",
        "",
        f"- Evolution: `{report['inputs']['evolution_report_json']}`",
        f"- Benchmark: `{report['inputs']['benchmark_report_json']}`",
        f"- Session matrix: `{report['inputs']['session_matrix_report_json']}`",
        f"- Runtime log: `{report['inputs']['runtime_log_file']}`",
        "",
        "## Gate Results",
        "",
        f"- Evolution: `{'PASS' if evolution['passed'] else 'FAIL'}`",
        f"- Benchmark: `{'PASS' if benchmark['passed'] else 'FAIL'}`",
        f"- Session matrix: `{'PASS' if session_matrix['passed'] else 'FAIL'}`",
        f"- Stream gate: `{'PASS' if stream['passed'] else 'FAIL'}` (enabled={stream['enabled']})",
        "",
        "## Key Metrics",
        "",
        (
            "- Evolution minima: "
            "planned_hits={ph}, successful_corrections={sc}, "
            "recall_credit_events={rc}, quality_score={qs}".format(
                ph=evolution["summary"]["min_planned_hits"],
                sc=evolution["summary"]["min_successful_corrections"],
                rc=evolution["summary"]["min_recall_credit_events"],
                qs=evolution["summary"]["min_quality_score"],
            )
        ),
        (
            "- Benchmark tool runtime errors: "
            f"{benchmark['summary']['total_tool_error_turns']} "
            f"(per-mode={benchmark['summary']['tool_error_turns_by_mode']})"
        ),
        (
            "- Session matrix: "
            f"total_steps={session_matrix['summary']['total_steps']} "
            f"failed_steps={session_matrix['summary']['failed_steps']}"
        ),
        (
            "- Stream health: "
            f"published={stream['summary']['published_events']} "
            f"processed={stream['summary']['processed_events']} "
            f"read_failed={stream['summary']['read_failed_events']} "
            f"ack_ratio={stream['summary']['ack_ratio']}"
        ),
    ]

    failures = report.get("failures", [])
    if failures:
        lines.extend(["", "## Failures", ""])
        lines.extend([f"- {item}" for item in failures])

    return "\n".join(lines)


def write_outputs(
    report: dict[str, Any], output_json: Path, output_markdown: Path
) -> None:
    """Persist JSON + Markdown outputs to disk."""
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_markdown.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(
        json.dumps(report, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    output_markdown.write_text(render_markdown(report) + "\n", encoding="utf-8")
