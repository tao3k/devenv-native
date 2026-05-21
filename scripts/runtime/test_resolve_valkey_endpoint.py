from __future__ import annotations

import importlib.util
import sys
from pathlib import Path


def _load_module():
    script_path = Path(__file__).resolve().with_name("resolve_valkey_endpoint.py")
    module_name = "test_resolve_valkey_endpoint_module"
    spec = importlib.util.spec_from_file_location(module_name, script_path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module


def test_resolve_valkey_endpoint_prefers_namespaced_env(monkeypatch) -> None:
    module = _load_module()
    monkeypatch.setenv(module.NAMESPACED_VALKEY_URL_ENV, "redis://198.51.100.30:6381/6")
    monkeypatch.setattr(module, "_xiuxian_toml_candidates", lambda: [])

    resolved = module.resolve_valkey_endpoint()
    assert resolved["host"] == "198.51.100.30"
    assert resolved["port"] == "6381"
    assert resolved["db"] == "6"


def test_resolve_valkey_endpoint_prefers_xiuxian_toml(monkeypatch, tmp_path) -> None:
    module = _load_module()
    xiuxian = tmp_path / "xiuxian.toml"
    xiuxian.write_text(
        '[wendao.link_graph.cache]\nvalkey_url = "redis://198.51.100.20:6385/7"\n',
        encoding="utf-8",
    )
    monkeypatch.delenv(module.NAMESPACED_VALKEY_URL_ENV, raising=False)
    monkeypatch.setattr(module, "_xiuxian_toml_candidates", lambda: [xiuxian])

    resolved = module.resolve_valkey_endpoint()
    assert resolved["host"] == "198.51.100.20"
    assert resolved["port"] == "6385"
    assert resolved["db"] == "7"


def test_resolve_valkey_endpoint_ignores_legacy_valkey_env(monkeypatch) -> None:
    module = _load_module()
    monkeypatch.delenv(module.NAMESPACED_VALKEY_URL_ENV, raising=False)
    monkeypatch.setattr(module, "_xiuxian_toml_candidates", lambda: [])
    monkeypatch.setenv("VALKEY_URL", f"redis://{module.DEFAULT_HOST}:6389/4")

    resolved = module.resolve_valkey_endpoint()
    assert resolved["url"] == f"redis://{module.DEFAULT_HOST}:6379/0"


def test_resolve_valkey_endpoint_has_stable_default(monkeypatch) -> None:
    module = _load_module()
    monkeypatch.delenv(module.NAMESPACED_VALKEY_URL_ENV, raising=False)
    monkeypatch.setattr(module, "_xiuxian_toml_candidates", lambda: [])

    resolved = module.resolve_valkey_endpoint()
    assert resolved["url"] == f"redis://{module.DEFAULT_HOST}:6379/0"


def test_resolve_valkey_endpoint_honors_xiuxian_local_host_env(monkeypatch) -> None:
    monkeypatch.setenv("XIUXIAN_WENDAO_LOCAL_HOST", "198.51.100.88")
    module = _load_module()
    monkeypatch.delenv(module.NAMESPACED_VALKEY_URL_ENV, raising=False)
    monkeypatch.setattr(module, "_xiuxian_toml_candidates", lambda: [])

    resolved = module.resolve_valkey_endpoint()
    assert resolved["host"] == "198.51.100.88"
    assert resolved["url"] == "redis://198.51.100.88:6379/0"
