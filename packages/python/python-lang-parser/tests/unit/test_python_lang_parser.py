from __future__ import annotations

from python_lang_parser import (
    PythonCallEffect,
    PythonDiagnosticSeverity,
    PythonExportContractKind,
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
    assert not any(item.is_wildcard for item in report.imports)
    assert [
        (
            symbol.kind,
            symbol.qualified_name,
            symbol.scope,
            symbol.decorators,
            symbol.is_public,
            symbol.is_top_level,
        )
        for symbol in report.symbols
    ] == [
        (PythonSymbolKind.CLASS, "Runner", "", ("decorator('value')",), True, True),
        (PythonSymbolKind.ASYNC_FUNCTION, "Runner.run", "Runner", (), True, False),
    ]
    assert report.shape is not None
    assert report.shape.responsibility_groups == ("types",)
    assert report.shape.public_symbol_count == 1
    assert report.metadata["parser"] == "cpython.ast"
    assert report.metadata["parser_authority"] == "python-native"
    assert report.metadata["symbol_table"] == "cpython.symtable"
    assert report.export_candidates == ("Runner", "abc", "path_mod")


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
            call.effect,
            call.expression,
        )
        for call in report.calls
    ] == [
        (
            "client.worker.process",
            "build",
            1,
            ("flag",),
            PythonCallEffect.UNKNOWN,
            "client.worker.process(item, flag=True)",
        ),
        (
            "helper",
            "build",
            1,
            (),
            PythonCallEffect.UNKNOWN,
            "helper(client.worker.status)",
        ),
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
    assert serialized["calls"][0]["effect"] == "unknown"


def test_parse_python_source_classifies_wildcard_import_and_builtin_call_effects() -> (
    None
):
    report = parse_python_source(
        """
from tools import *


def run() -> None:
    print("debug")
    breakpoint()
""",
        path="effects.py",
    )

    assert report.is_valid
    assert [(item.names, item.is_wildcard) for item in report.imports] == [
        (("*",), True),
    ]
    assert [(call.function, call.effect) for call in report.calls] == [
        ("print", PythonCallEffect.STANDARD_OUTPUT),
        ("breakpoint", PythonCallEffect.DEBUG_BREAKPOINT),
    ]


def test_parse_python_source_collects_annotations_assignments_and_exports() -> None:
    report = parse_python_source(
        """
from .api import Runner as PublicRunner

__all__ = ["PublicRunner", "build"]
VALUE: int = 1
cache = {}
_private = "hidden"


def build(name: str) -> PublicRunner:
    current = PublicRunner(name)
    for index, item in enumerate([current]):
        pass
    return current


class Service:
    endpoint: str
""",
        path="exports.py",
    )

    assert report.is_valid
    assert report.has_annotations
    assert report.export_contract.kind == PythonExportContractKind.STATIC
    assert report.export_contract.names == ("PublicRunner", "build")
    assert report.export_candidates == ("PublicRunner", "build")
    assert report.shape is not None
    assert report.shape.top_level_statement_count == 6
    assert report.shape.public_symbol_count == 2
    assert report.shape.public_assignment_count == 2

    symbol_annotations = {
        symbol.qualified_name: symbol.has_annotations for symbol in report.symbols
    }
    assert symbol_annotations == {
        "build": True,
        "Service": True,
    }

    assignments = {
        (assignment.scope, assignment.name): assignment
        for assignment in report.assignments
    }
    assert assignments[("", "__all__")].target_kind == "assign"
    assert assignments[("", "__all__")].value_expression == '["PublicRunner", "build"]'
    assert assignments[("", "VALUE")].target_kind == "annotated_assign"
    assert assignments[("", "VALUE")].value_expression == "1"
    assert assignments[("", "VALUE")].is_public
    assert assignments[("", "VALUE")].is_top_level
    assert assignments[("build", "current")].target_kind == "assign"
    assert not assignments[("build", "current")].is_top_level
    assert assignments[("build", "index")].target_kind == "for"
    assert assignments[("build", "item")].target_kind == "for"
    assert assignments[("Service", "endpoint")].target_kind == "annotated_assign"

    serialized = report.to_dict()
    assert serialized["has_annotations"] is True
    assert serialized["export_contract"]["kind"] == "static"
    assert serialized["export_contract"]["names"] == ["PublicRunner", "build"]
    assert serialized["export_candidates"] == ["PublicRunner", "build"]
    assert serialized["assignments"][0]["name"] == "__all__"


def test_parse_python_source_preserves_explicit_empty_exports() -> None:
    report = parse_python_source(
        """
__all__ = []


class Hidden:
    pass
""",
        path="empty_exports.py",
    )

    assert report.is_valid
    assert report.export_contract.kind == PythonExportContractKind.STATIC
    assert report.export_contract.names == ()
    assert report.export_candidates == ()


def test_parse_python_source_falls_back_for_dynamic_exports() -> None:
    report = parse_python_source(
        """
__all__ = ["Public", exported_name]


class Public:
    pass


class Other:
    pass
""",
        path="dynamic_exports.py",
    )

    assert report.is_valid
    assert report.export_contract.kind == PythonExportContractKind.DYNAMIC
    assert report.export_candidates == ("Other", "Public")
