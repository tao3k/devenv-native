"""Agent-oriented Python policy rules backed by native parser reports."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

from python_lang_parser import PythonSymbolKind

from ._agent_policy_catalog import (
    AGENT_POLICY_PACK_ID,
    PY_AGENT_R001,
    PY_AGENT_R002,
    PY_AGENT_R003,
    PY_AGENT_R004,
    agent_policy_rule,
)
from ._model import PythonHarnessFinding, PythonRulePackDescriptor
from ._source import path_location, source_line

if TYPE_CHECKING:
    from collections.abc import Iterable, Sequence

    from python_lang_parser import PythonModuleReport, PythonSymbol

    from ._model import PythonProjectHarnessScope


@dataclass(frozen=True, slots=True)
class PythonAgentPolicyRulePack:
    """Rules that keep Python projects legible for repair-oriented agents."""

    pack_id: str = AGENT_POLICY_PACK_ID

    def descriptor(self) -> PythonRulePackDescriptor:
        """Return stable metadata for this rule pack."""

        return PythonRulePackDescriptor(
            id=self.pack_id,
            version="v1",
            domains=("agent-policy", "project-shape", "python"),
        )

    def evaluate(self, report: PythonModuleReport) -> Iterable[PythonHarnessFinding]:
        """Evaluate agent-oriented policy rules for one parsed module report."""

        if not report.is_valid:
            return ()

        findings: list[PythonHarnessFinding] = []
        findings.extend(_module_docstring_findings(report, self.pack_id))
        findings.extend(_public_callable_annotation_findings(report, self.pack_id))
        return tuple(findings)

    def evaluate_project_modules(
        self,
        scope: PythonProjectHarnessScope,
        modules: Sequence[PythonModuleReport],
    ) -> Iterable[PythonHarnessFinding]:
        """Evaluate agent-oriented namespace rules across a project scope."""

        findings: list[PythonHarnessFinding] = []
        findings.extend(_duplicate_public_callable_findings(modules, self.pack_id))
        findings.extend(
            _repeated_namespace_segment_findings(scope, modules, self.pack_id)
        )
        return tuple(findings)


def _module_docstring_findings(
    report: PythonModuleReport,
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    if report.module_docstring or not _module_has_agent_surface(report):
        return ()
    rule = agent_policy_rule(PY_AGENT_R001)
    path = Path(report.path or "<memory>")
    return (
        PythonHarnessFinding(
            rule_id=rule.rule_id,
            pack_id=pack_id,
            severity=rule.severity,
            title=rule.title,
            summary=f"{path.name} has public module surface without a module intent docstring.",
            location=path_location(path),
            requirement=rule.requirement,
            source_line=source_line(report.path, 1),
            label="add a concise module responsibility docstring",
            labels=dict(rule.labels),
        ),
    )


def _public_callable_annotation_findings(
    report: PythonModuleReport,
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    findings: list[PythonHarnessFinding] = []
    rule = agent_policy_rule(PY_AGENT_R002)
    for symbol in report.symbols:
        if not _is_public_callable(symbol) or symbol.has_annotations:
            continue
        findings.append(
            PythonHarnessFinding(
                rule_id=rule.rule_id,
                pack_id=pack_id,
                severity=rule.severity,
                title=rule.title,
                summary=f"{symbol.qualified_name} exposes a public callable boundary without annotations.",
                location=symbol.location,
                requirement=rule.requirement,
                source_line=source_line(report.path, symbol.location.line),
                label="add parameter and return annotations to this public callable",
                labels=dict(rule.labels),
            )
        )
    return tuple(findings)


def _module_has_agent_surface(report: PythonModuleReport) -> bool:
    return report.shape is not None and report.shape.public_symbol_count > 0


def _is_public_callable(symbol: PythonSymbol) -> bool:
    if symbol.kind not in {PythonSymbolKind.FUNCTION, PythonSymbolKind.ASYNC_FUNCTION}:
        return False
    return symbol.is_public


def _duplicate_public_callable_findings(
    modules: Sequence[PythonModuleReport],
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    first_seen: dict[str, tuple[PythonModuleReport, PythonSymbol]] = {}
    emitted: set[tuple[str, str]] = set()
    findings: list[PythonHarnessFinding] = []
    rule = agent_policy_rule(PY_AGENT_R003)
    for report in modules:
        if not report.is_valid:
            continue
        for symbol in report.symbols:
            if not symbol.is_top_level or not _is_public_callable(symbol):
                continue
            first = first_seen.setdefault(symbol.name, (report, symbol))
            first_report, first_symbol = first
            if first_symbol is symbol:
                continue
            key = (symbol.name, report.path or "")
            if key in emitted:
                continue
            emitted.add(key)
            findings.append(
                PythonHarnessFinding(
                    rule_id=rule.rule_id,
                    pack_id=pack_id,
                    severity=rule.severity,
                    title=rule.title,
                    summary=(
                        f"Public callable {symbol.name!r} appears in "
                        f"{_display_module(first_report)} and {_display_module(report)}."
                    ),
                    location=symbol.location,
                    requirement=rule.requirement,
                    source_line=source_line(report.path, symbol.location.line),
                    label="rename or namespace this public callable boundary",
                    labels=dict(rule.labels),
                )
            )
    return tuple(findings)


def _repeated_namespace_segment_findings(
    scope: PythonProjectHarnessScope,
    modules: Sequence[PythonModuleReport],
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    emitted_branches: set[tuple[str, tuple[str, ...]]] = set()
    findings: list[PythonHarnessFinding] = []
    rule = agent_policy_rule(PY_AGENT_R004)
    for report in modules:
        if not report.path:
            continue
        path = Path(report.path)
        namespace = _module_namespace_parts(scope, path)
        repeated = _first_repeated_namespace_segment(namespace)
        if repeated is None:
            continue
        segment, branch = repeated
        key = (segment, branch)
        if key in emitted_branches:
            continue
        emitted_branches.add(key)
        findings.append(
            PythonHarnessFinding(
                rule_id=rule.rule_id,
                pack_id=pack_id,
                severity=rule.severity,
                title=rule.title,
                summary=f"Module namespace {'.'.join(namespace)!r} repeats {segment!r}.",
                location=path_location(path),
                requirement=rule.requirement,
                source_line=source_line(report.path, 1),
                label="rename one repeated namespace segment",
                labels=dict(rule.labels),
            )
        )
    return tuple(findings)


def _display_module(report: PythonModuleReport) -> str:
    if report.path is None:
        return "<memory>"
    return Path(report.path).stem


def _module_namespace_parts(
    scope: PythonProjectHarnessScope,
    path: Path,
) -> tuple[str, ...]:
    relative = _relative_module_path(scope, path)
    module_path = relative.with_suffix("")
    parts = module_path.parts
    if parts and parts[-1] == "__init__":
        return parts[:-1]
    return parts


def _relative_module_path(
    scope: PythonProjectHarnessScope,
    path: Path,
) -> Path:
    for root in scope.monitored_paths:
        try:
            return path.relative_to(root)
        except ValueError:
            continue
    try:
        return path.relative_to(scope.project_root)
    except ValueError:
        return path


def _first_repeated_namespace_segment(
    namespace: tuple[str, ...],
) -> tuple[str, tuple[str, ...]] | None:
    seen: dict[str, int] = {}
    for index, segment in enumerate(namespace):
        previous_index = seen.setdefault(segment, index)
        if previous_index == index:
            continue
        return segment, namespace[: index + 1]
    return None
