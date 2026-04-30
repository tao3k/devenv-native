"""Pytest-facing helpers for embedding the Python project harness."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

from ._runner import assert_python_project_harness_clean

if TYPE_CHECKING:
    from collections.abc import Callable, Sequence

    from python_lang_parser import PythonDiagnosticSeverity

    from ._model import PythonHarnessConfig, PythonLangRulePack


def python_project_harness_test(
    project_root: str | Path,
    *,
    config: PythonHarnessConfig | None = None,
    rule_packs: Sequence[PythonLangRulePack] | None = None,
    severities: frozenset[PythonDiagnosticSeverity] | None = None,
    include_tests: bool = True,
    source_dir_names: Sequence[str] = ("src",),
    test_dir_names: Sequence[str] = ("tests",),
    test_name: str = "test_python_project_harness_policy",
    include_advice: bool = True,
) -> Callable[[], None]:
    """Return a pytest-collectable test function for one Python project."""

    root = Path(project_root)

    def test_python_project_harness_policy() -> None:
        assert_python_project_harness_clean(
            root,
            config=config,
            rule_packs=rule_packs,
            severities=severities,
            include_tests=include_tests,
            source_dir_names=source_dir_names,
            test_dir_names=test_dir_names,
            include_advice=include_advice,
        )

    test_python_project_harness_policy.__name__ = test_name
    test_python_project_harness_policy.__qualname__ = test_name
    test_python_project_harness_policy.__doc__ = (
        "Run the Xiuxian Python project harness over src/** and tests/**."
    )
    return test_python_project_harness_policy
