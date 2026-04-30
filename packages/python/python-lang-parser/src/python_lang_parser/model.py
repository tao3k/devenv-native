"""Public model facade for Python native-syntax parser reports."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any

from ._diagnostic_model import (
    PythonDiagnostic,
    PythonDiagnosticSeverity,
    SourceLocation,
)
from ._export_model import PythonExportContract, PythonExportContractKind
from ._symbol_model import (
    PythonAssignmentTarget,
    PythonCall,
    PythonCallEffect,
    PythonImport,
    PythonModuleShape,
    PythonNameBinding,
    PythonReference,
    PythonReferenceKind,
    PythonScope,
    PythonSymbol,
    PythonSymbolKind,
)

if TYPE_CHECKING:
    from collections.abc import Mapping


@dataclass(frozen=True, slots=True)
class PythonModuleReport:
    """Structured parser report for one Python module."""

    path: str | None
    module_docstring: str | None
    imports: tuple[PythonImport, ...] = field(default_factory=tuple)
    symbols: tuple[PythonSymbol, ...] = field(default_factory=tuple)
    scopes: tuple[PythonScope, ...] = field(default_factory=tuple)
    bindings: tuple[PythonNameBinding, ...] = field(default_factory=tuple)
    references: tuple[PythonReference, ...] = field(default_factory=tuple)
    calls: tuple[PythonCall, ...] = field(default_factory=tuple)
    assignments: tuple[PythonAssignmentTarget, ...] = field(default_factory=tuple)
    export_contract: PythonExportContract = field(
        default_factory=lambda: PythonExportContract(
            kind=PythonExportContractKind.INFERRED
        )
    )
    export_candidates: tuple[str, ...] = field(default_factory=tuple)
    has_annotations: bool = False
    shape: PythonModuleShape | None = None
    diagnostics: tuple[PythonDiagnostic, ...] = field(default_factory=tuple)
    metadata: Mapping[str, str] = field(default_factory=dict)

    @property
    def is_valid(self) -> bool:
        """Return whether parsing completed without error diagnostics."""

        return not any(
            diagnostic.severity == PythonDiagnosticSeverity.ERROR
            for diagnostic in self.diagnostics
        )

    def to_dict(self) -> dict[str, Any]:
        """Return a JSON-compatible representation."""

        return {
            "path": self.path,
            "module_docstring": self.module_docstring,
            "imports": [item.to_dict() for item in self.imports],
            "symbols": [item.to_dict() for item in self.symbols],
            "scopes": [item.to_dict() for item in self.scopes],
            "bindings": [item.to_dict() for item in self.bindings],
            "references": [item.to_dict() for item in self.references],
            "calls": [item.to_dict() for item in self.calls],
            "assignments": [item.to_dict() for item in self.assignments],
            "export_contract": self.export_contract.to_dict(),
            "export_candidates": list(self.export_candidates),
            "has_annotations": self.has_annotations,
            "shape": None if self.shape is None else self.shape.to_dict(),
            "diagnostics": [item.to_dict() for item in self.diagnostics],
            "metadata": dict(self.metadata),
            "is_valid": self.is_valid,
        }


__all__ = [
    "PythonAssignmentTarget",
    "PythonCall",
    "PythonCallEffect",
    "PythonDiagnostic",
    "PythonDiagnosticSeverity",
    "PythonExportContract",
    "PythonExportContractKind",
    "PythonImport",
    "PythonModuleReport",
    "PythonModuleShape",
    "PythonNameBinding",
    "PythonReference",
    "PythonReferenceKind",
    "PythonScope",
    "PythonSymbol",
    "PythonSymbolKind",
    "SourceLocation",
]
