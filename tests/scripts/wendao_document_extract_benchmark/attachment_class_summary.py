"""Attachment class summary assembly."""

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
from .attachment_class_image import (
    aggregate_image_acceleration_candidates,
    aggregate_image_audit_strings,
    image_attachment_audit_count,
    image_known_dimension_count,
    max_image_dimension,
    max_image_pixel_count,
)
from .attachment_class_stats import (
    aggregate_artifact_counter,
    aggregate_document_timing_phases,
    aggregate_optional_bool,
    slowest_fixture,
)
from .precision_speed import precision_speed_summary

if TYPE_CHECKING:
    from .common import Any


def summarize_attachment_class(
    attachment_class: str,
    results: list[dict[str, Any]],
) -> dict[str, Any]:
    """Return one class-level report summary."""

    total_error_rows = sum(
        result.get("forceErrorRows", 0)
        + result.get("shardCacheReuseErrorRows", 0)
        + result.get("artifactRegistryReuseErrorRows", 0)
        + result.get("cacheErrorRows", 0)
        for result in results
    )
    artifact_error_count = sum(
        result.get("artifactErrorCount", 0) for result in results
    )
    structure_parity_error_count = sum(
        result.get("structureParityErrorCount", 0) for result in results
    )
    structure_sorted = aggregate_optional_bool(
        result.get("structureReadingOrderSorted") for result in results
    )
    structure_order_stable = aggregate_optional_bool(
        result.get("structureOrderStable") for result in results
    )
    structure_order_mismatch_count = sum(
        result.get("structureOrderMismatchCount", 0) for result in results
    )
    structure_parity_passed = aggregate_optional_bool(
        result.get("structureParityPassed") for result in results
    )
    return {
        "attachmentClass": attachment_class,
        "fixtureCount": len(results),
        "fixtures": [
            fixture
            for result in results
            if isinstance((fixture := result.get("fixture")), str)
        ],
        "totalRequests": sum(result.get("requestCount", 0) for result in results),
        "totalRows": sum(result.get("totalRows", 0) for result in results),
        "totalArrowIpcBytes": sum(result.get("arrowIpcBytes", 0) for result in results),
        "totalErrorRows": total_error_rows,
        "resourcesRows": sum(result.get("resourcesRows", 0) for result in results),
        "resourceTypeCounts": aggregate_artifact_counter(
            results,
            "resourceTypeCounts",
        ),
        "resourceStatusCounts": aggregate_artifact_counter(
            results,
            "resourceStatusCounts",
        ),
        "structureRows": sum(result.get("structureRows", 0) for result in results),
        "structureBboxBlocks": sum(
            result.get("structureBboxBlocks", 0) for result in results
        ),
        "structureBlockTypeCounts": aggregate_artifact_counter(
            results,
            "structureBlockTypeCounts",
        ),
        "metricsStatusCounts": aggregate_artifact_counter(
            results,
            "metricsStatusCounts",
        ),
        "documentTimingStatusCounts": aggregate_artifact_counter(
            results,
            "documentTimingStatusCounts",
        ),
        "documentTimingTotalElapsedMs": sum(
            result.get("documentTimingTotalElapsedMs", 0.0) for result in results
        ),
        "documentTimingOverheadMs": sum(
            result.get("documentTimingOverheadMs", 0.0) or 0.0 for result in results
        ),
        "documentTimingPhaseElapsedMs": aggregate_document_timing_phases(results),
        "imageAttachmentAuditCount": image_attachment_audit_count(results),
        "imageKnownDimensionCount": image_known_dimension_count(results),
        "imageFormatCounts": aggregate_image_audit_strings(results, "format"),
        "imageDimensionSourceCounts": aggregate_image_audit_strings(
            results,
            "dimensionSource",
        ),
        "imageAccelerationCandidates": aggregate_image_acceleration_candidates(
            results,
        ),
        "maxImageWidthPx": max_image_dimension(results, "widthPx"),
        "maxImageHeightPx": max_image_dimension(results, "heightPx"),
        "maxImagePixelCount": max_image_pixel_count(results),
        "archiveAttachmentAuditCount": archive_attachment_audit_count(results),
        "archiveMemberCount": sum_archive_audit_int(results, "memberCount"),
        "archiveRegularFileCount": sum_archive_audit_int(results, "regularFileCount"),
        "archiveXmlMemberCount": sum_archive_audit_int(results, "xmlMemberCount"),
        "archiveImageMemberCount": sum_archive_audit_int(results, "imageMemberCount"),
        "archiveTotalMemberSizeBytes": sum_archive_audit_int(
            results,
            "totalMemberSizeBytes",
        ),
        "archiveFormatCounts": aggregate_archive_audit_strings(
            results, "archiveFormat"
        ),
        "archiveAccelerationCandidates": aggregate_archive_acceleration_candidates(
            results,
        ),
        "archiveExtensionCounts": aggregate_archive_extension_counts(results),
        "maxArchiveLargestMemberSizeBytes": max_archive_largest_member_size(results),
        "slowestForceFixture": slowest_fixture(results, "forceRefreshMs"),
        "slowestTimingOverheadFixture": slowest_fixture(
            results,
            "documentTimingOverheadMs",
        ),
        "slowestCacheP95Fixture": slowest_fixture(results, "cacheHitP95Ms"),
        "precisionSpeedSummary": precision_speed_summary(
            results,
            None,
            total_error_rows=total_error_rows,
            artifact_error_count=artifact_error_count,
            structure_parity_error_count=structure_parity_error_count,
            structure_reading_order_sorted=structure_sorted,
            structure_order_stable=structure_order_stable,
            structure_order_mismatch_count=structure_order_mismatch_count,
            structure_parity_passed=structure_parity_passed,
        ),
    }
