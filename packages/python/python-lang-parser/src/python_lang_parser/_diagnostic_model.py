"""Diagnostic model objects for Python native parser reports."""

from __future__ import annotations

from dataclasses import asdict, dataclass
from enum import StrEnum


class PythonDiagnosticSeverity(StrEnum):
    """Severity levels used by parser diagnostics and harness findings."""

    INFO = "info"
    WARNING = "warning"
    ERROR = "error"


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
