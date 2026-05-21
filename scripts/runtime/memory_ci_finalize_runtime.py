#!/usr/bin/env python3
"""Runtime finalization helpers for memory CI gate artifacts."""

from __future__ import annotations

import json
import shutil
from typing import TYPE_CHECKING

from memory_ci_finalize_discovery import newest_failure
from memory_ci_finalize_payloads import (
    build_status_payload,
    write_fallback_failure_payload,
)

if TYPE_CHECKING:
    from pathlib import Path


def finalize_gate_run(
    *,
    reports_dir: Path,
    profile: str,
    start_stamp: int,
    exit_code: int,
    latest_failure_json: Path,
    latest_failure_md: Path,
    latest_run_json: Path,
    log_file: Path,
    finish_stamp: int,
) -> None:
    """Finalize latest pointers and run status payload for one gate execution."""
    profile_title = profile.capitalize()

    reports_dir.mkdir(parents=True, exist_ok=True)
    latest_failure_json.parent.mkdir(parents=True, exist_ok=True)
    latest_failure_md.parent.mkdir(parents=True, exist_ok=True)
    latest_run_json.parent.mkdir(parents=True, exist_ok=True)

    picked_json_path, picked_json_stamp = newest_failure(
        reports_dir,
        profile,
        extension="json",
        start_stamp=start_stamp,
    )
    picked_md_path, picked_md_stamp = newest_failure(
        reports_dir,
        profile,
        extension="md",
        start_stamp=start_stamp,
    )

    if exit_code != 0:
        if picked_json_path is not None:
            shutil.copy2(picked_json_path, latest_failure_json)
        else:
            write_fallback_failure_payload(
                latest_failure_json,
                profile=profile,
                exit_code=exit_code,
                log_file=log_file,
            )
        if picked_md_path is not None:
            shutil.copy2(picked_md_path, latest_failure_md)
        elif not latest_failure_md.exists():
            latest_failure_md.write_text(
                (
                    "# Omni Agent Memory CI Failure\n\n"
                    f"- profile: {profile}\n"
                    f"- exit_code: {exit_code}\n"
                    f"- log: {log_file}\n"
                ),
                encoding="utf-8",
            )

    status_payload = build_status_payload(
        profile=profile,
        start_stamp=start_stamp,
        finish_stamp=finish_stamp,
        exit_code=exit_code,
        log_file=log_file,
        latest_failure_json=latest_failure_json,
        latest_failure_md=latest_failure_md,
        picked_json_path=picked_json_path,
        picked_json_stamp=picked_json_stamp,
        picked_md_path=picked_md_path,
        picked_md_stamp=picked_md_stamp,
    )
    latest_run_json.write_text(
        json.dumps(status_payload, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )

    print(
        f"{profile_title} CI summary: "
        f"status={status_payload['status']} "
        f"exit_code={exit_code} "
        f"log={log_file} "
        f"latest_run={latest_run_json}"
    )
    if exit_code != 0:
        print(
            f"{profile_title} CI failure aggregates: "
            f"json={latest_failure_json} md={latest_failure_md}"
        )
