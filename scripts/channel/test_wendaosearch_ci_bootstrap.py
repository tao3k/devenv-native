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
        "WENDAOSEARCH_REV=903c9e03ed8da3ed8f71b68cf947eaa20894affc" in workflow
    )
    assert (
        'git -C "${WENDAOSEARCH_PACKAGE_DIR}" fetch --depth 1 origin "${WENDAOSEARCH_REV}"'
        in workflow
    )
    assert 'checkout --detach FETCH_HEAD' in workflow
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
    assert "wendaosearch-solver-demo" in workflow
    assert "run_search_service.jl" in workflow
    assert "WENDAOSEARCH_SOLVER_DEMO_BASE_URL=http://127.0.0.1:" in workflow
    assert "RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST=1" in workflow


def test_ci_no_longer_uses_workspace_wendaosearch_checkout() -> None:
    workflow = CI_WORKFLOW.read_text(encoding="utf-8")

    assert (
        'WENDAOSEARCH_JULIA_PROJECT="${GITHUB_WORKSPACE}/.data/WendaoSearch.jl"'
        not in workflow
    )
    assert "${RUNNER_TEMP}/WendaoSearch.jl" in workflow
