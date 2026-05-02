"""Image attachment aggregate helpers."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .common import Any


def image_attachment_audit_count(results: list[dict[str, Any]]) -> int:
    """Count image attachment audits across class fixture artifacts."""

    return sum(
        1
        for result in results
        for artifact in result.get("artifactReports", [])
        if isinstance(artifact.get("imageAttachmentAudit"), dict)
    )


def image_known_dimension_count(results: list[dict[str, Any]]) -> int:
    """Count image audits with dimensions proven by Rust bounded headers."""

    return sum(
        1
        for result in results
        for artifact in result.get("artifactReports", [])
        if isinstance((audit := artifact.get("imageAttachmentAudit")), dict)
        and isinstance(audit.get("widthPx"), int)
        and isinstance(audit.get("heightPx"), int)
    )


def aggregate_image_audit_strings(
    results: list[dict[str, Any]],
    key: str,
) -> dict[str, int]:
    """Aggregate string-valued image audit fields."""

    counts: dict[str, int] = {}
    for result in results:
        for artifact in result.get("artifactReports", []):
            audit = artifact.get("imageAttachmentAudit")
            if not isinstance(audit, dict):
                continue
            value = audit.get(key)
            if isinstance(value, str):
                counts[value] = counts.get(value, 0) + 1
    return dict(sorted(counts.items()))


def aggregate_image_acceleration_candidates(
    results: list[dict[str, Any]],
) -> dict[str, int]:
    """Aggregate Rust image acceleration candidate counts."""

    counts: dict[str, int] = {}
    for result in results:
        for artifact in result.get("artifactReports", []):
            audit = artifact.get("imageAttachmentAudit")
            if not isinstance(audit, dict):
                continue
            candidate = audit.get("rustAccelerationCandidate")
            if isinstance(candidate, str):
                counts[candidate] = counts.get(candidate, 0) + 1
    return dict(sorted(counts.items()))


def max_image_dimension(results: list[dict[str, Any]], key: str) -> int | None:
    """Return the largest image dimension value across class fixture artifacts."""

    values = []
    for result in results:
        for artifact in result.get("artifactReports", []):
            audit = artifact.get("imageAttachmentAudit")
            if not isinstance(audit, dict):
                continue
            value = audit.get(key)
            if isinstance(value, int):
                values.append(value)
    return max(values, default=None)


def max_image_pixel_count(results: list[dict[str, Any]]) -> int | None:
    """Return the largest image pixel count across class fixture artifacts."""

    values = []
    for result in results:
        for artifact in result.get("artifactReports", []):
            audit = artifact.get("imageAttachmentAudit")
            if not isinstance(audit, dict):
                continue
            pixel_count = audit.get("pixelCount")
            if isinstance(pixel_count, int):
                values.append(pixel_count)
    return max(values, default=None)
