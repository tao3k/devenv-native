"""Subprocess isolation for heavyweight document extraction profiles."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Sequence

DOCUMENT_EXTRACT_FULL_ISOLATION_ENV = "WENDAO_DOCUMENT_EXTRACT_FULL_ISOLATION"
DOCUMENT_EXTRACT_FULL_TIMEOUT_ENV = "WENDAO_DOCUMENT_EXTRACT_FULL_TIMEOUT_SECONDS"

_DEFAULT_FULL_TIMEOUT_SECONDS = 900.0
_FALSE_ENV_VALUES = {"0", "false", "no", "off", "inline"}


def full_profile_isolation_enabled() -> bool:
    """Return whether full-profile extraction should run in a child process."""

    value = os.environ.get(DOCUMENT_EXTRACT_FULL_ISOLATION_ENV, "true")
    return value.strip().lower() not in _FALSE_ENV_VALUES


def run_isolated_document_extract(
    source_path: str | Path,
    output_dir: str | Path,
    *,
    profile: str,
    force: bool,
    source_preparation: str | None = None,
) -> None:
    """Run one document extraction in a child Python process.

    # Errors

    Raises `RuntimeError` when the child exits unsuccessfully. Raises
    `TimeoutError` when the child exceeds the configured timeout.
    """

    command = [
        sys.executable,
        "-m",
        "xiuxian_wendao_analyzer.document_isolation",
        "--source-path",
        str(source_path),
        "--output-dir",
        str(output_dir),
        "--profile",
        profile,
    ]
    if force:
        command.append("--force")
    if source_preparation:
        command.extend(["--source-preparation", source_preparation])

    try:
        result = subprocess.run(
            command,
            capture_output=True,
            check=False,
            text=True,
            timeout=_full_profile_timeout_seconds(),
        )
    except subprocess.TimeoutExpired as exc:
        raise TimeoutError(
            "isolated document extraction timed out after "
            f"{_full_profile_timeout_seconds():.0f}s for {source_path}"
        ) from exc

    if result.returncode == 0:
        return

    raise RuntimeError(
        "isolated document extraction failed with exit code "
        f"{result.returncode}: {_compact_process_output(result)}"
    )


def _full_profile_timeout_seconds() -> float:
    value = os.environ.get(DOCUMENT_EXTRACT_FULL_TIMEOUT_ENV)
    if value is None or not value.strip():
        return _DEFAULT_FULL_TIMEOUT_SECONDS
    try:
        timeout = float(value)
    except ValueError as exc:
        raise ValueError(f"{DOCUMENT_EXTRACT_FULL_TIMEOUT_ENV} must be a positive number") from exc
    if timeout <= 0:
        raise ValueError(f"{DOCUMENT_EXTRACT_FULL_TIMEOUT_ENV} must be a positive number")
    return timeout


def _compact_process_output(result: subprocess.CompletedProcess[str]) -> str:
    stderr = (result.stderr or "").strip()
    stdout = (result.stdout or "").strip()
    detail = stderr or stdout or "child process exited without output"
    if len(detail) <= 1200:
        return detail
    return detail[-1200:]


def _run_child(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-path", required=True)
    parser.add_argument("--output-dir", required=True)
    parser.add_argument("--profile", required=True)
    parser.add_argument("--force", action="store_true")
    parser.add_argument("--source-preparation", default="")
    args = parser.parse_args(argv)

    try:
        from .document_extract import _extract_document_resources_inline

        rows = _extract_document_resources_inline(
            Path(args.source_path),
            Path(args.output_dir),
            converter=None,
            profile=args.profile,
            error_row=False,
            source_preparation=args.source_preparation or None,
        )
    except Exception as exc:
        print(
            json.dumps(
                {
                    "status": "error",
                    "errorType": type(exc).__name__,
                    "error": str(exc),
                },
                ensure_ascii=True,
            ),
            file=sys.stderr,
        )
        return 1

    print(
        json.dumps(
            {
                "status": "ok",
                "resourceRows": len(rows),
            },
            ensure_ascii=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(_run_child())
