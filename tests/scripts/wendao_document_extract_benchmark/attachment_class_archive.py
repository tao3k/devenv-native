"""Archive attachment aggregate helpers."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .common import Any


def archive_attachment_audit_count(results: list[dict[str, Any]]) -> int:
    """Count archive attachment audits across class fixture artifacts."""

    return sum(
        1
        for result in results
        for artifact in result.get("artifactReports", [])
        if isinstance(artifact.get("archiveAttachmentAudit"), dict)
    )


def sum_archive_audit_int(results: list[dict[str, Any]], key: str) -> int:
    """Sum an integer field across archive attachment audit reports."""

    return sum(
        value
        for result in results
        for artifact in result.get("artifactReports", [])
        if isinstance((audit := artifact.get("archiveAttachmentAudit")), dict)
        and isinstance((value := audit.get(key)), int)
    )


def aggregate_archive_audit_strings(
    results: list[dict[str, Any]],
    key: str,
) -> dict[str, int]:
    """Aggregate string counts across archive attachment audits."""

    counts: dict[str, int] = {}
    for result in results:
        for artifact in result.get("artifactReports", []):
            audit = artifact.get("archiveAttachmentAudit")
            if not isinstance(audit, dict):
                continue
            value = audit.get(key)
            if isinstance(value, str):
                counts[value] = counts.get(value, 0) + 1
    return dict(sorted(counts.items()))


def aggregate_archive_acceleration_candidates(
    results: list[dict[str, Any]],
) -> dict[str, int]:
    """Aggregate archive audit routing candidates across class fixtures."""

    counts: dict[str, int] = {}
    for result in results:
        for artifact in result.get("artifactReports", []):
            audit = artifact.get("archiveAttachmentAudit")
            if not isinstance(audit, dict):
                continue
            candidate = audit.get("rustAccelerationCandidate")
            if isinstance(candidate, str):
                counts[candidate] = counts.get(candidate, 0) + 1
    return dict(sorted(counts.items()))


def aggregate_archive_extension_counts(results: list[dict[str, Any]]) -> dict[str, int]:
    """Aggregate archive member suffix counts across class fixtures."""

    counts: dict[str, int] = {}
    for result in results:
        for artifact in result.get("artifactReports", []):
            audit = artifact.get("archiveAttachmentAudit")
            if not isinstance(audit, dict):
                continue
            extension_counts = audit.get("extensionCounts")
            if not isinstance(extension_counts, dict):
                continue
            for suffix, count in extension_counts.items():
                if isinstance(suffix, str) and isinstance(count, int):
                    counts[suffix] = counts.get(suffix, 0) + count
    return dict(sorted(counts.items()))


def max_archive_largest_member_size(results: list[dict[str, Any]]) -> int | None:
    """Return the largest regular archive member size observed."""

    values = []
    for result in results:
        for artifact in result.get("artifactReports", []):
            audit = artifact.get("archiveAttachmentAudit")
            if not isinstance(audit, dict):
                continue
            value = audit.get("largestMemberSizeBytes")
            if isinstance(value, int):
                values.append(value)
    return max(values, default=None)
