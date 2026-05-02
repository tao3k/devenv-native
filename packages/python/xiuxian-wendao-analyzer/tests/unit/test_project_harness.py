"""Project-level Python harness gate for xiuxian-wendao-analyzer."""

from __future__ import annotations

from pathlib import Path

from python_lang_project_harness import (
    PythonHarnessFinding,
    run_python_lang_harness,
    run_python_project_harness,
)

_BASELINED_BLOCKING_FINDINGS = frozenset(
    {
        ("PY-MOD-R006", "src/xiuxian_wendao_analyzer/document_service.py"),
        ("PY-MOD-R006", "src/xiuxian_wendao_analyzer/documents.py"),
        ("PY-MOD-R006", "src/xiuxian_wendao_analyzer/pdf_ocr.py"),
        ("PY-MOD-R006", "src/xiuxian_wendao_analyzer/runtime.py"),
        ("PY-MOD-R006", "tests/unit/test_document_service.py"),
        ("PY-MOD-R006", "tests/unit/test_documents.py"),
        ("PY-MOD-R006", "tests/unit/test_transport_runtime.py"),
        ("PY-MOD-R002", "examples/attachment_pdf_analyzer_workflow.py"),
        ("PY-MOD-R002", "examples/custom_repo_analyzer_workflow.py"),
        ("PY-MOD-R002", "examples/document_extraction_workflow.py"),
        ("PY-MOD-R002", "examples/host_backed_repo_search_beta_smoke.py"),
        ("PY-MOD-R002", "examples/repo_search_workflow.py"),
        ("PY-MOD-R002", "examples/scripted_repo_search_workflow.py"),
        ("PY-TEST-R003", "tests/unit/test_document_extract_perf_script.py"),
        ("PY-TEST-R003", "tests/unit/test_document_service.py"),
        ("PY-TEST-R003", "tests/unit/test_documents.py"),
        ("PY-TEST-R003", "tests/unit/test_examples.py"),
        ("PY-TEST-R003", "tests/unit/test_transport_runtime.py"),
    }
)

_BASELINED_BENCHMARK_BLOCKING_FINDINGS = frozenset(
    {
        (
            "PY-MOD-R006",
            "tests/scripts/wendao_document_extract_benchmark/artifact_summary.py",
        ),
        (
            "PY-MOD-R006",
            "tests/scripts/wendao_document_extract_benchmark/attachment_classes.py",
        ),
        ("PY-MOD-R006", "tests/scripts/wendao_document_extract_benchmark/reporting.py"),
    }
)


def test_python_project_harness_blocks_unbaselined_findings() -> None:
    package_root = Path(__file__).resolve().parents[2]
    report = run_python_project_harness(package_root)

    current = {
        _finding_key(package_root, finding) for finding in report.blocking_findings()
    }
    unexpected = current - _BASELINED_BLOCKING_FINDINGS
    retired = _BASELINED_BLOCKING_FINDINGS - current

    assert not unexpected, _render_finding_set(
        "unexpected Python project harness findings",
        unexpected,
    )
    assert not retired, _render_finding_set(
        "retired Python project harness baseline entries",
        retired,
    )


def test_benchmark_script_harness_blocks_unbaselined_findings() -> None:
    package_root = Path(__file__).resolve().parents[2]
    repo_root = package_root.parents[2]
    benchmark_root = repo_root / "tests/scripts"
    report = run_python_lang_harness(
        [
            benchmark_root / "benchmark_wendao_document_extract.py",
            benchmark_root / "wendao_document_extract_benchmark",
        ]
    )

    current = {
        _finding_key(repo_root, finding) for finding in report.blocking_findings()
    }
    unexpected = current - _BASELINED_BENCHMARK_BLOCKING_FINDINGS
    retired = _BASELINED_BENCHMARK_BLOCKING_FINDINGS - current

    assert not unexpected, _render_finding_set(
        "unexpected benchmark script harness findings",
        unexpected,
    )
    assert not retired, _render_finding_set(
        "retired benchmark script harness baseline entries",
        retired,
    )


def _finding_key(
    package_root: Path,
    finding: PythonHarnessFinding,
) -> tuple[str, str]:
    path = Path(finding.location.path or "")
    try:
        relative_path = path.relative_to(package_root)
    except ValueError:
        relative_path = path
    return finding.rule_id, relative_path.as_posix()


def _render_finding_set(
    title: str,
    findings: set[tuple[str, str]] | frozenset[tuple[str, str]],
) -> str:
    rendered = "\n".join(f"- {rule_id}: {path}" for rule_id, path in sorted(findings))
    return f"{title}:\n{rendered}"
