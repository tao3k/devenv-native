"""Artifact summary helpers for benchmark JSON reports."""

from __future__ import annotations

from .common import (
    Any,
    resource,
    sys,
)


def summarize_artifact_reports(reports: list[dict[str, Any]]) -> dict[str, Any]:
    structure_sorted_values = [
        report.get("structureReadingOrderSorted")
        for report in reports
        if report.get("structureReadingOrderSorted") is not None
    ]
    structure_parity_checked = any(
        report.get("structureParity") is not None or report.get("structureParityError")
        for report in reports
    )
    return {
        "resourcesArrowExists": any(
            bool(report.get("resourcesArrowExists")) for report in reports
        ),
        "resourcesRows": _sum_int_report_values(reports, "resourcesRowCount"),
        "structureArrowExists": any(
            bool(report.get("structureArrowExists")) for report in reports
        ),
        "structureRows": _sum_int_report_values(reports, "structureRowCount"),
        "structureOcrPageBlocks": _sum_int_report_values(
            reports,
            "structureOcrPageBlocks",
        ),
        "structureOcrRegionBlocks": _sum_int_report_values(
            reports,
            "structureOcrRegionBlocks",
        ),
        "structureBboxBlocks": _sum_int_report_values(
            reports,
            "structureBboxBlocks",
        ),
        "structureReadingOrderSorted": (
            all(bool(value) for value in structure_sorted_values)
            if structure_sorted_values
            else None
        ),
        "structureParityChecked": structure_parity_checked,
        "structureParityPassed": _structure_parity_passed(reports),
        "structureParityErrorCount": sum(
            1 for report in reports if report.get("structureParityError")
        ),
        "metricsArrowExists": any(
            bool(report.get("metricsArrowExists")) for report in reports
        ),
        "metricsRows": _sum_int_report_values(reports, "metricsRowCount"),
        "metricsResultChars": _sum_int_report_values(reports, "metricsResultChars"),
        "metricsBboxCount": _sum_int_report_values(reports, "metricsBboxCount"),
        "metricsRustSchedulerElapsedMs": _sum_float_report_values(
            reports,
            "metricsRustSchedulerElapsedMs",
        ),
        "documentTimingArrowExists": any(
            _document_timing_arrow_exists(report) for report in reports
        ),
        "documentTimingRows": _sum_int_report_values(
            reports,
            "documentTimingRowCount",
        ),
        "documentTimingTotalElapsedMs": _sum_float_report_values(
            reports,
            "documentTimingTotalElapsedMs",
        ),
        "documentTimingPhaseElapsedMs": _aggregate_float_report_maps(
            reports,
            "documentTimingPhaseElapsedMs",
        ),
        "imageAttachmentAuditCount": _image_attachment_audit_count(reports),
        "imageKnownDimensionCount": _image_known_dimension_count(reports),
        "imageFormatCounts": _image_format_counts(reports),
        "imageDimensionSourceCounts": _image_dimension_source_counts(reports),
        "imageAccelerationCandidates": _image_acceleration_candidates(reports),
        "maxImageWidthPx": _max_image_dimension(reports, "widthPx"),
        "maxImageHeightPx": _max_image_dimension(reports, "heightPx"),
        "maxImagePixelCount": _max_image_pixel_count(reports),
        "archiveAttachmentAuditCount": _archive_attachment_audit_count(reports),
        "archiveMemberCount": _sum_archive_audit_int(reports, "memberCount"),
        "archiveRegularFileCount": _sum_archive_audit_int(reports, "regularFileCount"),
        "archiveXmlMemberCount": _sum_archive_audit_int(reports, "xmlMemberCount"),
        "archiveImageMemberCount": _sum_archive_audit_int(reports, "imageMemberCount"),
        "archiveTotalMemberSizeBytes": _sum_archive_audit_int(
            reports,
            "totalMemberSizeBytes",
        ),
        "archiveFormatCounts": _archive_audit_string_counts(reports, "archiveFormat"),
        "archiveAccelerationCandidates": _archive_acceleration_candidates(reports),
        "archiveExtensionCounts": _archive_extension_counts(reports),
        "maxArchiveLargestMemberSizeBytes": _max_archive_largest_member_size(reports),
        "artifactErrorCount": sum(
            1 for report in reports if report.get("artifactError")
        ),
    }


def _structure_parity_passed(reports: list[dict[str, Any]]) -> bool | None:
    checked_reports = [
        report
        for report in reports
        if report.get("structureParity") is not None
        or report.get("structureParityError")
    ]
    if not checked_reports:
        return None
    return all(
        report.get("structureParity") is not None
        and not report.get("structureParityError")
        for report in checked_reports
    )


def _sum_int_report_values(reports: list[dict[str, Any]], key: str) -> int:
    return sum(
        value for report in reports if isinstance((value := report.get(key)), int)
    )


def _sum_float_report_values(reports: list[dict[str, Any]], key: str) -> float:
    return sum(
        float(value)
        for report in reports
        if isinstance((value := report.get(key)), int | float)
    )


def _aggregate_float_report_maps(
    reports: list[dict[str, Any]],
    key: str,
) -> dict[str, float]:
    totals: dict[str, float] = {}
    for report in reports:
        values = report.get(key)
        if not isinstance(values, dict):
            continue
        for item_key, item_value in values.items():
            if isinstance(item_key, str) and isinstance(item_value, int | float):
                totals[item_key] = totals.get(item_key, 0.0) + float(item_value)
    return dict(sorted(totals.items()))


