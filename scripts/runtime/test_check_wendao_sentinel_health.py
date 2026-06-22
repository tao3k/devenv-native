from __future__ import annotations

import importlib.util
import sys
from pathlib import Path


def _load_module():
    script_path = Path(__file__).resolve().with_name("check_wendao_sentinel_health.py")
    module_name = "test_check_wendao_sentinel_health_module"
    spec = importlib.util.spec_from_file_location(module_name, script_path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module


def test_resolve_sentinel_watch_paths_reads_link_graph_projects(tmp_path) -> None:
    module = _load_module()
    project_root = tmp_path / "workspace"
    project_root.mkdir()
    (project_root / "docs").mkdir()
    (project_root / "skills").mkdir()
    config_path = project_root / "wendao.toml"
    config_path.write_text(
        """
[sources.projects.main]
root = "."
dirs = ["docs", "skills"]
""".strip()
        + "\n",
        encoding="utf-8",
    )

    watch_paths = module.resolve_sentinel_watch_paths(project_root, config_path)

    assert watch_paths == [project_root / "docs", project_root / "skills"]


def test_is_sentinel_healthy_accepts_live_pid_and_existing_watch_root(tmp_path) -> None:
    module = _load_module()
    project_root = tmp_path / "workspace"
    project_root.mkdir()
    (project_root / "docs").mkdir()
    config_path = project_root / "wendao.toml"
    config_path.write_text(
        """
[sources.projects.main]
root = "."
dirs = ["docs"]
""".strip()
        + "\n",
        encoding="utf-8",
    )
    pidfile = tmp_path / "wendao-sentinel.pid"
    pidfile.write_text("4123\n", encoding="utf-8")

    healthy, message = module.is_sentinel_healthy(
        project_root=project_root,
        config_path=config_path,
        pidfile=pidfile,
        pid_checker=lambda pid: pid == 4123,
    )

    assert healthy is True
    assert message == "healthy"


def test_is_sentinel_healthy_rejects_dead_pid(tmp_path) -> None:
    module = _load_module()
    project_root = tmp_path / "workspace"
    project_root.mkdir()
    (project_root / "docs").mkdir()
    config_path = project_root / "wendao.toml"
    config_path.write_text(
        """
[sources.projects.main]
root = "."
dirs = ["docs"]
""".strip()
        + "\n",
        encoding="utf-8",
    )
    pidfile = tmp_path / "wendao-sentinel.pid"
    pidfile.write_text("4123\n", encoding="utf-8")

    healthy, message = module.is_sentinel_healthy(
        project_root=project_root,
        config_path=config_path,
        pidfile=pidfile,
        pid_checker=lambda _pid: False,
    )

    assert healthy is False
    assert "not alive" in message


def test_is_sentinel_healthy_rejects_missing_watch_roots(tmp_path) -> None:
    module = _load_module()
    project_root = tmp_path / "workspace"
    project_root.mkdir()
    config_path = project_root / "wendao.toml"
    config_path.write_text(
        """
[sources.projects.main]
root = "."
dirs = ["docs"]
""".strip()
        + "\n",
        encoding="utf-8",
    )
    pidfile = tmp_path / "wendao-sentinel.pid"
    pidfile.write_text("4123\n", encoding="utf-8")

    healthy, message = module.is_sentinel_healthy(
        project_root=project_root,
        config_path=config_path,
        pidfile=pidfile,
        pid_checker=lambda _pid: True,
    )

    assert healthy is False
    assert "no existing watch roots" in message
