"""Runner API for embedding the Python language harness in pytest."""

from __future__ import annotations

from dataclasses import replace
from pathlib import Path
from typing import TYPE_CHECKING

from python_lang_parser import PythonDiagnosticSeverity, parse_python_file

from ._agent_policy import PythonAgentPolicyRulePack
from ._agent_policy_catalog import PY_AGENT_R002
from ._discovery import discover_python_files, python_project_harness_scope
from ._model import (
    PythonHarnessConfig,
    PythonHarnessFinding,
    PythonHarnessReport,
    PythonLangRulePack,
    PythonProjectHarnessScope,
)
from ._modern_design import PythonModernDesignRulePack
from ._modularity import PythonModularityRulePack
from ._project_policy import PythonProjectPolicyRulePack
from ._project_policy_catalog import PY_PROJ_R004
from ._syntax import PythonSyntaxRulePack
from ._test_layout import PythonTestLayoutRulePack

if TYPE_CHECKING:
    from collections.abc import Sequence

    from python_lang_parser import PythonModuleReport


def default_python_lang_rule_packs() -> tuple[PythonLangRulePack, ...]:
    """Return the default deterministic Python language rule packs."""

    return (
        PythonSyntaxRulePack(),
        PythonProjectPolicyRulePack(),
        PythonModernDesignRulePack(),
        PythonAgentPolicyRulePack(),
        PythonModularityRulePack(),
        PythonTestLayoutRulePack(),
    )


def default_python_harness_config() -> PythonHarnessConfig:
    """Return the default Python language harness configuration."""

    return PythonHarnessConfig(rule_packs=default_python_lang_rule_packs())


def run_python_project_harness(
    project_root: str | Path,
    *,
    config: PythonHarnessConfig | None = None,
    rule_packs: Sequence[PythonLangRulePack] | None = None,
    include_tests: bool = True,
    source_dir_names: Sequence[str] = ("src",),
    test_dir_names: Sequence[str] = ("tests",),
) -> PythonHarnessReport:
    """Run the harness over conventional Python project paths."""

    selected_config = _resolve_harness_config(config, rule_packs=rule_packs)
    selected_rule_packs = _selected_rule_packs(selected_config)
    root = Path(project_root)
    scope = python_project_harness_scope(
        root,
        include_tests=include_tests,
        source_dir_names=source_dir_names,
        test_dir_names=test_dir_names,
    )
    report = run_python_lang_harness(
        scope.monitored_paths,
        config=selected_config,
    )
    project_findings = _evaluate_project_rule_packs(
        scope,
        selected_rule_packs,
        report.modules,
    )
    return replace(
        report,
        project_scope=scope,
        findings=_compact_project_findings(report.findings, project_findings),
    )


def assert_python_project_harness_clean(
    project_root: str | Path,
    *,
    config: PythonHarnessConfig | None = None,
    rule_packs: Sequence[PythonLangRulePack] | None = None,
    severities: frozenset[PythonDiagnosticSeverity] | None = None,
    include_tests: bool = True,
    source_dir_names: Sequence[str] = ("src",),
    test_dir_names: Sequence[str] = ("tests",),
    include_advice: bool = True,
) -> PythonHarnessReport:
    """Run the project harness and raise when configured-blocking findings exist."""

    selected_config = _resolve_harness_config(config, rule_packs=rule_packs)
    report = run_python_project_harness(
        project_root,
        config=selected_config,
        include_tests=include_tests,
        source_dir_names=source_dir_names,
        test_dir_names=test_dir_names,
    )
    report.assert_clean(
        severities=(
            severities
            if severities is not None
            else selected_config.blocking_severities
        ),
        include_advice=include_advice,
    )
    return report


def run_python_lang_harness(
    paths: Sequence[str | Path],
    *,
    config: PythonHarnessConfig | None = None,
    rule_packs: Sequence[PythonLangRulePack] | None = None,
) -> PythonHarnessReport:
    """Run the Python language harness over files or directories."""

    selected_config = _resolve_harness_config(config, rule_packs=rule_packs)
    selected_rule_packs = _selected_rule_packs(selected_config)
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
        blocking_severities=selected_config.blocking_severities,
    )


def assert_python_lang_harness_clean(
    paths: Sequence[str | Path],
    *,
    config: PythonHarnessConfig | None = None,
    rule_packs: Sequence[PythonLangRulePack] | None = None,
    severities: frozenset[PythonDiagnosticSeverity] | None = None,
    include_advice: bool = True,
) -> PythonHarnessReport:
    """Run the harness and raise when configured-blocking findings are present."""

    selected_config = _resolve_harness_config(config, rule_packs=rule_packs)
    report = run_python_lang_harness(paths, config=selected_config)
    report.assert_clean(
        severities=(
            severities
            if severities is not None
            else selected_config.blocking_severities
        ),
        include_advice=include_advice,
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


def _selected_rule_packs(
    config: PythonHarnessConfig,
) -> tuple[PythonLangRulePack, ...]:
    if config.rule_packs is not None:
        return config.rule_packs
    return default_python_lang_rule_packs()


def _evaluate_project_rule_packs(
    scope: PythonProjectHarnessScope,
    rule_packs: Sequence[PythonLangRulePack],
    modules: Sequence[PythonModuleReport],
) -> tuple[PythonHarnessFinding, ...]:
    findings: list[PythonHarnessFinding] = []
    for rule_pack in rule_packs:
        module_evaluator = getattr(rule_pack, "evaluate_project_modules", None)
        if module_evaluator is not None:
            findings.extend(module_evaluator(scope, modules))
            continue
        scope_evaluator = getattr(rule_pack, "evaluate_project_scope", None)
        if scope_evaluator is not None:
            findings.extend(scope_evaluator(scope))
            continue
        evaluator = getattr(rule_pack, "evaluate_project", None)
        if evaluator is None:
            continue
        findings.extend(evaluator(scope.project_root))
    return tuple(findings)


def _compact_project_findings(
    module_findings: Sequence[PythonHarnessFinding],
    project_findings: Sequence[PythonHarnessFinding],
) -> tuple[PythonHarnessFinding, ...]:
    typed_package_annotation_locations = {
        _finding_location_key(finding)
        for finding in project_findings
        if finding.rule_id == PY_PROJ_R004
    }
    compact_module_findings = tuple(
        finding
        for finding in module_findings
        if not (
            finding.rule_id == PY_AGENT_R002
            and _finding_location_key(finding) in typed_package_annotation_locations
        )
    )
    return (*compact_module_findings, *project_findings)


def _finding_location_key(
    finding: PythonHarnessFinding,
) -> tuple[str | None, int, int]:
    return (
        finding.location.path,
        finding.location.line,
        finding.location.column,
    )
