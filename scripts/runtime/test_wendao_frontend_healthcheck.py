from __future__ import annotations

import os
import subprocess
from pathlib import Path


def test_wendao_frontend_healthcheck_shell_prefers_pyo3_python(tmp_path: Path) -> None:
    project_root = Path(__file__).resolve().parents[2]
    script_path = Path(__file__).resolve().with_name("wendao-frontend-healthcheck.sh")
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
    env["WENDAO_FRONTEND_PIDFILE"] = str(tmp_path / "wendao-frontend.pid")
    env["WENDAO_FRONTEND_PORT"] = "9518"
    env["WENDAO_FRONTEND_HOST"] = "127.0.0.1"

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
    assert any("check_wendao_frontend_health.py" in line for line in calls)
    assert any("--host" in line for line in calls)
    assert any("--port" in line for line in calls)
    assert any("--pidfile" in line for line in calls)
