#!/usr/bin/env python3
"""Shared helpers for loading sibling modules in standalone channel scripts."""

from __future__ import annotations

import importlib
import importlib.util
import sys
from pathlib import Path


def load_sibling_module(
    *,
    module_name: str,
    file_name: str,
    caller_file: str,
    error_context: str,
) -> object:
    """Import `module_name`, or load it from the caller's sibling file.

    This keeps script-style execution and importlib-based test loading consistent.
    """
    try:
        return importlib.import_module(module_name)
    except ModuleNotFoundError as import_error:
        module_path = Path(caller_file).resolve().with_name(file_name)
        spec = importlib.util.spec_from_file_location(module_name, module_path)
        if spec is None or spec.loader is None:
            raise RuntimeError(
                f"failed to load {error_context} from {module_path}"
            ) from import_error

        loaded = sys.modules.get(spec.name)
        if loaded is not None:
            return loaded

        module = importlib.util.module_from_spec(spec)
        sys.modules[spec.name] = module
        spec.loader.exec_module(module)
        return module
