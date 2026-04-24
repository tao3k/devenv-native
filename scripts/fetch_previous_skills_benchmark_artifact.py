#!/usr/bin/env python3
"""Fetch previous skills benchmark artifact and materialize CLI summary baseline."""

from __future__ import annotations

import argparse
import io
import json
import os
import urllib.parse
import urllib.request
import zipfile
from pathlib import Path
from typing import Any

GITHUB_API_ROOT = "https://api.github.com"
DEFAULT_PREFERRED_MEMBER = "cli_runner_summary.base.json"
DEFAULT_FALLBACK_MEMBER = "cli_runner_summary.json"


def _as_int(value: Any) -> int | None:
    if isinstance(value, bool):
        return None
    if isinstance(value, int):
        return value
    if isinstance(value, str) and value.strip():
        try:
            return int(value.strip())
        except ValueError:
            return None
    return None


def _api_bytes(*, url: str, token: str, timeout_seconds: float = 30.0) -> bytes:
    request = urllib.request.Request(
        url,
        headers={
            "Accept": "application/vnd.github+json",
            "Authorization": f"Bearer {token}",
            "X-GitHub-Api-Version": "2022-11-28",
            "User-Agent": "xiuxian-skill-runtime-baseline-fetcher",
        },
    )
    with urllib.request.urlopen(request, timeout=timeout_seconds) as response:
        return response.read()


def _api_json(*, url: str, token: str, timeout_seconds: float = 30.0) -> dict[str, Any]:
    payload = _api_bytes(url=url, token=token, timeout_seconds=timeout_seconds)
    parsed = json.loads(payload.decode("utf-8"))
    if isinstance(parsed, dict):
        return parsed
    raise ValueError("GitHub API did not return a JSON object")


def _workflow_runs_url(
    *,
    repo: str,
    workflow_file: str,
    branch: str | None,
    run_status: str,
    per_page: int,
) -> str:
    encoded_workflow = urllib.parse.quote(workflow_file, safe="")
    params: dict[str, str] = {"status": run_status, "per_page": str(max(1, per_page))}
    if branch:
        params["branch"] = branch
    return (
        f"{GITHUB_API_ROOT}/repos/{repo}/actions/workflows/{encoded_workflow}/runs?"
        f"{urllib.parse.urlencode(params)}"
    )


def _list_workflow_runs(
    *,
    repo: str,
    workflow_file: str,
    branch: str | None,
    run_status: str,
    token: str,
    per_page: int = 30,
) -> list[dict[str, Any]]:
    payload = _api_json(
        url=_workflow_runs_url(
            repo=repo,
            workflow_file=workflow_file,
            branch=branch,
            run_status=run_status,
            per_page=per_page,
        ),
        token=token,
    )
    runs = payload.get("workflow_runs")
    if isinstance(runs, list):
        return [run for run in runs if isinstance(run, dict)]
    return []


def _select_candidate_runs(
    runs: list[dict[str, Any]],
    *,
    current_run_id: int | None,
    max_candidates: int,
) -> list[dict[str, Any]]:
    selected: list[dict[str, Any]] = []
    for run in runs:
        run_id = _as_int(run.get("id"))
        if run_id is None:
            continue
        if current_run_id is not None and run_id == current_run_id:
            continue
        selected.append(run)
        if len(selected) >= max(1, max_candidates):
            break
    return selected


def _list_run_artifacts(*, repo: str, run_id: int, token: str) -> list[dict[str, Any]]:
    url = f"{GITHUB_API_ROOT}/repos/{repo}/actions/runs/{run_id}/artifacts?per_page=100"
    payload = _api_json(url=url, token=token)
    artifacts = payload.get("artifacts")
    if isinstance(artifacts, list):
        return [artifact for artifact in artifacts if isinstance(artifact, dict)]
    return []


def _select_artifact_by_name(
    artifacts: list[dict[str, Any]],
    *,
    artifact_name: str,
) -> dict[str, Any] | None:
    for artifact in artifacts:
        if str(artifact.get("name", "")) != artifact_name:
            continue
        if bool(artifact.get("expired", False)):
            continue
        return artifact
    return None


def _normalize_member_name(name: str) -> str:
    return name.replace("\\", "/").strip("/")


def _select_member_name(
    member_names: list[str],
    *,
    preferred_member: str,
    fallback_member: str,
) -> str | None:
    normalized = [_normalize_member_name(name) for name in member_names]
    for candidate in (preferred_member, fallback_member):
        matches = [
            original
            for original, normalized_name in zip(member_names, normalized, strict=False)
            if normalized_name.endswith(candidate)
        ]
        if matches:
            return matches[0]
    return None


def _extract_member_from_zip(
    *,
    archive_bytes: bytes,
    preferred_member: str,
    fallback_member: str,
) -> tuple[str, bytes] | None:
    with zipfile.ZipFile(io.BytesIO(archive_bytes)) as archive:
        member_name = _select_member_name(
            archive.namelist(),
            preferred_member=preferred_member,
            fallback_member=fallback_member,
        )
        if member_name is None:
            return None
        return member_name, archive.read(member_name)


