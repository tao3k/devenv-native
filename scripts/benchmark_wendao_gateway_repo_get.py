#!/usr/bin/env python3
"""
Benchmark live Wendao gateway repo GET latency.

Examples:
  uv run python scripts/benchmark_wendao_gateway_repo_get.py
  uv run python scripts/benchmark_wendao_gateway_repo_get.py --repo ADTypes.jl --query ADType
  uv run python scripts/benchmark_wendao_gateway_repo_get.py --endpoint module-search --repo SciMLBase.jl --query solve
  uv run python scripts/benchmark_wendao_gateway_repo_get.py --json
"""

from __future__ import annotations

import argparse
import json
import statistics
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import asdict, dataclass
from typing import Any


DEFAULT_BASE_URL = "http://127.0.0.1:9517"
DEFAULT_ENDPOINT = "symbol-search"


@dataclass(frozen=True)
class GatewaySnapshot:
    project_count: int
    repo_project_count: int
    bootstrap_enabled: bool | None
    bootstrap_mode: str | None
    deferred_activation_observed: bool | None
    total_repos: int
    ready_repos: int
    unsupported_repos: int
    failed_repos: int
    target_concurrency: int | None
    max_concurrency: int | None
    sync_concurrency_limit: int | None
    first_repo_id: str | None


@dataclass(frozen=True)
class RepoSweepCase:
    repo_id: str
    query: str | None
    elapsed_ms: float
    ok: bool
    status: int
    result_count: int
    error: str | None


@dataclass(frozen=True)
class RunResult:
    elapsed_ms: float
    ok: bool
    status: int
    result_count: int
    error: str | None


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Benchmark live Wendao gateway repo GET latency",
    )
    parser.add_argument(
        "--base-url",
        default=DEFAULT_BASE_URL,
        help="Gateway base URL",
    )
    parser.add_argument(
        "--endpoint",
        default=DEFAULT_ENDPOINT,
        choices=(
            "symbol-search",
            "module-search",
            "example-search",
            "import-search",
            "overview",
            "index-status",
        ),
        help="GET endpoint family under /api/repo/",
    )
    parser.add_argument("--repo", default=None, help="Repository ID")
    parser.add_argument(
        "--all-repos",
        action="store_true",
        help="Benchmark every live repo project exposed by /api/ui/capabilities",
    )
    parser.add_argument(
        "--repo-limit",
        type=int,
        default=0,
        help="Optional cap for --all-repos sweeps (disabled when <=0)",
    )
    parser.add_argument(
        "--query",
        default=None,
        help="Search query for search endpoints",
    )
    parser.add_argument("--package", default=None, help="Import-search package filter")
    parser.add_argument("--module", default=None, help="Import-search module filter")
    parser.add_argument("--limit", type=int, default=10, help="Result limit")
    parser.add_argument("--warm-runs", type=int, default=2, help="Warm-up runs")
    parser.add_argument("--runs", type=int, default=7, help="Measured runs")
    parser.add_argument(
        "--timeout-s",
        type=float,
        default=30.0,
        help="Timeout per GET request in seconds",
    )
    parser.add_argument(
        "--max-p95-ms",
        type=float,
        default=0.0,
        help="Fail when P95 exceeds this threshold (disabled when <=0)",
    )
    parser.add_argument(
        "--max-avg-ms",
        type=float,
        default=0.0,
        help="Fail when average exceeds this threshold (disabled when <=0)",
    )
    parser.add_argument("--json", action="store_true", help="Print JSON output")
    return parser.parse_args(argv)


def normalize_base_url(base_url: str) -> str:
    return base_url.rstrip("/")


def default_query_for_repo(repo_id: str) -> str:
    query = repo_id.strip().removesuffix(".git").removesuffix(".jl")
    return query.replace("_", " ").replace("-", " ").replace("/", " ").strip()


def count_results(endpoint: str, payload: dict[str, Any]) -> int:
    if endpoint == "symbol-search":
        return len(payload.get("symbols") or [])
    if endpoint == "module-search":
        return len(payload.get("modules") or [])
    if endpoint == "example-search":
        return len(payload.get("examples") or [])
    if endpoint == "import-search":
        return len(payload.get("imports") or [])
    if endpoint == "overview":
        return int(payload.get("symbol_count") or 0)
    if endpoint == "index-status":
        return int(payload.get("total") or 0)
    return 0


