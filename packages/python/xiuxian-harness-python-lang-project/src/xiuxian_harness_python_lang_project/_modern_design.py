"""Modern Python design rule pack backed by native parser reports."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

from python_lang_parser import PythonCallEffect

from ._model import PythonHarnessFinding, PythonRulePackDescriptor
from ._modern_design_catalog import (
    MODERN_DESIGN_PACK_ID,
    PY_MOD_R001,
    PY_MOD_R002,
    PY_MOD_R003,
    PY_MOD_R004,
    modern_design_rule,
)
from ._source import source_line

if TYPE_CHECKING:
    from collections.abc import Iterable

    from python_lang_parser import PythonModuleReport


@dataclass(frozen=True, slots=True)
class PythonModernDesignRulePack:
    """Numbered modern Python design rules backed by native parser reports."""

    pack_id: str = MODERN_DESIGN_PACK_ID

    def descriptor(self) -> PythonRulePackDescriptor:
        """Return stable metadata for this rule pack."""

        return PythonRulePackDescriptor(
            id=self.pack_id,
            version="v1",
            domains=("modern-python", "design", "python"),
        )

    def evaluate(self, report: PythonModuleReport) -> Iterable[PythonHarnessFinding]:
        """Evaluate modern Python design rules for one parsed module report."""

        if not report.is_valid:
            return ()

        findings: list[PythonHarnessFinding] = []
        findings.extend(_wildcard_import_findings(report, self.pack_id))
        findings.extend(_bare_print_findings(report, self.pack_id))
        findings.extend(_debug_breakpoint_findings(report, self.pack_id))
        findings.extend(_facade_all_findings(report, self.pack_id))
        return tuple(findings)


def _wildcard_import_findings(
    report: PythonModuleReport,
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    findings: list[PythonHarnessFinding] = []
    rule = modern_design_rule(PY_MOD_R001)
    for import_record in report.imports:
        if not import_record.is_wildcard:
            continue
        module = "." * import_record.level + (import_record.module or "")
        findings.append(
            PythonHarnessFinding(
                rule_id=rule.rule_id,
                pack_id=pack_id,
                severity=rule.severity,
                title=rule.title,
                summary=f"Wildcard import from {module!r} makes exported names implicit.",
                location=import_record.location,
                requirement=f"Import explicit names from {module!r}; do not use `*` in project modules.",
                source_line=source_line(report.path, import_record.location.line),
                label="replace wildcard import with explicit imported names",
                labels=dict(rule.labels),
            )
        )
    return tuple(findings)


def _bare_print_findings(
    report: PythonModuleReport,
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    if not _module_has_project_surface(report):
        return ()

    findings: list[PythonHarnessFinding] = []
    rule = modern_design_rule(PY_MOD_R002)
    for call in report.calls:
        if call.effect != PythonCallEffect.STANDARD_OUTPUT:
            continue
        findings.append(
            PythonHarnessFinding(
                rule_id=rule.rule_id,
                pack_id=pack_id,
                severity=rule.severity,
                title=rule.title,
                summary="Bare print calls leak diagnostics to stdout.",
                location=call.location,
                requirement=rule.requirement,
                source_line=source_line(report.path, call.location.line),
                label="replace bare print with a project-owned reporting surface",
                labels=dict(rule.labels),
            )
        )
    return tuple(findings)


def _debug_breakpoint_findings(
    report: PythonModuleReport,
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    if not _module_has_project_surface(report):
        return ()

    findings: list[PythonHarnessFinding] = []
    rule = modern_design_rule(PY_MOD_R004)
    for call in report.calls:
        if call.effect != PythonCallEffect.DEBUG_BREAKPOINT:
            continue
        findings.append(
            PythonHarnessFinding(
                rule_id=rule.rule_id,
                pack_id=pack_id,
                severity=rule.severity,
                title=rule.title,
                summary="breakpoint() can halt library execution inside an interactive debugger.",
                location=call.location,
                requirement=rule.requirement,
                source_line=source_line(report.path, call.location.line),
                label="remove breakpoint() from library code",
                labels=dict(rule.labels),
            )
        )
    return tuple(findings)


def _facade_all_findings(
    report: PythonModuleReport,
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    if not report.path or Path(report.path).name != "__init__.py":
        return ()
    if not _has_facade_import(report) or report.export_contract.is_static:
        return ()
    rule = modern_design_rule(PY_MOD_R003)

    return (
        PythonHarnessFinding(
            rule_id=rule.rule_id,
            pack_id=pack_id,
            severity=rule.severity,
            title=rule.title,
            summary="Facade imports expose names without an explicit public contract.",
            location=report.imports[0].location,
            requirement=rule.requirement,
            source_line=source_line(report.path, report.imports[0].location.line),
            label="add an explicit __all__ for this facade export surface",
            labels=dict(rule.labels),
        ),
    )


def _has_facade_import(report: PythonModuleReport) -> bool:
    return any(
        import_record.scope == "" and import_record.level > 0
        for import_record in report.imports
    )


def _module_has_project_surface(report: PythonModuleReport) -> bool:
    return report.shape is not None and report.shape.public_symbol_count > 0
