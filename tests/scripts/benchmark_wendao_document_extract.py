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
_artifact_summary = importlib.import_module("wendao_document_extract_benchmark.artifact_summary")
_audio_trace = importlib.import_module("wendao_document_extract_benchmark.audio_trace")
_audio_transcript_org = importlib.import_module(
    "wendao_document_extract_benchmark.audio_transcript_org"
)
_attachment_classes = importlib.import_module(
    "wendao_document_extract_benchmark.attachment_classes"
)
_cache = importlib.import_module("wendao_document_extract_benchmark.cache")
_constants = importlib.import_module("wendao_document_extract_benchmark.constants")
_docling_groundtruth = importlib.import_module("xiuxian_wendao_analyzer.docling_groundtruth")
_fake_fixtures = importlib.import_module("wendao_document_extract_benchmark.fake_fixtures")
_features = importlib.import_module("wendao_document_extract_benchmark.features")
_fixtures = importlib.import_module("wendao_document_extract_benchmark.fixtures")
_http_status = importlib.import_module("wendao_document_extract_benchmark.http_status")
_ocr2_trace = importlib.import_module("wendao_document_extract_benchmark.ocr2_trace")
_pdf_render = importlib.import_module("wendao_document_extract_benchmark.pdf_render")
_pdfium = importlib.import_module("wendao_document_extract_benchmark.pdfium")
_precision_speed = importlib.import_module("wendao_document_extract_benchmark.precision_speed")
_probes = importlib.import_module("wendao_document_extract_benchmark.probes")
_provider_lifecycle = importlib.import_module(
    "wendao_document_extract_benchmark.provider_lifecycle"
)
_providers = importlib.import_module("wendao_document_extract_benchmark.providers")
_reporting = importlib.import_module("wendao_document_extract_benchmark.reporting")
_rust_status = importlib.import_module("wendao_document_extract_benchmark.rust_status")
_structure_consistency = importlib.import_module(
    "wendao_document_extract_benchmark.structure_consistency"
)
_workers = importlib.import_module("wendao_document_extract_benchmark.workers")
_cli = importlib.import_module("wendao_document_extract_benchmark.cli")
main = _cli.main

run_cargo_perf_test = _probes.run_cargo_perf_test

_EXPORTED_MODULES: tuple[Any, ...] = (
    _constants,
    _docling_groundtruth,
    _args,
    _audio_trace,
    _audio_transcript_org,
    _fixtures,
    _fake_fixtures,
    _attachment_classes,
    _pdf_render,
    _pdfium,
    _precision_speed,
    _features,
    _ocr2_trace,
    _workers,
    _provider_lifecycle,
    _providers,
    _http_status,
    _probes,
    _artifact_summary,
    _rust_status,
    _structure_consistency,
    _cache,
    _reporting,
    _cli,
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