def _document_timing_arrow_exists(report: dict[str, Any]) -> bool:
    if bool(report.get("documentTimingArrowExists")):
        return True
    arrow_bytes = report.get("documentTimingArrowBytes")
    row_count = report.get("documentTimingRowCount")
    return (isinstance(arrow_bytes, int) and arrow_bytes > 0) or (
        isinstance(row_count, int) and row_count > 0
    )


def _image_attachment_audit_count(reports: list[dict[str, Any]]) -> int:
    return sum(
        1 for report in reports if isinstance(report.get("imageAttachmentAudit"), dict)
    )


def _image_known_dimension_count(reports: list[dict[str, Any]]) -> int:
    return sum(
        1
        for report in reports
        if isinstance((audit := report.get("imageAttachmentAudit")), dict)
        and isinstance(audit.get("widthPx"), int)
        and isinstance(audit.get("heightPx"), int)
    )


def _image_format_counts(reports: list[dict[str, Any]]) -> dict[str, int]:
    return _image_audit_string_counts(reports, "format")


def _image_dimension_source_counts(reports: list[dict[str, Any]]) -> dict[str, int]:
    return _image_audit_string_counts(reports, "dimensionSource")


def _image_audit_string_counts(
    reports: list[dict[str, Any]],
    key: str,
) -> dict[str, int]:
    counts: dict[str, int] = {}
    for report in reports:
        audit = report.get("imageAttachmentAudit")
        if not isinstance(audit, dict):
            continue
        value = audit.get(key)
        if isinstance(value, str):
            counts[value] = counts.get(value, 0) + 1
    return dict(sorted(counts.items()))


def _image_acceleration_candidates(reports: list[dict[str, Any]]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for report in reports:
        audit = report.get("imageAttachmentAudit")
        if not isinstance(audit, dict):
            continue
        candidate = audit.get("rustAccelerationCandidate")
        if isinstance(candidate, str):
            counts[candidate] = counts.get(candidate, 0) + 1
    return dict(sorted(counts.items()))


def _max_image_dimension(reports: list[dict[str, Any]], key: str) -> int | None:
    values = []
    for report in reports:
        audit = report.get("imageAttachmentAudit")
        if not isinstance(audit, dict):
            continue
        value = audit.get(key)
        if isinstance(value, int):
            values.append(value)
    return max(values, default=None)


def _max_image_pixel_count(reports: list[dict[str, Any]]) -> int | None:
    values = []
    for report in reports:
        audit = report.get("imageAttachmentAudit")
        if not isinstance(audit, dict):
            continue
        pixel_count = audit.get("pixelCount")
        if isinstance(pixel_count, int):
            values.append(pixel_count)
    return max(values, default=None)


def _archive_attachment_audit_count(reports: list[dict[str, Any]]) -> int:
    return sum(
        1
        for report in reports
        if isinstance(report.get("archiveAttachmentAudit"), dict)
    )


def _sum_archive_audit_int(reports: list[dict[str, Any]], key: str) -> int:
    return sum(
        value
        for report in reports
        if isinstance((audit := report.get("archiveAttachmentAudit")), dict)
        and isinstance((value := audit.get(key)), int)
    )


def _archive_audit_string_counts(
    reports: list[dict[str, Any]],
    key: str,
) -> dict[str, int]:
    counts: dict[str, int] = {}
    for report in reports:
        audit = report.get("archiveAttachmentAudit")
        if not isinstance(audit, dict):
            continue
        value = audit.get(key)
        if isinstance(value, str):
            counts[value] = counts.get(value, 0) + 1
    return dict(sorted(counts.items()))


def _archive_acceleration_candidates(reports: list[dict[str, Any]]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for report in reports:
        audit = report.get("archiveAttachmentAudit")
        if not isinstance(audit, dict):
            continue
        candidate = audit.get("rustAccelerationCandidate")
        if isinstance(candidate, str):
            counts[candidate] = counts.get(candidate, 0) + 1
    return dict(sorted(counts.items()))


def _archive_extension_counts(reports: list[dict[str, Any]]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for report in reports:
        audit = report.get("archiveAttachmentAudit")
        if not isinstance(audit, dict):
            continue
        extension_counts = audit.get("extensionCounts")
        if not isinstance(extension_counts, dict):
            continue
        for suffix, count in extension_counts.items():
            if isinstance(suffix, str) and isinstance(count, int):
                counts[suffix] = counts.get(suffix, 0) + count
    return dict(sorted(counts.items()))


def _max_archive_largest_member_size(reports: list[dict[str, Any]]) -> int | None:
    values = []
    for report in reports:
        audit = report.get("archiveAttachmentAudit")
        if not isinstance(audit, dict):
            continue
        value = audit.get("largestMemberSizeBytes")
        if isinstance(value, int):
            values.append(value)
    return max(values, default=None)


def max_rss_kb() -> int | None:
    try:
        max_rss = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    except (AttributeError, OSError):
        return None
    if sys.platform == "darwin":
        return max_rss // 1024
    return max_rss


def percentile(values: list[float], percentile_value: int) -> float:
    if not values:
        return 0.0
    if len(values) == 1:
        return values[0]
    sorted_values = sorted(values)
    index = (len(sorted_values) - 1) * (percentile_value / 100)
    lower = int(index)
    upper = min(lower + 1, len(sorted_values) - 1)
    weight = index - lower
    return sorted_values[lower] * (1 - weight) + sorted_values[upper] * weight


def rows_per_second(row_count: int, wall_time_ms: float) -> float:
    return 0.0 if wall_time_ms <= 0 else row_count / (wall_time_ms / 1000)
