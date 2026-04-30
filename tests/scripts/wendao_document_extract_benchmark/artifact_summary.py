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
        "resourcesRows": sum_int_report_values(reports, "resourcesRowCount"),
        "structureArrowExists": any(
            bool(report.get("structureArrowExists")) for report in reports
        ),
        "structureRows": sum_int_report_values(reports, "structureRowCount"),
        "structureOcrPageBlocks": sum_int_report_values(
            reports,
            "structureOcrPageBlocks",
        ),
        "structureOcrRegionBlocks": sum_int_report_values(
            reports,
            "structureOcrRegionBlocks",
        ),
        "structureBboxBlocks": sum_int_report_values(
            reports,
            "structureBboxBlocks",
        ),
        "structureReadingOrderSorted": (
            all(bool(value) for value in structure_sorted_values)
            if structure_sorted_values
            else None
        ),
        "structureParityChecked": structure_parity_checked,
        "structureParityPassed": structure_parity_passed(reports),
        "structureParityErrorCount": sum(
            1 for report in reports if report.get("structureParityError")
        ),
        "metricsArrowExists": any(
            bool(report.get("metricsArrowExists")) for report in reports
        ),
        "metricsRows": sum_int_report_values(reports, "metricsRowCount"),
        "metricsResultChars": sum_int_report_values(reports, "metricsResultChars"),
        "metricsBboxCount": sum_int_report_values(reports, "metricsBboxCount"),
        "metricsRustSchedulerElapsedMs": sum_float_report_values(
            reports,
            "metricsRustSchedulerElapsedMs",
        ),
        "imageAttachmentAuditCount": image_attachment_audit_count(reports),
        "imageAccelerationCandidates": image_acceleration_candidates(reports),
        "maxImagePixelCount": max_image_pixel_count(reports),
        "artifactErrorCount": sum(
            1 for report in reports if report.get("artifactError")
        ),
    }


def structure_parity_passed(reports: list[dict[str, Any]]) -> bool | None:
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


def sum_int_report_values(reports: list[dict[str, Any]], key: str) -> int:
    return sum(
        value for report in reports if isinstance((value := report.get(key)), int)
    )


def sum_float_report_values(reports: list[dict[str, Any]], key: str) -> float:
    return sum(
        float(value)
        for report in reports
        if isinstance((value := report.get(key)), int | float)
    )


def image_attachment_audit_count(reports: list[dict[str, Any]]) -> int:
    return sum(
        1 for report in reports if isinstance(report.get("imageAttachmentAudit"), dict)
    )


def image_acceleration_candidates(reports: list[dict[str, Any]]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for report in reports:
        audit = report.get("imageAttachmentAudit")
        if not isinstance(audit, dict):
            continue
        candidate = audit.get("rustAccelerationCandidate")
        if isinstance(candidate, str):
            counts[candidate] = counts.get(candidate, 0) + 1
    return dict(sorted(counts.items()))


def max_image_pixel_count(reports: list[dict[str, Any]]) -> int | None:
    values = []
    for report in reports:
        audit = report.get("imageAttachmentAudit")
        if not isinstance(audit, dict):
            continue
        pixel_count = audit.get("pixelCount")
        if isinstance(pixel_count, int):
            values.append(pixel_count)
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
