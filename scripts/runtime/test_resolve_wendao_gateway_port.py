from __future__ import annotations

import importlib.util
import sys
from pathlib import Path


def _load_module():
    script_path = Path(__file__).resolve().with_name("resolve_wendao_gateway_port.py")
    module_name = "test_resolve_wendao_gateway_port_module"
    spec = importlib.util.spec_from_file_location(module_name, script_path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module


def test_resolve_gateway_port_reads_root_config_port(tmp_path) -> None:
    module = _load_module()
    config_path = tmp_path / "wendao.toml"
    config_path.write_text("[gateway]\nport = 9611\n", encoding="utf-8")

    assert module.resolve_gateway_port(config_path) == 9611


def test_resolve_gateway_port_prefers_studio_overlay_when_present(tmp_path) -> None:
    module = _load_module()
    base_path = tmp_path / "wendao.toml"
    overlay_path = tmp_path / "wendao.studio.overlay.toml"
    base_path.write_text("[gateway]\nport = 9517\n", encoding="utf-8")
    overlay_path.write_text(
        'imports = ["wendao.toml"]\n[gateway]\nport = 9620\n',
        encoding="utf-8",
    )

    assert module.resolve_gateway_port(base_path) == 9620


def test_resolve_gateway_port_supports_env_expanded_import_paths(
    monkeypatch, tmp_path
) -> None:
    module = _load_module()
    defaults_path = tmp_path / "defaults.toml"
    config_path = tmp_path / "wendao.toml"
    defaults_path.write_text("[gateway]\nport = 9633\n", encoding="utf-8")
    monkeypatch.setenv("PRJ_ROOT", str(tmp_path))
    config_path.write_text(
        'imports = ["${PRJ_ROOT}/defaults.toml"]\n', encoding="utf-8"
    )

    assert module.resolve_gateway_port(config_path) == 9633


def test_resolve_gateway_port_falls_back_to_bind(tmp_path) -> None:
    module = _load_module()
    config_path = tmp_path / "wendao.toml"
    config_path.write_text('[gateway]\nbind = "127.0.0.1:9644"\n', encoding="utf-8")

    assert module.resolve_gateway_port(config_path) == 9644
