from __future__ import annotations

from typing import TYPE_CHECKING

from xiuxian_harness_python_lang_project import (
    PythonProjectPolicyRulePack,
    python_project_policy_rules,
    run_python_project_harness,
)
from xiuxian_harness_python_lang_project._project_metadata import (
    read_python_project_metadata,
)

if TYPE_CHECKING:
    from pathlib import Path


def test_read_python_project_metadata_collects_hatch_package_roots(
    tmp_path: Path,
) -> None:
    package = tmp_path / "src" / "example_pkg"
    package.mkdir(parents=True)
    _write_pyproject(
        tmp_path,
        packages='["src/example_pkg"]',
        project_name="example-pkg",
        requires_python=">=3.12",
        build_backend="hatchling.build",
    )

    metadata = read_python_project_metadata(tmp_path)

    assert metadata is not None
    assert metadata.project_name == "example-pkg"
    assert metadata.requires_python == ">=3.12"
    assert metadata.build_backend == "hatchling.build"
    assert metadata.wheel_packages == ("src/example_pkg",)
    assert metadata.package_roots == (package,)


def test_read_python_project_metadata_returns_none_without_pyproject(
    tmp_path: Path,
) -> None:
    assert read_python_project_metadata(tmp_path) is None


def test_read_python_project_metadata_returns_none_for_malformed_pyproject(
    tmp_path: Path,
) -> None:
    (tmp_path / "pyproject.toml").write_text("[project\n", encoding="utf-8")

    assert read_python_project_metadata(tmp_path) is None


def test_read_python_project_metadata_ignores_unsupported_package_values(
    tmp_path: Path,
) -> None:
    _write_pyproject(tmp_path, packages="[42]")

    metadata = read_python_project_metadata(tmp_path)

    assert metadata is not None
    assert metadata.wheel_packages == ()
    assert metadata.package_roots == ()


def test_read_python_project_metadata_compacts_duplicate_package_roots(
    tmp_path: Path,
) -> None:
    package = tmp_path / "src" / "example_pkg"
    package.mkdir(parents=True)
    _write_pyproject(
        tmp_path,
        packages='["src/example_pkg", "src/./example_pkg", 42, "src/example_pkg"]',
    )

    metadata = read_python_project_metadata(tmp_path)

    assert metadata is not None
    assert metadata.wheel_packages == ("src/example_pkg",)
    assert metadata.package_roots == (package,)


def test_project_policy_noops_without_pyproject(tmp_path: Path) -> None:
    package = tmp_path / "pkg"
    package.mkdir()
    (package / "__init__.py").write_text(
        '"""Package public API."""\n\n\ndef build(value: int) -> int:\n    return value\n',
        encoding="utf-8",
    )

    report = run_python_project_harness(tmp_path)

    assert not any(
        finding.rule_id.startswith("PY-PROJ-") for finding in report.findings
    )


def test_project_policy_blocks_flat_layout_with_pyproject(tmp_path: Path) -> None:
    package = tmp_path / "pkg"
    package.mkdir()
    (package / "__init__.py").write_text(
        '"""Package public API."""\n', encoding="utf-8"
    )
    (package / "py.typed").write_text("", encoding="utf-8")
    _write_pyproject(tmp_path, packages='["pkg"]')

    report = run_python_project_harness(tmp_path)

    assert [
        (finding.rule_id, finding.location.path) for finding in report.findings
    ] == [
        ("PY-PROJ-R001", str(tmp_path / "pyproject.toml")),
    ]


def test_project_policy_blocks_missing_declared_package_root(
    tmp_path: Path,
) -> None:
    (tmp_path / "src").mkdir()
    _write_pyproject(tmp_path, packages='["src/missing_pkg"]')

    report = run_python_project_harness(tmp_path)

    assert [
        (finding.rule_id, finding.location.path) for finding in report.findings
    ] == [
        ("PY-PROJ-R002", str(tmp_path / "pyproject.toml")),
    ]


