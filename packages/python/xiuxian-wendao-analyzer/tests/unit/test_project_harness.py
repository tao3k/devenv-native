"""Project-level Python harness gate for xiuxian-wendao-analyzer."""

from __future__ import annotations

from pathlib import Path

from python_lang_project_harness import (
    PythonHarnessFinding,
    run_python_lang_harness,
    run_python_project_harness,
)


def test_python_project_harness_blocks_all_default_findings() -> None:
    package_root = Path(__file__).resolve().parents[2]
    report = run_python_project_harness(package_root)

    assert not report.findings, _render_finding_set(
        package_root,
        "Python project harness findings",
        report.findings,
    )


def test_benchmark_script_harness_blocks_all_default_findings() -> None:
    package_root = Path(__file__).resolve().parents[2]
    repo_root = package_root.parents[2]
    benchmark_root = repo_root / "tests/scripts"
    report = run_python_lang_harness(
        [
            benchmark_root / "benchmark_wendao_document_extract.py",
            benchmark_root / "wendao_document_extract_benchmark",
        ]
    )

    assert not report.findings, _render_finding_set(
        repo_root,
        "benchmark script harness findings",
        report.findings,
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
    root: Path,
    title: str,
    findings: tuple[PythonHarnessFinding, ...],
) -> str:
    rendered = "\n".join(
        f"- {rule_id}: {path}"
        for rule_id, path in sorted(_finding_key(root, finding) for finding in findings)
    )
    return f"{title}:\n{rendered}"
