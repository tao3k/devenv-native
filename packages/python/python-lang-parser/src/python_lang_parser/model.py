"""Data model for Python native-syntax parser reports."""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from enum import StrEnum
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from collections.abc import Mapping


class PythonDiagnosticSeverity(StrEnum):
    """Severity levels used by parser diagnostics and harness findings."""

    INFO = "info"
    WARNING = "warning"
    ERROR = "error"


class PythonSymbolKind(StrEnum):
    """Python symbol categories collected from the AST."""

    CLASS = "class"
    FUNCTION = "function"
    ASYNC_FUNCTION = "async_function"


class PythonReferenceKind(StrEnum):
    """Python reference categories collected from the AST."""

    NAME = "name"
    ATTRIBUTE = "attribute"


@dataclass(frozen=True, slots=True)
class SourceLocation:
    """A source location inside an optional file."""

    path: str | None
    line: int
    column: int

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-compatible representation."""

        return asdict(self)


@dataclass(frozen=True, slots=True)
class PythonDiagnostic:
    """One parser diagnostic."""

    code: str
    severity: PythonDiagnosticSeverity
    message: str
    location: SourceLocation
    source_line: str | None = None
    label: str = "repair Python syntax near this token"
    help: str = "Fix Python syntax before running the harness."

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-compatible representation."""

        payload = asdict(self)
        payload["severity"] = self.severity.value
        return payload


@dataclass(frozen=True, slots=True)
class PythonImport:
    """One import statement collected from a Python module."""

    module: str | None
    names: tuple[str, ...]
    level: int
    scope: str
    location: SourceLocation

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-compatible representation."""

        payload = asdict(self)
        payload["names"] = list(self.names)
        return payload


@dataclass(frozen=True, slots=True)
class PythonSymbol:
    """One class or function symbol collected from a Python module."""

    name: str
    kind: PythonSymbolKind
    qualified_name: str
    scope: str
    location: SourceLocation
    end_line: int | None
    decorators: tuple[str, ...] = field(default_factory=tuple)
    docstring: str | None = None

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-compatible representation."""

        payload = asdict(self)
        payload["kind"] = self.kind.value
        payload["decorators"] = list(self.decorators)
        return payload


@dataclass(frozen=True, slots=True)
class PythonScope:
    """One native Python compiler symbol-table scope."""

    id: str
    name: str
    kind: str
    parent_id: str | None
    location: SourceLocation
    identifiers: tuple[str, ...]
    nested: bool
    optimized: bool

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-compatible representation."""

        payload = asdict(self)
        payload["identifiers"] = list(self.identifiers)
        return payload


@dataclass(frozen=True, slots=True)
class PythonNameBinding:
    """One name binding from Python's native compiler symbol table."""

    name: str
    scope_id: str
    scope_name: str
    scope_kind: str
    flags: tuple[str, ...]
    namespace_ids: tuple[str, ...] = field(default_factory=tuple)

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-compatible representation."""

        payload = asdict(self)
        payload["flags"] = list(self.flags)
        payload["namespace_ids"] = list(self.namespace_ids)
        return payload


@dataclass(frozen=True, slots=True)
class PythonReference:
    """One AST-level Python name or attribute reference."""

    name: str
    kind: PythonReferenceKind
    scope: str
    location: SourceLocation
    end_line: int | None
    end_column: int | None
    context: str
    expression: str | None = None

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-compatible representation."""

        payload = asdict(self)
        payload["kind"] = self.kind.value
        return payload


@dataclass(frozen=True, slots=True)
class PythonCall:
    """One AST-level Python call site."""

    function: str
    scope: str
    location: SourceLocation
    end_line: int | None
    end_column: int | None
    positional_count: int
    keyword_names: tuple[str, ...]
    expression: str | None = None

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-compatible representation."""

        payload = asdict(self)
        payload["keyword_names"] = list(self.keyword_names)
        return payload


@dataclass(frozen=True, slots=True)
class PythonModuleReport:
    """Structured parser report for one Python module."""

    path: str | None
    module_docstring: str | None
    imports: tuple[PythonImport, ...] = field(default_factory=tuple)
    symbols: tuple[PythonSymbol, ...] = field(default_factory=tuple)
    scopes: tuple[PythonScope, ...] = field(default_factory=tuple)
    bindings: tuple[PythonNameBinding, ...] = field(default_factory=tuple)
    references: tuple[PythonReference, ...] = field(default_factory=tuple)
    calls: tuple[PythonCall, ...] = field(default_factory=tuple)
    diagnostics: tuple[PythonDiagnostic, ...] = field(default_factory=tuple)
    metadata: Mapping[str, str] = field(default_factory=dict)

    @property
    def is_valid(self) -> bool:
        """Return whether parsing completed without error diagnostics."""

        return not any(
            diagnostic.severity == PythonDiagnosticSeverity.ERROR
            for diagnostic in self.diagnostics
        )

    def to_dict(self) -> dict[str, Any]:
        """Return a JSON-compatible representation."""

        return {
            "path": self.path,
            "module_docstring": self.module_docstring,
            "imports": [item.to_dict() for item in self.imports],
            "symbols": [item.to_dict() for item in self.symbols],
            "scopes": [item.to_dict() for item in self.scopes],
            "bindings": [item.to_dict() for item in self.bindings],
            "references": [item.to_dict() for item in self.references],
            "calls": [item.to_dict() for item in self.calls],
            "diagnostics": [item.to_dict() for item in self.diagnostics],
            "metadata": dict(self.metadata),
            "is_valid": self.is_valid,
        }
