from __future__ import annotations

from typing import TYPE_CHECKING

from python_lang_parser import PythonDiagnosticSeverity, SourceLocation
from xiuxian_harness_python_lang_project import (
    PythonHarnessConfig,
    PythonHarnessFinding,
    assert_python_lang_harness_clean,
    default_python_harness_config,
    discover_python_files,
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
    good.write_text(
        '"""Good module."""\n\n\ndef ok() -> None:\n    return None\n', encoding="utf-8"
    )
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


def test_run_python_lang_harness_uses_configured_blocking_severities(
    tmp_path: Path,
) -> None:
    source = tmp_path / "module.py"
    source.write_text("VALUE = 1\n", encoding="utf-8")
    config = PythonHarnessConfig(
        blocking_severities=frozenset({PythonDiagnosticSeverity.ERROR}),
        rule_packs=(_WarningRulePack(),),
    )

    report = run_python_lang_harness([source], config=config)

    assert [finding.rule_id for finding in report.findings] == [
        "python.project.warning"
    ]
    assert report.blocking_findings() == ()
    assert report.is_clean
    assert report.to_dict()["blocking_severities"] == ["error"]
    assert render_python_lang_harness(report).startswith("[ok]")


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
    assert report.is_clean


def test_assert_python_lang_harness_clean_honors_legacy_severities_override(
    tmp_path: Path,
) -> None:
    source = tmp_path / "module.py"
    source.write_text("VALUE = 1\n", encoding="utf-8")
    config = PythonHarnessConfig(
        blocking_severities=frozenset({PythonDiagnosticSeverity.ERROR}),
        rule_packs=(_WarningRulePack(),),
    )

    try:
        assert_python_lang_harness_clean(
            [source],
            config=config,
            severities=frozenset({PythonDiagnosticSeverity.WARNING}),
        )
    except AssertionError as error:
        message = str(error)
    else:
        raise AssertionError("legacy severity override should still block warnings")

    assert "[lint:warning]" in message
    assert "[python.project.warning] Warning: Project warning" in message


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
        "python.agent_policy",
        "python.modularity",
        "python.test_layout",
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
