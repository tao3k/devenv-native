"""Attachment class helpers for mixed-format benchmark reports."""

from __future__ import annotations

from typing import TYPE_CHECKING

from .precision_speed import precision_speed_summary

if TYPE_CHECKING:
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
        "structureRows": sum(result.get("structureRows", 0) for result in results),
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


def aggregate_optional_bool(values: object) -> bool | None:
    """Return all-truthy bool for present values, or None when no value exists."""

    present_values = [value for value in values if value is not None]
    return all(bool(value) for value in present_values) if present_values else None
