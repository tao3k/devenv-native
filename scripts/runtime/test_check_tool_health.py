from __future__ import annotations

import importlib.util
import json
import os
import sys
import urllib.error
from pathlib import Path


class _FakeResponse:
    def __init__(self, status: int, payload: dict[str, object]) -> None:
        self.status = status
        self._payload_bytes = json.dumps(payload, ensure_ascii=True).encode("utf-8")

    def read(self) -> bytes:
        return self._payload_bytes

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb) -> bool:
        return False


def _load_module():
    script_path = Path(__file__).resolve().with_name("check_tool_health.py")
    module_name = "test_check_tool_health_module"
    spec = importlib.util.spec_from_file_location(module_name, script_path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module


def _local_host() -> str:
    return (
        os.environ.get("XIUXIAN_WENDAO_LOCAL_HOST", "localhost").strip() or "localhost"
    )


def test_is_tool_healthy_returns_true_for_status_healthy() -> None:
    module = _load_module()

    def _fake_open(_url: str, timeout: float):
        assert timeout == 2.0
        return _FakeResponse(200, {"status": "healthy"})

    assert module.is_tool_healthy(_local_host(), 18501, 2.0, opener=_fake_open) is True


def test_is_tool_healthy_returns_false_for_non_200_response() -> None:
    module = _load_module()

    def _fake_open(_url: str, timeout: float):
        assert timeout == 2.0
        return _FakeResponse(503, {"status": "healthy"})

    assert module.is_tool_healthy(_local_host(), 18501, 2.0, opener=_fake_open) is False


def test_is_tool_healthy_returns_false_on_http_error() -> None:
    module = _load_module()

    def _fake_open(_url: str, timeout: float):
        raise urllib.error.URLError("boom")

    assert module.is_tool_healthy(_local_host(), 18501, 2.0, opener=_fake_open) is False