def test_project_policy_blocks_package_root_without_init(tmp_path: Path) -> None:
    package = tmp_path / "src" / "example_pkg"
    package.mkdir(parents=True)
    _write_pyproject(tmp_path, packages='["src/example_pkg"]')

    report = run_python_project_harness(tmp_path)

    assert [
        (finding.rule_id, finding.location.path) for finding in report.findings
    ] == [
        ("PY-PROJ-R002", str(package)),
    ]


def test_project_policy_deduplicates_declared_package_findings(
    tmp_path: Path,
) -> None:
    (tmp_path / "src").mkdir()
    _write_pyproject(
        tmp_path,
        packages='["src/missing_pkg", "src/./missing_pkg", "src/missing_pkg"]',
    )

    report = run_python_project_harness(tmp_path)

    assert [
        (finding.rule_id, finding.location.path) for finding in report.findings
    ] == [
        ("PY-PROJ-R002", str(tmp_path / "pyproject.toml")),
    ]


def test_project_policy_blocks_missing_py_typed_for_public_package(
    tmp_path: Path,
) -> None:
    package = tmp_path / "src" / "example_pkg"
    package.mkdir(parents=True)
    (package / "__init__.py").write_text(
        '"""Package public API."""\n\n\ndef build(value: int) -> int:\n    return value\n',
        encoding="utf-8",
    )
    _write_pyproject(tmp_path, packages='["src/example_pkg"]')

    report = run_python_project_harness(tmp_path)

    assert [
        (finding.rule_id, finding.location.path) for finding in report.findings
    ] == [
        ("PY-PROJ-R003", str(package)),
    ]


def test_project_policy_blocks_missing_py_typed_for_public_facade_imports(
    tmp_path: Path,
) -> None:
    package = tmp_path / "src" / "example_pkg"
    package.mkdir(parents=True)
    (package / "__init__.py").write_text(
        '"""Package public API."""\n\nfrom .service import Service\n\n__all__ = ("Service",)\n',
        encoding="utf-8",
    )
    (package / "service.py").write_text(
        '"""Service implementation."""\n\n\nclass Service:\n    pass\n',
        encoding="utf-8",
    )
    _write_pyproject(tmp_path, packages='["src/example_pkg"]')

    report = run_python_project_harness(tmp_path)

    assert [
        (finding.rule_id, finding.location.path) for finding in report.findings
    ] == [
        ("PY-PROJ-R003", str(package)),
    ]


def test_project_policy_allows_private_package_without_py_typed(
    tmp_path: Path,
) -> None:
    package = tmp_path / "src" / "example_pkg"
    package.mkdir(parents=True)
    (package / "__init__.py").write_text('"""Internal package."""\n', encoding="utf-8")
    (package / "_internal.py").write_text(
        '"""Internal helpers."""\n\n\ndef _build(value: int) -> int:\n    return value\n',
        encoding="utf-8",
    )
    _write_pyproject(tmp_path, packages='["src/example_pkg"]')

    report = run_python_project_harness(tmp_path)

    assert report.is_clean


def test_project_policy_accepts_src_package_with_py_typed(tmp_path: Path) -> None:
    package = tmp_path / "src" / "example_pkg"
    package.mkdir(parents=True)
    (package / "__init__.py").write_text(
        '"""Package public API."""\n\n\ndef build(value: int) -> int:\n    return value\n',
        encoding="utf-8",
    )
    (package / "py.typed").write_text("", encoding="utf-8")
    _write_pyproject(tmp_path, packages='["src/example_pkg"]')

    report = run_python_project_harness(tmp_path)

    assert report.is_clean


