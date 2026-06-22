from __future__ import annotations

from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[2]
CI_WORKFLOW = PROJECT_ROOT / ".github/workflows/ci.yaml"


def test_ci_no_longer_bootstraps_process_managed_wendaocodeparser() -> None:
    workflow = CI_WORKFLOW.read_text(encoding="utf-8")

    assert "wendaosearch-common.sh" not in workflow
    assert "wendaosearch_materialize_package_repo" not in workflow
    assert "wendaosearch-launch.sh" not in workflow
    assert "wendaosearch-healthcheck.sh" not in workflow
    assert "WendaoCodeParser parser-summary service" not in workflow
    assert "WendaoCodeParser.jl" not in workflow
    assert "WENDAO_CODE_PARSER_PACKAGE_DIR" not in workflow
    assert "WENDAO_PARSER_SUMMARY_BASE_URL" not in workflow
    assert "RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST" not in workflow
    assert "WENDAOSEARCH_REV" not in workflow
    assert 'Pkg.update("Absyn")' not in workflow
    assert "OpenModelicaRegistry" not in workflow


def test_ci_no_longer_bootstraps_process_managed_wendaosearch_solver_demo() -> None:
    workflow = CI_WORKFLOW.read_text(encoding="utf-8")

    assert "Checkout WendaoSearch solver-demo source" not in workflow
    assert "repository: tao3k/WendaoSearch.jl" not in workflow
    assert "WendaoSearch.jl" not in workflow
    assert "WENDAOSEARCH_REPO_TOKEN" not in workflow
    assert "auth_header" not in workflow
    assert "x-access-token" not in workflow
    assert "http.https://github.com/.extraheader" not in workflow
    assert "https://github.com/tao3k/WendaoSearch.jl.git" not in workflow
    assert "WendaoSearch main Project.toml must declare HiGHS" not in workflow
    assert "ensure_registry" not in workflow
    assert "WENDAOSEARCH_JULIA_PROJECT" not in workflow
    assert "WendaoArrow" not in workflow
    assert "WENDAOSEARCH_PACKAGE_DIR" not in workflow
    assert "wendaosearch-solver-demo" not in workflow
    assert "run_search_service.jl" not in workflow
    assert "config/live/solver_demo.toml" not in workflow
    assert "WENDAOSEARCH_SOLVER_DEMO_CONFIG" not in workflow
    assert (
        '"--route-names" "capability_manifest,structural_rerank,constraint_filter"' not in workflow
    )
    assert '"--mode" "solver_demo"' not in workflow
    assert "WENDAOSEARCH_SOLVER_DEMO_BASE_URL" not in workflow


def test_ci_no_longer_uses_workspace_julia_checkouts() -> None:
    workflow = CI_WORKFLOW.read_text(encoding="utf-8")

    assert 'WENDAOSEARCH_JULIA_PROJECT="${GITHUB_WORKSPACE}/.data/WendaoSearch.jl"' not in workflow
    assert "${RUNNER_TEMP}/WendaoSearch.jl" not in workflow
    assert "${PRJ_CACHE_HOME}/ci/WendaoSearch.jl" not in workflow
    assert "${RUNNER_TEMP}/WendaoCodeParser.jl" not in workflow
