"""Project metadata and package-root layout policy helpers."""

from __future__ import annotations

from typing import TYPE_CHECKING

from ._model import PythonHarnessFinding
from ._project_policy_catalog import PY_PROJ_R001, PY_PROJ_R002, project_policy_rule
from ._source import path_location, source_line

if TYPE_CHECKING:
    from ._model import PythonProjectHarnessScope
    from ._project_metadata import PythonProjectMetadata


def project_layout_findings(
    scope: PythonProjectHarnessScope,
    metadata: PythonProjectMetadata,
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    """Return findings for project source and package-root layout."""

    return (
        *_src_layout_findings(scope, metadata, pack_id),
        *_declared_package_root_findings(metadata, pack_id),
    )


def _src_layout_findings(
    scope: PythonProjectHarnessScope,
    metadata: PythonProjectMetadata,
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    if _uses_src_layout(scope, metadata):
        return ()

    rule = project_policy_rule(PY_PROJ_R001)
    return (
        PythonHarnessFinding(
            rule_id=rule.rule_id,
            pack_id=pack_id,
            severity=rule.severity,
            title=rule.title,
            summary=f"{metadata.pyproject_path.name} is present but project sources are not under src/.",
            location=path_location(metadata.pyproject_path),
            requirement=rule.requirement,
            source_line=source_line(str(metadata.pyproject_path), 1),
            label="declare package code under src/",
            labels=dict(rule.labels),
        ),
    )


def _uses_src_layout(
    scope: PythonProjectHarnessScope,
    metadata: PythonProjectMetadata,
) -> bool:
    src_path = metadata.project_root / "src"
    if src_path not in scope.source_paths:
        return False
    if not metadata.package_roots:
        return True
    for package_root in metadata.package_roots:
        try:
            package_root.relative_to(src_path)
        except ValueError:
            return False
    return True


def _declared_package_root_findings(
    metadata: PythonProjectMetadata,
    pack_id: str,
) -> tuple[PythonHarnessFinding, ...]:
    findings: list[PythonHarnessFinding] = []
    rule = project_policy_rule(PY_PROJ_R002)
    for package_root in metadata.package_roots:
        init_file = package_root / "__init__.py"
        if package_root.is_dir() and init_file.is_file():
            continue
        finding_path = (
            package_root if package_root.exists() else metadata.pyproject_path
        )
        findings.append(
            PythonHarnessFinding(
                rule_id=rule.rule_id,
                pack_id=pack_id,
                severity=rule.severity,
                title=rule.title,
                summary=f"Declared wheel package root {package_root} is not an importable package directory.",
                location=path_location(finding_path),
                requirement=rule.requirement,
                source_line=source_line(str(finding_path), 1),
                label="make this declared package root importable",
                labels=dict(rule.labels),
            )
        )
    return tuple(findings)
