from __future__ import annotations

import importlib.util
import sys
from pathlib import Path


def _load_module():
    script_path = Path(__file__).resolve().with_name("resolve_tool_endpoint.py")
    module_name = "test_resolve_tool_endpoint_module"
    spec = importlib.util.spec_from_file_location(module_name, script_path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module


def test_resolve_tool_endpoint_prefers_namespaced_env(monkeypatch) -> None:
    module = _load_module()
    monkeypatch.setenv(module.NAMESPACED_TOOL_BASE_URL_ENV, "http://198.51.100.10:3902")
    monkeypatch.setenv(module.NAMESPACED_TOOL_PORT_ENV, "3903")
    monkeypatch.setattr(module, "_xiuxian_toml_candidates", lambda: [])

    resolved = module.resolve_tool_endpoint()
    assert resolved["host"] == "198.51.100.10"
    assert resolved["port"] == "3903"
    assert resolved["base_url"] == "http://198.51.100.10:3903"


def test_resolve_tool_endpoint_reads_xiuxian_toml(monkeypatch, tmp_path) -> None:
    module = _load_module()
    xiuxian = tmp_path / "xiuxian.toml"
    xiuxian.write_text(
        '[tool_runtime]\nbase_url = "http://198.51.100.9:3902"\npreferred_embed_port = 3904\n',
        encoding="utf-8",
    )
    monkeypatch.delenv(module.NAMESPACED_TOOL_BASE_URL_ENV, raising=False)
    monkeypatch.delenv(module.NAMESPACED_TOOL_PORT_ENV, raising=False)
    monkeypatch.setattr(module, "_xiuxian_toml_candidates", lambda: [xiuxian])

    resolved = module.resolve_tool_endpoint()
    assert resolved["host"] == "198.51.100.9"
    assert resolved["port"] == "3904"


def test_resolve_tool_endpoint_falls_back_to_defaults(monkeypatch) -> None:
    module = _load_module()
    monkeypatch.delenv(module.NAMESPACED_TOOL_BASE_URL_ENV, raising=False)
    monkeypatch.delenv(module.NAMESPACED_TOOL_PORT_ENV, raising=False)
    monkeypatch.setattr(module, "_xiuxian_toml_candidates", lambda: [])
    resolved = module.resolve_tool_endpoint()
    assert resolved["base_url"] == f"http://{module.DEFAULT_HOST}:3002"


def test_resolve_tool_endpoint_honors_xiuxian_local_host_env(monkeypatch) -> None:
    monkeypatch.setenv("XIUXIAN_WENDAO_LOCAL_HOST", "198.51.100.77")
    module = _load_module()
    monkeypatch.delenv(module.NAMESPACED_TOOL_BASE_URL_ENV, raising=False)
    monkeypatch.delenv(module.NAMESPACED_TOOL_PORT_ENV, raising=False)
    monkeypatch.setattr(module, "_xiuxian_toml_candidates", lambda: [])
    resolved = module.resolve_tool_endpoint()
    assert resolved["host"] == "198.51.100.77"
    assert resolved["base_url"] == "http://198.51.100.77:3002"