def build_request_url(
    *,
    base_url: str,
    endpoint: str,
    repo: str | None,
    query: str | None,
    limit: int,
    package: str | None,
    module: str | None,
) -> str:
    base = normalize_base_url(base_url)
    params: dict[str, str] = {}
    if endpoint != "index-status":
        if repo:
            params["repo"] = repo
    if endpoint in {"symbol-search", "module-search", "example-search"} and query:
        params["query"] = query
        params["limit"] = str(max(1, limit))
    elif endpoint == "import-search":
        if package:
            params["package"] = package
        if module:
            params["module"] = module
        params["limit"] = str(max(1, limit))

    path = "/api/repo/index/status" if endpoint == "index-status" else f"/api/repo/{endpoint}"
    if not params:
        return f"{base}{path}"
    return f"{base}{path}?{urllib.parse.urlencode(params)}"


def request_json(url: str, timeout_s: float) -> tuple[int, dict[str, Any], str | None]:
    request = urllib.request.Request(url, method="GET")
    try:
        with urllib.request.urlopen(request, timeout=timeout_s) as response:
            status = getattr(response, "status", 200)
            body = response.read().decode("utf-8")
    except urllib.error.HTTPError as error:
        status = error.code
        body = error.read().decode("utf-8", errors="replace")
    except urllib.error.URLError as error:
        return 0, {}, str(error.reason)

    try:
        payload = json.loads(body)
    except json.JSONDecodeError as error:
        return status, {}, f"invalid-json: {error}"

    if not isinstance(payload, dict):
        return status, {}, "unexpected-json-shape"
    return status, payload, None


def summarize_gateway_state(
    ui_capabilities: dict[str, Any],
    repo_index_status: dict[str, Any],
) -> GatewaySnapshot:
    repo_projects = ui_capabilities.get("repoProjects") or []
    first_repo_id = None
    if repo_projects and isinstance(repo_projects[0], dict):
        first_repo_id = repo_projects[0].get("id")
    return GatewaySnapshot(
        project_count=len(ui_capabilities.get("projects") or []),
        repo_project_count=len(repo_projects),
        bootstrap_enabled=ui_capabilities.get("studioBootstrapBackgroundIndexingEnabled"),
        bootstrap_mode=ui_capabilities.get("studioBootstrapBackgroundIndexingMode"),
        deferred_activation_observed=ui_capabilities.get(
            "studioBootstrapBackgroundIndexingDeferredActivationObserved"
        ),
        total_repos=int(repo_index_status.get("total") or 0),
        ready_repos=int(repo_index_status.get("ready") or 0),
        unsupported_repos=int(repo_index_status.get("unsupported") or 0),
        failed_repos=int(repo_index_status.get("failed") or 0),
        target_concurrency=repo_index_status.get("targetConcurrency"),
        max_concurrency=repo_index_status.get("maxConcurrency"),
        sync_concurrency_limit=repo_index_status.get("syncConcurrencyLimit"),
        first_repo_id=first_repo_id,
    )


def fetch_gateway_capabilities(base_url: str, timeout_s: float) -> dict[str, Any]:
    capabilities_status, capabilities_payload, capabilities_error = request_json(
        f"{normalize_base_url(base_url)}/api/ui/capabilities",
        timeout_s,
    )
    if capabilities_status < 200 or capabilities_status >= 300 or capabilities_error:
        raise RuntimeError(
            "failed to read /api/ui/capabilities: "
            f"status={capabilities_status} error={capabilities_error}"
        )
    return capabilities_payload


def fetch_gateway_snapshot(
    base_url: str, timeout_s: float
) -> tuple[GatewaySnapshot, dict[str, Any]]:
    capabilities_payload = fetch_gateway_capabilities(base_url, timeout_s)
    repo_status_code, repo_status_payload, repo_status_error = request_json(
        f"{normalize_base_url(base_url)}/api/repo/index/status",
        timeout_s,
    )
    if repo_status_code < 200 or repo_status_code >= 300 or repo_status_error:
        raise RuntimeError(
            "failed to read /api/repo/index/status: "
            f"status={repo_status_code} error={repo_status_error}"
        )
    return (
        summarize_gateway_state(capabilities_payload, repo_status_payload),
        capabilities_payload,
    )


def live_repo_ids(ui_capabilities: dict[str, Any]) -> list[str]:
    repo_projects = ui_capabilities.get("repoProjects") or []
    repo_ids: list[str] = []
    for repo_project in repo_projects:
        if not isinstance(repo_project, dict):
            continue
        repo_id = str(repo_project.get("id") or "").strip()
        if repo_id:
            repo_ids.append(repo_id)
    return repo_ids


def resolve_effective_request(
    *,
    snapshot: GatewaySnapshot,
    endpoint: str,
    repo: str | None,
    query: str | None,
    package: str | None,
    module: str | None,
) -> tuple[str | None, str | None]:
    effective_repo = repo
    effective_query = query
    if endpoint != "index-status" and not effective_repo:
        effective_repo = snapshot.first_repo_id
    if endpoint in {"symbol-search", "module-search", "example-search"} and not effective_query:
        if effective_repo:
            effective_query = default_query_for_repo(effective_repo)
    if endpoint == "import-search" and not package and not module:
        raise ValueError("--package or --module is required for import-search")
    if endpoint != "index-status" and not effective_repo:
        raise ValueError("no repository available to benchmark")
    if endpoint in {"symbol-search", "module-search", "example-search"} and not effective_query:
        raise ValueError("--query is required for this endpoint")
    return effective_repo, effective_query


