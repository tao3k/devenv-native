from __future__ import annotations

import importlib.util
import sys
from pathlib import Path


def _load_module():
    script_path = Path(__file__).resolve().with_name("extract_ngrok_public_url.py")
    module_name = "test_extract_ngrok_public_url_module"
    spec = importlib.util.spec_from_file_location(module_name, script_path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module


def test_extract_public_url_returns_first_tunnel_url() -> None:
    module = _load_module()
    payload = {
        "tunnels": [
            {"public_url": "https://abc.ngrok-free.app"},
            {"public_url": "https://def.ngrok-free.app"},
        ]
    }
    assert module.extract_public_url(payload) == "https://abc.ngrok-free.app"


def test_extract_public_url_returns_empty_on_invalid_payload() -> None:
    module = _load_module()
    assert module.extract_public_url({"tunnels": "bad"}) == ""
    assert module.extract_public_url({"tunnels": [{"foo": "bar"}]}) == ""
