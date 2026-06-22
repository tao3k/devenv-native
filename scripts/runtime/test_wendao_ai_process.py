from __future__ import annotations

import os
import subprocess
from pathlib import Path

from check_runtime_web_health import is_expected_web_app_command


def test_wendao_ai_launch_uses_rsbuild_not_wendao_frontend() -> None:
    project_root = Path(__file__).resolve().parents[2]
    launch_script = (project_root / "scripts/runtime/wendao-ai-launch.sh").read_text(
        encoding="utf-8"
    )

    assert "WENDAO_AI_DIR" in launch_script
    assert "wendao.ai.git" in launch_script
    assert "rsbuild" in launch_script
    assert "WENDAO_FRONTEND_DIR" not in launch_script
    assert "wendao-frontend.git" not in launch_script


def test_wendao_ai_healthcheck_accepts_rsbuild_command() -> None:
    assert is_expected_web_app_command("node ./node_modules/.bin/rsbuild dev", "wendao.ai")
    assert is_expected_web_app_command("rspack-node dev-server", "wendao.ai")


def test_wendao_ai_healthcheck_shell_prefers_pyo3_python(tmp_path: Path) -> None:
    project_root = Path(__file__).resolve().parents[2]
    script_path = project_root / "scripts/runtime/wendao-ai-healthcheck.sh"
    fake_bin = tmp_path / "bin"
    fake_bin.mkdir()

    bad_python3 = fake_bin / "python3"
    bad_python3.write_text("#!/usr/bin/env bash\nexit 23\n", encoding="utf-8")
    bad_python3.chmod(0o755)

    fake_python = fake_bin / "python-good"
    fake_log = tmp_path / "python.log"
    fake_python.write_text(
        '#!/usr/bin/env bash\nset -euo pipefail\nprintf "%s\\n" "$@" >> "$FAKE_PYTHON_LOG"\n',
        encoding="utf-8",
    )
    fake_python.chmod(0o755)

    env = dict(os.environ)
    env["PATH"] = f"{fake_bin}:/usr/bin:/bin"
    env["PYO3_PYTHON"] = str(fake_python)
    env["FAKE_PYTHON_LOG"] = str(fake_log)
    env["WENDAO_AI_PIDFILE"] = str(tmp_path / "wendao-ai.pid")
    env["WENDAO_AI_PORT"] = "9518"
    env["WENDAO_AI_HOST"] = "127.0.0.1"

    result = subprocess.run(
        ["bash", str(script_path)],
        cwd=project_root,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 0, result.stderr
    calls = fake_log.read_text(encoding="utf-8").splitlines()
    assert any("check_runtime_web_health.py" in line for line in calls)
    assert any("--service-name" in line for line in calls)
    assert any("wendao.ai" in line for line in calls)
