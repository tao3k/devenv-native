"""Rule catalog for agent-oriented Python project policy."""

from __future__ import annotations

from dataclasses import replace

from python_lang_parser import PythonDiagnosticSeverity

from ._model import PythonHarnessRule

AGENT_POLICY_PACK_ID = "python.agent_policy"
PY_AGENT_R001 = "PY-AGENT-R001"
PY_AGENT_R002 = "PY-AGENT-R002"
PY_AGENT_R003 = "PY-AGENT-R003"
PY_AGENT_R004 = "PY-AGENT-R004"
PY_AGENT_R005 = "PY-AGENT-R005"
PY_AGENT_R006 = "PY-AGENT-R006"

_RULE_LABELS = {
    "language": "python",
    "domain": "agent-policy",
}
_RULES = (
    PythonHarnessRule(
        rule_id=PY_AGENT_R001,
        pack_id=AGENT_POLICY_PACK_ID,
        severity=PythonDiagnosticSeverity.INFO,
        title="Library module lacks a module intent docstring",
        requirement="Add a concise module docstring that names the module responsibility for agent search and repair.",
        labels=dict(_RULE_LABELS),
    ),
    PythonHarnessRule(
        rule_id=PY_AGENT_R002,
        pack_id=AGENT_POLICY_PACK_ID,
        severity=PythonDiagnosticSeverity.INFO,
        title="Public callable lacks type annotations",
        requirement="Annotate public function and method boundaries so agents can reason from native syntax without guessing shapes.",
        labels=dict(_RULE_LABELS),
    ),
    PythonHarnessRule(
        rule_id=PY_AGENT_R003,
        pack_id=AGENT_POLICY_PACK_ID,
        severity=PythonDiagnosticSeverity.INFO,
        title="Public callable name conflicts across namespaces",
        requirement="Give project-level public callables unambiguous names or move them behind a clear domain namespace so agents can resolve intent without guessing.",
        labels=dict(_RULE_LABELS),
    ),
    PythonHarnessRule(
        rule_id=PY_AGENT_R004,
        pack_id=AGENT_POLICY_PACK_ID,
        severity=PythonDiagnosticSeverity.INFO,
        title="Module namespace repeats a path segment",
        requirement="Keep Python module namespaces branch-unique; rename repeated path segments so agents see one clear ownership path.",
        labels=dict(_RULE_LABELS),
    ),
    PythonHarnessRule(
        rule_id=PY_AGENT_R005,
        pack_id=AGENT_POLICY_PACK_ID,
        severity=PythonDiagnosticSeverity.INFO,
        title="Public type name conflicts across namespaces",
        requirement="Give project-level public classes unambiguous type names or move them behind a clear domain namespace so agents can resolve intent without guessing.",
        labels=dict(_RULE_LABELS),
    ),
    PythonHarnessRule(
        rule_id=PY_AGENT_R006,
        pack_id=AGENT_POLICY_PACK_ID,
        severity=PythonDiagnosticSeverity.INFO,
        title="Public value name conflicts across namespaces",
        requirement="Give project-level public values and configuration exports unambiguous names or move them behind a clear domain namespace so agents can resolve intent without guessing.",
        labels=dict(_RULE_LABELS),
    ),
)
_RULE_BY_ID = {rule.rule_id: rule for rule in _RULES}


def python_agent_policy_rules() -> tuple[PythonHarnessRule, ...]:
    """Return compact metadata for the default agent-oriented policy rules."""

    return tuple(replace(rule, labels=dict(rule.labels)) for rule in _RULES)


def agent_policy_rule(rule_id: str) -> PythonHarnessRule:
    """Return one agent-policy rule descriptor by stable rule id."""

    return _RULE_BY_ID[rule_id]
