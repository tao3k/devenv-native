"""Python-native AST-backed source parser."""

from __future__ import annotations

import ast
import sys
import tokenize
from pathlib import Path

from ._ast_annotations import module_has_annotations
from ._ast_collector import PythonAstCollector
from ._diagnostics import compile_diagnostic, diagnostic_from_parse_exception
from ._exports import export_candidates
from ._module_shape import collect_module_shape
from ._native_symbols import collect_native_symbol_table
from .model import (
    PythonDiagnostic,
    PythonDiagnosticSeverity,
    PythonModuleReport,
    SourceLocation,
)


def parse_python_file(path: str | Path) -> PythonModuleReport:
    """Parse one Python file and return a structured module report."""

    file_path = Path(path)
    try:
        with tokenize.open(file_path) as handle:
            source = handle.read()
    except OSError as exc:
        location = SourceLocation(path=str(file_path), line=1, column=0)
        diagnostic = PythonDiagnostic(
            code="python.file.read_error",
            severity=PythonDiagnosticSeverity.ERROR,
            message=str(exc),
            location=location,
            label="file could not be read",
            help="Check the file path and permissions.",
        )
        return PythonModuleReport(
            path=str(file_path),
            module_docstring=None,
            diagnostics=(diagnostic,),
        )

    return parse_python_source(source, path=file_path)


def parse_python_source(
    source: str, *, path: str | Path | None = None
) -> PythonModuleReport:
    """Parse Python source and return a structured module report."""

    path_text = None if path is None else str(path)
    filename = path_text or "<memory>"
    try:
        tree = ast.parse(source, filename=filename, type_comments=True)
    except (SyntaxError, ValueError) as exc:
        diagnostic = diagnostic_from_parse_exception(
            exc,
            code="python.syntax.invalid",
            path_text=path_text,
        )
        return PythonModuleReport(
            path=path_text,
            module_docstring=None,
            diagnostics=(diagnostic,),
        )

    diagnostic = compile_diagnostic(source, path_text=path_text, filename=filename)
    if diagnostic is not None:
        return PythonModuleReport(
            path=path_text,
            module_docstring=ast.get_docstring(tree),
            diagnostics=(diagnostic,),
        )

    native_scopes, native_bindings = collect_native_symbol_table(
        source,
        path_text=path_text,
        filename=filename,
    )
    collector = PythonAstCollector(path_text, source)
    collector.visit(tree)
    return PythonModuleReport(
        path=path_text,
        module_docstring=ast.get_docstring(tree),
        imports=tuple(collector.imports),
        symbols=tuple(collector.symbols),
        scopes=native_scopes,
        bindings=native_bindings,
        references=tuple(collector.references),
        calls=tuple(collector.calls),
        assignments=tuple(collector.assignments),
        export_contract=collector.export_contract,
        export_candidates=export_candidates(collector),
        has_annotations=module_has_annotations(tree),
        shape=collect_module_shape(tree, source, collector),
        diagnostics=(),
        metadata={
            "parser": "cpython.ast",
            "parser_authority": "python-native",
            "python_version": ".".join(str(part) for part in sys.version_info[:3]),
            "symbol_table": "cpython.symtable",
            "language": "python",
        },
    )
