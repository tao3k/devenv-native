"""Python module-shape rule pack for file-level modularity gates."""

from __future__ import annotations

from dataclasses import dataclass, replace
from pathlib import Path
from typing import TYPE_CHECKING

from python_lang_parser import PythonDiagnosticSeverity

from ._model import (
    PythonHarnessFinding,
    PythonHarnessRule,
    PythonRulePackDescriptor,
)
from ._source import path_location, source_line

if TYPE_CHECKING:
    from collections.abc import Iterable

    from python_lang_parser import PythonModuleReport

MODULARITY_PACK_ID = "python.modularity"
PY_MOD_R006 = "PY-MOD-R006"

_MAX_MODULE_EFFECTIVE_CODE_LINES = 220
_MAX_MODULE_TOP_LEVEL_ITEMS = 8
_MIN_MODULE_RESPONSIBILITY_GROUPS = 3
_MIN_MODULE_PUBLIC_SURFACE_ITEMS = 5
_RULE_LABELS = {
    "language": "python",
    "domain": "modularity",
}
_RULES = (
    PythonHarnessRule(
        rule_id=PY_MOD_R006,
        pack_id=MODULARITY_PACK_ID,
        severity=PythonDiagnosticSeverity.WARNING,
        title="Python module appears too large for one ownership seam",
        requirement="Split large multi-responsibility Python modules into focused modules behind an explicit package facade.",
        labels=dict(_RULE_LABELS),
    ),
)
_RULE_BY_ID = {rule.rule_id: rule for rule in _RULES}


@dataclass(frozen=True, slots=True)
class PythonModularityRulePack:
    """Numbered Python modularity rules backed by native parser reports."""

    pack_id: str = MODULARITY_PACK_ID

    def descriptor(self) -> PythonRulePackDescriptor:
        """Return stable metadata for this rule pack."""

        return PythonRulePackDescriptor(
            id=self.pack_id,
            version="v1",
            domains=("modularity", "architecture", "python"),
        )

    def evaluate(self, report: PythonModuleReport) -> Iterable[PythonHarnessFinding]:
        """Evaluate Python module-shape rules for one parsed module report."""

        if not report.is_valid:
            return ()
        return _file_modularity_findings(report, self.pack_id)


def python_modularity_rules() -> tuple[PythonHarnessRule, ...]:
    """Return compact metadata for the default Python modularity rules."""

    return tuple(replace(rule, labels=dict(rule.labels)) for rule in _RULES)


def _file_modularity_findings(
    report: PythonModuleReport,
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    shape = report.shape
    if shape is None:
        return ()

    if (
        shape.effective_code_lines < _MAX_MODULE_EFFECTIVE_CODE_LINES
        or shape.top_level_statement_count < _MAX_MODULE_TOP_LEVEL_ITEMS
        or (
            shape.responsibility_group_count < _MIN_MODULE_RESPONSIBILITY_GROUPS
            and shape.public_surface_count < _MIN_MODULE_PUBLIC_SURFACE_ITEMS
        )
    ):
        return ()

    rule = _rule(PY_MOD_R006)
    path = Path(report.path or "<memory>")
    return (
        PythonHarnessFinding(
            rule_id=rule.rule_id,
            pack_id=pack_id,
            severity=rule.severity,
            title=rule.title,
            summary=(
                f"{path.name} has {shape.effective_code_lines} effective lines, "
                f"{shape.top_level_statement_count} top-level items, "
                f"{shape.responsibility_group_count} responsibility groups, "
                f"and {shape.public_surface_count} public surface items."
            ),
            location=path_location(path),
            requirement=(
                f"Split {path.name} into focused modules; current size is "
                f"{shape.effective_code_lines} effective lines across "
                f"{shape.top_level_statement_count} top-level items."
            ),
            source_line=source_line(report.path, 1),
            label="split this module into focused ownership seams",
            labels=dict(rule.labels),
        ),
    )


def _rule(rule_id: str) -> PythonHarnessRule:
    return _RULE_BY_ID[rule_id]
