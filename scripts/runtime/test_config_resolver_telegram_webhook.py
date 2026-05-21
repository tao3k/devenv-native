from __future__ import annotations

import importlib
import sys
from pathlib import Path

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))


def _module():
    return importlib.import_module("config_resolver_telegram_webhook")


def test_telegram_webhook_bind_prefers_namespaced_env(monkeypatch) -> None:
    mod = _module()
    monkeypatch.setenv(mod.LEGACY_WEBHOOK_BIND_ENV, "legacy.example:1111")
    monkeypatch.setenv(mod.NAMESPACED_WEBHOOK_BIND_ENV, "namespaced.example:2222")

    assert mod.telegram_webhook_bind() == "namespaced.example:2222"


def test_telegram_webhook_port_prefers_namespaced_env(monkeypatch) -> None:
    mod = _module()
    monkeypatch.setenv(mod.LEGACY_WEBHOOK_PORT_ENV, "18081")
    monkeypatch.setenv(mod.NAMESPACED_WEBHOOK_PORT_ENV, "28081")

    assert mod.telegram_webhook_port() == 28081


def test_telegram_webhook_port_uses_legacy_env_when_namespaced_absent(
    monkeypatch,
) -> None:
    mod = _module()
    monkeypatch.delenv(mod.NAMESPACED_WEBHOOK_PORT_ENV, raising=False)
    monkeypatch.setenv(mod.LEGACY_WEBHOOK_PORT_ENV, "19090")

    assert mod.telegram_webhook_port() == 19090
