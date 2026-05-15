"""Local backend helper tests."""

from __future__ import annotations

from xiuxian_wendao_analyzer import local_backend
from xiuxian_wendao_analyzer.local_backend import BackendLaunch


def test_env_value_ignores_empty_overrides(monkeypatch) -> None:
    monkeypatch.delenv("WENDAO_TEST_BACKEND_ENV", raising=False)
    assert local_backend.env_value("WENDAO_TEST_BACKEND_ENV", "fallback") == "fallback"

    monkeypatch.setenv("WENDAO_TEST_BACKEND_ENV", "   ")
    assert local_backend.env_value("WENDAO_TEST_BACKEND_ENV", "fallback") == "fallback"

    monkeypatch.setenv("WENDAO_TEST_BACKEND_ENV", "configured")
    assert (
        local_backend.env_value("WENDAO_TEST_BACKEND_ENV", "fallback") == "configured"
    )


def test_project_roots_follow_environment(monkeypatch, tmp_path) -> None:
    data_home = tmp_path / "data-home"
    cache_home = tmp_path / "cache-home"
    monkeypatch.setenv("PRJ_DATA_HOME", str(data_home))
    monkeypatch.setenv("PRJ_CACHE_HOME", str(cache_home))

    assert local_backend.project_data_home() == data_home.resolve()
    assert local_backend.project_cache_home() == cache_home.resolve()


def test_module_path_resolves_beside_anchor(tmp_path) -> None:
    anchor = tmp_path / "wendao" / "backend" / "manager.py"

    assert local_backend.module_path(str(anchor), "adapter.py") == (
        tmp_path / "wendao" / "backend" / "adapter.py"
    )


def test_macos_apple_silicon_probe_uses_platform(monkeypatch) -> None:
    monkeypatch.setattr(local_backend.platform, "system", lambda: "Darwin")
    monkeypatch.setattr(local_backend.platform, "machine", lambda: "arm64")
    assert local_backend.is_macos_apple_silicon()

    monkeypatch.setattr(local_backend.platform, "machine", lambda: "x86_64")
    assert not local_backend.is_macos_apple_silicon()


def test_exec_backend_launch_merges_environment(monkeypatch) -> None:
    captured: dict[str, object] = {}

    def fake_execvpe(
        file: str,
        args: list[str],
        env: object,
    ) -> None:
        captured["file"] = file
        captured["args"] = args
        captured["env"] = dict(env)

    monkeypatch.setenv("KEEP_EXISTING", "yes")
    monkeypatch.setattr(local_backend.os, "execvpe", fake_execvpe)

    result = local_backend.exec_backend_launch(
        BackendLaunch(
            runner="test",
            command=("python", "-m", "wendao_backend"),
            message="start",
            env_updates={"BACKEND_MODE": "test"},
        )
    )

    assert result == 127
    assert captured["file"] == "python"
    assert captured["args"] == ["python", "-m", "wendao_backend"]
    assert captured["env"]["KEEP_EXISTING"] == "yes"
    assert captured["env"]["BACKEND_MODE"] == "test"
