"""Compact snapshot rendering for Python harness diagnostics."""

from __future__ import annotations

from typing import TYPE_CHECKING

from python_lang_parser import PythonDiagnosticSeverity

if TYPE_CHECKING:
    from ._model import PythonHarnessFinding, PythonHarnessReport


def render_python_lang_harness(
    report: PythonHarnessReport,
    *,
    severities: frozenset[PythonDiagnosticSeverity] | None = None,
) -> str:
    """Render a compact diagnostic report for humans and repair workflows."""

    blocking_findings = report.blocking_findings(severities=severities)
    rendered = _render_header(report, blocking_findings=blocking_findings)
    if not blocking_findings:
        return rendered

    for finding in blocking_findings:
        rendered += "\n" + _render_finding(finding)
    return rendered


def _render_header(
    report: PythonHarnessReport,
    *,
    blocking_findings: tuple[PythonHarnessFinding, ...],
) -> str:
    target = ", ".join(report.root_paths)
    if not blocking_findings:
        return f"[ok] {target} python\nSource: {target}\nNo blocking issues found.\n"
    status = _render_findings_status(blocking_findings)
    return f"[lint:{status}] {target} python\nSource: {target}\nIssues: {len(blocking_findings)}\n"


def _render_finding(finding: PythonHarnessFinding) -> str:
    path = finding.location.path or "<memory>"
    line = finding.location.line
    column = finding.location.column
    severity = finding.severity.value.title()
    display_column = column + 1
    rendered = (
        f"[{finding.rule_id}] {severity}: {finding.title}\n"
        f"   ,-[ {path}:{line}:{display_column} ]\n"
    )
    if finding.source_line:
        pointer_column = max(column, 0)
        rendered += f"{line:>2} | {finding.source_line}\n   | {' ' * pointer_column}`- {finding.label}\n"
    else:
        rendered += f"   | {finding.label}\n"
    rendered += f"   |Required: {finding.requirement}\n"
    return rendered


def _render_findings_status(findings: tuple[PythonHarnessFinding, ...]) -> str:
    if any(finding.severity == PythonDiagnosticSeverity.ERROR for finding in findings):
        return "error"
    if any(
        finding.severity == PythonDiagnosticSeverity.WARNING for finding in findings
    ):
        return "warning"
    return "info"
