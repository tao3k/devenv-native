from __future__ import annotations

from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parents[2]
CI_WORKFLOW = PROJECT_ROOT / ".github/workflows/ci.yaml"


def test_ci_bootstraps_wendaosearch_with_julia_pkg() -> None:
    workflow = CI_WORKFLOW.read_text(encoding="utf-8")

    assert "wendaosearch-common.sh" not in workflow
    assert "wendaosearch_materialize_package_repo" not in workflow
    assert "wendaosearch-launch.sh" not in workflow
    assert "wendaosearch-healthcheck.sh" not in workflow
    assert (
        'Pkg.add(Pkg.PackageSpec(url="https://github.com/tao3k/WendaoSearch.jl.git"))'
        not in workflow
    )
    assert (
        "git clone --depth 1 https://github.com/tao3k/WendaoSearch.jl.git"
        in workflow
    )
    assert 'WENDAOSEARCH_JULIA_PROJECT="${WENDAOSEARCH_PACKAGE_DIR}"' in workflow
    assert "WendaoCodeParser" not in workflow
    assert "WendaoArrow" not in workflow
    assert "Pkg.instantiate()" in workflow
    assert 'Pkg.update("Absyn")' in workflow
    assert "WENDAOSEARCH_PACKAGE_DIR" in workflow
    assert (
        "WENDAOSEARCH_CONFIG=${WENDAOSEARCH_PACKAGE_DIR}/config/live/parser_summary.toml"
        in workflow
    )
    assert "RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST=1" in workflow


def test_ci_no_longer_uses_workspace_wendaosearch_checkout() -> None:
    workflow = CI_WORKFLOW.read_text(encoding="utf-8")

    assert (
        'WENDAOSEARCH_JULIA_PROJECT="${GITHUB_WORKSPACE}/.data/WendaoSearch.jl"'
        not in workflow
    )
    assert "${RUNNER_TEMP}/WendaoSearch.jl" in workflow
