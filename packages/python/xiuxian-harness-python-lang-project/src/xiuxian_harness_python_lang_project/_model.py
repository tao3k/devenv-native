"""Data model for embedded Python language harness reports."""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from typing import TYPE_CHECKING, Protocol

from ._constants import DEFAULT_BLOCKING_SEVERITIES, IGNORED_DIR_NAMES

if TYPE_CHECKING:
    from collections.abc import Iterable
    from pathlib import Path

    from python_lang_parser import (
        PythonDiagnosticSeverity,
        PythonModuleReport,
        SourceLocation,
    )


@dataclass(frozen=True, slots=True)
class PythonRulePackDescriptor:
    """Stable metadata for one Python language harness rule pack."""

    id: str
    version: str
    domains: tuple[str, ...]
    default_mode: str = "deterministic"

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-compatible representation."""

        payload = asdict(self)
        payload["domains"] = list(self.domains)
        return payload


@dataclass(frozen=True, slots=True)
class PythonHarnessRule:
    """Compact metadata for one deterministic harness rule."""

    rule_id: str
    pack_id: str
    severity: PythonDiagnosticSeverity
    title: str
    requirement: str
    labels: dict[str, str] = field(default_factory=dict)

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-compatible representation."""

        payload = asdict(self)
        payload["severity"] = self.severity.value
        return payload


@dataclass(frozen=True, slots=True)
class PythonHarnessFinding:
    """One deterministic Python harness finding."""

    rule_id: str
    pack_id: str
    severity: PythonDiagnosticSeverity
    title: str
    summary: str
    location: SourceLocation
    requirement: str
    source_line: str | None = None
    label: str = "repair Python syntax near this token"
    labels: dict[str, str] = field(default_factory=dict)

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-compatible representation."""

        payload = asdict(self)
        payload["severity"] = self.severity.value
        return payload


@dataclass(frozen=True, slots=True)
class PythonProjectHarnessScope:
    """Concrete project paths monitored by an embedded Python harness run."""

    project_root: Path
    source_paths: tuple[Path, ...]
    test_paths: tuple[Path, ...]
    fallback_paths: tuple[Path, ...] = ()

    @property
    def monitored_paths(self) -> tuple[Path, ...]:
        """Return the concrete roots scanned by the parser and rule packs."""

        selected = (*self.source_paths, *self.test_paths)
        if selected:
            return selected
        return self.fallback_paths

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-compatible representation."""

        return {
            "project_root": str(self.project_root),
            "source_paths": [str(path) for path in self.source_paths],
            "test_paths": [str(path) for path in self.test_paths],
            "monitored_paths": [str(path) for path in self.monitored_paths],
        }


class PythonLangRulePack(Protocol):
    """Protocol for Python language harness rule packs."""

    pack_id: str

    def descriptor(self) -> PythonRulePackDescriptor:
        """Return stable metadata for this rule pack."""

    def evaluate(self, report: PythonModuleReport) -> Iterable[PythonHarnessFinding]:
        """Evaluate one parsed module report."""


@dataclass(frozen=True, slots=True)
class PythonHarnessConfig:
    """Configuration for an embedded Python language harness run."""

    ignored_dir_names: frozenset[str] = IGNORED_DIR_NAMES
    blocking_severities: frozenset[PythonDiagnosticSeverity] = (
        DEFAULT_BLOCKING_SEVERITIES
    )
    rule_packs: tuple[PythonLangRulePack, ...] | None = None


@dataclass(frozen=True, slots=True)
class PythonHarnessReport:
    """Aggregated Python language harness report."""

    modules: tuple[PythonModuleReport, ...]
    findings: tuple[PythonHarnessFinding, ...]
    root_paths: tuple[str, ...]
    blocking_severities: frozenset[PythonDiagnosticSeverity] = (
        DEFAULT_BLOCKING_SEVERITIES
    )
    project_scope: PythonProjectHarnessScope | None = None

    @property
    def parsed_count(self) -> int:
        """Return the number of parser-clean modules."""

        return sum(1 for module in self.modules if module.is_valid)

    @property
    def file_count(self) -> int:
        """Return the number of modules included in the report."""

        return len(self.modules)

    @property
    def is_clean(self) -> bool:
        """Return whether the report contains no configured-blocking findings."""

        return not self.blocking_findings()

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-compatible representation."""

        return {
            "root_paths": list(self.root_paths),
            "project_scope": (
                None if self.project_scope is None else self.project_scope.to_dict()
            ),
            "file_count": self.file_count,
            "parsed_count": self.parsed_count,
            "is_clean": self.is_clean,
            "blocking_severities": [
                severity.value for severity in sorted(self.blocking_severities)
            ],
            "findings": [finding.to_dict() for finding in self.findings],
            "modules": [module.to_dict() for module in self.modules],
        }

    def blocking_findings(
        self,
        *,
        severities: frozenset[PythonDiagnosticSeverity] | None = None,
    ) -> tuple[PythonHarnessFinding, ...]:
        """Return findings that should block a pytest assertion."""

        blocking_severities = (
            self.blocking_severities if severities is None else severities
        )
        return tuple(
            finding
            for finding in self.findings
            if finding.severity in blocking_severities
        )

    def assert_clean(
        self,
        *,
        severities: frozenset[PythonDiagnosticSeverity] | None = None,
    ) -> None:
        """Raise `AssertionError` when blocking findings are present."""

        if self.blocking_findings(severities=severities):
            from ._render import render_python_lang_harness

            raise AssertionError(
                render_python_lang_harness(self, severities=severities)
            )
