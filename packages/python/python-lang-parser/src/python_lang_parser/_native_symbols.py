"""CPython symbol-table collection helpers."""

from __future__ import annotations

import symtable

from .model import PythonNameBinding, PythonScope, SourceLocation

_SYMBOL_FLAG_METHODS = (
    ("referenced", "is_referenced"),
    ("imported", "is_imported"),
    ("parameter", "is_parameter"),
    ("type_parameter", "is_type_parameter"),
    ("global", "is_global"),
    ("declared_global", "is_declared_global"),
    ("nonlocal", "is_nonlocal"),
    ("local", "is_local"),
    ("annotated", "is_annotated"),
    ("free", "is_free"),
    ("cell", "is_cell"),
    ("free_class", "is_free_class"),
    ("assigned", "is_assigned"),
    ("comp_iter", "is_comp_iter"),
    ("comp_cell", "is_comp_cell"),
    ("namespace", "is_namespace"),
)


def collect_native_symbol_table(
    source: str,
    *,
    path_text: str | None,
    filename: str,
) -> tuple[tuple[PythonScope, ...], tuple[PythonNameBinding, ...]]:
    """Collect native CPython symbol-table scopes and bindings."""

    table = symtable.symtable(source, filename, "exec")
    scopes: list[PythonScope] = []
    bindings: list[PythonNameBinding] = []
    _collect_scope(
        table, path_text=path_text, parent_id=None, scopes=scopes, bindings=bindings
    )
    return tuple(scopes), tuple(bindings)


def _collect_scope(
    table: symtable.SymbolTable,
    *,
    path_text: str | None,
    parent_id: str | None,
    scopes: list[PythonScope],
    bindings: list[PythonNameBinding],
) -> None:
    scope_id = str(table.get_id())
    scope_kind = _scope_kind(table)
    identifiers = tuple(sorted(table.get_identifiers()))
    scopes.append(
        PythonScope(
            id=scope_id,
            name=table.get_name(),
            kind=scope_kind,
            parent_id=parent_id,
            location=SourceLocation(
                path=path_text,
                line=max(table.get_lineno(), 1),
                column=0,
            ),
            identifiers=identifiers,
            nested=table.is_nested(),
            optimized=table.is_optimized(),
        )
    )
    for name in identifiers:
        symbol = table.lookup(name)
        namespace_ids = tuple(
            str(namespace.get_id()) for namespace in symbol.get_namespaces()
        )
        bindings.append(
            PythonNameBinding(
                name=name,
                scope_id=scope_id,
                scope_name=table.get_name(),
                scope_kind=scope_kind,
                flags=_symbol_flags(symbol),
                namespace_ids=namespace_ids,
            )
        )
    for child in table.get_children():
        _collect_scope(
            child,
            path_text=path_text,
            parent_id=scope_id,
            scopes=scopes,
            bindings=bindings,
        )


def _scope_kind(table: symtable.SymbolTable) -> str:
    raw_kind = table.get_type()
    return getattr(raw_kind, "value", str(raw_kind))


def _symbol_flags(symbol: symtable.Symbol) -> tuple[str, ...]:
    flags: list[str] = []
    for flag, method_name in _SYMBOL_FLAG_METHODS:
        method = getattr(symbol, method_name, None)
        if method is not None and method():
            flags.append(flag)
    return tuple(flags)
