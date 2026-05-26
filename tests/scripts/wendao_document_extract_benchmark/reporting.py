"""Benchmark summary and Markdown rendering helpers."""

from __future__ import annotations

import json
from typing import TYPE_CHECKING

from .attachment_classes import (
    aggregate_archive_acceleration_candidates,
    aggregate_archive_audit_strings,
    aggregate_archive_extension_counts,
    aggregate_image_acceleration_candidates,
    aggregate_image_audit_strings,
    archive_attachment_audit_count,
    attachment_class_summaries,
    image_attachment_audit_count,
    image_known_dimension_count,
    max_archive_largest_member_size,
    max_image_dimension,
    max_image_pixel_count,
    sum_archive_audit_int,
)
from .precision_speed import (
    all_structure_order_stable,
    all_structure_parity_passed,
    all_structure_reading_order_sorted,
    precision_speed_summary,
    structure_order_mismatch_count,
)
from .rust_status import combine_rust_jobs_status_summaries

if TYPE_CHECKING:
    from .common import Any, argparse

OCR2_REGION_RENDER_ARTIFACT_KIND_SUFFIXES = (
    "PageRasterHitCount",
    "PageRasterMissCount",
    "PageRasterThrottledCount",
    "PageRasterByteCount",
    "RegionCropHitCount",
    "RegionCropMissCount",
    "RegionCropThrottledCount",
    "RegionCropByteCount",
    "RegionManifestProjectionHitCount",
    "RegionManifestProjectionMissCount",
    "RegionManifestProjectionThrottledCount",
    "RegionManifestProjectionByteCount",
    "RegionManifestProjectionRowHitCount",
    "RegionManifestProjectionRowMissCount",
    "RegionManifestProjectionRowThrottledCount",
    "RegionManifestProjectionRowByteCount",
)


def _sum_prefixed_ocr2_region_render_artifact_kind_counts(
    results: list[dict[str, Any]],
    result_prefix: str,
) -> dict[str, int]:
    return {
        f"{result_prefix}Ocr2RegionRenderArtifactCache{suffix}": sum(
            _numeric_result_value(
                result,
                f"{result_prefix}Ocr2RegionRenderArtifactCache{suffix}",
            )
            for result in results
        )
        for suffix in OCR2_REGION_RENDER_ARTIFACT_KIND_SUFFIXES
    }


