"""Python-native AST-backed source parser for modern Python projects."""

from __future__ import annotations

from .model import (
    PythonCall,
    PythonDiagnostic,
    PythonDiagnosticSeverity,
    PythonImport,
    PythonModuleReport,
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
    "PythonCall",
    "PythonDiagnostic",
    "PythonDiagnosticSeverity",
    "PythonImport",
    "PythonModuleReport",
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
