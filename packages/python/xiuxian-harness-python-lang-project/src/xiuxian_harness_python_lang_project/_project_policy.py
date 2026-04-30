"""Modern Python project-shape rule pack."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from ._model import PythonHarnessFinding, PythonRulePackDescriptor
from ._project_metadata import read_python_project_metadata
from ._project_policy_catalog import PROJECT_POLICY_PACK_ID
from ._project_policy_layout import project_layout_findings
from ._project_policy_typed import typed_package_findings

if TYPE_CHECKING:
    from collections.abc import Iterable, Sequence

    from python_lang_parser import PythonModuleReport

    from ._model import PythonProjectHarnessScope


@dataclass(frozen=True, slots=True)
class PythonProjectPolicyRulePack:
    """Rules for modern Python project metadata and package shape."""

    pack_id: str = PROJECT_POLICY_PACK_ID

    def descriptor(self) -> PythonRulePackDescriptor:
        """Return stable metadata for this rule pack."""

        return PythonRulePackDescriptor(
            id=self.pack_id,
            version="v1",
            domains=("project-policy", "packaging", "python"),
        )

    def evaluate(self, report: PythonModuleReport) -> Iterable[PythonHarnessFinding]:
        """Evaluate per-module rules."""

        return ()

    def evaluate_project_modules(
        self,
        scope: PythonProjectHarnessScope,
        modules: Sequence[PythonModuleReport],
    ) -> Iterable[PythonHarnessFinding]:
        """Evaluate project-shape rules over a parsed project."""

        metadata = read_python_project_metadata(scope.project_root)
        if metadata is None:
            return ()

        findings: list[PythonHarnessFinding] = []
        findings.extend(project_layout_findings(scope, metadata, self.pack_id))
        findings.extend(typed_package_findings(metadata, modules, self.pack_id))
        return tuple(findings)
