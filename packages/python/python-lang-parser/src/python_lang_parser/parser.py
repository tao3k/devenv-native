"""Python-native AST-backed source parser."""

from __future__ import annotations

import ast
import symtable
import sys
import tokenize
from pathlib import Path

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
        diagnostic = _diagnostic_from_parse_exception(
            exc,
            code="python.syntax.invalid",
            path_text=path_text,
        )
        return PythonModuleReport(
            path=path_text,
            module_docstring=None,
            diagnostics=(diagnostic,),
        )

    compile_diagnostic = _compile_diagnostic(
        source, path_text=path_text, filename=filename
    )
    if compile_diagnostic is not None:
        return PythonModuleReport(
            path=path_text,
            module_docstring=ast.get_docstring(tree),
            diagnostics=(compile_diagnostic,),
        )

    native_scopes, native_bindings = _collect_native_symbol_table(
        source,
        path_text=path_text,
        filename=filename,
    )
    collector = _PythonAstCollector(path_text, source)
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
        diagnostics=(),
        metadata={
            "parser": "cpython.ast",
            "parser_authority": "python-native",
            "python_version": ".".join(str(part) for part in sys.version_info[:3]),
            "symbol_table": "cpython.symtable",
            "language": "python",
        },
    )


class _PythonAstCollector(ast.NodeVisitor):
    def __init__(self, path: str | None, source: str) -> None:
        self._path = path
        self._source = source
        self._scope_stack: list[str] = []
        self.imports: list[PythonImport] = []
        self.symbols: list[PythonSymbol] = []
        self.references: list[PythonReference] = []
        self.calls: list[PythonCall] = []

    @property
    def _scope(self) -> str:
        return ".".join(self._scope_stack)

    def visit_Import(self, node: ast.Import) -> None:
        self.imports.append(
            PythonImport(
                module=None,
                names=tuple(alias.asname or alias.name for alias in node.names),
                level=0,
                scope=self._scope,
                location=self._location(node),
            )
        )
        self.generic_visit(node)

    def visit_ImportFrom(self, node: ast.ImportFrom) -> None:
        self.imports.append(
            PythonImport(
                module=node.module,
                names=tuple(alias.asname or alias.name for alias in node.names),
                level=node.level,
                scope=self._scope,
                location=self._location(node),
            )
        )
        self.generic_visit(node)

    def visit_ClassDef(self, node: ast.ClassDef) -> None:
        self._visit_symbol(node, PythonSymbolKind.CLASS)

    def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
        self._visit_symbol(node, PythonSymbolKind.FUNCTION)

    def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef) -> None:
        self._visit_symbol(node, PythonSymbolKind.ASYNC_FUNCTION)

    def visit_Name(self, node: ast.Name) -> None:
        self.references.append(
            PythonReference(
                name=node.id,
                kind=PythonReferenceKind.NAME,
                scope=self._scope,
                location=self._location(node),
                end_line=_end_line(node),
                end_column=_end_column(node),
                context=_expr_context(node.ctx),
                expression=_source_segment(self._source, node),
            )
        )
        self.generic_visit(node)

    def visit_Attribute(self, node: ast.Attribute) -> None:
        self.references.append(
            PythonReference(
                name=_qualified_expr_name(node),
                kind=PythonReferenceKind.ATTRIBUTE,
                scope=self._scope,
                location=self._location(node),
                end_line=_end_line(node),
                end_column=_end_column(node),
                context=_expr_context(node.ctx),
                expression=_source_segment(self._source, node),
            )
        )
        self.generic_visit(node)

    def visit_Call(self, node: ast.Call) -> None:
        self.calls.append(
            PythonCall(
                function=_qualified_expr_name(node.func),
                scope=self._scope,
                location=self._location(node),
                end_line=_end_line(node),
                end_column=_end_column(node),
                positional_count=len(node.args),
                keyword_names=tuple(keyword.arg or "**" for keyword in node.keywords),
                expression=_source_segment(self._source, node),
            )
        )
        self.generic_visit(node)

    def _visit_symbol(
        self,
        node: ast.ClassDef | ast.FunctionDef | ast.AsyncFunctionDef,
        kind: PythonSymbolKind,
    ) -> None:
        scope = self._scope
        qualified_name = ".".join([*self._scope_stack, node.name])
        self.symbols.append(
            PythonSymbol(
                name=node.name,
                kind=kind,
                qualified_name=qualified_name,
                scope=scope,
                location=self._location(node),
                end_line=getattr(node, "end_lineno", None),
                decorators=tuple(
                    _unparse(decorator) for decorator in node.decorator_list
                ),
                docstring=ast.get_docstring(node),
            )
        )
        self._scope_stack.append(node.name)
        self.generic_visit(node)
        self._scope_stack.pop()

    def _location(self, node: ast.AST) -> SourceLocation:
        return SourceLocation(
            path=self._path,
            line=getattr(node, "lineno", 1),
            column=getattr(node, "col_offset", 0),
        )


def _unparse(node: ast.AST) -> str:
    try:
        return ast.unparse(node)
    except Exception:  # pragma: no cover - defensive fallback for exotic AST nodes.
        return node.__class__.__name__


def _qualified_expr_name(node: ast.AST) -> str:
    if isinstance(node, ast.Name):
        return node.id
    if isinstance(node, ast.Attribute):
        base = _qualified_expr_name(node.value)
        if base:
            return f"{base}.{node.attr}"
        return node.attr
    if isinstance(node, ast.Call):
        return _qualified_expr_name(node.func)
    if isinstance(node, ast.Subscript):
        return _qualified_expr_name(node.value)
    return _unparse(node)


def _source_segment(source: str, node: ast.AST) -> str | None:
    try:
        return ast.get_source_segment(source, node)
    except Exception:  # pragma: no cover - defensive fallback for exotic AST nodes.
        return None


def _expr_context(context: ast.expr_context) -> str:
    if isinstance(context, ast.Load):
        return "load"
    if isinstance(context, ast.Store):
        return "store"
    if isinstance(context, ast.Del):
        return "del"
    return context.__class__.__name__.lower()


def _end_line(node: ast.AST) -> int | None:
    return getattr(node, "end_lineno", None)


def _end_column(node: ast.AST) -> int | None:
    return getattr(node, "end_col_offset", None)


def _compile_diagnostic(
    source: str,
    *,
    path_text: str | None,
    filename: str,
) -> PythonDiagnostic | None:
    try:
        compile(source, filename, "exec", dont_inherit=True, optimize=0)
    except SyntaxError as exc:
        return _diagnostic_from_parse_exception(
            exc,
            code="python.compile.invalid",
            path_text=path_text,
        )
    return None


def _diagnostic_from_parse_exception(
    exc: SyntaxError | ValueError,
    *,
    code: str,
    path_text: str | None,
) -> PythonDiagnostic:
    if isinstance(exc, SyntaxError):
        location = SourceLocation(
            path=exc.filename,
            line=exc.lineno or 1,
            column=max((exc.offset or 1) - 1, 0),
        )
        return PythonDiagnostic(
            code=code,
            severity=PythonDiagnosticSeverity.ERROR,
            message=exc.msg,
            location=location,
            source_line=(exc.text or "").rstrip("\n") or None,
        )
    return PythonDiagnostic(
        code=code,
        severity=PythonDiagnosticSeverity.ERROR,
        message=str(exc),
        location=SourceLocation(path=path_text, line=1, column=0),
    )


def _collect_native_symbol_table(
    source: str,
    *,
    path_text: str | None,
    filename: str,
) -> tuple[tuple[PythonScope, ...], tuple[PythonNameBinding, ...]]:
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
