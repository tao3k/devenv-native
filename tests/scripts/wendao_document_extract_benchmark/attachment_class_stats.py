"""Attachment class aggregate statistics."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Iterable

    from .common import Any


def aggregate_artifact_counter(
    results: list[dict[str, Any]],
    counter_key: str,
) -> dict[str, int]:
    """Aggregate one artifact report counter across class fixtures."""

    counts: dict[str, int] = {}
    for result in results:
        for artifact in result.get("artifactReports", []):
            counter = artifact.get(counter_key, {})
            if not isinstance(counter, dict):
                continue
            for key, value in counter.items():
                if isinstance(key, str) and isinstance(value, int):
                    counts[key] = counts.get(key, 0) + value
    return dict(sorted(counts.items()))


def aggregate_document_timing_phases(
    results: list[dict[str, Any]],
) -> dict[str, float]:
    """Aggregate document timing phase elapsed milliseconds."""

    counts: dict[str, float] = {}
    for result in results:
        for artifact in result.get("artifactReports", []):
            counter = artifact.get("documentTimingPhaseElapsedMs", {})
            if not isinstance(counter, dict):
                continue
            for key, value in counter.items():
                if isinstance(key, str) and isinstance(value, int | float):
                    counts[key] = counts.get(key, 0.0) + float(value)
    return dict(sorted(counts.items()))


def slowest_fixture(
    results: list[dict[str, Any]],
    latency_key: str,
) -> dict[str, Any] | None:
    """Return the fixture with the highest numeric latency for a class."""

    candidates = [
        {
            "fixture": result.get("fixture"),
            "latencyMs": float(latency),
        }
        for result in results
        if isinstance(result.get("fixture"), str)
        and isinstance((latency := result.get(latency_key)), int | float)
    ]
    return max(candidates, key=lambda item: item["latencyMs"], default=None)


def aggregate_optional_bool(values: Iterable[Any]) -> bool | None:
    """Return all-truthy bool for present values, or None when no value exists."""

    present_values = [value for value in values if value is not None]
    return all(bool(value) for value in present_values) if present_values else None
