"""Native AST export contract helpers."""

from __future__ import annotations

import ast
from typing import TYPE_CHECKING

from ._ast_names import is_public_name
from ._export_model import PythonExportContractKind

if TYPE_CHECKING:
    from ._ast_collector import PythonAstCollector


def literal_string_sequence(value: ast.AST | None) -> tuple[str, ...] | None:
    """Return literal string sequence values from a static AST node."""

    if value is None:
        return None
    if not isinstance(value, ast.List | ast.Tuple):
        return None
    exports: list[str] = []
    for item in value.elts:
        if not isinstance(item, ast.Constant) or not isinstance(item.value, str):
            return None
        exports.append(item.value)
    return tuple(dict.fromkeys(exports))


def export_candidates(collector: PythonAstCollector) -> tuple[str, ...]:
    """Return explicit or inferred top-level export candidates."""

    if collector.export_contract.kind == PythonExportContractKind.STATIC:
        return collector.export_contract.names

    candidates: set[str] = set()
    candidates.update(
        symbol.name
        for symbol in collector.symbols
        if symbol.is_top_level and symbol.is_public
    )
    candidates.update(
        assignment.name
        for assignment in collector.assignments
        if assignment.is_top_level and assignment.is_public
    )
    for import_record in collector.imports:
        if import_record.scope != "":
            continue
        candidates.update(name for name in import_record.names if is_public_name(name))
    return tuple(sorted(candidates))
