from __future__ import annotations

from typing import TYPE_CHECKING

from python_lang_parser import PythonDiagnosticSeverity, SourceLocation
from xiuxian_harness_python_lang_project import (
    PythonHarnessConfig,
    PythonHarnessFinding,
    PythonModernDesignRulePack,
    assert_python_lang_harness_clean,
    default_python_harness_config,
    discover_python_files,
    python_modern_design_rules,
    render_python_lang_harness,
    run_python_lang_harness,
)

if TYPE_CHECKING:
    from pathlib import Path

    from python_lang_parser import PythonModuleReport


def test_discover_python_files_skips_cache_dirs(tmp_path: Path) -> None:
    src = tmp_path / "src"
    cache = src / "__pycache__"
    src.mkdir()
    cache.mkdir()
    good = src / "good.py"
    ignored = cache / "ignored.py"
    good.write_text("VALUE = 1\n", encoding="utf-8")
    ignored.write_text("VALUE = 2\n", encoding="utf-8")

    assert discover_python_files([tmp_path]) == (good,)


def test_discover_python_files_accepts_custom_ignored_dirs(tmp_path: Path) -> None:
    src = tmp_path / "src"
    generated = src / "generated"
    src.mkdir()
    generated.mkdir()
    good = src / "good.py"
    ignored = generated / "ignored.py"
    good.write_text("VALUE = 1\n", encoding="utf-8")
    ignored.write_text("VALUE = 2\n", encoding="utf-8")

    assert discover_python_files([tmp_path], ignored_dir_names={"generated"}) == (good,)


def test_run_python_lang_harness_collects_parse_findings(tmp_path: Path) -> None:
    good = tmp_path / "good.py"
    bad = tmp_path / "bad.py"
    good.write_text("def ok() -> None:\n    return None\n", encoding="utf-8")
    bad.write_text("def broken(:\n    pass\n", encoding="utf-8")

    report = run_python_lang_harness([tmp_path])

    assert report.file_count == 2
    assert report.parsed_count == 1
    assert not report.is_clean
    assert [
        (finding.rule_id, finding.location.path) for finding in report.findings
    ] == [
        ("python.syntax.invalid", str(bad)),
    ]
    assert report.to_dict()["is_clean"] is False


def test_run_python_lang_harness_uses_configured_discovery(tmp_path: Path) -> None:
    generated = tmp_path / "generated"
    generated.mkdir()
    ignored = generated / "debug.py"
    ignored.write_text('def run():\n    print("debug")\n', encoding="utf-8")

    report = run_python_lang_harness(
        [tmp_path],
        config=PythonHarnessConfig(ignored_dir_names=frozenset({"generated"})),
    )

    assert report.file_count == 0
    assert report.is_clean


def test_render_python_lang_harness_uses_compact_source_diagnostic(
    tmp_path: Path,
) -> None:
    bad = tmp_path / "bad.py"
    bad.write_text("def broken(:\n    pass\n", encoding="utf-8")

    output = render_python_lang_harness(run_python_lang_harness([bad]))

    assert output.startswith("[lint:error]")
    assert "[python.syntax.invalid] Error: Python source did not parse" in output
    assert "def broken(:" in output
    assert "Required: Python modules must parse with CPython native syntax" in output
    assert "Action:" not in output
    assert "Fix:" not in output
    assert "Evidence:" not in output


def test_modern_design_rule_pack_reports_numbered_rules_in_compact_snapshot(
    tmp_path: Path,
) -> None:
    source = tmp_path / "module.py"
    source.write_text(
        'from tools import *\n\n\ndef run():\n    print("debug")\n    breakpoint()\n',
        encoding="utf-8",
    )

    output = render_python_lang_harness(run_python_lang_harness([source]))
    output = output.replace(str(source), "$TMP/module.py")

    assert (
        output
        == """[lint:warning] $TMP/module.py python
Source: $TMP/module.py
Issues: 3

[PY-MOD-R001] Warning: Wildcard import hides the dependency surface
   ,-[ $TMP/module.py:1:1 ]
 1 | from tools import *
   | `- replace wildcard import with explicit imported names
   |Required: Import explicit names from 'tools'; do not use `*` in project modules.

[PY-MOD-R002] Warning: Library module uses bare print
   ,-[ $TMP/module.py:5:5 ]
 5 |     print("debug")
   |     `- replace bare print with a project-owned reporting surface
   |Required: Use a logger, returned value, or explicit test assertion instead of bare `print` in library modules.

[PY-MOD-R004] Warning: Library module contains breakpoint()
   ,-[ $TMP/module.py:6:5 ]
 6 |     breakpoint()
   |     `- remove breakpoint() from library code
   |Required: Remove `breakpoint()` from library modules; use test-only debug tooling or a project-owned diagnostic surface.
"""
    )