def test_project_policy_blocks_unannotated_public_callable_in_typed_package(
    tmp_path: Path,
) -> None:
    package = tmp_path / "src" / "example_pkg"
    package.mkdir(parents=True)
    (package / "__init__.py").write_text(
        '"""Package public API."""\n', encoding="utf-8"
    )
    (package / "service.py").write_text(
        '"""Service helpers."""\n\n\ndef build(value):\n    return value\n',
        encoding="utf-8",
    )
    (package / "py.typed").write_text("", encoding="utf-8")
    _write_pyproject(tmp_path, packages='["src/example_pkg"]')

    report = run_python_project_harness(tmp_path)

    assert [
        (finding.rule_id, finding.location.path) for finding in report.findings
    ] == [
        ("PY-PROJ-R004", str(package / "service.py")),
    ]
    assert "PY-AGENT-R002" not in {finding.rule_id for finding in report.findings}


def test_project_policy_blocks_unannotated_public_method_in_typed_package(
    tmp_path: Path,
) -> None:
    package = tmp_path / "src" / "example_pkg"
    package.mkdir(parents=True)
    (package / "__init__.py").write_text(
        '"""Package public API."""\n', encoding="utf-8"
    )
    (package / "service.py").write_text(
        '"""Service helpers."""\n\n\nclass Service:\n    def build(self, value):\n        return value\n',
        encoding="utf-8",
    )
    (package / "py.typed").write_text("", encoding="utf-8")
    _write_pyproject(tmp_path, packages='["src/example_pkg"]')

    report = run_python_project_harness(tmp_path)

    assert [
        (finding.rule_id, finding.location.path) for finding in report.findings
    ] == [
        ("PY-PROJ-R004", str(package / "service.py")),
    ]
    assert report.findings[0].source_line == "    def build(self, value):"


def test_project_policy_allows_private_callable_without_annotations_in_typed_package(
    tmp_path: Path,
) -> None:
    package = tmp_path / "src" / "example_pkg"
    package.mkdir(parents=True)
    (package / "__init__.py").write_text(
        '"""Package public API."""\n', encoding="utf-8"
    )
    (package / "service.py").write_text(
        '"""Service helpers."""\n\n\ndef _build(value):\n    return value\n',
        encoding="utf-8",
    )
    (package / "py.typed").write_text("", encoding="utf-8")
    _write_pyproject(tmp_path, packages='["src/example_pkg"]')

    report = run_python_project_harness(tmp_path)

    assert report.is_clean


def test_project_policy_accepts_annotated_method_in_typed_package(
    tmp_path: Path,
) -> None:
    package = tmp_path / "src" / "example_pkg"
    package.mkdir(parents=True)
    (package / "__init__.py").write_text(
        '"""Package public API."""\n', encoding="utf-8"
    )
    (package / "service.py").write_text(
        '"""Service helpers."""\n\n\nclass Service:\n    def build(self, value: int) -> int:\n        return value\n',
        encoding="utf-8",
    )
    (package / "py.typed").write_text("", encoding="utf-8")
    _write_pyproject(tmp_path, packages='["src/example_pkg"]')

    report = run_python_project_harness(tmp_path)

    assert report.is_clean


def test_project_policy_rule_pack_descriptor_and_catalog_are_stable() -> None:
    descriptor = PythonProjectPolicyRulePack().descriptor()
    rules = python_project_policy_rules()

    assert descriptor.id == "python.project_policy"
    assert descriptor.to_dict()["domains"] == [
        "project-policy",
        "packaging",
        "python",
    ]
    assert [rule.rule_id for rule in rules] == [
        "PY-PROJ-R001",
        "PY-PROJ-R002",
        "PY-PROJ-R003",
        "PY-PROJ-R004",
    ]


def _write_pyproject(
    project_root: Path,
    *,
    packages: str,
    project_name: str = "example-pkg",
    requires_python: str = ">=3.12",
    build_backend: str = "hatchling.build",
) -> None:
    (project_root / "pyproject.toml").write_text(
        f"""
[project]
name = "{project_name}"
requires-python = "{requires_python}"

[build-system]
build-backend = "{build_backend}"

[tool.hatch.build.targets.wheel]
packages = {packages}
""".lstrip(),
        encoding="utf-8",
    )