def _emit(status: dict[str, Any]) -> None:
    print(json.dumps(status, ensure_ascii=False, indent=2))


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Fetch previous workflow artifact and extract CLI summary baseline."
    )
    parser.add_argument(
        "--artifact-name", required=True, help="Artifact name to fetch."
    )
    parser.add_argument(
        "--output", required=True, help="Destination baseline JSON path."
    )
    parser.add_argument(
        "--workflow-file",
        default="ci.yaml",
        help="Workflow file name under .github/workflows (default: ci.yaml).",
    )
    parser.add_argument("--repo", default=os.environ.get("GITHUB_REPOSITORY", ""))
    parser.add_argument("--branch", default=os.environ.get("GITHUB_REF_NAME", ""))
    parser.add_argument("--token", default=os.environ.get("GITHUB_TOKEN", ""))
    parser.add_argument("--current-run-id", default=os.environ.get("GITHUB_RUN_ID", ""))
    parser.add_argument(
        "--run-status",
        default="success",
        help="Workflow run status filter (default: success). Use completed for rollout streak tracking.",
    )
    parser.add_argument(
        "--preferred-member",
        default=DEFAULT_PREFERRED_MEMBER,
        help=f"Preferred file in artifact zip (default: {DEFAULT_PREFERRED_MEMBER}).",
    )
    parser.add_argument(
        "--fallback-member",
        default=DEFAULT_FALLBACK_MEMBER,
        help=f"Fallback file in artifact zip (default: {DEFAULT_FALLBACK_MEMBER}).",
    )
    parser.add_argument(
        "--max-candidate-runs",
        type=int,
        default=20,
        help="Maximum previous successful runs to inspect (default: 20).",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Exit non-zero on fetch failure instead of emitting skipped status.",
    )
    args = parser.parse_args()

    repo = str(args.repo).strip()
    token = str(args.token).strip()
    branch = str(args.branch).strip() or None
    run_status = str(args.run_status).strip() or "success"
    artifact_name = str(args.artifact_name).strip()
    workflow_file = str(args.workflow_file).strip()
    current_run_id = _as_int(args.current_run_id)
    output_path = Path(str(args.output)).expanduser().resolve()

    if not repo or not token or not artifact_name or not workflow_file:
        _emit(
            {
                "schema": "omni.skills.cli_runner_summary.fetch.v1",
                "status": "skipped",
                "reason": "missing_required_context",
                "repo": repo,
                "artifact_name": artifact_name,
                "workflow_file": workflow_file,
                "run_status": run_status,
            }
        )
        return 1 if args.strict else 0

    try:
        runs = _list_workflow_runs(
            repo=repo,
            workflow_file=workflow_file,
            branch=branch,
            run_status=run_status,
            token=token,
        )
        if not runs and branch is not None:
            runs = _list_workflow_runs(
                repo=repo,
                workflow_file=workflow_file,
                branch=None,
                run_status=run_status,
                token=token,
            )
        candidates = _select_candidate_runs(
            runs,
            current_run_id=current_run_id,
            max_candidates=max(1, int(args.max_candidate_runs)),
        )

        for run in candidates:
            run_id = _as_int(run.get("id"))
            if run_id is None:
                continue
            artifacts = _list_run_artifacts(repo=repo, run_id=run_id, token=token)
            artifact = _select_artifact_by_name(artifacts, artifact_name=artifact_name)
            if artifact is None:
                continue

            archive_url = str(artifact.get("archive_download_url", "")).strip()
            if not archive_url:
                continue
            archive_bytes = _api_bytes(url=archive_url, token=token)
            extracted = _extract_member_from_zip(
                archive_bytes=archive_bytes,
                preferred_member=str(args.preferred_member),
                fallback_member=str(args.fallback_member),
            )
            if extracted is None:
                continue

            member_name, content = extracted
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_bytes(content)
            _emit(
                {
                    "schema": "omni.skills.cli_runner_summary.fetch.v1",
                    "status": "ok",
                    "repo": repo,
                    "workflow_file": workflow_file,
                    "branch": branch or "",
                    "run_status": run_status,
                    "run_id": run_id,
                    "artifact_id": _as_int(artifact.get("id")),
                    "artifact_name": artifact_name,
                    "member_name": member_name,
                    "output": str(output_path),
                }
            )
            return 0

        _emit(
            {
                "schema": "omni.skills.cli_runner_summary.fetch.v1",
                "status": "skipped",
                "reason": "artifact_or_member_not_found",
                "repo": repo,
                "workflow_file": workflow_file,
                "branch": branch or "",
                "run_status": run_status,
                "artifact_name": artifact_name,
                "output": str(output_path),
                "candidate_run_count": len(candidates),
            }
        )
        return 1 if args.strict else 0
    except Exception as exc:
        _emit(
            {
                "schema": "omni.skills.cli_runner_summary.fetch.v1",
                "status": "error",
                "error": str(exc),
                "repo": repo,
                "workflow_file": workflow_file,
                "branch": branch or "",
                "run_status": run_status,
                "artifact_name": artifact_name,
                "output": str(output_path),
            }
        )
        return 1 if args.strict else 0


if __name__ == "__main__":
    raise SystemExit(main())
