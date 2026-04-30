"""Rule catalog for pytest layout checks."""

from __future__ import annotations

from dataclasses import replace

from python_lang_parser import PythonDiagnosticSeverity

from ._model import PythonHarnessRule

TEST_LAYOUT_PACK_ID = "python.test_layout"
PY_TEST_R001 = "PY-TEST-R001"
PY_TEST_R002 = "PY-TEST-R002"
PY_TEST_R003 = "PY-TEST-R003"

ALLOWED_TEST_DIR_NAMES = frozenset(
    {
        "__pycache__",
        "e2e",
        "fixtures",
        "integration",
        "performance",
        "scenarios",
        "snapshots",
        "support",
        "unit",
    }
)
ALLOWED_TEST_ROOT_FILES = frozenset({"__init__.py", "conftest.py"})
MAX_UNIT_TEST_EFFECTIVE_LINES = 260
MIN_UNIT_TEST_FUNCTIONS = 8

_RULE_LABELS = {
    "language": "python",
    "domain": "pytest-layout",
}
_RULES = (
    PythonHarnessRule(
        rule_id=PY_TEST_R001,
        pack_id=TEST_LAYOUT_PACK_ID,
        severity=PythonDiagnosticSeverity.WARNING,
        title="Pytest file is scattered in tests root",
        requirement="Move pytest modules under `tests/unit/` or `tests/integration/` so the project harness owns suite shape.",
        labels=dict(_RULE_LABELS),
    ),
    PythonHarnessRule(
        rule_id=PY_TEST_R002,
        pack_id=TEST_LAYOUT_PACK_ID,
        severity=PythonDiagnosticSeverity.WARNING,
        title="Unexpected tests root entry",
        requirement="Keep tests root limited to harness configuration and owned suite directories.",
        labels=dict(_RULE_LABELS),
    ),
    PythonHarnessRule(
        rule_id=PY_TEST_R003,
        pack_id=TEST_LAYOUT_PACK_ID,
        severity=PythonDiagnosticSeverity.WARNING,
        title="Unit test leaf is too large",
        requirement="Split large unit-test modules into a folder-first `tests/unit/<topic>/` suite with focused leaves.",
        labels=dict(_RULE_LABELS),
    ),
)
_RULE_BY_ID = {rule.rule_id: rule for rule in _RULES}


def python_test_layout_rules() -> tuple[PythonHarnessRule, ...]:
    """Return compact metadata for the default pytest-layout rules."""

    return tuple(replace(rule, labels=dict(rule.labels)) for rule in _RULES)


def test_layout_rule(rule_id: str) -> PythonHarnessRule:
    """Return one pytest-layout rule descriptor by stable rule id."""

    return _RULE_BY_ID[rule_id]
