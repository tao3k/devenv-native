from __future__ import annotations

from typing import TYPE_CHECKING

from xiuxian_harness_python_lang_project import python_project_harness_test

if TYPE_CHECKING:
    from pathlib import Path


def test_python_project_harness_test_returns_pytest_collectable_callable(
    tmp_path: Path,
) -> None:
    src = tmp_path / "src"
    tests = tmp_path / "tests" / "unit"
    src.mkdir()
    tests.mkdir(parents=True)
    (src / "library.py").write_text(
        '"""Library docs."""\n\nVALUE = 1\n', encoding="utf-8"
    )
    (tests / "test_library.py").write_text(
        "def test_value() -> None:\n    assert True\n",
        encoding="utf-8",
    )

    harness_test = python_project_harness_test(tmp_path)

    assert harness_test.__name__ == "test_python_project_harness_policy"
    assert harness_test.__qualname__ == "test_python_project_harness_policy"
    harness_test()


def test_python_project_harness_test_blocks_with_compact_snapshot(
    tmp_path: Path,
) -> None:
    src = tmp_path / "src"
    src.mkdir()
    source = src / "library.py"
    source.write_text('def run() -> None:\n    print("debug")\n', encoding="utf-8")
    harness_test = python_project_harness_test(tmp_path)

    try:
        harness_test()
    except AssertionError as error:
        message = str(error)
    else:
        raise AssertionError("pytest harness callable should block policy findings")

    assert "[PY-MOD-R002] Warning: Library module uses bare print" in message
    assert str(source) in message
