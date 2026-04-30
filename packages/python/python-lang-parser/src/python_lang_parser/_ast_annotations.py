"""Native AST annotation fact collection."""

from __future__ import annotations

import ast


def module_has_annotations(tree: ast.AST) -> bool:
    """Return whether a parsed module carries any annotation marker."""

    return any(_node_has_annotation_marker(node) for node in ast.walk(tree))


def symbol_has_annotations(
    node: ast.ClassDef | ast.FunctionDef | ast.AsyncFunctionDef,
) -> bool:
    """Return whether one class or function symbol carries annotations."""

    if isinstance(node, ast.FunctionDef | ast.AsyncFunctionDef):
        return _function_has_annotations(node)
    return any(_node_has_annotation_marker(item) for item in node.body)


def _function_has_annotations(node: ast.FunctionDef | ast.AsyncFunctionDef) -> bool:
    return (
        node.returns is not None
        or node.type_comment is not None
        or any(_arg_has_annotation(item) for item in _iter_function_args(node.args))
    )


def _iter_function_args(arguments: ast.arguments) -> tuple[ast.arg, ...]:
    items = [
        *arguments.posonlyargs,
        *arguments.args,
        *arguments.kwonlyargs,
    ]
    if arguments.vararg is not None:
        items.append(arguments.vararg)
    if arguments.kwarg is not None:
        items.append(arguments.kwarg)
    return tuple(items)


def _arg_has_annotation(arg: ast.arg) -> bool:
    return arg.annotation is not None or arg.type_comment is not None


def _node_has_annotation_marker(node: ast.AST) -> bool:
    if isinstance(node, ast.AnnAssign):
        return True
    if isinstance(node, ast.Assign | ast.For | ast.AsyncFor | ast.With | ast.AsyncWith):
        return getattr(node, "type_comment", None) is not None
    if isinstance(node, ast.FunctionDef | ast.AsyncFunctionDef):
        return _function_has_annotations(node)
    return False
