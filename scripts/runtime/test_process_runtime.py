from __future__ import annotations

import subprocess
from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parents[2]
PROCESS_RUNTIME = PROJECT_ROOT / "scripts/runtime/process-runtime.sh"


def _run_runtime_check(command: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["bash", "-lc", command],
        cwd=PROJECT_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )


def test_managed_pid_matches_patterns_accepts_root_relative_process_display(
    tmp_path: Path,
) -> None:
    project_root = tmp_path / "project"
    project_root.mkdir()

    result = _run_runtime_check(
        f"""
set -euo pipefail
source "{PROCESS_RUNTIME}"
export PRJ_ROOT="{project_root}"
managed_process_command() {{
  printf '%s\\n' 'target/debug/wendao gateway start'
}}
managed_pid_matches_patterns 4805 "{project_root}/target/debug/wendao" " gateway start"
"""
    )

    assert result.returncode == 0, result.stderr


def test_managed_pid_matches_patterns_accepts_exact_absolute_command(
    tmp_path: Path,
) -> None:
    project_root = tmp_path / "project"
    project_root.mkdir()

    result = _run_runtime_check(
        f"""
set -euo pipefail
source "{PROCESS_RUNTIME}"
export PRJ_ROOT="{project_root}"
managed_process_command() {{
  printf '%s\\n' '{project_root}/target/debug/wendao gateway start'
}}
managed_pid_matches_patterns 4805 "{project_root}/target/debug/wendao" " gateway start"
"""
    )

    assert result.returncode == 0, result.stderr


def test_managed_pid_matches_patterns_rejects_foreign_command(tmp_path: Path) -> None:
    project_root = tmp_path / "project"
    project_root.mkdir()

    result = _run_runtime_check(
        f"""
set -euo pipefail
source "{PROCESS_RUNTIME}"
export PRJ_ROOT="{project_root}"
managed_process_command() {{
  printf '%s\\n' 'python -m http.server 9517'
}}
managed_pid_matches_patterns 4805 "{project_root}/target/debug/wendao" " gateway start"
"""
    )

    assert result.returncode != 0
