#!/usr/bin/env python3
"""Payload builders for memory CI finalization."""

from __future__ import annotations

import json
import time
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


def build_fallback_failure_payload(
    *,
    profile: str,
    exit_code: int,
    log_file: Path,
) -> dict[str, object]:
    """Build fallback failure payload when triage output is missing."""
    return {
        "generated_at_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "profile": profile,
        "category": "runner_unknown_failure",
        "summary": f"{profile} gate failed before triage json emission",
        "error": f"exit_code={exit_code}",
        "artifacts": [
            {
                "name": "runtime_log",
                "path": str(log_file),
                "exists": bool(log_file.exists()),
            }
        ],
        "repro_commands": [f"tail -n 200 {log_file}"],
    }


def write_fallback_failure_payload(
    latest_failure_json: Path,
    *,
    profile: str,
    exit_code: int,
    log_file: Path,
) -> None:
    """Write fallback failure payload to latest-failure json file."""
    payload = build_fallback_failure_payload(
        profile=profile,
        exit_code=exit_code,
        log_file=log_file,
    )
    latest_failure_json.write_text(
        json.dumps(payload, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )


def build_status_payload(
    *,
    profile: str,
    start_stamp: int,
    finish_stamp: int,
    exit_code: int,
    log_file: Path,
    latest_failure_json: Path,
    latest_failure_md: Path,
    picked_json_path: Path | None,
    picked_json_stamp: int,
    picked_md_path: Path | None,
    picked_md_stamp: int,
) -> dict[str, object]:
    """Build latest-run status payload for finalized CI gate."""
    return {
        "profile": profile,
        "started_at_ms": start_stamp,
        "finished_at_ms": finish_stamp,
        "duration_ms": max(0, finish_stamp - start_stamp),
        "exit_code": exit_code,
        "status": "passed" if exit_code == 0 else "failed",
        "log_file": str(log_file),
        "latest_failure_json": (
            str(latest_failure_json) if latest_failure_json.exists() else ""
        ),
        "latest_failure_markdown": (
            str(latest_failure_md) if latest_failure_md.exists() else ""
        ),
        "selected_failure_report_json": (
            str(picked_json_path) if picked_json_path is not None else ""
        ),
        "selected_failure_report_json_stamp": (
            picked_json_stamp if picked_json_stamp >= 0 else None
        ),
        "selected_failure_report_markdown": (
            str(picked_md_path) if picked_md_path is not None else ""
        ),
        "selected_failure_report_markdown_stamp": (
            picked_md_stamp if picked_md_stamp >= 0 else None
        ),
    }