def run_once(url: str, endpoint: str, timeout_s: float) -> RunResult:
    start = time.perf_counter()
    status, payload, error = request_json(url, timeout_s)
    elapsed_ms = (time.perf_counter() - start) * 1000.0
    if error is not None:
        return RunResult(
            elapsed_ms=elapsed_ms,
            ok=False,
            status=status,
            result_count=0,
            error=error,
        )
    ok = 200 <= status < 300
    return RunResult(
        elapsed_ms=elapsed_ms,
        ok=ok,
        status=status,
        result_count=count_results(endpoint, payload),
        error=None if ok else f"http-{status}",
    )


def p95_ms(values: list[float]) -> float:
    if not values:
        return 0.0
    if len(values) == 1:
        return values[0]
    ordered = sorted(values)
    index = max(0, round(0.95 * (len(ordered) - 1)))
    return ordered[index]


def summarize_repo_sweep(cases: list[RepoSweepCase]) -> dict[str, Any]:
    elapsed = [case.elapsed_ms for case in cases]
    non_empty = [case.repo_id for case in cases if case.ok and case.result_count > 0]
    empty = [case.repo_id for case in cases if case.ok and case.result_count == 0]
    failed = [case.repo_id for case in cases if not case.ok]
    return {
        "total_repos": len(cases),
        "non_empty_repos": len(non_empty),
        "empty_repos": len(empty),
        "failed_repos": len(failed),
        "avg_ms": statistics.fmean(elapsed) if elapsed else 0.0,
        "median_ms": statistics.median(elapsed) if elapsed else 0.0,
        "p95_ms": p95_ms(elapsed),
        "min_ms": min(elapsed) if elapsed else 0.0,
        "max_ms": max(elapsed) if elapsed else 0.0,
        "non_empty_repo_ids": non_empty,
        "empty_repo_ids": empty,
        "failed_repo_ids": failed,
    }


