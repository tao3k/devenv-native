from __future__ import annotations

import importlib.util
import sys
from pathlib import Path


def _load_module():
    script_path = (
        Path(__file__)
        .resolve()
        .with_name("resolve_wendao_document_extract_endpoint.py")
    )
    module_name = "test_resolve_wendao_document_extract_endpoint_module"
    spec = importlib.util.spec_from_file_location(module_name, script_path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module


def test_resolve_document_extract_endpoint_reads_root_config(tmp_path) -> None:
    module = _load_module()
    config_path = tmp_path / "wendao.toml"
    config_path.write_text(
        '[document_extract]\nendpoint = "http://127.0.0.1:56051/"\n',
        encoding="utf-8",
    )

    assert (
        module.resolve_document_extract_endpoint(config_path)
        == "http://127.0.0.1:56051"
    )
    assert (
        module.document_extract_endpoint_host("http://127.0.0.1:56051") == "127.0.0.1"
    )
    assert module.document_extract_endpoint_port("http://127.0.0.1:56051") == 56051


def test_resolve_document_extract_endpoint_prefers_overlay_importing_base(
    tmp_path,
) -> None:
    module = _load_module()
    base_path = tmp_path / "wendao.toml"
    overlay_path = tmp_path / "wendao.studio.overlay.toml"
    base_path.write_text(
        '[document_extract]\nendpoint = "http://127.0.0.1:50051"\n',
        encoding="utf-8",
    )
    overlay_path.write_text(
        'imports = ["wendao.toml"]\n[document_extract]\nendpoint = "http://127.0.0.1:56052"\n',
        encoding="utf-8",
    )

    assert (
        module.resolve_document_extract_endpoint(base_path) == "http://127.0.0.1:56052"
    )


def test_resolve_document_extract_endpoint_uses_default_when_absent(tmp_path) -> None:
    module = _load_module()
    config_path = tmp_path / "wendao.toml"
    config_path.write_text("[gateway]\nport = 9517\n", encoding="utf-8")

    assert (
        module.resolve_document_extract_endpoint(config_path) == module.DEFAULT_ENDPOINT
    )
