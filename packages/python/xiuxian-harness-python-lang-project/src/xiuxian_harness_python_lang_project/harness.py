"""Harness runner for Python native-syntax parser reports."""

from __future__ import annotations

from dataclasses import asdict, dataclass, field, replace
from pathlib import Path
from typing import TYPE_CHECKING, Protocol

from python_lang_parser import (
    PythonDiagnosticSeverity,
    PythonModuleReport,
    SourceLocation,
    parse_python_file,
)

if TYPE_CHECKING:
    from collections.abc import Iterable, Sequence

_IGNORED_DIR_NAMES = frozenset(
    {
        ".git",
        ".hg",
        ".mypy_cache",
        ".pytest_cache",
        ".ruff_cache",
        ".tox",
        ".venv",
        "__pycache__",
        "build",
        "dist",
        "node_modules",
        "target",
        "venv",
    }
)
_DEFAULT_BLOCKING_SEVERITIES = frozenset(
    {
        PythonDiagnosticSeverity.ERROR,
        PythonDiagnosticSeverity.WARNING,
    }
)
_MODERN_DESIGN_PACK_ID = "python.modern_design"
_PY_MOD_R001 = "PY-MOD-R001"
_PY_MOD_R002 = "PY-MOD-R002"
_PY_MOD_R003 = "PY-MOD-R003"
_PY_MOD_R004 = "PY-MOD-R004"


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


_MODERN_DESIGN_RULE_LABELS = {
    "language": "python",
    "domain": "modern-python",
}
_MODERN_DESIGN_RULES = (
    PythonHarnessRule(
        rule_id=_PY_MOD_R001,
        pack_id=_MODERN_DESIGN_PACK_ID,
        severity=PythonDiagnosticSeverity.WARNING,
        title="Wildcard import hides the dependency surface",
        requirement="Import explicit names instead of `*` in project modules.",
        labels=dict(_MODERN_DESIGN_RULE_LABELS),
    ),
    PythonHarnessRule(
        rule_id=_PY_MOD_R002,
        pack_id=_MODERN_DESIGN_PACK_ID,
        severity=PythonDiagnosticSeverity.WARNING,
        title="Library module uses bare print",
        requirement="Use a logger, returned value, or explicit test assertion instead of bare `print` in library modules.",
        labels=dict(_MODERN_DESIGN_RULE_LABELS),
    ),
    PythonHarnessRule(
        rule_id=_PY_MOD_R003,
        pack_id=_MODERN_DESIGN_PACK_ID,
        severity=PythonDiagnosticSeverity.WARNING,
        title="Package facade re-exports without __all__",
        requirement="Declare `__all__` beside package facade imports so public exports stay explicit.",
        labels=dict(_MODERN_DESIGN_RULE_LABELS),
    ),
    PythonHarnessRule(
        rule_id=_PY_MOD_R004,
        pack_id=_MODERN_DESIGN_PACK_ID,
        severity=PythonDiagnosticSeverity.WARNING,
        title="Library module contains breakpoint()",
        requirement="Remove `breakpoint()` from library modules; use test-only debug tooling or a project-owned diagnostic surface.",
        labels=dict(_MODERN_DESIGN_RULE_LABELS),
    ),
)
_MODERN_DESIGN_RULE_BY_ID = {rule.rule_id: rule for rule in _MODERN_DESIGN_RULES}


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


class PythonLangRulePack(Protocol):
    """Protocol for Python language harness rule packs."""

    pack_id: str

    def descriptor(self) -> PythonRulePackDescriptor:
        """Return stable metadata for this rule pack."""

    def evaluate(self, report: PythonModuleReport) -> Iterable[PythonHarnessFinding]:
        """Evaluate one parsed module report."""


