"""Python harness gate for wendao-knowledge-retrieval-benchmark."""

from __future__ import annotations

from pathlib import Path

from python_lang_project_harness import (
    PythonDiagnosticSeverity,
    PythonHarnessConfig,
    python_project_harness_test,
    run_python_project_harness,
)

ERROR_ONLY_HARNESS_CONFIG = PythonHarnessConfig(
    blocking_severities=frozenset({PythonDiagnosticSeverity.ERROR})
)

test_python_project_harness_policy = python_project_harness_test(
    Path(__file__).resolve().parents[1],
)


def test_python_project_harness_blocks_no_error_findings() -> None:
    package_root = Path(__file__).resolve().parents[1]
    report = run_python_project_harness(
        package_root,
        config=ERROR_ONLY_HARNESS_CONFIG,
    )

    assert not report.blocking_findings()
