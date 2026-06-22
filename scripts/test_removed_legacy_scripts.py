from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[1]

REMOVED_LEGACY_SCRIPTS = (
    "scripts/runtime/memory_ci_finalize.py",
    "scripts/runtime/memory_ci_finalize_cli.py",
    "scripts/runtime/memory_ci_finalize_payloads.py",
    "scripts/runtime/memory_ci_finalize_runtime.py",
)


def test_removed_legacy_memory_ci_finalizer_scripts_stay_removed() -> None:
    for relative_path in REMOVED_LEGACY_SCRIPTS:
        assert not (PROJECT_ROOT / relative_path).exists(), relative_path


def test_scripts_readme_does_not_list_removed_legacy_memory_finalizer() -> None:
    readme = (PROJECT_ROOT / "scripts" / "README.md").read_text(encoding="utf-8")
    assert "channel/memory_ci_finalize.py" not in readme
    assert "scripts/runtime/memory_ci_finalize.py" not in readme