@dataclass(frozen=True, slots=True)
class PythonSyntaxRulePack:
    """Rule pack that turns parser diagnostics into harness findings."""

    pack_id: str = "python.syntax"

    def descriptor(self) -> PythonRulePackDescriptor:
        """Return stable metadata for this rule pack."""

        return PythonRulePackDescriptor(
            id=self.pack_id,
            version="v1",
            domains=("syntax", "python"),
        )

    def evaluate(self, report: PythonModuleReport) -> Iterable[PythonHarnessFinding]:
        """Evaluate parse diagnostics for one module report."""

        for diagnostic in report.diagnostics:
            if diagnostic.severity != PythonDiagnosticSeverity.ERROR:
                continue
            yield PythonHarnessFinding(
                rule_id=diagnostic.code,
                pack_id=self.pack_id,
                severity=diagnostic.severity,
                title="Python source did not parse",
                summary=diagnostic.message,
                location=diagnostic.location,
                requirement="Python modules must parse with CPython native syntax before project rules run.",
                source_line=diagnostic.source_line,
                label=diagnostic.message or diagnostic.label,
                labels={"language": "python"},
            )


@dataclass(frozen=True, slots=True)
class PythonModernDesignRulePack:
    """Numbered modern Python design rules backed by native parser reports."""

    pack_id: str = _MODERN_DESIGN_PACK_ID

    def descriptor(self) -> PythonRulePackDescriptor:
        """Return stable metadata for this rule pack."""

        return PythonRulePackDescriptor(
            id=self.pack_id,
            version="v1",
            domains=("modern-python", "design", "python"),
        )

    def evaluate(self, report: PythonModuleReport) -> Iterable[PythonHarnessFinding]:
        """Evaluate modern Python design rules for one parsed module report."""

        if not report.is_valid:
            return ()

        findings: list[PythonHarnessFinding] = []
        findings.extend(_wildcard_import_findings(report, self.pack_id))
        findings.extend(_bare_print_findings(report, self.pack_id))
        findings.extend(_debug_breakpoint_findings(report, self.pack_id))
        findings.extend(_facade_all_findings(report, self.pack_id))
        return tuple(findings)


def default_python_lang_rule_packs() -> tuple[PythonLangRulePack, ...]:
    """Return the default deterministic Python language rule packs."""

    return (PythonSyntaxRulePack(), PythonModernDesignRulePack())


@dataclass(frozen=True, slots=True)
class PythonHarnessConfig:
    """Configuration for an embedded Python language harness run."""

    ignored_dir_names: frozenset[str] = _IGNORED_DIR_NAMES
    blocking_severities: frozenset[PythonDiagnosticSeverity] = (
        _DEFAULT_BLOCKING_SEVERITIES
    )
    rule_packs: tuple[PythonLangRulePack, ...] | None = None


def default_python_harness_config() -> PythonHarnessConfig:
    """Return the default Python language harness configuration."""

    return PythonHarnessConfig(rule_packs=default_python_lang_rule_packs())


def python_modern_design_rules() -> tuple[PythonHarnessRule, ...]:
    """Return compact metadata for the default modern-design rules."""

    return tuple(
        replace(rule, labels=dict(rule.labels)) for rule in _MODERN_DESIGN_RULES
    )


