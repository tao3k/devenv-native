"""Python-native AST-backed source parser for modern Python projects."""

from __future__ import annotations

from .model import (
    PythonAssignmentTarget,
    PythonCall,
    PythonCallEffect,
    PythonDiagnostic,
    PythonDiagnosticSeverity,
    PythonExportContract,
    PythonExportContractKind,
    PythonImport,
    PythonModuleReport,
    PythonModuleShape,
    PythonNameBinding,
    PythonReference,
    PythonReferenceKind,
    PythonScope,
    PythonSymbol,
    PythonSymbolKind,
    SourceLocation,
)
from .parser import parse_python_file, parse_python_source

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
    "parse_python_file",
    "parse_python_source",
]
