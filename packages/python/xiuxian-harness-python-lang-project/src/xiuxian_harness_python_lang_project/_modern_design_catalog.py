"""Rule catalog for modern Python design checks."""

from __future__ import annotations

from dataclasses import replace

from python_lang_parser import PythonDiagnosticSeverity

from ._model import PythonHarnessRule

MODERN_DESIGN_PACK_ID = "python.modern_design"
PY_MOD_R001 = "PY-MOD-R001"
PY_MOD_R002 = "PY-MOD-R002"
PY_MOD_R003 = "PY-MOD-R003"
PY_MOD_R004 = "PY-MOD-R004"

_RULE_LABELS = {
    "language": "python",
    "domain": "modern-python",
}
_RULES = (
    PythonHarnessRule(
        rule_id=PY_MOD_R001,
        pack_id=MODERN_DESIGN_PACK_ID,
        severity=PythonDiagnosticSeverity.WARNING,
        title="Wildcard import hides the dependency surface",
        requirement="Import explicit names instead of `*` in project modules.",
        labels=dict(_RULE_LABELS),
    ),
    PythonHarnessRule(
        rule_id=PY_MOD_R002,
        pack_id=MODERN_DESIGN_PACK_ID,
        severity=PythonDiagnosticSeverity.WARNING,
        title="Library module uses bare print",
        requirement="Use a logger, returned value, or explicit test assertion instead of bare `print` in library modules.",
        labels=dict(_RULE_LABELS),
    ),
    PythonHarnessRule(
        rule_id=PY_MOD_R003,
        pack_id=MODERN_DESIGN_PACK_ID,
        severity=PythonDiagnosticSeverity.WARNING,
        title="Package facade re-exports without __all__",
        requirement="Declare `__all__` beside package facade imports so public exports stay explicit.",
        labels=dict(_RULE_LABELS),
    ),
    PythonHarnessRule(
        rule_id=PY_MOD_R004,
        pack_id=MODERN_DESIGN_PACK_ID,
        severity=PythonDiagnosticSeverity.WARNING,
        title="Library module contains breakpoint()",
        requirement="Remove `breakpoint()` from library modules; use test-only debug tooling or a project-owned diagnostic surface.",
        labels=dict(_RULE_LABELS),
    ),
)
_RULE_BY_ID = {rule.rule_id: rule for rule in _RULES}


def python_modern_design_rules() -> tuple[PythonHarnessRule, ...]:
    """Return compact metadata for the default modern-design rules."""

    return tuple(replace(rule, labels=dict(rule.labels)) for rule in _RULES)


def modern_design_rule(rule_id: str) -> PythonHarnessRule:
    """Return one modern-design rule descriptor by stable rule id."""

    return _RULE_BY_ID[rule_id]
