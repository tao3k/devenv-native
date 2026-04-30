"""Typed-package project policy helpers."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

from python_lang_parser import PythonSymbolKind

from ._model import PythonHarnessFinding
from ._project_policy_catalog import PY_PROJ_R003, PY_PROJ_R004, project_policy_rule
from ._source import path_location, source_line

if TYPE_CHECKING:
    from collections.abc import Sequence

    from python_lang_parser import PythonModuleReport, PythonSymbol

    from ._project_metadata import PythonProjectMetadata


def typed_package_findings(
    metadata: PythonProjectMetadata,
    modules: Sequence[PythonModuleReport],
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    """Return findings for typed package marker and annotation contracts."""

    return (
        *_typed_package_marker_findings(metadata, modules, pack_id),
        *_typed_package_annotation_findings(metadata, modules, pack_id),
    )


def _typed_package_marker_findings(
    metadata: PythonProjectMetadata,
    modules: Sequence[PythonModuleReport],
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    findings: list[PythonHarnessFinding] = []
    rule = project_policy_rule(PY_PROJ_R003)
    for package_root in metadata.package_roots:
        if not (package_root / "__init__.py").is_file():
            continue
        if (package_root / "py.typed").is_file():
            continue
        if not _package_has_public_parser_surface(package_root, modules):
            continue
        findings.append(
            PythonHarnessFinding(
                rule_id=rule.rule_id,
                pack_id=pack_id,
                severity=rule.severity,
                title=rule.title,
                summary=f"{package_root.name} exposes public Python surface without a py.typed marker.",
                location=path_location(package_root),
                requirement=rule.requirement,
                source_line=None,
                label="add py.typed to this package root",
                labels=dict(rule.labels),
            )
        )
    return tuple(findings)


def _package_has_public_parser_surface(
    package_root: Path,
    modules: Sequence[PythonModuleReport],
) -> bool:
    return any(
        _module_is_inside_package(module, package_root)
        and _module_has_public_parser_surface(module)
        for module in modules
    )


def _module_has_public_parser_surface(module: PythonModuleReport) -> bool:
    if module.export_candidates:
        return True
    return module.shape is not None and module.shape.has_public_surface


def _typed_package_annotation_findings(
    metadata: PythonProjectMetadata,
    modules: Sequence[PythonModuleReport],
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    findings: list[PythonHarnessFinding] = []
    rule = project_policy_rule(PY_PROJ_R004)
    for package_root in metadata.package_roots:
        if not (package_root / "py.typed").is_file():
            continue
        for module in modules:
            if not _module_is_inside_package(module, package_root):
                continue
            public_class_names = _public_class_names(module)
            findings.extend(
                _unannotated_public_callable_findings(
                    module,
                    public_class_names=public_class_names,
                    pack_id=pack_id,
                    rule=rule,
                )
            )
    return tuple(findings)


def _unannotated_public_callable_findings(
    module: PythonModuleReport,
    *,
    public_class_names: frozenset[str],
    pack_id: str,
    rule: object,
) -> tuple[PythonHarnessFinding, ...]:
    findings: list[PythonHarnessFinding] = []
    for symbol in module.symbols:
        if not _is_public_callable_boundary(symbol, public_class_names):
            continue
        if symbol.has_annotations:
            continue
        findings.append(
            PythonHarnessFinding(
                rule_id=rule.rule_id,
                pack_id=pack_id,
                severity=rule.severity,
                title=rule.title,
                summary=f"{symbol.qualified_name} is public in a py.typed package but lacks annotations.",
                location=symbol.location,
                requirement=rule.requirement,
                source_line=source_line(module.path, symbol.location.line),
                label="annotate this typed-package public callable",
                labels=dict(rule.labels),
            )
        )
    return tuple(findings)


def _module_is_inside_package(module: PythonModuleReport, package_root: Path) -> bool:
    if module.path is None or module.shape is None:
        return False
    try:
        Path(module.path).relative_to(package_root)
    except ValueError:
        return False
    return True


def _public_class_names(module: PythonModuleReport) -> frozenset[str]:
    return frozenset(
        symbol.qualified_name
        for symbol in module.symbols
        if symbol.kind == PythonSymbolKind.CLASS
        and symbol.is_public
        and _scope_is_public(symbol.scope)
    )


def _is_public_callable_boundary(
    symbol: PythonSymbol,
    public_class_names: frozenset[str],
) -> bool:
    if symbol.kind not in {PythonSymbolKind.FUNCTION, PythonSymbolKind.ASYNC_FUNCTION}:
        return False
    if not symbol.is_public:
        return False
    return symbol.scope == "" or symbol.scope in public_class_names


def _scope_is_public(scope: str) -> bool:
    return all(_name_is_public(part) for part in scope.split(".") if part)


def _name_is_public(name: str) -> bool:
    return bool(name) and not name.startswith(("_", "test_"))