def test_modern_design_rule_pack_requires_all_for_package_facade(
    tmp_path: Path,
) -> None:
    package = tmp_path / "pkg"
    package.mkdir()
    init_file = package / "__init__.py"
    init_file.write_text("from .api import Runner\n", encoding="utf-8")
    (package / "api.py").write_text("class Runner:\n    pass\n", encoding="utf-8")

    report = run_python_lang_harness([package])

    assert [
        (finding.rule_id, finding.location.path) for finding in report.findings
    ] == [
        ("PY-MOD-R003", str(init_file)),
    ]


def test_modern_design_rule_pack_accepts_explicit_facade_all(tmp_path: Path) -> None:
    package = tmp_path / "pkg"
    package.mkdir()
    (package / "__init__.py").write_text(
        'from .api import Runner\n\n__all__ = ["Runner"]\n',
        encoding="utf-8",
    )
    (package / "api.py").write_text("class Runner:\n    pass\n", encoding="utf-8")

    report = run_python_lang_harness([package])

    assert report.is_clean


def test_modern_design_rule_pack_skips_prints_in_tests(tmp_path: Path) -> None:
    tests = tmp_path / "tests"
    tests.mkdir()
    source = tests / "test_debug.py"
    source.write_text(
        'def test_debug():\n    print("debug")\n    breakpoint()\n',
        encoding="utf-8",
    )

    report = run_python_lang_harness([tmp_path])

    assert report.is_clean


def test_modern_design_rule_pack_descriptor_is_stable() -> None:
    descriptor = PythonModernDesignRulePack().descriptor()

    assert descriptor.id == "python.modern_design"
    assert descriptor.version == "v1"
    assert descriptor.default_mode == "deterministic"
    assert descriptor.to_dict()["domains"] == ["modern-python", "design", "python"]


def test_modern_design_rule_catalog_is_compact_and_stable() -> None:
    rules = python_modern_design_rules()

    assert [rule.rule_id for rule in rules] == [
        "PY-MOD-R001",
        "PY-MOD-R002",
        "PY-MOD-R003",
        "PY-MOD-R004",
    ]
    assert {rule.pack_id for rule in rules} == {"python.modern_design"}
    assert {rule.severity for rule in rules} == {PythonDiagnosticSeverity.WARNING}
    assert rules[-1].to_dict() == {
        "rule_id": "PY-MOD-R004",
        "pack_id": "python.modern_design",
        "severity": "warning",
        "title": "Library module contains breakpoint()",
        "requirement": "Remove `breakpoint()` from library modules; use test-only debug tooling or a project-owned diagnostic surface.",
        "labels": {"language": "python", "domain": "modern-python"},
    }


def test_assert_python_lang_harness_clean_blocks_for_pytest(tmp_path: Path) -> None:
    bad = tmp_path / "bad.py"
    bad.write_text("def broken(:\n    pass\n", encoding="utf-8")

    try:
        assert_python_lang_harness_clean([bad])
    except AssertionError as error:
        message = str(error)
    else:
        raise AssertionError("harness should block invalid Python source")

    assert "[lint:error]" in message
    assert "python.syntax.invalid" in message


def test_assert_python_lang_harness_clean_blocks_warning_findings(
    tmp_path: Path,
) -> None:
    source = tmp_path / "module.py"
    source.write_text("VALUE = 1\n", encoding="utf-8")

    try:
        assert_python_lang_harness_clean([source], rule_packs=(_WarningRulePack(),))
    except AssertionError as error:
        message = str(error)
    else:
        raise AssertionError("harness should block warning findings")

    assert "[lint:warning]" in message
    assert "[python.project.warning] Warning: Project warning" in message


def test_assert_python_lang_harness_clean_honors_configured_blocking_severities(
    tmp_path: Path,
) -> None:
    source = tmp_path / "module.py"
    source.write_text("VALUE = 1\n", encoding="utf-8")
    config = PythonHarnessConfig(
        blocking_severities=frozenset({PythonDiagnosticSeverity.ERROR}),
        rule_packs=(_WarningRulePack(),),
    )

    report = assert_python_lang_harness_clean([source], config=config)

    assert [finding.rule_id for finding in report.findings] == [
        "python.project.warning"
    ]


def test_default_python_harness_config_uses_default_rule_packs() -> None:
    config = default_python_harness_config()

    assert config.ignored_dir_names
    assert config.blocking_severities == {
        PythonDiagnosticSeverity.ERROR,
        PythonDiagnosticSeverity.WARNING,
    }
    assert [rule_pack.pack_id for rule_pack in config.rule_packs or ()] == [
        "python.syntax",
        "python.modern_design",
    ]


class _WarningRulePack:
    pack_id = "test.warning"

    def evaluate(self, report: PythonModuleReport) -> tuple[PythonHarnessFinding, ...]:
        return (
            PythonHarnessFinding(
                rule_id="python.project.warning",
                pack_id=self.pack_id,
                severity=PythonDiagnosticSeverity.WARNING,
                title="Project warning",
                summary="warning emitted by a project rule",
                location=SourceLocation(path=report.path, line=1, column=0),
                requirement="Fix the project rule warning.",
                source_line="VALUE = 1",
            ),
        )
