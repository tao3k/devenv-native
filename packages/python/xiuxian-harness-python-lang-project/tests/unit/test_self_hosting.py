from __future__ import annotations

from pathlib import Path

from xiuxian_harness_python_lang_project import python_project_harness_test

_REPO_ROOT = next(
    parent
    for parent in Path(__file__).resolve().parents
    if (parent / "packages").exists() and (parent / "pyproject.toml").exists()
)
_PYTHON_LANG_PARSER_ROOT = _REPO_ROOT / "packages/python/python-lang-parser"
_PYTHON_HARNESS_ROOT = (
    _REPO_ROOT / "packages/python/xiuxian-harness-python-lang-project"
)


test_python_lang_parser_package_harness = python_project_harness_test(
    _PYTHON_LANG_PARSER_ROOT
)
test_python_harness_package_harness = python_project_harness_test(_PYTHON_HARNESS_ROOT)