def run_all_repo_sweep(
    *,
    base_url: str,
    endpoint: str,
    repo_ids: list[str],
    shared_query: str | None,
    limit: int,
    timeout_s: float,
    package: str | None,
    module: str | None,
) -> list[RepoSweepCase]:
    if endpoint == "index-status":
        raise ValueError("--all-repos is not meaningful with index-status")
    cases: list[RepoSweepCase] = []
    for repo_id in repo_ids:
        query = shared_query
        if endpoint in {"symbol-search", "module-search", "example-search"} and not query:
            query = default_query_for_repo(repo_id)
        url = build_request_url(
            base_url=base_url,
            endpoint=endpoint,
            repo=repo_id,
            query=query,
            limit=limit,
            package=package,
            module=module,
        )
        result = run_once(url, endpoint, timeout_s)
        cases.append(
            RepoSweepCase(
                repo_id=repo_id,
                query=query,
                elapsed_ms=result.elapsed_ms,
                ok=result.ok,
                status=result.status,
                result_count=result.result_count,
                error=result.error,
            )
        )
    return cases


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)

    try:
        snapshot, capabilities_payload = fetch_gateway_snapshot(
            args.base_url,
            float(args.timeout_s),
        )
        repo, query = resolve_effective_request(
            snapshot=snapshot,
            endpoint=args.endpoint,
            repo=args.repo,
            query=args.query,
            package=args.package,
            module=args.module,
        )
    except Exception as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 2

    if args.all_repos:
        repo_ids = live_repo_ids(capabilities_payload)
        if args.repo_limit > 0:
            repo_ids = repo_ids[: max(1, int(args.repo_limit))]
        try:
            cases = run_all_repo_sweep(
                base_url=args.base_url,
                endpoint=args.endpoint,
                repo_ids=repo_ids,
                shared_query=args.query,
                limit=args.limit,
                timeout_s=float(args.timeout_s),
                package=args.package,
                module=args.module,
            )
        except Exception as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 2
        summary = summarize_repo_sweep(cases)
        report = {
            "gateway": asdict(snapshot),
            "request": {
                "base_url": normalize_base_url(args.base_url),
                "endpoint": args.endpoint,
                "shared_query": args.query,
                "package": args.package,
                "module": args.module,
                "limit": max(1, int(args.limit)),
                "all_repos": True,
                "repo_count": len(repo_ids),
            },
            "sweep": summary,
            "cases": [asdict(case) for case in cases],
        }
        if args.json:
            print(json.dumps(report, ensure_ascii=False, indent=2))
        else:
            print(
                "gateway "
                f"repoProjects={snapshot.repo_project_count} "
                f"ready={snapshot.ready_repos}/{snapshot.total_repos} "
                f"unsupported={snapshot.unsupported_repos} "
                f"failed={snapshot.failed_repos} "
                f"bootstrap={snapshot.bootstrap_mode}"
            )
            print(
                "sweep "
                f"endpoint={args.endpoint} "
                f"repos={summary['total_repos']} "
                f"nonEmpty={summary['non_empty_repos']} "
                f"empty={summary['empty_repos']} "
                f"failed={summary['failed_repos']} "
                f"avgMs={summary['avg_ms']:.3f} "
                f"p95Ms={summary['p95_ms']:.3f}"
            )
            if summary["empty_repo_ids"]:
                print(f"emptyRepoIds={summary['empty_repo_ids']}")
            if summary["failed_repo_ids"]:
                print(f"failedRepoIds={summary['failed_repo_ids']}")
        return 0

    url = build_request_url(
        base_url=args.base_url,
        endpoint=args.endpoint,
        repo=repo,
        query=query,
        limit=args.limit,
        package=args.package,
        module=args.module,
    )

    cold_run = run_once(url, args.endpoint, float(args.timeout_s))
    for _ in range(max(0, int(args.warm_runs))):
        run_once(url, args.endpoint, float(args.timeout_s))

    measured = [
        run_once(url, args.endpoint, float(args.timeout_s)) for _ in range(max(1, int(args.runs)))
    ]

    elapsed_values = [run.elapsed_ms for run in measured]
    ok_runs = [run for run in measured if run.ok]
    failures = [run.error for run in measured if run.error]
    avg_ms = statistics.fmean(elapsed_values) if elapsed_values else 0.0
    median_ms = statistics.median(elapsed_values) if elapsed_values else 0.0
    p95 = p95_ms(elapsed_values)
    min_ms = min(elapsed_values) if elapsed_values else 0.0
    max_ms = max(elapsed_values) if elapsed_values else 0.0
    avg_result_count = (
        statistics.fmean([float(run.result_count) for run in ok_runs]) if ok_runs else 0.0
    )

    report = {
        "gateway": asdict(snapshot),
        "request": {
            "base_url": normalize_base_url(args.base_url),
            "endpoint": args.endpoint,
            "repo": repo,
            "query": query,
            "package": args.package,
            "module": args.module,
            "limit": max(1, int(args.limit)),
            "url": url,
        },
        "cold_run": asdict(cold_run),
        "measured": {
            "runs": len(measured),
            "ok_runs": len(ok_runs),
            "failed_runs": len(measured) - len(ok_runs),
            "avg_ms": avg_ms,
            "median_ms": median_ms,
            "p95_ms": p95,
            "min_ms": min_ms,
            "max_ms": max_ms,
            "avg_result_count": avg_result_count,
            "failures": failures,
        },
    }

    exit_code = 0
    if args.max_p95_ms > 0 and p95 > args.max_p95_ms:
        exit_code = 1
    if args.max_avg_ms > 0 and avg_ms > args.max_avg_ms:
        exit_code = 1

    if args.json:
        print(json.dumps(report, ensure_ascii=False, indent=2))
    else:
        gateway = report["gateway"]
        measured_summary = report["measured"]
        print(
            "gateway "
            f"repoProjects={gateway['repo_project_count']} "
            f"ready={gateway['ready_repos']}/{gateway['total_repos']} "
            f"unsupported={gateway['unsupported_repos']} "
            f"failed={gateway['failed_repos']} "
            f"bootstrap={gateway['bootstrap_mode']} "
            f"deferredActivationObserved={gateway['deferred_activation_observed']}"
        )
        print(f"request endpoint={args.endpoint} repo={repo} query={query!r} url={url}")
        print(
            "cold "
            f"elapsedMs={cold_run.elapsed_ms:.3f} "
            f"status={cold_run.status} "
            f"ok={cold_run.ok} "
            f"results={cold_run.result_count}"
        )
        print(
            "warm "
            f"avgMs={measured_summary['avg_ms']:.3f} "
            f"medianMs={measured_summary['median_ms']:.3f} "
            f"p95Ms={measured_summary['p95_ms']:.3f} "
            f"minMs={measured_summary['min_ms']:.3f} "
            f"maxMs={measured_summary['max_ms']:.3f} "
            f"avgResults={measured_summary['avg_result_count']:.2f} "
            f"okRuns={measured_summary['ok_runs']}/{measured_summary['runs']}"
        )
        if failures:
            print(f"failures {failures}")

    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
