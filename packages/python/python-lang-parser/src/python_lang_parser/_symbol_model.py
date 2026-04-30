"""Symbol and shape model objects for Python native parser reports."""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from enum import StrEnum
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from ._diagnostic_model import SourceLocation


class PythonSymbolKind(StrEnum):
    """Python symbol categories collected from the AST."""

    CLASS = "class"
    FUNCTION = "function"
    ASYNC_FUNCTION = "async_function"


class PythonReferenceKind(StrEnum):
    """Python reference categories collected from the AST."""

    NAME = "name"
    ATTRIBUTE = "attribute"


class PythonCallEffect(StrEnum):
    """Known runtime effects for parser-recognized Python call sites."""

    UNKNOWN = "unknown"
    STANDARD_OUTPUT = "standard_output"
    DEBUG_BREAKPOINT = "debug_breakpoint"


@dataclass(frozen=True, slots=True)
class PythonImport:
    """One import statement collected from a Python module."""

    module: str | None
    names: tuple[str, ...]
    level: int
    scope: str
    location: SourceLocation
    is_wildcard: bool = False

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
    has_annotations: bool = False
    is_public: bool = False
    is_top_level: bool = False

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
    effect: PythonCallEffect = PythonCallEffect.UNKNOWN

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-compatible representation."""

        payload = asdict(self)
        payload["keyword_names"] = list(self.keyword_names)
        payload["effect"] = self.effect.value
        return payload


@dataclass(frozen=True, slots=True)
class PythonAssignmentTarget:
    """One AST-level assignment target."""

    name: str
    scope: str
    location: SourceLocation
    end_line: int | None
    end_column: int | None
    target_kind: str
    expression: str | None = None
    value_expression: str | None = None
    is_public: bool = False
    is_top_level: bool = False

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-compatible representation."""

        return asdict(self)


@dataclass(frozen=True, slots=True)
class PythonModuleShape:
    """Native-parser module shape facts for project policy rules."""

    effective_code_lines: int
    top_level_statement_count: int
    responsibility_groups: tuple[str, ...]
    public_symbol_count: int
    public_assignment_count: int
    public_surface_count: int

    @property
    def responsibility_group_count(self) -> int:
        """Return the number of top-level responsibility groups."""

        return len(self.responsibility_groups)

    @property
    def has_public_surface(self) -> bool:
        """Return whether the module exposes parser-recognized public surface."""

        return self.public_surface_count > 0

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-compatible representation."""

        return {
            "effective_code_lines": self.effective_code_lines,
            "top_level_statement_count": self.top_level_statement_count,
            "responsibility_groups": list(self.responsibility_groups),
            "responsibility_group_count": self.responsibility_group_count,
            "public_symbol_count": self.public_symbol_count,
            "public_assignment_count": self.public_assignment_count,
            "public_surface_count": self.public_surface_count,
            "has_public_surface": self.has_public_surface,
        }
