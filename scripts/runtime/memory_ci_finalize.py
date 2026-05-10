#!/usr/bin/env python3
"""Compatibility facade for memory CI gate finalization script."""

from __future__ import annotations

from pathlib import Path

from memory_ci_finalize_cli import parse_args
from memory_ci_finalize_runtime import finalize_gate_run as _finalize_gate_run_impl

# Backward-compatible private alias.
_parse_args = parse_args


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
    _finalize_gate_run_impl(
        reports_dir=reports_dir,
        profile=profile,
        start_stamp=start_stamp,
        exit_code=exit_code,
        latest_failure_json=latest_failure_json,
        latest_failure_md=latest_failure_md,
        latest_run_json=latest_run_json,
        log_file=log_file,
        finish_stamp=finish_stamp,
    )


def main() -> int:
    args = parse_args()
    finalize_gate_run(
        reports_dir=Path(args.reports_dir),
        profile=str(args.profile),
        start_stamp=int(args.start_stamp),
        exit_code=int(args.exit_code),
        latest_failure_json=Path(args.latest_failure_json),
        latest_failure_md=Path(args.latest_failure_md),
        latest_run_json=Path(args.latest_run_json),
        log_file=Path(args.log_file),
        finish_stamp=int(args.finish_stamp),
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