@dataclass(frozen=True, slots=True)
class PythonHarnessReport:
    """Aggregated Python language harness report."""

    modules: tuple[PythonModuleReport, ...]
    findings: tuple[PythonHarnessFinding, ...]
    root_paths: tuple[str, ...]

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
        """Return whether the report contains no default-blocking findings."""

        return not self.blocking_findings()

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-compatible representation."""

        return {
            "root_paths": list(self.root_paths),
            "file_count": self.file_count,
            "parsed_count": self.parsed_count,
            "is_clean": self.is_clean,
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
            _DEFAULT_BLOCKING_SEVERITIES if severities is None else severities
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

        blocking = self.blocking_findings(severities=severities)
        if blocking:
            raise AssertionError(render_python_lang_harness(self))


def render_python_lang_harness(report: PythonHarnessReport) -> str:
    """Render a compact diagnostic report for humans and repair workflows."""

    rendered = _render_header(report)
    if report.is_clean:
        return rendered

    for finding in report.findings:
        rendered += "\n" + _render_finding(finding)
    return rendered


def discover_python_files(
    paths: Sequence[str | Path],
    *,
    ignored_dir_names: Iterable[str] | None = None,
) -> tuple[Path, ...]:
    """Discover Python files below the provided paths."""

    ignored_names = (
        _IGNORED_DIR_NAMES
        if ignored_dir_names is None
        else frozenset(ignored_dir_names)
    )
    discovered: list[Path] = []
    for raw_path in paths:
        path = Path(raw_path)
        if path.is_file():
            if path.suffix == ".py":
                discovered.append(path)
            continue
        if path.is_dir():
            discovered.extend(
                candidate
                for candidate in path.rglob("*.py")
                if _is_scannable_python_file(candidate, ignored_dir_names=ignored_names)
            )
    return tuple(sorted(discovered, key=lambda item: item.as_posix()))


def run_python_lang_harness(
    paths: Sequence[str | Path],
    *,
    config: PythonHarnessConfig | None = None,
    rule_packs: Sequence[PythonLangRulePack] | None = None,
) -> PythonHarnessReport:
    """Run the Python language harness over files or directories."""

    selected_config = _resolve_harness_config(config, rule_packs=rule_packs)
    selected_rule_packs = (
        selected_config.rule_packs
        if selected_config.rule_packs is not None
        else default_python_lang_rule_packs()
    )
    modules = tuple(
        parse_python_file(path)
        for path in discover_python_files(
            paths,
            ignored_dir_names=selected_config.ignored_dir_names,
        )
    )
    findings = tuple(
        finding
        for module in modules
        for rule_pack in selected_rule_packs
        for finding in rule_pack.evaluate(module)
    )
    return PythonHarnessReport(
        modules=modules,
        findings=findings,
        root_paths=tuple(str(Path(path)) for path in paths),
    )


def assert_python_lang_harness_clean(
    paths: Sequence[str | Path],
    *,
    config: PythonHarnessConfig | None = None,
    rule_packs: Sequence[PythonLangRulePack] | None = None,
    severities: frozenset[PythonDiagnosticSeverity] | None = None,
) -> PythonHarnessReport:
    """Run the harness and raise when error or warning findings are present."""

    selected_config = _resolve_harness_config(config, rule_packs=rule_packs)
    report = run_python_lang_harness(paths, config=selected_config)
    report.assert_clean(
        severities=(
            severities
            if severities is not None
            else selected_config.blocking_severities
        )
    )
    return report


def _resolve_harness_config(
    config: PythonHarnessConfig | None,
    *,
    rule_packs: Sequence[PythonLangRulePack] | None,
) -> PythonHarnessConfig:
    selected_config = default_python_harness_config() if config is None else config
    if rule_packs is None:
        return selected_config
    return replace(selected_config, rule_packs=tuple(rule_packs))


def _is_scannable_python_file(
    path: Path,
    *,
    ignored_dir_names: frozenset[str],
) -> bool:
    return not any(part in ignored_dir_names for part in path.parts)


def _render_header(report: PythonHarnessReport) -> str:
    target = ", ".join(report.root_paths)
    if report.is_clean:
        return f"[ok] {target} python\nSource: {target}\nNo blocking issues found.\n"
    status = _render_report_status(report)
    return f"[lint:{status}] {target} python\nSource: {target}\nIssues: {len(report.findings)}\n"


def _render_finding(finding: PythonHarnessFinding) -> str:
    path = finding.location.path or "<memory>"
    line = finding.location.line
    column = finding.location.column
    severity = finding.severity.value.title()
    display_column = column + 1
    rendered = (
        f"[{finding.rule_id}] {severity}: {finding.title}\n"
        f"   ,-[ {path}:{line}:{display_column} ]\n"
    )
    if finding.source_line:
        pointer_column = max(column, 0)
        rendered += f"{line:>2} | {finding.source_line}\n   | {' ' * pointer_column}`- {finding.label}\n"
    else:
        rendered += f"   | {finding.label}\n"
    rendered += f"   |Required: {finding.requirement}\n"
    return rendered


def _render_report_status(report: PythonHarnessReport) -> str:
    if any(
        finding.severity == PythonDiagnosticSeverity.ERROR
        for finding in report.findings
    ):
        return "error"
    if any(
        finding.severity == PythonDiagnosticSeverity.WARNING
        for finding in report.findings
    ):
        return "warning"
    return "info"


def _wildcard_import_findings(
    report: PythonModuleReport,
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    findings: list[PythonHarnessFinding] = []
    rule = _modern_design_rule(_PY_MOD_R001)
    for import_record in report.imports:
        if "*" not in import_record.names:
            continue
        module = "." * import_record.level + (import_record.module or "")
        findings.append(
            PythonHarnessFinding(
                rule_id=rule.rule_id,
                pack_id=pack_id,
                severity=rule.severity,
                title=rule.title,
                summary=f"Wildcard import from {module!r} makes exported names implicit.",
                location=import_record.location,
                requirement=f"Import explicit names from {module!r}; do not use `*` in project modules.",
                source_line=_source_line(report.path, import_record.location.line),
                label="replace wildcard import with explicit imported names",
                labels=dict(rule.labels),
            )
        )
    return tuple(findings)


def _bare_print_findings(
    report: PythonModuleReport,
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    if not _is_library_module(report.path):
        return ()

    findings: list[PythonHarnessFinding] = []
    rule = _modern_design_rule(_PY_MOD_R002)
    for call in report.calls:
        if call.function != "print":
            continue
        findings.append(
            PythonHarnessFinding(
                rule_id=rule.rule_id,
                pack_id=pack_id,
                severity=rule.severity,
                title=rule.title,
                summary="Bare print calls leak diagnostics to stdout.",
                location=call.location,
                requirement=rule.requirement,
                source_line=_source_line(report.path, call.location.line),
                label="replace bare print with a project-owned reporting surface",
                labels=dict(rule.labels),
            )
        )
    return tuple(findings)


def _debug_breakpoint_findings(
    report: PythonModuleReport,
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    if not _is_library_module(report.path):
        return ()

    findings: list[PythonHarnessFinding] = []
    rule = _modern_design_rule(_PY_MOD_R004)
    for call in report.calls:
        if call.function != "breakpoint":
            continue
        findings.append(
            PythonHarnessFinding(
                rule_id=rule.rule_id,
                pack_id=pack_id,
                severity=rule.severity,
                title=rule.title,
                summary="breakpoint() can halt library execution inside an interactive debugger.",
                location=call.location,
                requirement=rule.requirement,
                source_line=_source_line(report.path, call.location.line),
                label="remove breakpoint() from library code",
                labels=dict(rule.labels),
            )
        )
    return tuple(findings)


def _facade_all_findings(
    report: PythonModuleReport,
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    if not report.path or Path(report.path).name != "__init__.py":
        return ()
    if not _has_facade_import(report) or _has_top_level_all_binding(report):
        return ()
    rule = _modern_design_rule(_PY_MOD_R003)

    return (
        PythonHarnessFinding(
            rule_id=rule.rule_id,
            pack_id=pack_id,
            severity=rule.severity,
            title=rule.title,
            summary="Facade imports expose names without an explicit public contract.",
            location=report.imports[0].location,
            requirement=rule.requirement,
            source_line=_source_line(report.path, report.imports[0].location.line),
            label="add an explicit __all__ for this facade export surface",
            labels=dict(rule.labels),
        ),
    )


def _modern_design_rule(rule_id: str) -> PythonHarnessRule:
    return _MODERN_DESIGN_RULE_BY_ID[rule_id]


def _has_facade_import(report: PythonModuleReport) -> bool:
    return any(
        import_record.scope == "" and import_record.level > 0
        for import_record in report.imports
    )


def _has_top_level_all_binding(report: PythonModuleReport) -> bool:
    return any(
        binding.scope_name == "top"
        and binding.name == "__all__"
        and "assigned" in binding.flags
        for binding in report.bindings
    )


def _is_library_module(path: str | None) -> bool:
    if path is None:
        return True
    candidate = Path(path)
    if candidate.name == "__main__.py":
        return False
    return not any(
        part == "tests" for part in candidate.parts
    ) and not candidate.name.startswith("test_")


def _source_line(path: str | None, line: int) -> str | None:
    if path is None or line < 1:
        return None
    try:
        return Path(path).read_text(encoding="utf-8").splitlines()[line - 1]
    except (OSError, IndexError, UnicodeDecodeError):
        return None
