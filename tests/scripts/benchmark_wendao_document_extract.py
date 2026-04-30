#!/usr/bin/env python3
"""Benchmark Wendao document extraction across Python Flight and Rust tests."""

from __future__ import annotations

import argparse
import importlib
import sys
from pathlib import Path
from typing import Any

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

_args = importlib.import_module("wendao_document_extract_benchmark.args")
_artifact_summary = importlib.import_module(
    "wendao_document_extract_benchmark.artifact_summary"
)
_cache = importlib.import_module("wendao_document_extract_benchmark.cache")
_constants = importlib.import_module("wendao_document_extract_benchmark.constants")
_fake_fixtures = importlib.import_module(
    "wendao_document_extract_benchmark.fake_fixtures"
)
_features = importlib.import_module("wendao_document_extract_benchmark.features")
_fixtures = importlib.import_module("wendao_document_extract_benchmark.fixtures")
_http_status = importlib.import_module("wendao_document_extract_benchmark.http_status")
_pdf_render = importlib.import_module("wendao_document_extract_benchmark.pdf_render")
_pdfium = importlib.import_module("wendao_document_extract_benchmark.pdfium")
_probes = importlib.import_module("wendao_document_extract_benchmark.probes")
_providers = importlib.import_module("wendao_document_extract_benchmark.providers")
_reporting = importlib.import_module("wendao_document_extract_benchmark.reporting")
_rust_status = importlib.import_module("wendao_document_extract_benchmark.rust_status")
_workers = importlib.import_module("wendao_document_extract_benchmark.workers")
main = importlib.import_module("wendao_document_extract_benchmark.cli").main

run_cargo_perf_test = _probes.run_cargo_perf_test

_EXPORTED_MODULES: tuple[Any, ...] = (
    _constants,
    _args,
    _fixtures,
    _fake_fixtures,
    _pdf_render,
    _pdfium,
    _features,
    _workers,
    _providers,
    _http_status,
    _probes,
    _artifact_summary,
    _rust_status,
    _cache,
    _reporting,
)


def run_distinct_miss_probe(*args: Any, **kwargs: Any) -> Any:
    return _run_probe_with_overrides(_probes.run_distinct_miss_probe, *args, **kwargs)


def run_structure_baseline_probe(*args: Any, **kwargs: Any) -> Any:
    return _run_probe_with_overrides(
        _probes.run_structure_baseline_probe,
        *args,
        **kwargs,
    )


def run_fixture_probe(*args: Any, **kwargs: Any) -> Any:
    return _run_probe_with_overrides(_probes.run_fixture_probe, *args, **kwargs)


def _run_probe_with_overrides(callback: Any, *args: Any, **kwargs: Any) -> Any:
    previous_run_cargo_perf_test = _probes.run_cargo_perf_test
    _probes.run_cargo_perf_test = run_cargo_perf_test
    try:
        return callback(*args, **kwargs)
    finally:
        _probes.run_cargo_perf_test = previous_run_cargo_perf_test


def __getattr__(name: str) -> Any:
    if name == "argparse":
        return argparse
    for module in _EXPORTED_MODULES:
        if hasattr(module, name):
            return getattr(module, name)
    raise AttributeError(name)


if __name__ == "__main__":
    raise SystemExit(main())
