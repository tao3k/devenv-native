"""Pytest layout rule pack aligned with the project harness."""

from __future__ import annotations

import ast
from dataclasses import dataclass
from typing import TYPE_CHECKING

from ._constants import IGNORED_DIR_NAMES
from ._discovery import is_scannable_python_file, python_project_harness_scope
from ._model import PythonHarnessFinding, PythonRulePackDescriptor
from ._source import (
    count_effective_python_code_lines,
    path_location,
    read_text,
    source_line,
)
from ._test_layout_catalog import (
    ALLOWED_TEST_DIR_NAMES,
    ALLOWED_TEST_ROOT_FILES,
    MAX_UNIT_TEST_EFFECTIVE_LINES,
    MIN_UNIT_TEST_FUNCTIONS,
    PY_TEST_R001,
    PY_TEST_R002,
    PY_TEST_R003,
    TEST_LAYOUT_PACK_ID,
    test_layout_rule,
)

if TYPE_CHECKING:
    from collections.abc import Iterable
    from pathlib import Path

    from python_lang_parser import PythonModuleReport

    from ._model import PythonProjectHarnessScope


@dataclass(frozen=True, slots=True)
class PythonTestLayoutRulePack:
    """Project-level pytest layout rules aligned with the Rust unit harness gate."""

    pack_id: str = TEST_LAYOUT_PACK_ID

    def descriptor(self) -> PythonRulePackDescriptor:
        """Return stable metadata for this rule pack."""

        return PythonRulePackDescriptor(
            id=self.pack_id,
            version="v1",
            domains=("pytest-layout", "unit-tests", "python"),
        )

    def evaluate(self, report: PythonModuleReport) -> Iterable[PythonHarnessFinding]:
        """Module-level parser reports do not carry project layout authority."""

        return ()

    def evaluate_project(self, project_root: Path) -> Iterable[PythonHarnessFinding]:
        """Evaluate project-level pytest layout rules."""

        return self.evaluate_project_scope(python_project_harness_scope(project_root))

    def evaluate_project_scope(
        self,
        scope: PythonProjectHarnessScope,
    ) -> Iterable[PythonHarnessFinding]:
        """Evaluate project-level pytest layout rules for monitored test roots."""

        return _test_layout_findings(scope, self.pack_id)


def _test_layout_findings(
    scope: PythonProjectHarnessScope,
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    findings: list[PythonHarnessFinding] = []
    for tests_dir in scope.test_paths:
        if not tests_dir.exists():
            continue
        findings.extend(_tests_root_entry_findings(tests_dir, pack_id))
        findings.extend(_bloated_unit_test_findings(tests_dir, pack_id))
    return tuple(findings)


def _tests_root_entry_findings(
    tests_dir: Path,
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    findings: list[PythonHarnessFinding] = []
    for path in sorted(tests_dir.iterdir(), key=lambda item: item.as_posix()):
        name = path.name
        if name.startswith("."):
            continue
        if path.is_dir():
            if name not in ALLOWED_TEST_DIR_NAMES:
                findings.append(_unexpected_tests_root_entry_finding(path, pack_id))
            continue
        if path.suffix != ".py" or name in ALLOWED_TEST_ROOT_FILES:
            continue
        if name.startswith("test_"):
            findings.append(_root_pytest_file_finding(path, pack_id))
        else:
            findings.append(_unexpected_tests_root_entry_finding(path, pack_id))
    return tuple(findings)


def _bloated_unit_test_findings(
    tests_dir: Path,
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    unit_dir = tests_dir / "unit"
    if not unit_dir.exists():
        return ()

    findings: list[PythonHarnessFinding] = []
    for path in sorted(unit_dir.rglob("*.py"), key=lambda item: item.as_posix()):
        if not is_scannable_python_file(path, ignored_dir_names=IGNORED_DIR_NAMES):
            continue
        if path.name == "__init__.py":
            continue
        content = read_text(path)
        if content is None:
            continue
        effective_code_lines = count_effective_python_code_lines(content)
        if effective_code_lines < MAX_UNIT_TEST_EFFECTIVE_LINES:
            continue
        test_functions = _count_python_test_functions(content)
        if test_functions < MIN_UNIT_TEST_FUNCTIONS:
            continue
        findings.append(
            _bloated_unit_test_finding(
                path,
                pack_id,
                effective_code_lines=effective_code_lines,
                test_functions=test_functions,
            )
        )
    return tuple(findings)


def _root_pytest_file_finding(path: Path, pack_id: str) -> PythonHarnessFinding:
    rule = test_layout_rule(PY_TEST_R001)
    return PythonHarnessFinding(
        rule_id=rule.rule_id,
        pack_id=pack_id,
        severity=rule.severity,
        title=rule.title,
        summary=f"{path.name} is a pytest module directly under tests root.",
        location=path_location(path),
        requirement=rule.requirement,
        source_line=source_line(str(path), 1),
        label="move this pytest module under tests/unit/ or tests/integration/",
        labels=dict(rule.labels),
    )


def _unexpected_tests_root_entry_finding(
    path: Path, pack_id: str
) -> PythonHarnessFinding:
    rule = test_layout_rule(PY_TEST_R002)
    return PythonHarnessFinding(
        rule_id=rule.rule_id,
        pack_id=pack_id,
        severity=rule.severity,
        title=rule.title,
        summary=f"{path.name} is not an owned tests root entry.",
        location=path_location(path),
        requirement=rule.requirement,
        source_line=source_line(str(path), 1) if path.is_file() else None,
        label="move this entry into an owned tests suite directory",
        labels=dict(rule.labels),
    )


def _bloated_unit_test_finding(
    path: Path,
    pack_id: str,
    *,
    effective_code_lines: int,
    test_functions: int,
) -> PythonHarnessFinding:
    rule = test_layout_rule(PY_TEST_R003)
    return PythonHarnessFinding(
        rule_id=rule.rule_id,
        pack_id=pack_id,
        severity=rule.severity,
        title=rule.title,
        summary=(
            f"{path.name} has {effective_code_lines} effective lines across "
            f"{test_functions} test functions."
        ),
        location=path_location(path),
        requirement=(
            f"Split {path.name} into a folder-first unit suite; "
            f"current size is {effective_code_lines} effective lines across "
            f"{test_functions} tests."
        ),
        source_line=source_line(str(path), 1),
        label="split this large unit test leaf into focused pytest modules",
        labels=dict(rule.labels),
    )


def _count_python_test_functions(content: str) -> int:
    try:
        tree = ast.parse(content)
    except SyntaxError:
        return 0
    return sum(
        1
        for node in ast.walk(tree)
        if isinstance(node, ast.FunctionDef | ast.AsyncFunctionDef)
        and node.name.startswith("test_")
    )
