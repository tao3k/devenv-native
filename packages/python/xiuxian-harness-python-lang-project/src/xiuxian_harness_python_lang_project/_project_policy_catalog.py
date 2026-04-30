"""Rule catalog for modern Python project-shape policy."""

from __future__ import annotations

from dataclasses import replace

from python_lang_parser import PythonDiagnosticSeverity

from ._model import PythonHarnessRule

PROJECT_POLICY_PACK_ID = "python.project_policy"
PY_PROJ_R001 = "PY-PROJ-R001"
PY_PROJ_R002 = "PY-PROJ-R002"
PY_PROJ_R003 = "PY-PROJ-R003"
PY_PROJ_R004 = "PY-PROJ-R004"

_RULE_LABELS = {
    "language": "python",
    "domain": "project-policy",
}
_RULES = (
    PythonHarnessRule(
        rule_id=PY_PROJ_R001,
        pack_id=PROJECT_POLICY_PACK_ID,
        severity=PythonDiagnosticSeverity.WARNING,
        title="Python project should use src layout",
        requirement="Use a `src/` source layout so package imports resolve through the installed project shape instead of the repository root.",
        labels=dict(_RULE_LABELS),
    ),
    PythonHarnessRule(
        rule_id=PY_PROJ_R002,
        pack_id=PROJECT_POLICY_PACK_ID,
        severity=PythonDiagnosticSeverity.WARNING,
        title="Declared wheel package root is not importable",
        requirement="Ensure each declared wheel package root exists and contains `__init__.py` so agents can map project metadata to import namespaces.",
        labels=dict(_RULE_LABELS),
    ),
    PythonHarnessRule(
        rule_id=PY_PROJ_R003,
        pack_id=PROJECT_POLICY_PACK_ID,
        severity=PythonDiagnosticSeverity.WARNING,
        title="Typed public package lacks py.typed marker",
        requirement="Add `py.typed` to declared package roots that expose public parser surface so downstream agents and type checkers can trust inline types.",
        labels=dict(_RULE_LABELS),
    ),
    PythonHarnessRule(
        rule_id=PY_PROJ_R004,
        pack_id=PROJECT_POLICY_PACK_ID,
        severity=PythonDiagnosticSeverity.WARNING,
        title="Typed package public callable lacks annotations",
        requirement="Annotate public callable boundaries in `py.typed` packages so the declared typed surface remains complete and agent-readable.",
        labels=dict(_RULE_LABELS),
    ),
)
_RULE_BY_ID = {rule.rule_id: rule for rule in _RULES}


def python_project_policy_rules() -> tuple[PythonHarnessRule, ...]:
    """Return compact metadata for default project-shape rules."""

    return tuple(replace(rule, labels=dict(rule.labels)) for rule in _RULES)


def project_policy_rule(rule_id: str) -> PythonHarnessRule:
    """Return one project-policy rule descriptor by stable rule id."""

    return _RULE_BY_ID[rule_id]
