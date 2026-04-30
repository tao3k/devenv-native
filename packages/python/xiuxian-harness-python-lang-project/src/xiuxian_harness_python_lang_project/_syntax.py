"""Native parser diagnostic rule pack."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from python_lang_parser import PythonDiagnosticSeverity

from ._model import (
    PythonHarnessFinding,
    PythonRulePackDescriptor,
)

if TYPE_CHECKING:
    from collections.abc import Iterable

    from python_lang_parser import PythonModuleReport


@dataclass(frozen=True, slots=True)
class PythonSyntaxRulePack:
    """Rule pack that turns parser diagnostics into harness findings."""

    pack_id: str = "python.syntax"

    def descriptor(self) -> PythonRulePackDescriptor:
        """Return stable metadata for this rule pack."""

        return PythonRulePackDescriptor(
            id=self.pack_id,
            version="v1",
            domains=("syntax", "python"),
        )

    def evaluate(self, report: PythonModuleReport) -> Iterable[PythonHarnessFinding]:
        """Evaluate parse diagnostics for one module report."""

        for diagnostic in report.diagnostics:
            if diagnostic.severity != PythonDiagnosticSeverity.ERROR:
                continue
            yield PythonHarnessFinding(
                rule_id=diagnostic.code,
                pack_id=self.pack_id,
                severity=diagnostic.severity,
                title="Python source did not parse",
                summary=diagnostic.message,
                location=diagnostic.location,
                requirement="Python modules must parse with CPython native syntax before project rules run.",
                source_line=diagnostic.source_line,
                label=diagnostic.message or diagnostic.label,
                labels={"language": "python"},
            )
