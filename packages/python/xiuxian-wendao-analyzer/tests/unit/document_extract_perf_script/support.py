"""Shared helpers for document_extract_perf_script tests."""

from __future__ import annotations

import importlib.util
import tomllib
from pathlib import Path

import pytest


def _load_benchmark_module() -> object:
    repo_root = Path(__file__).resolve().parents[6]
    script_path = (
        repo_root / "tests" / "scripts" / "benchmark_wendao_document_extract.py"
    )
    spec = importlib.util.spec_from_file_location(
        "benchmark_wendao_document_extract",
        script_path,
    )
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


__all__ = [
    "Path",
    "_load_benchmark_module",
    "importlib",
    "pytest",
    "tomllib",
]
