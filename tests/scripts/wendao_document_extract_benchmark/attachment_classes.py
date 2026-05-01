"""Attachment class helpers for mixed-format benchmark reports."""

from __future__ import annotations

from typing import TYPE_CHECKING

from .precision_speed import precision_speed_summary

if TYPE_CHECKING:
    from collections.abc import Iterable

    from .common import Any, Path

PDF_CLASS = "pdf"
OFFICE_CLASS = "office"
IMAGE_CLASS = "image"
STRUCTURED_TEXT_CLASS = "structured_text"
WEB_CLASS = "web"
TABLE_DATA_CLASS = "table_data"
XML_CLASS = "xml"
SUBTITLE_CLASS = "subtitle"
AUDIO_CLASS = "audio"
DOCLING_JSON_CLASS = "docling_json"
ARCHIVE_DOCUMENT_CLASS = "archive_document"
UNKNOWN_CLASS = "unknown"

FIXTURE_CLASS_OVERRIDES = {
    "pdf": PDF_CLASS,
    "docx": OFFICE_CLASS,
    "xlsx": OFFICE_CLASS,
    "pptx": OFFICE_CLASS,
    "markdown": STRUCTURED_TEXT_CLASS,
    "asciidoc": STRUCTURED_TEXT_CLASS,
    "latex": STRUCTURED_TEXT_CLASS,
    "html": WEB_CLASS,
    "csv": TABLE_DATA_CLASS,
    "image-png": IMAGE_CLASS,
    "image-tiff": IMAGE_CLASS,
    "image-webp": IMAGE_CLASS,
    "uspto-xml": XML_CLASS,
    "jats-xml": XML_CLASS,
    "xbrl-xml": XML_CLASS,
    "mets-gbs": ARCHIVE_DOCUMENT_CLASS,
    "docling-json": DOCLING_JSON_CLASS,
    "webvtt": SUBTITLE_CLASS,
    "audio": AUDIO_CLASS,
}

SUFFIX_CLASS_OVERRIDES = {
    ".pdf": PDF_CLASS,
    ".docx": OFFICE_CLASS,
    ".xlsx": OFFICE_CLASS,
    ".pptx": OFFICE_CLASS,
    ".md": STRUCTURED_TEXT_CLASS,
    ".markdown": STRUCTURED_TEXT_CLASS,
    ".adoc": STRUCTURED_TEXT_CLASS,
    ".asciidoc": STRUCTURED_TEXT_CLASS,
    ".tex": STRUCTURED_TEXT_CLASS,
    ".latex": STRUCTURED_TEXT_CLASS,
    ".txt": STRUCTURED_TEXT_CLASS,
    ".text": STRUCTURED_TEXT_CLASS,
    ".qmd": STRUCTURED_TEXT_CLASS,
    ".rmd": STRUCTURED_TEXT_CLASS,
    ".html": WEB_CLASS,
    ".htm": WEB_CLASS,
    ".xhtml": WEB_CLASS,
    ".csv": TABLE_DATA_CLASS,
    ".tsv": TABLE_DATA_CLASS,
    ".png": IMAGE_CLASS,
    ".jpg": IMAGE_CLASS,
    ".jpeg": IMAGE_CLASS,
    ".tif": IMAGE_CLASS,
    ".tiff": IMAGE_CLASS,
    ".bmp": IMAGE_CLASS,
    ".webp": IMAGE_CLASS,
    ".xml": XML_CLASS,
    ".xbrl": XML_CLASS,
    ".vtt": SUBTITLE_CLASS,
    ".webvtt": SUBTITLE_CLASS,
    ".mp3": AUDIO_CLASS,
    ".wav": AUDIO_CLASS,
    ".m4a": AUDIO_CLASS,
    ".json": DOCLING_JSON_CLASS,
}

COMPOUND_SUFFIX_CLASS_OVERRIDES = {
    ".tar.gz": ARCHIVE_DOCUMENT_CLASS,
    ".docling.json": DOCLING_JSON_CLASS,
}


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
    artifact_error_count = sum(result.get("artifactErrorCount", 0) for result in results)
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
            fixture for result in results if isinstance((fixture := result.get("fixture")), str)
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
        "structureBboxBlocks": sum(result.get("structureBboxBlocks", 0) for result in results),
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
