"""Run-to-run structure order consistency helpers."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .common import Any


def fixture_structure_order_consistency(
    force_report: dict[str, Any],
    cached_report: dict[str, Any],
    shard_cache_reuse_report: dict[str, Any] | None = None,
    artifact_registry_reuse_report: dict[str, Any] | None = None,
) -> dict[str, Any]:
    runs = [
        structure_order_run("force", force_report),
        structure_order_run("cache", cached_report),
    ]
    if shard_cache_reuse_report is not None:
        runs.append(structure_order_run("shard_cache_reuse", shard_cache_reuse_report))
    if artifact_registry_reuse_report is not None:
        runs.append(
            structure_order_run(
                "artifact_registry_reuse",
                artifact_registry_reuse_report,
            )
        )
    comparable_runs = [run for run in runs if run["signature"]]
    distinct_signatures = {
        run["signature"] for run in comparable_runs if isinstance(run["signature"], str)
    }
    stable = len(distinct_signatures) == 1 if len(comparable_runs) >= 2 else None
    representative = comparable_runs[-1] if comparable_runs else None
    return {
        "structureOrderStable": stable,
        "structureOrderComparedRuns": len(comparable_runs),
        "structureOrderMismatchCount": max(len(distinct_signatures) - 1, 0),
        "structureOrderSignature": (
            representative["signature"] if representative is not None else None
        ),
        "structureOrderFirstKey": (
            representative["firstKey"] if representative is not None else None
        ),
        "structureOrderLastKey": (
            representative["lastKey"] if representative is not None else None
        ),
        "structureOrderRuns": comparable_runs,
    }


def structure_order_run(label: str, report: dict[str, Any]) -> dict[str, Any]:
    artifact = first_structure_artifact(report)
    return {
        "run": label,
        "signature": artifact.get("structureOrderSignature") if artifact else None,
        "firstKey": artifact.get("structureOrderFirstKey") if artifact else None,
        "lastKey": artifact.get("structureOrderLastKey") if artifact else None,
        "rowCount": artifact.get("structureRowCount") if artifact else None,
    }


def first_structure_artifact(report: dict[str, Any]) -> dict[str, Any] | None:
    for artifact in report.get("artifactReports", []):
        if artifact.get("structureArrowExists"):
            return artifact
    return None