def summarize_results(
    results: list[dict[str, Any]],
    distinct_miss_report: dict[str, Any] | None = None,
) -> dict[str, Any]:
    rust_jobs_status = combine_rust_jobs_status_summaries(
        [result.get("rustJobsStatusSummary", {}) for result in results]
        + [(distinct_miss_report.get("rustJobsStatusSummary", {}) if distinct_miss_report else {})]
    )
    distinct_error_rows = distinct_miss_report.get("errorRows", 0) if distinct_miss_report else 0
    total_error_rows = (
        sum(
            result["forceErrorRows"]
            + result.get("shardCacheReuseErrorRows", 0)
            + result.get("regionProjectionReuseErrorRows", 0)
            + result.get("artifactRegistryReuseErrorRows", 0)
            + result["cacheErrorRows"]
            for result in results
        )
        + distinct_error_rows
    )
    artifact_error_count = sum(result.get("artifactErrorCount", 0) for result in results)
    structure_parity_error_count = sum(
        result.get("structureParityErrorCount", 0) for result in results
    )
    structure_reading_order_sorted = all_structure_reading_order_sorted(results)
    structure_order_stable = all_structure_order_stable(results)
    structure_order_mismatches = structure_order_mismatch_count(results)
    structure_parity_passed = all_structure_parity_passed(results)
    docling_groundtruth_passed = all_docling_groundtruth_passed(results)
    docling_groundtruth_failure_count = sum(
        result.get("doclingGroundtruthFailureCount", 0) for result in results
    )
    return {
        "fixtureCount": len(results),
        "attachmentClassSummary": attachment_class_summaries(results),
        "totalRows": sum(result["totalRows"] for result in results),
        "totalErrorRows": total_error_rows,
        "totalRequests": sum(result["requestCount"] for result in results),
        "totalArrowIpcBytes": sum(result["arrowIpcBytes"] for result in results),
        "totalAudioTranscriptChars": sum(
            result.get("audioTranscriptChars", 0) for result in results
        ),
        "totalAudioTranscriptTimelineMarkerCount": sum(
            result.get("audioTranscriptTimelineMarkerCount", 0) for result in results
        ),
        "totalAudioTranscriptTimelineMarkedRows": sum(
            result.get("audioTranscriptTimelineMarkedRows", 0) for result in results
        ),
        "totalAudioTranscriptOrgRows": sum(
            result.get("audioTranscriptOrgRows", 0) for result in results
        ),
        "totalAudioTranscriptOrgChars": sum(
            result.get("audioTranscriptOrgChars", 0) for result in results
        ),
        "totalAudioTranscriptOrgTimelineMarkerCount": sum(
            result.get("audioTranscriptOrgTimelineMarkerCount", 0) for result in results
        ),
        "totalAudioTranscriptReferenceDraftRows": sum(
            result.get("audioTranscriptReferenceDraftRows", 0) for result in results
        ),
        "totalAudioTranscriptReferenceDraftChars": sum(
            result.get("audioTranscriptReferenceDraftChars", 0) for result in results
        ),
        "totalAudioTranscriptReferenceDraftEmptyRows": sum(
            result.get("audioTranscriptReferenceDraftEmptyRows", 0) for result in results
        ),
        "totalAudioTranscriptReferenceDraftDuplicateTextHashCount": sum(
            result.get("audioTranscriptReferenceDraftDuplicateTextHashCount", 0)
            for result in results
        ),
        "minAudioTranscriptReferenceDraftChars": min(
            (
                result.get("audioTranscriptReferenceDraftMinChars", 0)
                for result in results
                if result.get("audioTranscriptReferenceDraftRows", 0) > 0
            ),
            default=0,
        ),
        "maxAudioTranscriptReferenceDraftChars": max(
            (
                result.get("audioTranscriptReferenceDraftMaxChars", 0)
                for result in results
                if result.get("audioTranscriptReferenceDraftRows", 0) > 0
            ),
            default=0,
        ),
        "totalForceAudioMaterializationShardCount": sum(
            result.get("forceAudioMaterializationShardCount", 0) for result in results
        ),
        "totalForceAudioMaterializationByteCount": sum(
            result.get("forceAudioMaterializationByteCount", 0) for result in results
        ),
        "forceAudioMaterializationArtifactCacheBackendCounts": _combine_int_counts(
            result.get("forceAudioMaterializationArtifactCacheBackendCounts", {})
            for result in results
        ),
        "totalForceAudioMaterializationArtifactCacheHitCount": sum(
            result.get("forceAudioMaterializationArtifactCacheHitCount", 0) for result in results
        ),
        "totalForceAudioMaterializationArtifactCacheHitBytes": sum(
            result.get("forceAudioMaterializationArtifactCacheHitBytes", 0) for result in results
        ),
        "totalForceAudioMaterializationMediaSplitterCount": sum(
            result.get("forceAudioMaterializationMediaSplitterCount", 0) for result in results
        ),
        "totalForceAudioMaterializationMediaSplitterBytes": sum(
            result.get("forceAudioMaterializationMediaSplitterBytes", 0) for result in results
        ),
        "totalForceAudioMaterializationArtifactCacheConfigErrors": sum(
            result.get("forceAudioMaterializationArtifactCacheConfigErrorCount", 0)
            for result in results
        ),
        "forceAudioMaterializationWorkflowStageElapsedMs": _combine_float_counts(
            result.get("forceAudioMaterializationWorkflowStageElapsedMs", {}) for result in results
        ),
        "totalForceAudioMaterializationWorkflowElapsedMs": sum(
            result.get("forceAudioMaterializationWorkflowTotalElapsedMs", 0.0) for result in results
        ),
        "totalArtifactReuseAudioMaterializationShardCount": sum(
            result.get("artifactRegistryReuseAudioMaterializationShardCount", 0)
            for result in results
        ),
        "totalArtifactReuseAudioMaterializationByteCount": sum(
            result.get("artifactRegistryReuseAudioMaterializationByteCount", 0)
            for result in results
        ),
        "artifactReuseAudioMaterializationArtifactCacheBackendCounts": _combine_int_counts(
            result.get(
                "artifactRegistryReuseAudioMaterializationArtifactCacheBackendCounts",
                {},
            )
            for result in results
        ),
        "totalArtifactReuseAudioMaterializationArtifactCacheHitCount": sum(
            result.get(
                "artifactRegistryReuseAudioMaterializationArtifactCacheHitCount",
                0,
            )
            for result in results
        ),
        "totalArtifactReuseAudioMaterializationArtifactCacheHitBytes": sum(
            result.get(
                "artifactRegistryReuseAudioMaterializationArtifactCacheHitBytes",
                0,
            )
            for result in results
        ),
        "totalArtifactReuseAudioMaterializationMediaSplitterCount": sum(
            result.get("artifactRegistryReuseAudioMaterializationMediaSplitterCount", 0)
            for result in results
        ),
        "totalArtifactReuseAudioMaterializationMediaSplitterBytes": sum(
            result.get("artifactRegistryReuseAudioMaterializationMediaSplitterBytes", 0)
            for result in results
        ),
        "totalArtifactReuseAudioMaterializationArtifactCacheConfigErrors": sum(
            result.get(
                "artifactRegistryReuseAudioMaterializationArtifactCacheConfigErrorCount",
                0,
            )
            for result in results
        ),
        "artifactReuseAudioMaterializationWorkflowStageElapsedMs": _combine_float_counts(
            result.get(
                "artifactRegistryReuseAudioMaterializationWorkflowStageElapsedMs",
                {},
            )
            for result in results
        ),
        "totalArtifactReuseAudioMaterializationWorkflowElapsedMs": sum(
            result.get(
                "artifactRegistryReuseAudioMaterializationWorkflowTotalElapsedMs",
                0.0,
            )
            for result in results
        ),
        "totalForceAudioTranscriptAdmissionHitCount": sum(
            result.get("forceAudioTranscriptAdmissionHitCount", 0) for result in results
        ),
        "totalForceAudioTranscriptAdmissionMissCount": sum(
            result.get("forceAudioTranscriptAdmissionMissCount", 0) for result in results
        ),
        "totalForceAudioTranscriptAdmissionStoredCount": sum(
            result.get("forceAudioTranscriptAdmissionStoredCount", 0) for result in results
        ),
        "totalForceAudioTranscriptAdmissionStaleCount": sum(
            result.get("forceAudioTranscriptAdmissionStaleCount", 0) for result in results
        ),
        "totalForceAudioTranscriptAdmissionPlannedHitCount": sum(
            result.get("forceAudioTranscriptAdmissionPlannedHitCount", 0) for result in results
        ),
        "totalForceAudioTranscriptAdmissionPlannedMissCount": sum(
            result.get("forceAudioTranscriptAdmissionPlannedMissCount", 0) for result in results
        ),
        "totalForceAudioTranscriptAdmissionPlannedStoredCount": sum(
            result.get("forceAudioTranscriptAdmissionPlannedStoredCount", 0) for result in results
        ),
        "totalForceAudioTranscriptAdmissionPlannedStaleCount": sum(
            result.get("forceAudioTranscriptAdmissionPlannedStaleCount", 0) for result in results
        ),
        "totalArtifactReuseAudioTranscriptAdmissionHitCount": sum(
            result.get("artifactRegistryReuseAudioTranscriptAdmissionHitCount", 0)
            for result in results
        ),
        "totalArtifactReuseAudioTranscriptAdmissionMissCount": sum(
            result.get("artifactRegistryReuseAudioTranscriptAdmissionMissCount", 0)
            for result in results
        ),
        "totalArtifactReuseAudioTranscriptAdmissionStoredCount": sum(
            result.get("artifactRegistryReuseAudioTranscriptAdmissionStoredCount", 0)
            for result in results
        ),
        "totalArtifactReuseAudioTranscriptAdmissionStaleCount": sum(
            result.get("artifactRegistryReuseAudioTranscriptAdmissionStaleCount", 0)
            for result in results
        ),
        "totalArtifactReuseAudioTranscriptAdmissionPlannedHitCount": sum(
            result.get("artifactRegistryReuseAudioTranscriptAdmissionPlannedHitCount", 0)
            for result in results
        ),
        "totalArtifactReuseAudioTranscriptAdmissionPlannedMissCount": sum(
            result.get("artifactRegistryReuseAudioTranscriptAdmissionPlannedMissCount", 0)
            for result in results
        ),
        "totalArtifactReuseAudioTranscriptAdmissionPlannedStoredCount": sum(
            result.get("artifactRegistryReuseAudioTranscriptAdmissionPlannedStoredCount", 0)
            for result in results
        ),
        "totalArtifactReuseAudioTranscriptAdmissionPlannedStaleCount": sum(
            result.get("artifactRegistryReuseAudioTranscriptAdmissionPlannedStaleCount", 0)
            for result in results
        ),
        "totalStructureRows": sum(result.get("structureRows", 0) for result in results),
        "totalStructureOcrPageBlocks": sum(
            result.get("structureOcrPageBlocks", 0) for result in results
        ),
        "totalStructureOcrRegionBlocks": sum(
            result.get("structureOcrRegionBlocks", 0) for result in results
        ),
        "totalStructureBboxBlocks": sum(result.get("structureBboxBlocks", 0) for result in results),
        "allStructureReadingOrderSorted": structure_reading_order_sorted,
        "allStructureOrderStable": structure_order_stable,
        "totalStructureOrderMismatches": structure_order_mismatches,
        "structureParityCheckedFixtures": sum(
            1 for result in results if result.get("structureParityChecked")
        ),
        "allStructureParityPassed": structure_parity_passed,
        "totalStructureParityErrors": structure_parity_error_count,
        "doclingGroundtruthCheckedFixtures": sum(
            1 for result in results if result.get("doclingGroundtruthChecked")
        ),
        "allDoclingGroundtruthPassed": docling_groundtruth_passed,
        "totalDoclingGroundtruthMissing": sum(
            result.get("doclingGroundtruthMissingCount", 0) for result in results
        ),
        "totalDoclingGroundtruthFailures": docling_groundtruth_failure_count,
        "minDoclingGroundtruthMarkdownSimilarity": min(
            (
                value
                for result in results
                if isinstance(
                    (value := result.get("doclingGroundtruthMinMarkdownSimilarity")),
                    int | float,
                )
            ),
            default=None,
        ),
        "minDoclingGroundtruthCharCoverageRatio": min(
            (
                value
                for result in results
                if isinstance(
                    (value := result.get("doclingGroundtruthMinCharCoverageRatio")),
                    int | float,
                )
            ),
            default=None,
        ),
        "totalMetricsRows": sum(result.get("metricsRows", 0) for result in results),
        "totalMetricsResultChars": sum(result.get("metricsResultChars", 0) for result in results),
        "totalMetricsBboxCount": sum(result.get("metricsBboxCount", 0) for result in results),
        "structureAuthorityPages": sum(
            result.get("structureAuthorityPages", 0) for result in results
        ),
        "textShortcutPages": sum(result.get("textShortcutPages", 0) for result in results),
        "ocrPatchRegions": sum(result.get("ocrPatchRegions", 0) for result in results),
        "pageRangeDoclingFallbackPages": sum(
            result.get("pageRangeDoclingFallbackPages", 0) for result in results
        ),
        "pageRangeDoclingFallbackChunkCount": sum(
            result.get("pageRangeDoclingFallbackChunkCount", 0) for result in results
        ),
        "pageRangeDoclingFallbackPlanStrategies": _combine_string_counts(
            (
                plan.get("strategy")
                if isinstance(
                    (plan := result.get("forceHybridPageOcrTimingPageRangeDoclingFallbackPlan")),
                    dict,
                )
                else None
            )
            for result in results
        ),
        "pageRangeDoclingFallbackChunkSummary": (
            _combine_page_range_docling_fallback_chunk_summaries(results)
        ),
        "fullDoclingFallbackCount": sum(
            result.get("fullDoclingFallbackCount", 0) for result in results
        ),
        "totalMetricsRustSchedulerElapsedMs": sum(
            result.get("metricsRustSchedulerElapsedMs", 0.0) for result in results
        ),
        "totalDocumentTimingRows": sum(result.get("documentTimingRows", 0) for result in results),
        "totalDocumentTimingElapsedMs": sum(
            result.get("documentTimingTotalElapsedMs", 0.0) for result in results
        ),
        "totalDocumentTimingOverheadMs": sum(
            result.get("documentTimingOverheadMs", 0.0) or 0.0 for result in results
        ),
        "documentTimingPhaseElapsedMs": _combine_float_counts(
            result.get("documentTimingPhaseElapsedMs", {}) for result in results
        ),
        "forceHybridPageOcrTimingPhaseElapsedMs": _combine_float_counts(
            result.get("forceHybridPageOcrTimingPhaseElapsedMs", {}) for result in results
        ),
        "forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheHitCount": sum(
            _numeric_result_value(
                result,
                "forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheHitCount",
            )
            for result in results
        ),
        "forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheMissCount": sum(
            _numeric_result_value(
                result,
                "forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheMissCount",
            )
            for result in results
        ),
        "forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheThrottledCount": sum(
            _numeric_result_value(
                result,
                "forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheThrottledCount",
            )
            for result in results
        ),
        "forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheByteCount": sum(
            _numeric_result_value(
                result,
                "forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheByteCount",
            )
            for result in results
        ),
        **_sum_prefixed_ocr2_region_render_artifact_kind_counts(
            results,
            "forceHybridPageOcrTiming",
        ),
        "shardCacheReuseHybridPageOcrTimingPhaseElapsedMs": _combine_float_counts(
            result.get("shardCacheReuseHybridPageOcrTimingPhaseElapsedMs", {}) for result in results
        ),
        "shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheHitCount": sum(
            _numeric_result_value(
                result,
                "shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheHitCount",
            )
            for result in results
        ),
        "shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheMissCount": sum(
            _numeric_result_value(
                result,
                "shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheMissCount",
            )
            for result in results
        ),
        "shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheThrottledCount": sum(
            _numeric_result_value(
                result,
                "shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheThrottledCount",
            )
            for result in results
        ),
        "shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheByteCount": sum(
            _numeric_result_value(
                result,
                "shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheByteCount",
            )
            for result in results
        ),
        **_sum_prefixed_ocr2_region_render_artifact_kind_counts(
            results,
            "shardCacheReuseHybridPageOcrTiming",
        ),
        "regionProjectionReuseHybridPageOcrTimingPhaseElapsedMs": _combine_float_counts(
            result.get("regionProjectionReuseHybridPageOcrTimingPhaseElapsedMs", {})
            for result in results
        ),
        "regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheHitCount": sum(
            _numeric_result_value(
                result,
                "regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheHitCount",
            )
            for result in results
        ),
        "regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheMissCount": sum(
            _numeric_result_value(
                result,
                "regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheMissCount",
            )
            for result in results
        ),
        "regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheThrottledCount": sum(
            _numeric_result_value(
                result,
                "regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheThrottledCount",
            )
            for result in results
        ),
        "regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheByteCount": sum(
            _numeric_result_value(
                result,
                "regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheByteCount",
            )
            for result in results
        ),
        **_sum_prefixed_ocr2_region_render_artifact_kind_counts(
            results,
            "regionProjectionReuseHybridPageOcrTiming",
        ),
        "maxRegionProjectionReuseMetricsRustSchedulerElapsedMs": max(
            (
                value
                for result in results
                if isinstance(
                    (value := result.get("regionProjectionReuseMetricsRustSchedulerElapsedMs")),
                    int | float,
                )
            ),
            default=None,
        ),
        "maxShardCacheReuseMetricsRustSchedulerElapsedMs": max(
            (
                value
                for result in results
                if isinstance(
                    (value := result.get("shardCacheReuseMetricsRustSchedulerElapsedMs")),
                    int | float,
                )
            ),
            default=None,
        ),
        "hybridPageOcrFallbackReasons": [
            reason
            for result in results
            for reason in result.get("hybridPageOcrFallbackReasons", [])
        ],
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
        "archiveFormatCounts": aggregate_archive_audit_strings(results, "archiveFormat"),
        "archiveAccelerationCandidates": aggregate_archive_acceleration_candidates(
            results,
        ),
        "archiveExtensionCounts": aggregate_archive_extension_counts(results),
        "maxArchiveLargestMemberSizeBytes": max_archive_largest_member_size(results),
        "artifactErrorCount": artifact_error_count,
        "minCacheSpeedup": min((result["cacheSpeedup"] for result in results), default=0.0),
        "totalDuplicateMissConverterCalls": sum(
            result["duplicateMissConverterCalls"] or 0 for result in results
        ),
        "maxDuplicateMissConverterCalls": max(
            (
                result["duplicateMissConverterCalls"]
                for result in results
                if result["duplicateMissConverterCalls"] is not None
            ),
            default=None,
        ),
        "distinctMissFixtureCount": (
            distinct_miss_report.get("fixtureCount", 0) if distinct_miss_report else 0
        ),
        "distinctMissConverterCalls": (
            distinct_miss_report.get("converterCalls") if distinct_miss_report else None
        ),
        "distinctMissErrorRows": distinct_error_rows,
        "rustJobsStatusSummary": rust_jobs_status,
        "precisionSpeedSummary": precision_speed_summary(
            results,
            distinct_miss_report,
            total_error_rows=total_error_rows,
            artifact_error_count=artifact_error_count,
            structure_parity_error_count=structure_parity_error_count,
            structure_reading_order_sorted=structure_reading_order_sorted,
            structure_order_stable=structure_order_stable,
            structure_order_mismatch_count=structure_order_mismatches,
            structure_parity_passed=structure_parity_passed,
            docling_groundtruth_passed=docling_groundtruth_passed,
            docling_groundtruth_failure_count=docling_groundtruth_failure_count,
        ),
    }


