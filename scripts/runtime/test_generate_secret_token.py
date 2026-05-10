from __future__ import annotations

import importlib.util
import re
import sys
from pathlib import Path


def _load_module():
    script_path = Path(__file__).resolve().with_name("generate_secret_token.py")
    module_name = "test_generate_secret_token_module"
    spec = importlib.util.spec_from_file_location(module_name, script_path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module


def test_generate_secret_token_is_non_empty_and_urlsafe() -> None:
    module = _load_module()
    token = module.generate_secret_token(32)
    assert token
    assert re.fullmatch(r"[A-Za-z0-9_-]+", token) is not None


def test_generate_secret_token_clamps_non_positive_length() -> None:
    module = _load_module()
    token = module.generate_secret_token(0)
    assert token
