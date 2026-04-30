"""AST collection helpers for Python native parser reports."""

from __future__ import annotations

import ast

from ._ast_annotations import symbol_has_annotations
from ._ast_names import (
    end_column,
    end_line,
    expr_context,
    is_public_name,
    iter_assignment_target_nodes,
    qualified_expr_name,
    source_segment,
    target_assigns_name,
    unparse,
)
from ._call_effects import call_effect
from ._export_model import PythonExportContract, PythonExportContractKind
from ._exports import literal_string_sequence
from .model import (
    PythonAssignmentTarget,
    PythonCall,
    PythonImport,
    PythonReference,
    PythonReferenceKind,
    PythonSymbol,
    PythonSymbolKind,
    SourceLocation,
)


class PythonAstCollector(ast.NodeVisitor):
    """Collect compact AST-backed module facts."""

    def __init__(self, path: str | None, source: str) -> None:
        self._path = path
        self._source = source
        self._scope_stack: list[str] = []
        self.imports: list[PythonImport] = []
        self.symbols: list[PythonSymbol] = []
        self.references: list[PythonReference] = []
        self.calls: list[PythonCall] = []
        self.assignments: list[PythonAssignmentTarget] = []
        self.export_contract = PythonExportContract(
            kind=PythonExportContractKind.INFERRED
        )

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
                is_wildcard=False,
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
                is_wildcard=any(alias.name == "*" for alias in node.names),
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
                end_line=end_line(node),
                end_column=end_column(node),
                context=expr_context(node.ctx),
                expression=source_segment(self._source, node),
            )
        )
        self.generic_visit(node)

    def visit_Attribute(self, node: ast.Attribute) -> None:
        self.references.append(
            PythonReference(
                name=qualified_expr_name(node),
                kind=PythonReferenceKind.ATTRIBUTE,
                scope=self._scope,
                location=self._location(node),
                end_line=end_line(node),
                end_column=end_column(node),
                context=expr_context(node.ctx),
                expression=source_segment(self._source, node),
            )
        )
        self.generic_visit(node)

    def visit_Call(self, node: ast.Call) -> None:
        self.calls.append(
            PythonCall(
                function=qualified_expr_name(node.func),
                scope=self._scope,
                location=self._location(node),
                end_line=end_line(node),
                end_column=end_column(node),
                positional_count=len(node.args),
                keyword_names=tuple(keyword.arg or "**" for keyword in node.keywords),
                expression=source_segment(self._source, node),
                effect=call_effect(qualified_expr_name(node.func)),
            )
        )
        self.generic_visit(node)

    def visit_Assign(self, node: ast.Assign) -> None:
        self._visit_export_contract(node.targets, "assign", node.value, node)
        for target in node.targets:
            self._visit_assignment_target(
                target,
                "assign",
                value_expression=source_segment(self._source, node.value),
            )
        self.generic_visit(node)

    def visit_AnnAssign(self, node: ast.AnnAssign) -> None:
        self._visit_export_contract(
            (node.target,), "annotated_assign", node.value, node
        )
        self._visit_assignment_target(
            node.target,
            "annotated_assign",
            value_expression=(
                source_segment(self._source, node.value)
                if node.value is not None
                else None
            ),
        )
        self.generic_visit(node)

    def visit_AugAssign(self, node: ast.AugAssign) -> None:
        self._visit_export_contract((node.target,), "aug_assign", node.value, node)
        self._visit_assignment_target(
            node.target,
            "aug_assign",
            value_expression=source_segment(self._source, node.value),
        )
        self.generic_visit(node)

    def visit_For(self, node: ast.For) -> None:
        self._visit_assignment_target(
            node.target,
            "for",
            value_expression=source_segment(self._source, node.iter),
        )
        self.generic_visit(node)

    def visit_AsyncFor(self, node: ast.AsyncFor) -> None:
        self._visit_assignment_target(
            node.target,
            "async_for",
            value_expression=source_segment(self._source, node.iter),
        )
        self.generic_visit(node)

    def visit_With(self, node: ast.With) -> None:
        for item in node.items:
            if item.optional_vars is not None:
                self._visit_assignment_target(
                    item.optional_vars,
                    "with",
                    value_expression=source_segment(self._source, item.context_expr),
                )
        self.generic_visit(node)

    def visit_AsyncWith(self, node: ast.AsyncWith) -> None:
        for item in node.items:
            if item.optional_vars is not None:
                self._visit_assignment_target(
                    item.optional_vars,
                    "async_with",
                    value_expression=source_segment(self._source, item.context_expr),
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
                    unparse(decorator) for decorator in node.decorator_list
                ),
                docstring=ast.get_docstring(node),
                has_annotations=symbol_has_annotations(node),
                is_public=is_public_name(node.name),
                is_top_level=scope == "",
            )
        )
        self._scope_stack.append(node.name)
        self.generic_visit(node)
        self._scope_stack.pop()

    def _visit_assignment_target(
        self,
        target: ast.AST,
        target_kind: str,
        *,
        value_expression: str | None,
    ) -> None:
        for item in iter_assignment_target_nodes(target):
            name = qualified_expr_name(item)
            self.assignments.append(
                PythonAssignmentTarget(
                    name=name,
                    scope=self._scope,
                    location=self._location(item),
                    end_line=end_line(item),
                    end_column=end_column(item),
                    target_kind=target_kind,
                    expression=source_segment(self._source, item),
                    value_expression=value_expression,
                    is_public=is_public_name(name),
                    is_top_level=self._scope == "",
                )
            )

    def _visit_export_contract(
        self,
        targets: tuple[ast.AST, ...] | list[ast.AST],
        target_kind: str,
        value: ast.AST | None,
        node: ast.AST,
    ) -> None:
        if self._scope != "" or not any(
            target_assigns_name(target, "__all__") for target in targets
        ):
            return
        if self.export_contract.kind == PythonExportContractKind.DYNAMIC:
            return
        if target_kind not in {"assign", "annotated_assign"}:
            self.export_contract = PythonExportContract(
                kind=PythonExportContractKind.DYNAMIC,
                location=self._location(node),
            )
            return
        literal_exports = literal_string_sequence(value)
        if literal_exports is None:
            self.export_contract = PythonExportContract(
                kind=PythonExportContractKind.DYNAMIC,
                location=self._location(node),
            )
            return
        self.export_contract = PythonExportContract(
            kind=PythonExportContractKind.STATIC,
            names=literal_exports,
            location=self._location(node),
        )

    def _location(self, node: ast.AST) -> SourceLocation:
        return SourceLocation(
            path=self._path,
            line=getattr(node, "lineno", 1),
            column=getattr(node, "col_offset", 0),
        )