def pdf_ocr_profile_label(args: argparse.Namespace) -> str:
    if args.pdf_ocr_worker == "skip":
        return "skip"
    if args.pdf_ocr_worker == "fixture":
        return "fixture"
    if args.flight_mode != "hybrid-page-ocr":
        return "docling-full-document"
    return "source-page-range-or-parallel-image"


def all_docling_groundtruth_passed(results: list[dict[str, Any]]) -> bool | None:
    values = [
        result.get("doclingGroundtruthPassed")
        for result in results
        if result.get("doclingGroundtruthPassed") is not None
    ]
    return all(bool(value) for value in values) if values else None


def _format_optional_float(value: Any) -> str:
    if isinstance(value, (int, float)):
        return f"{float(value):.3f}"
    return ""


def _format_optional_percent(value: Any) -> str:
    if isinstance(value, (int, float)):
        return f"{float(value) * 100.0:.1f}%"
    return ""


def _format_counts(value: Any) -> str:
    if not isinstance(value, dict) or not value:
        return ""
    return ", ".join(
        f"{key}={count}"
        for key, count in sorted(value.items())
        if isinstance(key, str) and isinstance(count, int)
    )


def _format_string_list(value: Any) -> str:
    if not isinstance(value, list) or not value:
        return ""
    return ", ".join(sorted(item for item in value if isinstance(item, str)))


def _format_json_object(value: Any) -> str:
    if not isinstance(value, dict) or not value:
        return ""
    return json.dumps(value, sort_keys=True, separators=(",", ":"))


def _format_slowest_hosted_requests(value: Any) -> str:
    if not isinstance(value, list) or not value:
        return ""
    rendered = []
    for request in value[:5]:
        if not isinstance(request, dict):
            continue
        rendered.append(
            "page={page} region={region} latencyMs={latency} kind={kind} "
            "attempts={attempts} imageBytes={image_bytes} sourcePixels={source_pixels} "
            "chars={chars}{hedge}".format(
                page=request.get("pageIndex"),
                region=request.get("regionIndex"),
                latency=_format_optional_float(request.get("latencyMs")),
                kind=request.get("requestKind"),
                attempts=request.get("httpAttemptCount"),
                image_bytes=request.get("imageBytes"),
                source_pixels=request.get("sourcePixelArea"),
                chars=request.get("markdownChars"),
                hedge=_format_slowest_hosted_request_hedge(request),
            )
        )
    return "; ".join(rendered)


def _format_slowest_hosted_audio_requests(value: Any) -> str:
    if not isinstance(value, list) or not value:
        return ""
    rendered = []
    for request in value[:5]:
        if not isinstance(request, dict):
            continue
        rendered.append(
            "shard={shard} profile={profile} latencyMs={latency} kind={kind} "
            "attempts={attempts} durationMs={duration} mediaStartMs={media_start} "
            "mediaDurationMs={media_duration} chars={chars} endpoint={endpoint}".format(
                shard=request.get("shardElementId"),
                profile=request.get("shardProfile"),
                latency=_format_optional_float(request.get("latencyMs")),
                kind=request.get("requestKind"),
                attempts=request.get("httpAttemptCount"),
                duration=request.get("durationMs"),
                media_start=request.get("mediaStartMs"),
                media_duration=request.get("mediaDurationMs"),
                chars=request.get("textChars"),
                endpoint=request.get("endpointKind"),
            )
        )
    return "; ".join(rendered)


def _format_slowest_hosted_request_hedge(request: dict[str, Any]) -> str:
    hedge_winner = request.get("hedgeWinner")
    if not hedge_winner:
        return ""
    return (
        " hedgeWinner={winner} hedgeDelaySeconds={delay} "
        "hedgePrimaryMs={primary} hedgeSecondaryMs={secondary}"
    ).format(
        winner=hedge_winner,
        delay=_format_optional_float(request.get("hedgeDelaySeconds")),
        primary=_format_optional_float(request.get("hedgePrimaryLatencyMs")),
        secondary=_format_optional_float(request.get("hedgeSecondaryLatencyMs")),
    )


def _format_float_counts(value: Any) -> str:
    if not isinstance(value, dict) or not value:
        return ""
    return ", ".join(
        f"{key}={float(count):.3f}"
        for key, count in sorted(value.items())
        if isinstance(key, str) and isinstance(count, int | float)
    )


def _format_fixture_latency(value: Any) -> str:
    if not isinstance(value, dict):
        return ""
    fixture = value.get("fixture")
    latency = value.get("latencyMs")
    if isinstance(fixture, str) and isinstance(latency, (int, float)):
        return f"{fixture}:{float(latency):.3f}"
    return ""


def _summary_or_first_result_value(payload: dict[str, Any], key: str) -> Any:
    value = payload["summary"].get(key)
    if value is not None:
        return value
    results = payload.get("results")
    if isinstance(results, list) and results and isinstance(results[0], dict):
        return results[0].get(key)
    return None


