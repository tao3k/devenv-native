"""Shared helpers for audio ASR diagnostic tests."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from types import ModuleType


def _package_root() -> Path:
    return Path(__file__).resolve().parents[3]


def _load_script(script_name: str) -> ModuleType:
    script_path = _package_root() / "tests" / "scripts" / script_name
    spec = importlib.util.spec_from_file_location(
        script_name.removesuffix(".py"), script_path
    )
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def _load_audio_asr_diagnostic() -> ModuleType:
    return _load_script("audio_asr_diagnostic.py")


def _load_fireredasr2s_local_setup() -> ModuleType:
    return _load_script("fireredasr2s_local_setup.py")


__all__ = [
    "Path",
    "_load_audio_asr_diagnostic",
    "_load_fireredasr2s_local_setup",
]
