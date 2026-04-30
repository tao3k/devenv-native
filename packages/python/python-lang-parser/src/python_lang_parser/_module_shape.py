"""Native-parser module shape facts for harness policy rules."""

from __future__ import annotations

import ast
import io
import tokenize
from typing import TYPE_CHECKING

from .model import PythonModuleShape

if TYPE_CHECKING:
    from ._ast_collector import PythonAstCollector


def collect_module_shape(
    tree: ast.Module,
    source: str,
    collector: PythonAstCollector,
) -> PythonModuleShape:
    """Collect compact module-shape facts from native parser surfaces."""

    groups: set[str] = set()
    top_level_statement_count = 0
    for statement in tree.body:
        if _is_ignored_top_level_statement(statement):
            continue
        top_level_statement_count += 1
        groups.add(_responsibility_group(statement))

    public_symbol_count, public_assignment_count = _public_surface_counts(collector)
    return PythonModuleShape(
        effective_code_lines=_count_effective_code_lines(source),
        top_level_statement_count=top_level_statement_count,
        responsibility_groups=tuple(sorted(groups)),
        public_symbol_count=public_symbol_count,
        public_assignment_count=public_assignment_count,
        public_surface_count=public_symbol_count + public_assignment_count,
    )


def _count_effective_code_lines(source: str) -> int:
    lines: set[int] = set()
    for token in tokenize.generate_tokens(io.StringIO(source).readline):
        if token.type in _NON_CODE_TOKEN_TYPES:
            continue
        lines.add(token.start[0])
    return len(lines)


def _public_surface_counts(collector: PythonAstCollector) -> tuple[int, int]:
    public_symbol_count = sum(
        1 for symbol in collector.symbols if symbol.is_top_level and symbol.is_public
    )
    public_assignment_count = sum(
        1
        for assignment in collector.assignments
        if assignment.is_top_level and assignment.is_public
    )
    return public_symbol_count, public_assignment_count


def _is_ignored_top_level_statement(statement: ast.stmt) -> bool:
    if isinstance(statement, ast.Import | ast.ImportFrom):
        return True
    return (
        isinstance(statement, ast.Expr)
        and isinstance(statement.value, ast.Constant)
        and isinstance(statement.value.value, str)
    )


def _responsibility_group(statement: ast.stmt) -> str:
    if isinstance(statement, ast.ClassDef):
        return "types"
    if isinstance(statement, ast.FunctionDef | ast.AsyncFunctionDef):
        return "functions"
    if isinstance(statement, ast.Assign | ast.AnnAssign | ast.AugAssign):
        return "state"
    if isinstance(
        statement, ast.If | ast.For | ast.AsyncFor | ast.With | ast.AsyncWith
    ):
        return "runtime-control"
    if isinstance(statement, ast.Try | ast.Match):
        return "runtime-control"
    return "other"


_NON_CODE_TOKEN_TYPES = frozenset(
    {
        tokenize.COMMENT,
        tokenize.DEDENT,
        tokenize.ENCODING,
        tokenize.ENDMARKER,
        tokenize.INDENT,
        tokenize.NEWLINE,
        tokenize.NL,
    }
)
