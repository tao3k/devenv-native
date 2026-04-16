from __future__ import annotations

import importlib.util
import sys
from pathlib import Path
from urllib.parse import parse_qs, urlparse


PROJECT_ROOT = Path(__file__).resolve().parents[1]
SCRIPT_PATH = PROJECT_ROOT / "scripts" / "benchmark_wendao_gateway_repo_get.py"
SPEC = importlib.util.spec_from_file_location(
    "benchmark_wendao_gateway_repo_get",
    SCRIPT_PATH,
)
assert SPEC is not None
assert SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


def test_parse_args_defaults() -> None:
    args = MODULE.parse_args([])
    assert args.base_url == "http://127.0.0.1:9517"
    assert args.endpoint == "symbol-search"
    assert args.all_repos is False
    assert args.limit == 10
    assert args.warm_runs == 2
    assert args.runs == 7


def test_build_request_url_encodes_symbol_search_query() -> None:
    url = MODULE.build_request_url(
        base_url="http://127.0.0.1:9517/",
        endpoint="symbol-search",
        repo="ADTypes.jl",
        query="AD Type",
        limit=5,
        package=None,
        module=None,
    )
    parsed = urlparse(url)
    query = parse_qs(parsed.query)

    assert parsed.scheme == "http"
    assert parsed.netloc == "127.0.0.1:9517"
    assert parsed.path == "/api/repo/symbol-search"
    assert query == {
        "repo": ["ADTypes.jl"],
        "query": ["AD Type"],
        "limit": ["5"],
    }


def test_summarize_gateway_state_and_count_results() -> None:
    snapshot = MODULE.summarize_gateway_state(
        {
            "projects": [{"name": "frontend"}],
            "repoProjects": [{"id": "ADTypes.jl"}, {"id": "SciMLBase.jl"}],
            "studioBootstrapBackgroundIndexingEnabled": True,
            "studioBootstrapBackgroundIndexingMode": "enabled",
            "studioBootstrapBackgroundIndexingDeferredActivationObserved": False,
        },
        {
            "total": 2,
            "ready": 1,
            "unsupported": 1,
            "failed": 0,
            "targetConcurrency": 1,
            "maxConcurrency": 12,
            "syncConcurrencyLimit": 8,
        },
    )

    assert snapshot.project_count == 1
    assert snapshot.repo_project_count == 2
    assert snapshot.bootstrap_enabled is True
    assert snapshot.bootstrap_mode == "enabled"
    assert snapshot.first_repo_id == "ADTypes.jl"
    assert MODULE.count_results("symbol-search", {"symbols": [1, 2, 3]}) == 3
    assert MODULE.count_results("module-search", {"modules": [1]}) == 1
    assert MODULE.count_results("example-search", {"examples": [1, 2]}) == 2
    assert MODULE.count_results("import-search", {"imports": [1, 2, 3, 4]}) == 4
    assert MODULE.count_results("index-status", {"total": 178}) == 178


def test_live_repo_ids_and_repo_sweep_summary() -> None:
    repo_ids = MODULE.live_repo_ids(
        {
            "repoProjects": [
                {"id": "ADTypes.jl"},
                {"id": "SciMLBase.jl"},
                {"id": ""},
                {"id": "OrdinaryDiffEq.jl"},
            ]
        }
    )
    assert repo_ids == ["ADTypes.jl", "SciMLBase.jl", "OrdinaryDiffEq.jl"]

    summary = MODULE.summarize_repo_sweep(
        [
            MODULE.RepoSweepCase(
                repo_id="ADTypes.jl",
                query="ADTypes",
                elapsed_ms=2.0,
                ok=True,
                status=200,
                result_count=3,
                error=None,
            ),
            MODULE.RepoSweepCase(
                repo_id="SciMLBase.jl",
                query="SciMLBase",
                elapsed_ms=4.0,
                ok=True,
                status=200,
                result_count=0,
                error=None,
            ),
            MODULE.RepoSweepCase(
                repo_id="BrokenRepo",
                query="BrokenRepo",
                elapsed_ms=8.0,
                ok=False,
                status=500,
                result_count=0,
                error="http-500",
            ),
        ]
    )

    assert summary["total_repos"] == 3
    assert summary["non_empty_repos"] == 1
    assert summary["empty_repos"] == 1
    assert summary["failed_repos"] == 1
    assert summary["non_empty_repo_ids"] == ["ADTypes.jl"]
    assert summary["empty_repo_ids"] == ["SciMLBase.jl"]
    assert summary["failed_repo_ids"] == ["BrokenRepo"]
