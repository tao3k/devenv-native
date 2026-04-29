from __future__ import annotations

from python_lang_parser import (
    PythonDiagnosticSeverity,
    PythonReferenceKind,
    PythonSymbolKind,
    parse_python_source,
)


def test_parse_python_source_collects_symbols_imports_and_scopes() -> None:
    report = parse_python_source(
        '''
"""Module docs."""

import pathlib as path_mod
from collections import abc


@decorator("value")
class Runner:
    """Runner docs."""

    async def run(self) -> None:
        import json
        return None
''',
        path="runner.py",
    )

    assert report.is_valid
    assert report.module_docstring == "Module docs."
    assert [(item.module, item.names, item.scope) for item in report.imports] == [
        (None, ("path_mod",), ""),
        ("collections", ("abc",), ""),
        (None, ("json",), "Runner.run"),
    ]
    assert [
        (symbol.kind, symbol.qualified_name, symbol.scope, symbol.decorators)
        for symbol in report.symbols
    ] == [
        (PythonSymbolKind.CLASS, "Runner", "", ("decorator('value')",)),
        (PythonSymbolKind.ASYNC_FUNCTION, "Runner.run", "Runner", ()),
    ]
    assert report.metadata["parser"] == "cpython.ast"
    assert report.metadata["parser_authority"] == "python-native"
    assert report.metadata["symbol_table"] == "cpython.symtable"


def test_parse_python_source_reports_syntax_error_as_diagnostic() -> None:
    report = parse_python_source("def broken(:\n    pass\n", path="broken.py")

    assert not report.is_valid
    assert len(report.diagnostics) == 1
    diagnostic = report.diagnostics[0]
    assert diagnostic.code == "python.syntax.invalid"
    assert diagnostic.severity == PythonDiagnosticSeverity.ERROR
    assert diagnostic.location.path == "broken.py"
    assert diagnostic.location.line == 1
    assert diagnostic.source_line == "def broken(:"


def test_parse_python_source_reports_compile_invalid_scope_as_diagnostic() -> None:
    report = parse_python_source("return 1\n", path="bad_scope.py")

    assert not report.is_valid
    assert len(report.diagnostics) == 1
    diagnostic = report.diagnostics[0]
    assert diagnostic.code == "python.compile.invalid"
    assert diagnostic.severity == PythonDiagnosticSeverity.ERROR
    assert diagnostic.location.path == "bad_scope.py"
    assert "outside function" in diagnostic.message


def test_parse_python_source_collects_native_symbol_table_bindings() -> None:
    report = parse_python_source(
        """
import os
VALUE = 1


def outer(x: int) -> int:
    y = x + VALUE

    def inner(z: int) -> int:
        return y + z

    return inner(1)
""",
        path="symbols.py",
    )

    assert report.is_valid
    assert [(scope.name, scope.kind, scope.parent_id) for scope in report.scopes] == [
        ("top", "module", None),
        ("outer", "function", report.scopes[0].id),
        ("inner", "function", report.scopes[1].id),
    ]

    bindings = {
        (binding.scope_name, binding.name): set(binding.flags)
        for binding in report.bindings
    }
    assert {"imported", "global", "local"} <= bindings[("top", "os")]
    assert {"assigned", "global", "local"} <= bindings[("top", "VALUE")]
    assert {"parameter", "local", "referenced"} <= bindings[("outer", "x")]
    assert {"assigned", "local"} <= bindings[("outer", "y")]
    assert {"global", "referenced"} <= bindings[("outer", "VALUE")]
    assert {"assigned", "local", "referenced", "namespace"} <= bindings[
        ("outer", "inner")
    ]
    assert {"free", "referenced"} <= bindings[("inner", "y")]


def test_parse_python_source_collects_agent_reference_and_call_index() -> None:
    report = parse_python_source(
        """
def build(client, items):
    for item in items:
        client.worker.process(item, flag=True)
    return helper(client.worker.status)
""",
        path="calls.py",
    )

    assert report.is_valid
    assert [
        (
            call.function,
            call.scope,
            call.positional_count,
            call.keyword_names,
            call.expression,
        )
        for call in report.calls
    ] == [
        (
            "client.worker.process",
            "build",
            1,
            ("flag",),
            "client.worker.process(item, flag=True)",
        ),
        ("helper", "build", 1, (), "helper(client.worker.status)"),
    ]

    references = {
        (reference.kind, reference.name, reference.context, reference.scope)
        for reference in report.references
    }
    assert (PythonReferenceKind.NAME, "item", "store", "build") in references
    assert (PythonReferenceKind.NAME, "items", "load", "build") in references
    assert (
        PythonReferenceKind.ATTRIBUTE,
        "client.worker.process",
        "load",
        "build",
    ) in references
    assert (
        PythonReferenceKind.ATTRIBUTE,
        "client.worker.status",
        "load",
        "build",
    ) in references
    serialized = report.to_dict()
    assert serialized["references"]
    assert serialized["calls"][0]["function"] == "client.worker.process"
