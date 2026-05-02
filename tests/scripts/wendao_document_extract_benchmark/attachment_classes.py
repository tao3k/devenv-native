"""Attachment class helpers for mixed-format benchmark reports."""

from __future__ import annotations

from typing import TYPE_CHECKING

from .attachment_class_archive import (
    aggregate_archive_acceleration_candidates,
    aggregate_archive_audit_strings,
    aggregate_archive_extension_counts,
    archive_attachment_audit_count,
    max_archive_largest_member_size,
    sum_archive_audit_int,
)
from .attachment_class_constants import (
    COMPOUND_SUFFIX_CLASS_OVERRIDES,
    FIXTURE_CLASS_OVERRIDES,
    PDF_CLASS,
    SUFFIX_CLASS_OVERRIDES,
    UNKNOWN_CLASS,
)
from .attachment_class_image import (
    aggregate_image_acceleration_candidates,
    aggregate_image_audit_strings,
    image_attachment_audit_count,
    image_known_dimension_count,
    max_image_dimension,
    max_image_pixel_count,
)
from .attachment_class_summary import summarize_attachment_class

if TYPE_CHECKING:
    from .common import Any, Path


def classify_attachment(fixture_name: str, source: Path) -> str:
    """Return the benchmark attachment class for one fixture."""

    if fixture_name.startswith("pdf-"):
        return PDF_CLASS
    if fixture_name in FIXTURE_CLASS_OVERRIDES:
        return FIXTURE_CLASS_OVERRIDES[fixture_name]

    source_name = source.name.lower()
    for suffix, attachment_class in COMPOUND_SUFFIX_CLASS_OVERRIDES.items():
        if source_name.endswith(suffix):
            return attachment_class

    return SUFFIX_CLASS_OVERRIDES.get(source.suffix.lower(), UNKNOWN_CLASS)


def attachment_class_summaries(results: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Summarize precision and speed by attachment class."""

    grouped: dict[str, list[dict[str, Any]]] = {}
    for result in results:
        grouped.setdefault(result.get("attachmentClass") or UNKNOWN_CLASS, []).append(
            result,
        )
    return [
        summarize_attachment_class(attachment_class, class_results)
        for attachment_class, class_results in sorted(grouped.items())
    ]


__all__ = [
    "aggregate_archive_acceleration_candidates",
    "aggregate_archive_audit_strings",
    "aggregate_archive_extension_counts",
    "aggregate_image_acceleration_candidates",
    "aggregate_image_audit_strings",
    "archive_attachment_audit_count",
    "attachment_class_summaries",
    "classify_attachment",
    "image_attachment_audit_count",
    "image_known_dimension_count",
    "max_archive_largest_member_size",
    "max_image_dimension",
    "max_image_pixel_count",
    "sum_archive_audit_int",
]
