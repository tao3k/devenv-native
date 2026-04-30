"""Agent-oriented Python policy rules backed by native parser reports."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

from python_lang_parser import PythonSymbolKind

from ._agent_namespace import agent_namespace_findings
from ._agent_policy_catalog import (
    AGENT_POLICY_PACK_ID,
    PY_AGENT_R001,
    PY_AGENT_R002,
    agent_policy_rule,
)
from ._model import PythonHarnessFinding, PythonRulePackDescriptor
from ._source import path_location, source_line

if TYPE_CHECKING:
    from collections.abc import Iterable, Sequence

    from python_lang_parser import PythonModuleReport, PythonSymbol

    from ._model import PythonProjectHarnessScope


@dataclass(frozen=True, slots=True)
class PythonAgentPolicyRulePack:
    """Rules that keep Python projects legible for repair-oriented agents."""

    pack_id: str = AGENT_POLICY_PACK_ID

    def descriptor(self) -> PythonRulePackDescriptor:
        """Return stable metadata for this rule pack."""

        return PythonRulePackDescriptor(
            id=self.pack_id,
            version="v1",
            domains=("agent-policy", "project-shape", "python"),
        )

    def evaluate(self, report: PythonModuleReport) -> Iterable[PythonHarnessFinding]:
        """Evaluate agent-oriented policy rules for one parsed module report."""

        if not report.is_valid:
            return ()

        findings: list[PythonHarnessFinding] = []
        findings.extend(_module_docstring_findings(report, self.pack_id))
        findings.extend(_public_callable_annotation_findings(report, self.pack_id))
        return tuple(findings)

    def evaluate_project_modules(
        self,
        scope: PythonProjectHarnessScope,
        modules: Sequence[PythonModuleReport],
    ) -> Iterable[PythonHarnessFinding]:
        """Evaluate agent-oriented namespace rules across a project scope."""

        return agent_namespace_findings(scope, modules, self.pack_id)


def _module_docstring_findings(
    report: PythonModuleReport,
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    if report.module_docstring or not _module_has_agent_surface(report):
        return ()
    rule = agent_policy_rule(PY_AGENT_R001)
    path = Path(report.path or "<memory>")
    return (
        PythonHarnessFinding(
            rule_id=rule.rule_id,
            pack_id=pack_id,
            severity=rule.severity,
            title=rule.title,
            summary=f"{path.name} has public module surface without a module intent docstring.",
            location=path_location(path),
            requirement=rule.requirement,
            source_line=source_line(report.path, 1),
            label="add a concise module responsibility docstring",
            labels=dict(rule.labels),
        ),
    )


def _public_callable_annotation_findings(
    report: PythonModuleReport,
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    findings: list[PythonHarnessFinding] = []
    rule = agent_policy_rule(PY_AGENT_R002)
    for symbol in report.symbols:
        if not _is_public_callable(symbol) or symbol.has_annotations:
            continue
        findings.append(
            PythonHarnessFinding(
                rule_id=rule.rule_id,
                pack_id=pack_id,
                severity=rule.severity,
                title=rule.title,
                summary=f"{symbol.qualified_name} exposes a public callable boundary without annotations.",
                location=symbol.location,
                requirement=rule.requirement,
                source_line=source_line(report.path, symbol.location.line),
                label="add parameter and return annotations to this public callable",
                labels=dict(rule.labels),
            )
        )
    return tuple(findings)


def _module_has_agent_surface(report: PythonModuleReport) -> bool:
    return report.shape is not None and report.shape.public_symbol_count > 0


def _is_public_callable(symbol: PythonSymbol) -> bool:
    if symbol.kind not in {PythonSymbolKind.FUNCTION, PythonSymbolKind.ASYNC_FUNCTION}:
        return False
    return symbol.is_public
