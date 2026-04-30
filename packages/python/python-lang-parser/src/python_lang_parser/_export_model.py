"""Export contract model for Python native parser reports."""

from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from ._diagnostic_model import SourceLocation


class PythonExportContractKind(StrEnum):
    """How a module export surface was determined."""

    INFERRED = "inferred"
    STATIC = "static"
    DYNAMIC = "dynamic"


@dataclass(frozen=True, slots=True)
class PythonExportContract:
    """Structured module export contract recognized by the native parser."""

    kind: PythonExportContractKind
    names: tuple[str, ...] = ()
    location: SourceLocation | None = None

    @property
    def is_explicit(self) -> bool:
        """Return whether the module declares `__all__`."""

        return self.kind != PythonExportContractKind.INFERRED

    @property
    def is_static(self) -> bool:
        """Return whether the explicit export contract is literal and complete."""

        return self.kind == PythonExportContractKind.STATIC

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-compatible representation."""

        return {
            "kind": self.kind.value,
            "names": list(self.names),
            "location": None if self.location is None else self.location.to_dict(),
            "is_explicit": self.is_explicit,
            "is_static": self.is_static,
        }
