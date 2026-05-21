from __future__ import annotations

import importlib.util
import sys
from pathlib import Path


def _load_module():
    script_path = Path(__file__).resolve().with_name("read_telegram_setting.py")
    module_name = "test_read_telegram_setting_module"
    spec = importlib.util.spec_from_file_location(module_name, script_path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module


def test_read_telegram_setting_returns_trimmed_string_from_config(
    tmp_path, monkeypatch
) -> None:
    module = _load_module()
    xiuxian = tmp_path / "xiuxian.toml"
    xiuxian.write_text(
        '[telegram]\nwebhook_secret_token = "  my-secret  "\n',
        encoding="utf-8",
    )
    monkeypatch.setattr(module, "repo_root_from", lambda _path: tmp_path)
    monkeypatch.setattr(module, "settings_candidates", lambda _root: [xiuxian])
    assert module.read_telegram_setting("webhook_secret_token") == "my-secret"


def test_read_telegram_setting_returns_empty_when_none(monkeypatch) -> None:
    module = _load_module()
    monkeypatch.setattr(module, "repo_root_from", lambda _path: Path("."))
    monkeypatch.setattr(module, "settings_candidates", lambda _root: [])
    assert module.read_telegram_setting("webhook_bind") == ""


def test_read_telegram_setting_prefers_xiuxian_candidate(tmp_path, monkeypatch) -> None:
    module = _load_module()
    xiuxian = tmp_path / "xiuxian.toml"
    xiuxian.write_text(
        '[telegram]\nwebhook_secret_token = "from-xiuxian"\n',
        encoding="utf-8",
    )

    monkeypatch.setattr(module, "repo_root_from", lambda _path: tmp_path)
    monkeypatch.setattr(module, "settings_candidates", lambda _root: [xiuxian])
    assert module.read_telegram_setting("webhook_secret_token") == "from-xiuxian"
