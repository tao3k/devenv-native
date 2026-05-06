"""Project-level Python harness gate for xiuxian-wendao-analyzer."""

from __future__ import annotations

import json
import os
from pathlib import Path

from python_lang_project_harness import (
    PythonDiagnosticSeverity,
    PythonHarnessConfig,
    PythonHarnessFinding,
    PythonHarnessReport,
    default_python_harness_config,
    python_agent_policy_rules,
    python_modularity_rules,
    python_project_harness_test,
    python_project_policy_rules,
    python_reasoning_tree_facts,
    run_python_lang_harness,
    run_python_project_harness,
)

ERROR_ONLY_HARNESS_CONFIG = PythonHarnessConfig(
    blocking_severities=frozenset({PythonDiagnosticSeverity.ERROR})
)
EXPECTED_DEFAULT_RULE_PACKS = {
    "python.agent_policy",
    "python.modern_design",
    "python.modularity",
    "python.project_policy",
    "python.syntax",
    "python.test_layout",
}

test_python_project_harness_policy = python_project_harness_test(
    Path(__file__).resolve().parents[2],
)


def test_python_project_harness_uses_all_default_rule_packs() -> None:
    config = default_python_harness_config()

    assert not config.disabled_rule_ids
    assert {
        rule_pack.pack_id for rule_pack in config.rule_packs or ()
    } == EXPECTED_DEFAULT_RULE_PACKS
    assert tuple(python_agent_policy_rules())
    assert tuple(python_modularity_rules())
    assert tuple(python_project_policy_rules())


def test_python_project_harness_blocks_no_error_findings() -> None:
    package_root = Path(__file__).resolve().parents[2]
    report = run_python_project_harness(package_root)
    blocking_findings = report.blocking_findings()

    assert not blocking_findings, _render_finding_set(
        package_root,
        "Python project harness blocking findings",
        blocking_findings,
    )
    assert_python_harness_baseline(package_root, report)


def test_benchmark_script_harness_blocks_no_error_findings() -> None:
    package_root = Path(__file__).resolve().parents[2]
    repo_root = package_root.parents[2]
    benchmark_root = repo_root / "tests/scripts"
    report = run_python_lang_harness(
        [
            benchmark_root / "benchmark_wendao_document_extract.py",
            benchmark_root / "wendao_document_extract_benchmark",
        ],
        config=ERROR_ONLY_HARNESS_CONFIG,
    )
    blocking_findings = report.blocking_findings()

    assert not blocking_findings, _render_finding_set(
        repo_root,
        "benchmark script harness blocking findings",
        blocking_findings,
    )


def _finding_key(
    package_root: Path,
    finding: PythonHarnessFinding,
) -> tuple[str, str]:
    path = Path(finding.location.path or "")
    try:
        relative_path = path.relative_to(package_root)
    except ValueError:
        relative_path = path
    return finding.rule_id, relative_path.as_posix()


def _render_finding_set(
    root: Path,
    title: str,
    findings: tuple[PythonHarnessFinding, ...],
) -> str:
    rendered = "\n".join(
        f"- {rule_id}: {path}"
        for rule_id, path in sorted(_finding_key(root, finding) for finding in findings)
    )
    return f"{title}:\n{rendered}"


def assert_python_harness_baseline(
    package_root: Path,
    report: PythonHarnessReport,
) -> None:
    report_dir = package_root / "resources" / "verification" / "reports"
    manifest_path = report_dir / "python_harness_report_manifest.json"
    summary_path = report_dir / "python_harness_summary.json"
    manifest = python_harness_report_manifest()
    summary = python_harness_summary(report)

    if os.environ.get("XIUXIAN_WRITE_PYTHON_HARNESS_REPORTS"):
        report_dir.mkdir(parents=True, exist_ok=True)
        write_json(manifest_path, manifest)
        write_json(summary_path, summary)

    assert json.loads(manifest_path.read_text(encoding="utf-8")) == manifest
    assert json.loads(summary_path.read_text(encoding="utf-8")) == summary


def python_harness_report_manifest() -> dict[str, object]:
    return {
        "artifacts": [
            {
                "key": "python_harness_summary_json",
                "artifact_name": "python_harness_summary.json",
                "persistence": "source_baseline",
                "renderer": "xiuxian-wendao-analyzer test_project_harness.py",
                "reason": (
                    "persist compact Python harness policy state so parser, "
                    "modularity, project, test-layout, and agent-policy drift "
                    "stay reviewable"
                ),
            }
        ],
        "schema_version": 1,
    }


def python_harness_summary(report: PythonHarnessReport) -> dict[str, object]:
    project_scope = report.project_scope
    project_metadata = None if project_scope is None else project_scope.project_metadata
    reasoning_tree = python_reasoning_tree_facts(
        report.modules,
        import_roots=() if project_scope is None else project_scope.source_paths,
        project_root=None if project_scope is None else project_scope.project_root,
        project_metadata=project_metadata,
    )
    return {
        "blocking_rule_ids": sorted(report.blocking_rule_ids),
        "blocking_severities": sorted(
            severity.value for severity in report.blocking_severities
        ),
        "disabled_rule_ids": sorted(report.disabled_rule_ids),
        "finding_counts": {
            "advisory": len(report.advisory_findings()),
            "blocking": len(report.blocking_findings()),
            "non_blocking": len(report.findings) - len(report.blocking_findings()),
            "total": len(report.findings),
        },
        "finding_rule_counts": _finding_rule_counts(report),
        "is_clean": report.is_clean,
        "parsed_count": report.parsed_count,
        "project": {
            "import_names": (
                []
                if project_metadata is None
                else sorted(
                    import_name.name for import_name in project_metadata.import_names
                )
            ),
            "name": None if project_metadata is None else project_metadata.project_name,
            "package_roots": (
                []
                if project_metadata is None
                else sorted(
                    _package_relative_path(project_metadata.project_root, path)
                    for path in project_metadata.package_roots
                )
            ),
            "requires_python": (
                None if project_metadata is None else project_metadata.requires_python
            ),
            "scripts": (
                []
                if project_metadata is None
                else sorted(script.name for script in project_metadata.scripts)
            ),
        },
        "reasoning_tree": {
            "import_edge_count": len(reasoning_tree.import_edges),
            "node_count": len(reasoning_tree.nodes),
            "shadowed_module_source_count": len(reasoning_tree.shadowed_module_sources),
        },
        "rule_packs": _default_rule_pack_ids(),
        "source": {
            "file_count": report.file_count,
            "module_count": len(report.modules),
        },
    }


def _default_rule_pack_ids() -> list[str]:
    rule_packs = default_python_harness_config().rule_packs or ()
    return [rule_pack.pack_id for rule_pack in rule_packs]


def _finding_rule_counts(report: PythonHarnessReport) -> dict[str, int]:
    counts: dict[str, int] = {}
    for finding in report.findings:
        counts[finding.rule_id] = counts.get(finding.rule_id, 0) + 1
    return dict(sorted(counts.items()))


def _package_relative_path(project_root: Path, path: Path) -> str:
    try:
        return path.relative_to(project_root).as_posix()
    except ValueError:
        return path.as_posix()


def write_json(path: Path, payload: object) -> None:
    path.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
