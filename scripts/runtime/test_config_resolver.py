#!/usr/bin/env python3
"""
Compatibility shim for legacy imports.

Preferred module path:
  scripts/runtime/config_resolver.py
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path


def _load_impl():
    impl_path = Path(__file__).resolve().with_name("config_resolver.py")
    spec = importlib.util.spec_from_file_location("config_resolver", impl_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"failed to load config resolver module from {impl_path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules.setdefault(spec.name, module)
    spec.loader.exec_module(module)
    return module


_impl = _load_impl()

# Re-export all non-dunder names for backward compatibility.
for _name in dir(_impl):
    if _name.startswith("__") and _name.endswith("__"):
        continue
    globals()[_name] = getattr(_impl, _name)

__all__ = [
    name for name in dir(_impl) if not (name.startswith("__") and name.endswith("__"))
]
