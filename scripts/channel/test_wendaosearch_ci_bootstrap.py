from __future__ import annotations

from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parents[2]
CI_WORKFLOW = PROJECT_ROOT / ".github/workflows/ci.yaml"


def test_ci_bootstraps_parser_summary_with_wendaocodeparser() -> None:
    workflow = CI_WORKFLOW.read_text(encoding="utf-8")

    assert "wendaosearch-common.sh" not in workflow
    assert "wendaosearch_materialize_package_repo" not in workflow
    assert "wendaosearch-launch.sh" not in workflow
    assert "wendaosearch-healthcheck.sh" not in workflow
    assert (
        'Pkg.add(Pkg.PackageSpec(url="https://github.com/tao3k/WendaoSearch.jl.git"))'
        not in workflow
    )
    assert "WENDAOSEARCH_REV" not in workflow
    assert (
        "git clone --depth 1 --branch main https://github.com/tao3k/WendaoCodeParser.jl.git"
        in workflow
    )
    assert 'git -C "${WENDAO_CODE_PARSER_PACKAGE_DIR}" rev-parse HEAD' in workflow
    assert (
        'WENDAO_CODE_PARSER_JULIA_PROJECT="${WENDAO_CODE_PARSER_PACKAGE_DIR}"'
        in workflow
    )
    assert "scripts/run_service.jl" in workflow
    assert "WendaoCodeParser parser-summary service" in workflow
    assert 'Pkg.update("Absyn")' not in workflow
    assert "OpenModelicaRegistry" not in workflow
    assert "WENDAO_PARSER_SUMMARY_BASE_URL=http://127.0.0.1:41081" in workflow
    assert (
        "WENDAOSEARCH_CONFIG=${WENDAO_CODE_PARSER_PACKAGE_DIR}/config/live/parser_summary.toml"
        in workflow
    )
    assert "RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST=1" in workflow


def test_ci_bootstraps_wendaosearch_solver_demo_with_julia_pkg() -> None:
    workflow = CI_WORKFLOW.read_text(encoding="utf-8")

    assert (
        "git clone --depth 1 --branch main https://github.com/tao3k/WendaoSearch.jl.git"
        in workflow
    )
    assert 'git -C "${WENDAOSEARCH_PACKAGE_DIR}" rev-parse HEAD' in workflow
    assert "WendaoSearch main Project.toml must declare HiGHS" in workflow
    assert 'WENDAOSEARCH_JULIA_PROJECT="${WENDAOSEARCH_PACKAGE_DIR}"' in workflow
    assert "WendaoArrow" not in workflow
    assert "Pkg.instantiate()" in workflow
    assert "WENDAOSEARCH_PACKAGE_DIR" in workflow
    assert "wendaosearch-solver-demo" in workflow
    assert "run_search_service.jl" in workflow
    assert "WENDAOSEARCH_SOLVER_DEMO_BASE_URL=http://127.0.0.1:" in workflow


def test_ci_no_longer_uses_workspace_julia_checkouts() -> None:
    workflow = CI_WORKFLOW.read_text(encoding="utf-8")

    assert (
        'WENDAOSEARCH_JULIA_PROJECT="${GITHUB_WORKSPACE}/.data/WendaoSearch.jl"'
        not in workflow
    )
    assert "${RUNNER_TEMP}/WendaoSearch.jl" in workflow
    assert "${RUNNER_TEMP}/WendaoCodeParser.jl" in workflow
