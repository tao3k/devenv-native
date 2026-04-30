"""Native AST name and source helpers."""

from __future__ import annotations

import ast


def unparse(node: ast.AST) -> str:
    """Return a compact Python expression for an AST node."""

    try:
        return ast.unparse(node)
    except Exception:  # pragma: no cover - defensive fallback for exotic AST nodes.
        return node.__class__.__name__


def qualified_expr_name(node: ast.AST) -> str:
    """Return the parser-recognized dotted name for an expression."""

    if isinstance(node, ast.Name):
        return node.id
    if isinstance(node, ast.Attribute):
        base = qualified_expr_name(node.value)
        if base:
            return f"{base}.{node.attr}"
        return node.attr
    if isinstance(node, ast.Call):
        return qualified_expr_name(node.func)
    if isinstance(node, ast.Subscript):
        return qualified_expr_name(node.value)
    return unparse(node)


def source_segment(source: str, node: ast.AST) -> str | None:
    """Return the source segment for an AST node when CPython exposes it."""

    try:
        return ast.get_source_segment(source, node)
    except Exception:  # pragma: no cover - defensive fallback for exotic AST nodes.
        return None


def iter_assignment_target_nodes(target: ast.AST) -> tuple[ast.AST, ...]:
    """Return concrete name-like target nodes from an assignment target."""

    if isinstance(target, ast.Starred):
        return iter_assignment_target_nodes(target.value)
    if isinstance(target, ast.Tuple | ast.List):
        nodes: list[ast.AST] = []
        for element in target.elts:
            nodes.extend(iter_assignment_target_nodes(element))
        return tuple(nodes)
    if isinstance(target, ast.Name | ast.Attribute | ast.Subscript):
        return (target,)
    return ()


def target_assigns_name(target: ast.AST, name: str) -> bool:
    """Return whether an assignment target binds one exact name."""

    return any(
        isinstance(item, ast.Name) and item.id == name
        for item in iter_assignment_target_nodes(target)
    )


def is_public_name(name: str) -> bool:
    """Return whether a parser-visible name is part of public project surface."""

    return bool(name) and not name.startswith(("_", "test_")) and "." not in name


def expr_context(context: ast.expr_context) -> str:
    """Return a compact expression-context label."""

    if isinstance(context, ast.Load):
        return "load"
    if isinstance(context, ast.Store):
        return "store"
    if isinstance(context, ast.Del):
        return "del"
    return context.__class__.__name__.lower()


def end_line(node: ast.AST) -> int | None:
    """Return the native AST end line when available."""

    return getattr(node, "end_lineno", None)


def end_column(node: ast.AST) -> int | None:
    """Return the native AST end column when available."""

    return getattr(node, "end_col_offset", None)