def render_markdown(payload: dict[str, Any]) -> str:
    rust_status = payload["summary"]["rustJobsStatusSummary"]
    ocr_shard_cache = payload.get("ocrShardCache", {})
    structure_baseline = payload.get("structureBaseline") or {}
    precision_speed = payload["summary"].get("precisionSpeedSummary", {})
    pdf_milestone = precision_speed.get("pdfOcrMilestoneGuard", {})
    hosted_vlm_promotion = payload.get("hostedVlmPromotionGate") or {}
    candidate_taxonomy = payload.get("candidateTaxonomy") or {}
    hosted_vlm_ocr = payload.get("hostedVlmOcr") or {}
    hosted_audio = payload.get("hostedAudio") or {}
    hosted_vlm_ocr_requests = hosted_vlm_ocr.get("requestSummary") or {}
    hosted_audio_requests = hosted_audio.get("requestSummary") or {}
    page_range_chunk_summary = payload["summary"].get("pageRangeDoclingFallbackChunkSummary") or {}
    page_range_chunk_phases = (
        phases
        if isinstance(
            (phases := page_range_chunk_summary.get("documentTimingPhaseElapsedMs")),
            dict,
        )
        else {}
    )
    longest_page_range_chunk_phases = (
        phases
        if isinstance(
            (phases := page_range_chunk_summary.get("longestDocumentTimingPhaseElapsedMs")),
            dict,
        )
        else {}
    )
    lines = [
        "# Wendao Document Extract Performance",
        "",
        f"- Schema: `{payload['schema']}`",
        f"- Mode: `{payload['mode']}`",
        f"- Endpoint: `{payload['endpoint']}`",
        f"- Rust REST endpoint: `{payload['rustRestEndpoint']}`",
        f"- Iterations: `{payload['iterations']}`",
        f"- Concurrency: `{payload['concurrency']}`",
        f"- Flight mode: `{payload['flightMode']}`",
        f"- Wait ms: `{payload['waitMs']}`",
        f"- PDF OCR worker: `{payload['pdfOcrWorker']}`",
        f"- PDF OCR workers: `{payload['pdfOcrWorkers']}`",
        f"- Audio worker: `{payload.get('audioWorker')}`",
        f"- Audio workers: `{payload.get('audioWorkers')}`",
        f"- PDF OCR prewarm profiles: `{payload.get('pdfOcrPrewarmProfiles')}`",
        f"- PDF OCR prewarm source path: `{payload.get('pdfOcrPrewarmSourcePath')}`",
        f"- PDF OCR prewarm page index: `{payload.get('pdfOcrPrewarmPageIndex')}`",
        f"- PDF OCR prewarm page indices: `{payload.get('pdfOcrPrewarmPageIndices')}`",
        f"- PDF OCR prewarm endpoint count: `{payload.get('pdfOcrPrewarmEndpointCount')}`",
        "- Document extract prewarm source path: "
        f"`{payload.get('documentExtractPrewarmSourcePath')}`",
        "- Document extract prewarm page ranges: "
        f"`{payload.get('documentExtractPrewarmPageRanges')}`",
        "- Document extract prewarm page ranges resolved: "
        f"`{payload.get('documentExtractPrewarmPageRangesResolved')}`",
        f"- PDF OCR backend-text page fallback: `{payload.get('pdfOcrBackendTextPageFallback')}`",
        f"- Local Python OCR endpoints: `{payload.get('localPythonOcrEndpointCount', 1)}`",
        f"- Rust PDF OCR worker pool: `{payload['rustPdfOcrWorkers']}`",
        f"- Rust PDF OCR source-range workers: `{payload['rustPdfOcrSourceRangeWorkers']}`",
        f"- Rust audio backend profile: `{payload.get('rustAudioBackendProfile')}`",
        f"- Rust audio chunk ms: `{payload.get('rustAudioChunkMs')}`",
        f"- Rust audio context before ms: `{payload.get('rustAudioContextBeforeMs')}`",
        f"- Rust audio context after ms: `{payload.get('rustAudioContextAfterMs')}`",
        f"- Rust audio recovery split ms: `{payload.get('rustAudioRecoverySplitMs')}`",
        f"- Rust audio sample rate Hz: `{payload.get('rustAudioSampleRateHz')}`",
        f"- Rust audio channels: `{payload.get('rustAudioChannels')}`",
        f"- Rust audio format: `{payload.get('rustAudioFormat')}`",
        f"- Rust audio bitrate: `{payload.get('rustAudioBitrate')}`",
        f"- Rust audio artifact cache dir: `{payload.get('rustAudioArtifactCacheDir')}`",
        "- Rust audio transcript admission dir: "
        f"`{payload.get('rustAudioTranscriptAdmissionDir')}`",
        f"- Rust audio base workers: `{payload.get('rustAudioBaseWorkers')}`",
        f"- Rust audio recovery workers: `{payload.get('rustAudioRecoveryWorkers')}`",
        f"- Rust audio speech segments JSONL: `{payload.get('rustAudioSpeechSegmentsJsonl')}`",
        f"- Rust audio speech merge gap ms: `{payload.get('rustAudioSpeechMergeGapMs')}`",
        f"- Rust audio speech min window ms: `{payload.get('rustAudioSpeechMinWindowMs')}`",
        f"- Rust audio speech limit chunks: `{payload.get('rustAudioSpeechLimitChunks')}`",
        "- Rust PDF Docling page-range chunk plan: "
        f"`{payload.get('rustPdfDoclingPageRangeChunkPlan')}`",
        "- Rust PDF Docling page-range profile: "
        f"`{payload.get('rustPdfDoclingPageRangeProfile', 'full')}`",
        "- Rust PDF Docling page-range hedge delay ms: "
        f"`{payload.get('rustPdfDoclingPageRangeHedgeDelayMs')}`",
        "- Rust PDF Docling page-range structure-cost budget: "
        f"`{payload.get('rustPdfDoclingPageRangeStructureCostBudget')}`",
        "- Rust PDF Docling text-shortcut promotion: "
        f"`{payload.get('rustPdfDoclingTextShortcutPromotion', 'range-fill')}`",
        f"- Rust PDF local backend text: `{payload.get('rustPdfLocalBackendText')}`",
        "- Rust PDF local backend-text empty mode: "
        f"`{payload.get('rustPdfLocalBackendTextEmpty')}`",
        f"- Rust PDF local fast text: `{payload.get('rustPdfLocalFastText')}`",
        "- Rust PDF fast-text source-range split: "
        f"`{payload.get('rustPdfFastTextSourceRangeSplit')}`",
        "- Rust PDF fast-text endpoint affinity: "
        f"`{payload.get('rustPdfFastTextEndpointAffinity')}`",
        "- Rust PDF OCR scheduler lane fairness: "
        f"`{payload.get('rustPdfOcrSchedulerLaneFairness')}`",
        f"- Rust PDF backend-text top-up: `{payload.get('rustPdfBackendTextTopup')}`",
        f"- Rust PDF failed-page recovery: `{payload.get('rustPdfFailedPageRecovery')}`",
        f"- Rust PDF OCR profile planner: `{payload.get('rustPdfOcrProfilePlanner')}`",
        f"- Rust PDF Hosted VLM/OCR render DPI: `{payload.get('rustPdfHostedVlmRenderDpi')}`",
        f"- Rust PDF Hosted VLM/OCR region planner: `{payload.get('rustPdfHostedVlmRegionPlanner')}`",
        "- Rust PDF Hosted VLM/OCR region target pixels: "
        f"`{payload.get('rustPdfHostedVlmRegionTargetPixels')}`",
        "- Rust PDF Hosted VLM/OCR region max slices: "
        f"`{payload.get('rustPdfHostedVlmRegionMaxSlices')}`",
        f"- Rust PDF Hosted VLM/OCR region pipeline: `{payload.get('rustPdfHostedVlmRegionPipeline')}`",
        f"- Rust PDF Hosted VLM/OCR region render ahead: `{payload.get('rustPdfHostedVlmRegionRenderAhead')}`",
        f"- Rust PDF Hosted VLM/OCR region render chunk: `{payload.get('rustPdfHostedVlmRegionRenderChunk')}`",
        f"- Rust PDF region render mode: `{payload.get('rustPdfRegionRenderMode')}`",
        f"- Hosted VLM/OCR backend: `{hosted_vlm_ocr.get('backend')}`",
        f"- Hosted VLM/OCR provider: `{hosted_vlm_ocr.get('provider')}`",
        f"- Hosted VLM/OCR base URL: `{hosted_vlm_ocr.get('baseUrl')}`",
        f"- Hosted VLM/OCR model: `{hosted_vlm_ocr.get('model')}`",
        f"- OpenRouter model: `{hosted_vlm_ocr.get('openRouterModel')}`",
        f"- OpenRouter key configured: `{hosted_vlm_ocr.get('openRouterApiKeyConfigured')}`",
        "- OpenRouter provider routing: "
        f"`{_format_json_object(hosted_vlm_ocr.get('openRouterProvider'))}`",
        f"- Hosted VLM/OCR max tokens: `{hosted_vlm_ocr.get('maxTokens')}`",
        f"- Hosted VLM/OCR region max tokens: `{hosted_vlm_ocr.get('regionMaxTokens')}`",
        f"- Hosted VLM/OCR region prompt mode: `{hosted_vlm_ocr.get('regionPromptMode')}`",
        f"- Hosted VLM/OCR region composite size: `{hosted_vlm_ocr.get('regionCompositeSize')}`",
        f"- Hosted VLM/OCR region composite mode: `{hosted_vlm_ocr.get('regionCompositeMode')}`",
        "- Hosted VLM/OCR region composite observed requests: "
        f"`{hosted_vlm_promotion.get('observed', {}).get('regionCompositeRequestCount')}`",
        "- Hosted VLM/OCR region composite budgets: "
        f"`sourcePixels={hosted_vlm_ocr.get('regionCompositeMaxSourcePixels')}, "
        f"imageBytes={hosted_vlm_ocr.get('regionCompositeMaxImageBytes')}`",
        f"- Hosted VLM/OCR region atlas mode: `{hosted_vlm_ocr.get('regionAtlasMode')}`",
        "- Hosted VLM/OCR image optimization mode: "
        f"`{hosted_vlm_ocr.get('imageOptimizationMode')}`",
        f"- Hosted VLM/OCR timeout seconds: `{hosted_vlm_ocr.get('timeoutSeconds')}`",
        f"- Hosted VLM/OCR request concurrency: `{hosted_vlm_ocr.get('requestConcurrency')}`",
        "- Hosted VLM/OCR speculative retry delay seconds: "
        f"`{hosted_vlm_ocr.get('speculativeRetryDelaySeconds')}`",
        "- Hosted VLM/OCR speculative retry minimums: "
        f"`sourcePixels={hosted_vlm_ocr.get('speculativeRetryMinSourcePixels')}, "
        f"imageBytes={hosted_vlm_ocr.get('speculativeRetryMinImageBytes')}`",
        f"- Hosted VLM/OCR page window size: `{hosted_vlm_ocr.get('pageWindowSize')}`",
        f"- Hosted audio backend: `{hosted_audio.get('backend')}`",
        f"- Hosted audio provider: `{hosted_audio.get('provider')}`",
        f"- Hosted audio base URL: `{hosted_audio.get('baseUrl')}`",
        f"- Hosted audio model: `{hosted_audio.get('model')}`",
        f"- Hosted audio key configured: `{hosted_audio.get('apiKeyConfigured')}`",
        f"- Hosted audio timeout seconds: `{hosted_audio.get('timeoutSeconds')}`",
        f"- Hosted audio request concurrency: `{hosted_audio.get('requestConcurrency')}`",
        "- Audio transcript evidence: "
        f"`chars={payload['summary'].get('totalAudioTranscriptChars')}, "
        "timelineMarkers="
        f"{payload['summary'].get('totalAudioTranscriptTimelineMarkerCount')}, "
        "timelineRows="
        f"{payload['summary'].get('totalAudioTranscriptTimelineMarkedRows')}, "
        f"orgRows={payload['summary'].get('totalAudioTranscriptOrgRows')}, "
        f"orgChars={payload['summary'].get('totalAudioTranscriptOrgChars')}, "
        "orgTimelineMarkers="
        f"{payload['summary'].get('totalAudioTranscriptOrgTimelineMarkerCount')}, "
        "referenceDraftRows="
        f"{payload['summary'].get('totalAudioTranscriptReferenceDraftRows')}, "
        "referenceDraftChars="
        f"{payload['summary'].get('totalAudioTranscriptReferenceDraftChars')}, "
        "referenceDraftMinChars="
        f"{payload['summary'].get('minAudioTranscriptReferenceDraftChars')}, "
        "referenceDraftMaxChars="
        f"{payload['summary'].get('maxAudioTranscriptReferenceDraftChars')}, "
        "referenceDraftEmptyRows="
        f"{payload['summary'].get('totalAudioTranscriptReferenceDraftEmptyRows')}, "
        "referenceDraftDuplicateTextHashes="
        f"{payload['summary'].get('totalAudioTranscriptReferenceDraftDuplicateTextHashCount')}`",
        "- Audio materialization cache: "
        f"`forceShards={payload['summary'].get('totalForceAudioMaterializationShardCount')}, "
        f"forceBytes={payload['summary'].get('totalForceAudioMaterializationByteCount')}, "
        "forceBackends="
        f"{_format_counts(payload['summary'].get('forceAudioMaterializationArtifactCacheBackendCounts'))}, "
        "forceArtifactHits="
        f"{payload['summary'].get('totalForceAudioMaterializationArtifactCacheHitCount')}, "
        "forceArtifactHitBytes="
        f"{payload['summary'].get('totalForceAudioMaterializationArtifactCacheHitBytes')}, "
        "forceMediaSplitter="
        f"{payload['summary'].get('totalForceAudioMaterializationMediaSplitterCount')}, "
        "forceMediaSplitterBytes="
        f"{payload['summary'].get('totalForceAudioMaterializationMediaSplitterBytes')}, "
        "forceConfigErrors="
        f"{payload['summary'].get('totalForceAudioMaterializationArtifactCacheConfigErrors')}, "
        "reuseShards="
        f"{payload['summary'].get('totalArtifactReuseAudioMaterializationShardCount')}, "
        "reuseBytes="
        f"{payload['summary'].get('totalArtifactReuseAudioMaterializationByteCount')}, "
        "reuseBackends="
        f"{_format_counts(payload['summary'].get('artifactReuseAudioMaterializationArtifactCacheBackendCounts'))}, "
        "reuseArtifactHits="
        f"{payload['summary'].get('totalArtifactReuseAudioMaterializationArtifactCacheHitCount')}, "
        "reuseArtifactHitBytes="
        f"{payload['summary'].get('totalArtifactReuseAudioMaterializationArtifactCacheHitBytes')}, "
        "reuseMediaSplitter="
        f"{payload['summary'].get('totalArtifactReuseAudioMaterializationMediaSplitterCount')}, "
        "reuseMediaSplitterBytes="
        f"{payload['summary'].get('totalArtifactReuseAudioMaterializationMediaSplitterBytes')}, "
        "reuseConfigErrors="
        f"{payload['summary'].get('totalArtifactReuseAudioMaterializationArtifactCacheConfigErrors')}, "
        "forceWorkflowMs="
        f"{payload['summary'].get('totalForceAudioMaterializationWorkflowElapsedMs')}, "
        "forceWorkflowStages="
        f"{_format_float_counts(payload['summary'].get('forceAudioMaterializationWorkflowStageElapsedMs'))}, "
        "reuseWorkflowMs="
        f"{payload['summary'].get('totalArtifactReuseAudioMaterializationWorkflowElapsedMs')}, "
        "reuseWorkflowStages="
        f"{_format_float_counts(payload['summary'].get('artifactReuseAudioMaterializationWorkflowStageElapsedMs'))}`",
        "- Audio hosted non-model timing: "
        f"`requestWallMs={_format_optional_float(payload['summary'].get('forceAudioHostedRequestWallSpanMs'))}, "
        "analyzerCallMs="
        f"{_format_optional_float(payload['summary'].get('forceAudioHostedAnalyzerCallMs'))}, "
        "analyzerRequestWallGapMs="
        f"{_format_optional_float(payload['summary'].get('forceAudioHostedAnalyzerRequestWallGapMs'))}, "
        "workflowRequestWallGapMs="
        f"{_format_optional_float(payload['summary'].get('forceAudioHostedWorkflowRequestWallGapMs'))}`",
        "- Audio transcript admission: "
        f"`forceHits={payload['summary'].get('totalForceAudioTranscriptAdmissionHitCount')}, "
        f"forceMisses={payload['summary'].get('totalForceAudioTranscriptAdmissionMissCount')}, "
        f"forceStored={payload['summary'].get('totalForceAudioTranscriptAdmissionStoredCount')}, "
        f"forceStale={payload['summary'].get('totalForceAudioTranscriptAdmissionStaleCount')}, "
        f"forcePlannedHits={payload['summary'].get('totalForceAudioTranscriptAdmissionPlannedHitCount')}, "
        "reuseHits="
        f"{payload['summary'].get('totalArtifactReuseAudioTranscriptAdmissionHitCount')}, "
        "reuseMisses="
        f"{payload['summary'].get('totalArtifactReuseAudioTranscriptAdmissionMissCount')}, "
        "reuseStored="
        f"{payload['summary'].get('totalArtifactReuseAudioTranscriptAdmissionStoredCount')}, "
        "reuseStale="
        f"{payload['summary'].get('totalArtifactReuseAudioTranscriptAdmissionStaleCount')}, "
        "reusePlannedHits="
        f"{payload['summary'].get('totalArtifactReuseAudioTranscriptAdmissionPlannedHitCount')}`",
        "- Hosted audio requests: "
        f"`count={hosted_audio_requests.get('requestCount')}, "
        f"httpAttempts={hosted_audio_requests.get('httpAttemptCountTotal')}, "
        "uniqueShardIds="
        f"{hosted_audio_requests.get('uniqueShardElementIdCount')}, "
        "duplicateShardIds="
        f"{hosted_audio_requests.get('duplicateShardElementIdExtraCount')}, "
        "uniqueMediaStarts="
        f"{hosted_audio_requests.get('uniqueMediaStartMsCount')}, "
        "duplicateMediaStarts="
        f"{hosted_audio_requests.get('duplicateMediaStartMsExtraCount')}, "
        f"success={hosted_audio_requests.get('successCount')}, "
        f"failed={hosted_audio_requests.get('failureCount')}, "
        f"durationMs={hosted_audio_requests.get('durationMsTotal')}, "
        f"mediaDurationMs={hosted_audio_requests.get('mediaDurationMsTotal')}, "
        f"p50Ms={_format_optional_float(hosted_audio_requests.get('latencyMsP50'))}, "
        f"p95Ms={_format_optional_float(hosted_audio_requests.get('latencyMsP95'))}, "
        f"maxMs={_format_optional_float(hosted_audio_requests.get('latencyMsMax'))}, "
        f"wallSpanMs={_format_optional_float(hosted_audio_requests.get('requestWallSpanMs'))}, "
        f"overlapRatio={_format_optional_float(hosted_audio_requests.get('requestLatencyOverlapRatio'))}, "
        f"chars={hosted_audio_requests.get('textCharCountTotal')}, "
        f"models={_format_counts(hosted_audio_requests.get('modelCounts'))}, "
        f"endpoints={_format_counts(hosted_audio_requests.get('endpointKindCounts'))}, "
        f"shardProfiles={_format_counts(hosted_audio_requests.get('shardProfileCounts'))}, "
        "duplicateMediaStartMs="
        f"{_format_counts(hosted_audio_requests.get('duplicateMediaStartMsCounts'))}, "
        f"formats={_format_counts(hosted_audio_requests.get('audioFormatCounts'))}, "
        f"status={_format_counts(hosted_audio_requests.get('statusCounts'))}`",
        "- Hosted audio slowest requests: "
        f"`{_format_slowest_hosted_audio_requests(hosted_audio_requests.get('slowestRequests'))}`",
        "- Hosted VLM/OCR requests: "
        f"`count={hosted_vlm_ocr_requests.get('requestCount')}, "
        f"httpAttempts={hosted_vlm_ocr_requests.get('httpAttemptCountTotal')}, "
        f"pages={hosted_vlm_ocr_requests.get('pageCountTotal')}, "
        f"shards={hosted_vlm_ocr_requests.get('shardCountTotal')}, "
        f"regions={hosted_vlm_ocr_requests.get('regionShardCount')}, "
        f"sourcePixels={hosted_vlm_ocr_requests.get('sourcePixelAreaTotal')}, "
        f"sourcePixelsMax={hosted_vlm_ocr_requests.get('sourcePixelAreaMax')}, "
        f"sourcePixelsAvg={_format_optional_float(hosted_vlm_ocr_requests.get('sourcePixelAreaPerRequestAvg'))}, "
        f"imageBytes={hosted_vlm_ocr_requests.get('imageBytesTotal')}, "
        f"imageBytesMax={hosted_vlm_ocr_requests.get('imageBytesMax')}, "
        f"imageBytesAvg={_format_optional_float(hosted_vlm_ocr_requests.get('imageBytesPerRequestAvg'))}, "
        f"success={hosted_vlm_ocr_requests.get('successCount')}, "
        f"failed={hosted_vlm_ocr_requests.get('failureCount')}, "
        f"p50Ms={_format_optional_float(hosted_vlm_ocr_requests.get('latencyMsP50'))}, "
        f"p95Ms={_format_optional_float(hosted_vlm_ocr_requests.get('latencyMsP95'))}, "
        f"maxMs={_format_optional_float(hosted_vlm_ocr_requests.get('latencyMsMax'))}, "
        f"wallSpanMs={_format_optional_float(hosted_vlm_ocr_requests.get('requestWallSpanMs'))}, "
        f"overlapRatio={_format_optional_float(hosted_vlm_ocr_requests.get('requestLatencyOverlapRatio'))}, "
        f"chars={hosted_vlm_ocr_requests.get('charCountTotal')}, "
        f"kinds={_format_counts(hosted_vlm_ocr_requests.get('requestKindCounts'))}, "
        f"hedgeWinners={_format_counts(hosted_vlm_ocr_requests.get('hedgeWinnerCounts'))}, "
        f"imageModes={_format_counts(hosted_vlm_ocr_requests.get('imageOptimizationModeCounts'))}, "
        f"http={_format_counts(hosted_vlm_ocr_requests.get('httpStatusCounts'))}`",
        "- Hosted VLM/OCR slowest requests: "
        f"`{_format_slowest_hosted_requests(hosted_vlm_ocr_requests.get('slowestRequests'))}`",
        f"- Rust document extract endpoints: `{payload.get('rustDocumentExtractEndpoints', [])}`",
        f"- Rust PDF OCR endpoints: `{payload.get('rustPdfOcrEndpoints', [])}`",
        "- Docling full-profile threads: "
        f"`{payload.get('documentExtractFullThreads', 'auto')}` "
        f"(resolved `{payload.get('documentExtractFullThreadsResolved')}`)",
        f"- Structure baseline root: `{payload.get('structureBaselineRoot')}`",
        f"- PDF OCR profile: `{payload['pdfOcrProfile']}`",
        "- PDF OCR fast-text source converter: "
        f"`{payload.get('pdfOcrFastTextSourceConverter', 'default')}`",
        "- Shard-cache reuse probe: "
        f"`{any(result.get('shardCacheReuseEnabled') for result in payload['results'])}`",
        "- Region-projection reuse probe: "
        f"`{any(result.get('regionProjectionReuseEnabled') for result in payload['results'])}`",
        "- Artifact-registry reuse probe: "
        f"`{any(result.get('artifactRegistryReuseEnabled') for result in payload['results'])}`",
        "- OCR shard cache: "
        f"`files={ocr_shard_cache.get('fileCount')}, "
        f"bytes={ocr_shard_cache.get('totalBytes')}, "
        f"maxBytes={ocr_shard_cache.get('maxBytes')}`",
        "- Duplicate miss converter calls: "
        f"`{payload['summary']['totalDuplicateMissConverterCalls']}`",
        "- Distinct cold-miss converter calls: "
        f"`{payload['summary']['distinctMissConverterCalls']}`",
        f"- Rust job status samples: `{rust_status['sampleCount']}`",
        "- Rust job pressure: "
        f"`queued={rust_status['maxQueuedJobs']}, "
        f"running={rust_status['maxRunningJobs']}, "
        f"inProcessRunning={rust_status['maxInProcessRunningConversions']}, "
        f"minAvailablePermits={rust_status['minAvailableConversionPermits']}`",
        "- Rust adaptive OCR: "
        f"`max={rust_status.get('maxPdfOcrWorkers')}, "
        f"budget={rust_status.get('maxCurrentPdfOcrWorkerBudget')}, "
        f"inProcess={rust_status.get('maxInProcessPdfOcrWorkers')}, "
        f"inFlight={rust_status.get('maxInFlightPdfOcrShards')}`",
        "- Rust adaptive audio: "
        f"`max={rust_status.get('maxAudioShardWorkers')}, "
        f"budget={rust_status.get('maxCurrentAudioShardWorkerBudget')}, "
        f"healthyStreak={rust_status.get('maxAudioShardHealthyStreak')}, "
        f"budgetUp={rust_status.get('maxAudioShardBudgetIncreaseEvents')}, "
        f"budgetDown={rust_status.get('maxAudioShardBudgetDecreaseEvents')}`",
        "- Rust OCR cache/live: "
        f"`hits={rust_status.get('maxPdfOcrCacheHits')}, "
        f"misses={rust_status.get('maxPdfOcrCacheMisses')}, "
        f"liveRequests={rust_status.get('maxPdfOcrLiveRequests')}`",
        "- Rust OCR lanes: "
        f"`sourceRange={rust_status.get('maxPdfOcrSourcePdfPageRangeShards')}, "
        f"renderedPage={rust_status.get('maxPdfOcrRenderedPageShards')}, "
        f"renderedRegion={rust_status.get('maxPdfOcrRenderedRegionShards')}`",
        "- Rust OCR pressure: "
        f"`queueP95Ms={rust_status.get('maxPdfOcrQueueWaitP95Ms')}, "
        f"latencyP95Ms={rust_status.get('maxPdfOcrLatencyP95Ms')}, "
        f"budgetUp={rust_status.get('maxPdfOcrBudgetIncreaseEvents')}, "
        f"budgetDown={rust_status.get('maxPdfOcrBudgetDecreaseEvents')}`",
        "- Rust OCR source-range trace: "
        f"`chunks={precision_speed.get('totalForceHybridPageOcrSourceRangeChunkCount')}, "
        f"maxChunkMs={_format_optional_float(precision_speed.get('maxForceHybridPageOcrSourceRangeChunkMs'))}, "
        f"longestPages={_format_optional_float(precision_speed.get('maxForceHybridPageOcrSourceRangeChunkPageStart'))}-"
        f"{_format_optional_float(precision_speed.get('maxForceHybridPageOcrSourceRangeChunkPageEnd'))}, "
        f"profile={precision_speed.get('maxForceHybridPageOcrSourceRangeChunkProfile')}, "
        f"shardType={precision_speed.get('maxForceHybridPageOcrSourceRangeChunkShardType')}, "
        f"longestChars={_format_optional_float(precision_speed.get('maxForceHybridPageOcrSourceRangeChunkTextChars'))}, "
        f"chars={precision_speed.get('totalForceHybridPageOcrSourceRangeTraceChars')}`",
        "- Rust hosted region render trace: "
        f"`reportedMs={_format_optional_float(_summary_or_first_result_value(payload, 'forceHybridPageOcrTimingOcr2RegionRenderReportedElapsedMs'))}, "
        f"plannedChunks={_summary_or_first_result_value(payload, 'forceHybridPageOcrTimingOcr2RegionPipelinePlannedRenderChunkCount')}, "
        f"endpoints={_summary_or_first_result_value(payload, 'forceHybridPageOcrTimingOcr2RegionPipelineEndpointCount')}, "
        f"renderAhead={_summary_or_first_result_value(payload, 'forceHybridPageOcrTimingOcr2RegionPipelineRenderAheadLimit')}, "
        f"renderSpawns={_summary_or_first_result_value(payload, 'forceHybridPageOcrTimingOcr2RegionPipelineRenderSpawnCount')}, "
        f"renderChunks={_summary_or_first_result_value(payload, 'forceHybridPageOcrTimingOcr2RegionPipelineRenderChunkCount')}, "
        f"dispatchChunks={_summary_or_first_result_value(payload, 'forceHybridPageOcrTimingOcr2RegionPipelineRegionDispatchCount')}, "
        f"cacheHits={_summary_or_first_result_value(payload, 'forceHybridPageOcrTimingOcr2RegionRenderCacheHitCount')}, "
        f"cacheMisses={_summary_or_first_result_value(payload, 'forceHybridPageOcrTimingOcr2RegionRenderCacheMissCount')}, "
        "artifactHits="
        f"{_summary_or_first_result_value(payload, 'forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheHitCount')}, "
        "artifactMisses="
        f"{_summary_or_first_result_value(payload, 'forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheMissCount')}, "
        "artifactThrottled="
        f"{_summary_or_first_result_value(payload, 'forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheThrottledCount')}, "
        "artifactBytes="
        f"{_summary_or_first_result_value(payload, 'forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheByteCount')}`",
        "- Rust hosted region projection reuse: "
        "artifactHits="
        f"`{_summary_or_first_result_value(payload, 'regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheHitCount')}, "
        "artifactMisses="
        f"{_summary_or_first_result_value(payload, 'regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheMissCount')}, "
        "crop="
        f"{_summary_or_first_result_value(payload, 'regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheRegionCropHitCount')}/"
        f"{_summary_or_first_result_value(payload, 'regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheRegionCropMissCount')}, "
        "rowProjection="
        f"{_summary_or_first_result_value(payload, 'regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheRegionManifestProjectionRowHitCount')}/"
        f"{_summary_or_first_result_value(payload, 'regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheRegionManifestProjectionRowMissCount')}, "
        "pageProjection="
        f"{_summary_or_first_result_value(payload, 'regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheRegionManifestProjectionHitCount')}/"
        f"{_summary_or_first_result_value(payload, 'regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheRegionManifestProjectionMissCount')}, "
        "artifactBytes="
        f"{_summary_or_first_result_value(payload, 'regionProjectionReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheByteCount')}, "
        "schedulerMs="
        f"{_format_optional_float(payload['summary'].get('maxRegionProjectionReuseMetricsRustSchedulerElapsedMs'))}`",
        f"- Structure sidecar rows: `{payload['summary']['totalStructureRows']}`",
        "- Structure OCR blocks: "
        f"`page={payload['summary']['totalStructureOcrPageBlocks']}, "
        f"region={payload['summary']['totalStructureOcrRegionBlocks']}`",
        "- Structure reading order sorted: "
        f"`{payload['summary']['allStructureReadingOrderSorted']}`",
        "- Structure order stable across runs: "
        f"`stable={payload['summary'].get('allStructureOrderStable')}, "
        f"mismatches={payload['summary'].get('totalStructureOrderMismatches')}`",
        "- Structure parity: "
        f"`checked={payload['summary'].get('structureParityCheckedFixtures')}, "
        f"passed={payload['summary'].get('allStructureParityPassed')}, "
        f"errors={payload['summary'].get('totalStructureParityErrors')}`",
        "- Structure baseline generation: "
        f"`enabled={bool(structure_baseline.get('enabled'))}, "
        f"fixtures={structure_baseline.get('fixtureCount')}, "
        f"errors={structure_baseline.get('totalErrorRows')}`",
        "- Docling upstream groundtruth: "
        f"`root={payload.get('doclingGroundtruthRoot')}, "
        f"checked={payload['summary'].get('doclingGroundtruthCheckedFixtures')}, "
        f"passed={payload['summary'].get('allDoclingGroundtruthPassed')}, "
        f"missing={payload['summary'].get('totalDoclingGroundtruthMissing')}, "
        f"failures={payload['summary'].get('totalDoclingGroundtruthFailures')}, "
        "minCoverage="
        f"{_format_optional_percent(payload['summary'].get('minDoclingGroundtruthCharCoverageRatio'))}, "
        "minSimilarity="
        f"{_format_optional_percent(payload['summary'].get('minDoclingGroundtruthMarkdownSimilarity'))}`",
        "- Metrics sidecar: "
        f"`rows={payload['summary'].get('totalMetricsRows')}, "
        f"chars={payload['summary'].get('totalMetricsResultChars')}, "
        f"bbox={payload['summary'].get('totalMetricsBboxCount')}, "
        "rustSchedulerElapsedMs="
        f"{_format_optional_float(payload['summary'].get('totalMetricsRustSchedulerElapsedMs'))}`",
        "- Docling-centered routing counts: "
        f"`structureAuthorityPages={payload['summary'].get('structureAuthorityPages')}, "
        f"textShortcutPages={payload['summary'].get('textShortcutPages')}, "
        f"ocrPatchRegions={payload['summary'].get('ocrPatchRegions')}, "
        f"pageRangeDoclingFallbackPages={payload['summary'].get('pageRangeDoclingFallbackPages')}, "
        f"pageRangeDoclingFallbackChunks={payload['summary'].get('pageRangeDoclingFallbackChunkCount')}, "
        f"fullDoclingFallbackCount={payload['summary'].get('fullDoclingFallbackCount')}`",
        "- Document timing sidecar: "
        f"`rows={payload['summary'].get('totalDocumentTimingRows')}, "
        "totalElapsedMs="
        f"{_format_optional_float(payload['summary'].get('totalDocumentTimingElapsedMs'))}, "
        "overheadMs="
        f"{_format_optional_float(payload['summary'].get('totalDocumentTimingOverheadMs'))}, "
        "phases="
        f"{_format_float_counts(payload['summary'].get('documentTimingPhaseElapsedMs'))}`",
        "- Hybrid OCR fallback reasons: "
        f"`{'; '.join(payload['summary'].get('hybridPageOcrFallbackReasons', []))}`",
        "- Hybrid OCR timing sidecar: "
        "forcePhases="
        f"`{_format_float_counts(payload['summary'].get('forceHybridPageOcrTimingPhaseElapsedMs'))}`, "
        "shardReusePhases="
        f"`{_format_float_counts(payload['summary'].get('shardCacheReuseHybridPageOcrTimingPhaseElapsedMs'))}`, "
        "regionProjectionReusePhases="
        f"`{_format_float_counts(payload['summary'].get('regionProjectionReuseHybridPageOcrTimingPhaseElapsedMs'))}`, "
        "shardReuseSchedulerMs="
        f"`{_format_optional_float(payload['summary'].get('maxShardCacheReuseMetricsRustSchedulerElapsedMs'))}`, "
        "regionProjectionReuseSchedulerMs="
        f"`{_format_optional_float(payload['summary'].get('maxRegionProjectionReuseMetricsRustSchedulerElapsedMs'))}`, "
        "hostedLocalGapMs="
        f"`{_format_optional_float(hosted_vlm_promotion.get('observed', {}).get('forceHostedVlmLocalOverheadMs'))}`, "
        "schedulerNonHostedMs="
        f"`{_format_optional_float(hosted_vlm_promotion.get('observed', {}).get('forceHostedVlmSchedulerNonRequestMs'))}`, "
        "baseResultMs="
        f"`{_format_optional_float(hosted_vlm_promotion.get('observed', {}).get('forceHostedVlmRegionPipelineLastBaseResultMs'))}`, "
        "regionResultMs="
        f"`{_format_optional_float(hosted_vlm_promotion.get('observed', {}).get('forceHostedVlmRegionPipelineLastRegionResultMs'))}`, "
        "doclingChunkMaxMs="
        f"`{_format_optional_float(page_range_chunk_summary.get('elapsedMsMax'))}`, "
        "doclingChunkDoclingConvertMaxMs="
        f"`{_format_optional_float(longest_page_range_chunk_phases.get('doclingConvert'))}`, "
        "doclingChunkDoclingConvertTotalMs="
        f"`{_format_optional_float(page_range_chunk_phases.get('doclingConvert'))}`, "
        "doclingChunkProfiles="
        f"`{_format_counts(page_range_chunk_summary.get('documentExtractProfileCounts'))}`, "
        "doclingPlanStrategies="
        f"`{_format_counts(payload['summary'].get('pageRangeDoclingFallbackPlanStrategies'))}`",
        "- Image audit summary: "
        f"`audits={payload['summary'].get('imageAttachmentAuditCount')}, "
        f"knownDims={payload['summary'].get('imageKnownDimensionCount')}, "
        f"formats={_format_counts(payload['summary'].get('imageFormatCounts'))}, "
        "dimensionSources="
        f"{_format_counts(payload['summary'].get('imageDimensionSourceCounts'))}, "
        f"candidates={_format_counts(payload['summary'].get('imageAccelerationCandidates'))}, "
        f"maxWidthPx={payload['summary'].get('maxImageWidthPx')}, "
        f"maxHeightPx={payload['summary'].get('maxImageHeightPx')}, "
        f"maxPixels={payload['summary'].get('maxImagePixelCount')}`",
        "- Archive audit summary: "
        f"`audits={payload['summary'].get('archiveAttachmentAuditCount')}, "
        f"members={payload['summary'].get('archiveMemberCount')}, "
        f"xml={payload['summary'].get('archiveXmlMemberCount')}, "
        f"images={payload['summary'].get('archiveImageMemberCount')}, "
        f"formats={_format_counts(payload['summary'].get('archiveFormatCounts'))}, "
        f"suffixes={_format_counts(payload['summary'].get('archiveExtensionCounts'))}, "
        f"candidates={_format_counts(payload['summary'].get('archiveAccelerationCandidates'))}, "
        "largestMemberBytes="
        f"{payload['summary'].get('maxArchiveLargestMemberSizeBytes')}`",
        "- Precision-speed summary: "
        f"`precisionPassed={precision_speed.get('precisionGatePassed')}, "
        f"errorRows={precision_speed.get('errorRows')}, "
        f"orderSorted={precision_speed.get('structureReadingOrderSorted')}, "
        f"orderStable={precision_speed.get('structureOrderStable')}, "
        f"parityPassed={precision_speed.get('structureParityPassed')}, "
        f"doclingGroundtruthPassed={precision_speed.get('doclingGroundtruthPassed')}, "
        f"maxForceMs={_format_optional_float(precision_speed.get('maxForceRefreshMs'))}, "
        "maxDoclingConvertMs="
        f"{_format_optional_float(precision_speed.get('maxDoclingConvertMs'))}, "
        "maxDoclingShare="
        f"{_format_optional_percent(precision_speed.get('maxDoclingConvertShare'))}, "
        "maxTimingOverheadMs="
        f"{_format_optional_float(precision_speed.get('maxDocumentTimingOverheadMs'))}, "
        "maxBoundaryOverheadShare="
        f"{_format_optional_percent(precision_speed.get('maxDocumentTimingOverheadShare'))}, "
        f"maxCacheP95Ms={_format_optional_float(precision_speed.get('maxCacheHitP95Ms'))}`",
        "- PDF OCR milestone guard: "
        f"`checked={pdf_milestone.get('checked')}, "
        f"passed={pdf_milestone.get('passed')}, "
        f"reason={pdf_milestone.get('reason')}, "
        f"regressions={len(pdf_milestone.get('regressions', []))}`",
        "- Hosted VLM/OCR promotion gate: "
        f"`checked={hosted_vlm_promotion.get('checked')}, "
        f"passed={hosted_vlm_promotion.get('passed')}, "
        f"reasons={len(hosted_vlm_promotion.get('reasons', []))}`",
        "- Candidate taxonomy: "
        f"`precisionCandidate={candidate_taxonomy.get('precisionCandidate')}, "
        f"speedCandidate={candidate_taxonomy.get('speedCandidate')}, "
        f"promotionCandidate={candidate_taxonomy.get('promotionCandidate')}, "
        f"defaultPromotionCandidate={candidate_taxonomy.get('defaultPromotionCandidate')}, "
        "optInPromotionControls="
        f"{_format_string_list(candidate_taxonomy.get('optInPromotionControls'))}, "
        f"rejectedStructureLoss={candidate_taxonomy.get('rejectedStructureLoss')}`",
        f"- Artifact errors: `{payload['summary']['artifactErrorCount']}`",
        "",
        "| Fixture | Requests | Rows/request | Error rows | Duplicate conversions | Queue max | Running max | Permits min | Total rows | Structure rows | OCR blocks | Order sorted | IPC bytes | Force ms | Artifact reuse ms | Region projection reuse ms | Shard reuse force ms | Cache p50 ms | Cache p95 ms | Wall ms | Max RSS KB | Speedup |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for result in payload["results"]:
        error_rows = (
            result["forceErrorRows"]
            + result.get("shardCacheReuseErrorRows", 0)
            + result.get("regionProjectionReuseErrorRows", 0)
            + result.get("artifactRegistryReuseErrorRows", 0)
            + result["cacheErrorRows"]
        )
        ocr_blocks = result.get("structureOcrPageBlocks", 0) + result.get(
            "structureOcrRegionBlocks", 0
        )
        row = {
            **result,
            "errorRows": error_rows,
            "duplicateConversions": result["duplicateMissConverterCalls"],
            "structureRows": result.get("structureRows", 0),
            "ocrBlocks": ocr_blocks,
            "orderSorted": result.get("structureReadingOrderSorted"),
            "shardCacheReuseForceMs": _format_optional_float(result.get("shardCacheReuseForceMs")),
            "regionProjectionReuseForceMs": _format_optional_float(
                result.get("regionProjectionReuseForceMs")
            ),
            "artifactRegistryReuseForceMs": _format_optional_float(
                result.get("artifactRegistryReuseForceMs")
            ),
        }
        lines.append(
            "| {fixture} | {requestCount} | {rows} | {errorRows} | "
            "{duplicateConversions} | {rustJobsMaxQueuedJobs} | "
            "{rustJobsMaxRunningJobs} | {rustJobsMinAvailableConversionPermits} | "
            "{totalRows} | {structureRows} | {ocrBlocks} | {orderSorted} | "
            "{arrowIpcBytes} | "
            "{forceRefreshMs:.3f} | {artifactRegistryReuseForceMs} | "
            "{regionProjectionReuseForceMs} | {shardCacheReuseForceMs} | "
            "{cacheHitP50Ms:.3f} | {cacheHitP95Ms:.3f} | "
            "{wallTimeMs:.3f} | {cacheMaxRssKb} | {cacheSpeedup:.2f} |".format(**row)
        )
    if payload["summary"].get("attachmentClassSummary"):
        lines.extend(
            [
                "",
                "## Attachment Class Summary",
                "",
                "| Class | Fixtures | Error rows | Resource types | Block types | BBox blocks | Image formats | Image dim sources | Image dims | Rust image candidates | Archive formats | Archive members | Rust archive candidates | Order sorted | Order stable | Slowest force | Docling max ms | Docling max share | Boundary overhead max share | Slowest cache p95 | Speedup min |",
                "| --- | ---: | ---: | --- | --- | ---: | --- | --- | ---: | --- | --- | ---: | --- | --- | --- | --- | ---: | ---: | ---: | --- | ---: |",
            ]
        )
        for class_summary in payload["summary"]["attachmentClassSummary"]:
            precision_speed = class_summary["precisionSpeedSummary"]
            lines.append(
                "| {attachmentClass} | {fixtureCount} | {totalErrorRows} | "
                "{resourceTypes} | {blockTypes} | {bboxBlocks} | "
                "{imageFormats} | {imageDimensionSources} | {imageDimensions} | "
                "{imageCandidates} | {archiveFormats} | {archiveMembers} | "
                "{archiveCandidates} | "
                "{orderSorted} | {orderStable} | {slowestForce} | "
                "{maxDoclingConvert} | {maxDoclingShare} | "
                "{maxBoundaryOverheadShare} | {slowestCacheP95} | "
                "{minCacheSpeedup} |".format(
                    **class_summary,
                    resourceTypes=_format_counts(class_summary.get("resourceTypeCounts")),
                    blockTypes=_format_counts(class_summary.get("structureBlockTypeCounts")),
                    bboxBlocks=class_summary.get("structureBboxBlocks", 0),
                    imageFormats=_format_counts(class_summary.get("imageFormatCounts")),
                    imageDimensionSources=_format_counts(
                        class_summary.get("imageDimensionSourceCounts")
                    ),
                    imageDimensions=class_summary.get("imageKnownDimensionCount", 0),
                    imageCandidates=_format_counts(
                        class_summary.get("imageAccelerationCandidates")
                    ),
                    archiveFormats=_format_counts(class_summary.get("archiveFormatCounts")),
                    archiveMembers=class_summary.get("archiveMemberCount", 0),
                    archiveCandidates=_format_counts(
                        class_summary.get("archiveAccelerationCandidates")
                    ),
                    orderSorted=precision_speed.get("structureReadingOrderSorted"),
                    orderStable=precision_speed.get("structureOrderStable"),
                    slowestForce=_format_fixture_latency(class_summary.get("slowestForceFixture")),
                    maxDoclingConvert=_format_optional_float(
                        precision_speed.get("maxDoclingConvertMs")
                    ),
                    maxDoclingShare=_format_optional_percent(
                        precision_speed.get("maxDoclingConvertShare")
                    ),
                    maxBoundaryOverheadShare=_format_optional_percent(
                        precision_speed.get("maxDocumentTimingOverheadShare")
                    ),
                    slowestCacheP95=_format_fixture_latency(
                        class_summary.get("slowestCacheP95Fixture")
                    ),
                    minCacheSpeedup=_format_optional_float(precision_speed.get("minCacheSpeedup")),
                )
            )
    distinct_miss = payload.get("distinctMiss")
    if distinct_miss:
        distinct_status = distinct_miss["rustJobsStatusSummary"]
        lines.extend(
            [
                "",
                "## Distinct Cold Miss Burst",
                "",
                "| Fixtures | Requests | Error rows | Converter calls | Queue max | Running max | In-process running max | Permits min | Capacity | Wall ms | Max conversion ms |",
                "| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
                "| {fixtureCount} | {requestCount} | {errorRows} | {converterCalls} | "
                "{maxQueuedJobs} | {maxRunningJobs} | "
                "{maxInProcessRunningConversions} | {minAvailablePermits} | "
                "{maxRunningConversions} | {wallTimeMs:.3f} | "
                "{maxConversionDurationMs} |".format(
                    **distinct_miss,
                    maxQueuedJobs=distinct_status["maxQueuedJobs"],
                    maxRunningJobs=distinct_status["maxRunningJobs"],
                    maxInProcessRunningConversions=distinct_status[
                        "maxInProcessRunningConversions"
                    ],
                    minAvailablePermits=distinct_status["minAvailableConversionPermits"],
                    maxRunningConversions=distinct_status["maxRunningConversions"],
                    maxConversionDurationMs=distinct_status["maxConversionDurationMs"],
                ),
                "",
                "Fixtures: " + ", ".join(f"`{fixture}`" for fixture in distinct_miss["fixtures"]),
            ]
        )
    lines.append("")
    return "\n".join(lines)


def _combine_float_counts(values: Any) -> dict[str, float]:
    totals: dict[str, float] = {}
    for value in values:
        if not isinstance(value, dict):
            continue
        for key, count in value.items():
            if isinstance(key, str) and isinstance(count, int | float):
                totals[key] = totals.get(key, 0.0) + float(count)
    return dict(sorted(totals.items()))


def _numeric_result_value(result: dict[str, Any], key: str) -> int | float:
    value = result.get(key)
    if isinstance(value, int | float) and not isinstance(value, bool):
        return value
    return 0


def _combine_string_counts(values: Any) -> dict[str, int]:
    totals: dict[str, int] = {}
    for value in values:
        if isinstance(value, str) and value:
            totals[value] = totals.get(value, 0) + 1
    return dict(sorted(totals.items()))


def _combine_int_counts(values: Any) -> dict[str, int]:
    totals: dict[str, int] = {}
    for value in values:
        if not isinstance(value, dict):
            continue
        for key, count in value.items():
            if isinstance(key, str) and isinstance(count, int):
                totals[key] = totals.get(key, 0) + count
    return dict(sorted(totals.items()))


def _combine_page_range_docling_fallback_chunk_summaries(
    results: list[dict[str, Any]],
) -> dict[str, Any]:
    summaries = [
        summary
        for result in results
        if isinstance(
            (summary := result.get("forceHybridPageOcrTimingPageRangeDoclingFallbackChunkSummary")),
            dict,
        )
    ]
    longest = max(
        summaries,
        key=lambda summary: float(summary.get("elapsedMsMax") or 0.0),
        default={},
    )
    chunk_count = sum(
        count for summary in summaries if isinstance((count := summary.get("chunkCount")), int)
    )
    elapsed_total = sum(
        float(value)
        for summary in summaries
        if isinstance((value := summary.get("elapsedMsTotal")), int | float)
    )
    elapsed_max = (
        float(longest["elapsedMsMax"])
        if isinstance(longest.get("elapsedMsMax"), int | float)
        else None
    )
    elapsed_min_values = [
        float(value)
        for summary in summaries
        if isinstance((value := summary.get("elapsedMsMin")), int | float)
    ]
    elapsed_min = min(elapsed_min_values, default=None)
    elapsed_mean = elapsed_total / chunk_count if chunk_count else None
    document_timing_total = sum(
        float(value)
        for summary in summaries
        if isinstance((value := summary.get("documentTimingTotalElapsedMs")), int | float)
    )
    return {
        "chunkCount": chunk_count,
        "elapsedMsMax": elapsed_max,
        "elapsedMsMin": elapsed_min,
        "elapsedMsMean": elapsed_mean,
        "elapsedMsSpread": (
            elapsed_max - elapsed_min
            if elapsed_max is not None and elapsed_min is not None
            else None
        ),
        "elapsedMsMaxToMeanRatio": (
            elapsed_max / elapsed_mean
            if elapsed_max is not None and elapsed_mean is not None and elapsed_mean > 0
            else None
        ),
        "elapsedMsTotal": elapsed_total,
        "documentTimingTotalElapsedMs": document_timing_total,
        "documentTimingPhaseElapsedMs": _combine_float_counts(
            summary.get("documentTimingPhaseElapsedMs", {}) for summary in summaries
        ),
        "documentExtractProfileCounts": _combine_int_counts(
            summary.get("documentExtractProfileCounts", {}) for summary in summaries
        ),
        "resourceRows": sum(
            count
            for summary in summaries
            if isinstance((count := summary.get("resourceRows")), int)
        ),
        "sourceProfilePageCount": sum(
            count
            for summary in summaries
            if isinstance((count := summary.get("sourceProfilePageCount")), int)
        ),
        "sourceProfileEstimatedWeightTotal": sum(
            count
            for summary in summaries
            if isinstance((count := summary.get("sourceProfileEstimatedWeightTotal")), int)
        ),
        "longestPageStart": longest.get("longestPageStart"),
        "longestPageEnd": longest.get("longestPageEnd"),
        "longestOneBasedStart": longest.get("longestOneBasedStart"),
        "longestOneBasedEnd": longest.get("longestOneBasedEnd"),
        "longestResourceRows": longest.get("longestResourceRows"),
        "longestDocumentTimingTotalElapsedMs": longest.get("longestDocumentTimingTotalElapsedMs"),
        "longestDocumentTimingPhaseElapsedMs": longest.get("longestDocumentTimingPhaseElapsedMs"),
        "longestSourceProfile": longest.get("longestSourceProfile"),
    }
