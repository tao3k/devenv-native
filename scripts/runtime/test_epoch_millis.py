from __future__ import annotations

import importlib.util
import sys
from pathlib import Path


def _load_module():
    script_path = Path(__file__).resolve().with_name("epoch_millis.py")
    module_name = "test_epoch_millis_module"
    spec = importlib.util.spec_from_file_location(module_name, script_path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module


def test_epoch_millis_returns_non_negative_int() -> None:
    module = _load_module()
    value = module.epoch_millis()
    assert isinstance(value, int)
    assert value >= 0


def test_epoch_millis_is_non_decreasing_for_consecutive_calls() -> None:
    module = _load_module()
    first = module.epoch_millis()
    second = module.epoch_millis()
    assert second >= first
