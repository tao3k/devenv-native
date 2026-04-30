from __future__ import annotations

from typing import TYPE_CHECKING

from python_lang_parser import PythonDiagnosticSeverity
from xiuxian_harness_python_lang_project import (
    PythonAgentPolicyRulePack,
    python_agent_policy_rules,
    render_python_lang_harness,
    run_python_lang_harness,
    run_python_project_harness,
)

if TYPE_CHECKING:
    from pathlib import Path


def test_agent_policy_reports_compact_repairable_snapshot(tmp_path: Path) -> None:
    source = tmp_path / "service.py"
    source.write_text("def build(value):\n    return value\n", encoding="utf-8")

    output = render_python_lang_harness(run_python_lang_harness([source]))
    output = output.replace(str(source), "$TMP/service.py")

    assert (
        output
        == """[lint:warning] $TMP/service.py python
Source: $TMP/service.py
Issues: 2

[PY-AGENT-R001] Warning: Library module lacks a module intent docstring
   ,-[ $TMP/service.py:1:1 ]
 1 | def build(value):
   | `- add a concise module responsibility docstring
   |Required: Add a concise module docstring that names the module responsibility for agent search and repair.

[PY-AGENT-R002] Warning: Public callable lacks type annotations
   ,-[ $TMP/service.py:1:1 ]
 1 | def build(value):
   | `- add parameter and return annotations to this public callable
   |Required: Annotate public function and method boundaries so agents can reason from native syntax without guessing shapes.
"""
    )


def test_agent_policy_accepts_documented_annotated_public_callable(
    tmp_path: Path,
) -> None:
    source = tmp_path / "service.py"
    source.write_text(
        '"""Service helpers for tests."""\n\n\ndef build(value: int) -> int:\n    return value\n',
        encoding="utf-8",
    )

    report = run_python_lang_harness([source])

    assert report.is_clean


def test_agent_policy_skips_test_modules(tmp_path: Path) -> None:
    tests = tmp_path / "tests"
    tests.mkdir()
    source = tests / "test_service.py"
    source.write_text("def test_value():\n    assert True\n", encoding="utf-8")

    report = run_python_lang_harness([tmp_path])

    assert report.is_clean


def test_agent_policy_blocks_duplicate_public_callable_names(
    tmp_path: Path,
) -> None:
    package = tmp_path / "src" / "pkg"
    package.mkdir(parents=True)
    (package / "alpha.py").write_text(
        '"""Alpha namespace."""\n\n\ndef build(value: int) -> int:\n    return value\n',
        encoding="utf-8",
    )
    (package / "beta.py").write_text(
        '"""Beta namespace."""\n\n\ndef build(value: int) -> int:\n    return value\n',
        encoding="utf-8",
    )

    report = run_python_project_harness(tmp_path)

    assert [
        (finding.rule_id, finding.location.path) for finding in report.findings
    ] == [
        ("PY-AGENT-R003", str(package / "beta.py")),
    ]
    assert "unambiguous names" in report.findings[0].requirement
    assert "namespace this public callable" in report.findings[0].label


def test_agent_policy_blocks_repeated_module_namespace_segments(
    tmp_path: Path,
) -> None:
    namespace = tmp_path / "src" / "domain" / "domain"
    namespace.mkdir(parents=True)
    source = namespace / "service.py"
    source.write_text(
        '"""Domain service namespace."""\n\nVALUE = 1\n', encoding="utf-8"
    )

    report = run_python_project_harness(tmp_path)

    assert [
        (finding.rule_id, finding.location.path) for finding in report.findings
    ] == [
        ("PY-AGENT-R004", str(source)),
    ]
    assert "branch-unique" in report.findings[0].requirement
    assert "rename one repeated namespace segment" in report.findings[0].label


def test_agent_policy_deduplicates_repeated_namespace_branches(
    tmp_path: Path,
) -> None:
    namespace = tmp_path / "src" / "domain" / "domain"
    namespace.mkdir(parents=True)
    (namespace / "alpha.py").write_text('"""Alpha."""\n\nVALUE = 1\n', encoding="utf-8")
    (namespace / "beta.py").write_text('"""Beta."""\n\nVALUE = 2\n', encoding="utf-8")

    report = run_python_project_harness(tmp_path)

    assert [finding.rule_id for finding in report.findings] == ["PY-AGENT-R004"]


def test_agent_policy_descriptor_and_catalog_are_stable() -> None:
    descriptor = PythonAgentPolicyRulePack().descriptor()
    rules = python_agent_policy_rules()

    assert descriptor.id == "python.agent_policy"
    assert descriptor.to_dict()["domains"] == [
        "agent-policy",
        "project-shape",
        "python",
    ]
    assert [rule.rule_id for rule in rules] == [
        "PY-AGENT-R001",
        "PY-AGENT-R002",
        "PY-AGENT-R003",
        "PY-AGENT-R004",
    ]
    assert {rule.severity for rule in rules} == {PythonDiagnosticSeverity.WARNING}
