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
        "audioTranscriptChars": _sum_int_report_values(
            reports,
            "audioTranscriptChars",
        ),
        "audioTranscriptTimelineMarkerCount": _sum_int_report_values(
            reports,
            "audioTranscriptTimelineMarkerCount",
        ),
        "audioTranscriptTimelineMarkedRows": _sum_int_report_values(
            reports,
            "audioTranscriptTimelineMarkedRows",
        ),
        "audioMaterializationReportExists": any(
            bool(report.get("audioMaterializationReportBytes")) for report in reports
        ),
        "audioMaterializationArtifactCacheConfigured": any(
            bool(report.get("audioMaterializationArtifactCacheConfigured"))
            for report in reports
        ),
        "audioMaterializationArtifactCacheBackendCounts": _string_report_counts(
            reports,
            "audioMaterializationArtifactCacheBackend",
        ),
        "audioMaterializationArtifactCacheMemoryBytes": _max_int_report_value(
            reports,
            "audioMaterializationArtifactCacheMemoryBytes",
        ),
        "audioMaterializationArtifactCacheStorageBytes": _max_int_report_value(
            reports,
            "audioMaterializationArtifactCacheStorageBytes",
        ),
        "audioMaterializationArtifactCacheConfigErrorCount": sum(
            1
            for report in reports
            if report.get("audioMaterializationArtifactCacheConfigError")
        ),
        "audioMaterializationShardCount": _sum_int_report_values(
            reports,
            "audioMaterializationShardCount",
        ),
        "audioMaterializationByteCount": _sum_int_report_values(
            reports,
            "audioMaterializationByteCount",
        ),
        "audioMaterializationArtifactCacheHitCount": _sum_int_report_values(
            reports,
            "audioMaterializationArtifactCacheHitCount",
        ),
        "audioMaterializationArtifactCacheHitBytes": _sum_int_report_values(
            reports,
            "audioMaterializationArtifactCacheHitBytes",
        ),
        "audioMaterializationExistingOutputCount": _sum_int_report_values(
            reports,
            "audioMaterializationExistingOutputCount",
        ),
        "audioMaterializationExistingOutputBytes": _sum_int_report_values(
            reports,
            "audioMaterializationExistingOutputBytes",
        ),
        "audioMaterializationMediaSplitterCount": _sum_int_report_values(
            reports,
            "audioMaterializationMediaSplitterCount",
        ),
        "audioMaterializationMediaSplitterBytes": _sum_int_report_values(
            reports,
            "audioMaterializationMediaSplitterBytes",
        ),
        "audioMaterializationSourceCounts": _aggregate_int_report_maps(
            reports,
            "audioMaterializationSourceCounts",
        ),
        "audioMaterializationSourceBytes": _aggregate_int_report_maps(
            reports,
            "audioMaterializationSourceBytes",
        ),
        "audioTranscriptAdmissionReportExists": any(
            bool(report.get("audioTranscriptAdmissionReportBytes")) for report in reports
        ),
        "audioTranscriptAdmissionEnabled": any(
            bool(report.get("audioTranscriptAdmissionEnabled")) for report in reports
        ),
        "audioTranscriptAdmissionHitCount": _sum_int_report_values(
            reports,
            "audioTranscriptAdmissionHitCount",
        ),
        "audioTranscriptAdmissionMissCount": _sum_int_report_values(
            reports,
            "audioTranscriptAdmissionMissCount",
        ),
        "audioTranscriptAdmissionStoredCount": _sum_int_report_values(
            reports,
            "audioTranscriptAdmissionStoredCount",
        ),
        "audioTranscriptAdmissionStaleCount": _sum_int_report_values(
            reports,
            "audioTranscriptAdmissionStaleCount",
        ),
        "audioTranscriptAdmissionPlannedHitCount": _sum_int_report_values(
            reports,
            "audioTranscriptAdmissionPlannedHitCount",
        ),
        "audioTranscriptAdmissionPlannedMissCount": _sum_int_report_values(
            reports,
            "audioTranscriptAdmissionPlannedMissCount",
        ),
        "audioTranscriptAdmissionPlannedStoredCount": _sum_int_report_values(
            reports,
            "audioTranscriptAdmissionPlannedStoredCount",
        ),
        "audioTranscriptAdmissionPlannedStaleCount": _sum_int_report_values(
            reports,
            "audioTranscriptAdmissionPlannedStaleCount",
        ),
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
        "hybridPageOcrFallbackReasons": _hybrid_page_ocr_fallback_reasons(reports),
        "hybridPageOcrTimingReportExists": any(
            _hybrid_page_ocr_timing_report_exists(report) for report in reports
        ),
        "hybridPageOcrTimingTotalElapsedMs": _sum_float_report_values(
            reports,
            "hybridPageOcrTimingTotalElapsedMs",
        ),
        "hybridPageOcrTimingPhaseElapsedMs": _aggregate_float_report_maps(
            reports,
            "hybridPageOcrTimingPhaseElapsedMs",
        ),
        "hybridPageOcrTimingOcrShardCount": _sum_int_report_values(
            reports,
            "hybridPageOcrTimingOcrShardCount",
        ),
        "hybridPageOcrTimingOcr2RegionShardCount": _sum_int_report_values(
            reports,
            "hybridPageOcrTimingOcr2RegionShardCount",
        ),
        "hybridPageOcrTimingOcr2RegionRequestCount": _sum_int_report_values(
            reports,
            "hybridPageOcrTimingOcr2RegionRequestCount",
        ),
        "hybridPageOcrTimingOcr2RegionRenderedShardCount": _sum_int_report_values(
            reports,
            "hybridPageOcrTimingOcr2RegionRenderedShardCount",
        ),
        "hybridPageOcrTimingOcr2RegionRenderCacheHitCount": _sum_int_report_values(
            reports,
            "hybridPageOcrTimingOcr2RegionRenderCacheHitCount",
        ),
        "hybridPageOcrTimingOcr2RegionRenderCacheMissCount": _sum_int_report_values(
            reports,
            "hybridPageOcrTimingOcr2RegionRenderCacheMissCount",
        ),
        "hybridPageOcrTimingOcr2RegionRenderReportedElapsedMs": _sum_float_report_values(
            reports,
            "hybridPageOcrTimingOcr2RegionRenderReportedElapsedMs",
        ),
        "hybridPageOcrTimingOcr2RegionPipelinePlannedRenderChunkCount": (
            _sum_int_report_values(
                reports,
                "hybridPageOcrTimingOcr2RegionPipelinePlannedRenderChunkCount",
            )
        ),
        "hybridPageOcrTimingOcr2RegionPipelineEndpointCount": _max_int_report_value(
            reports,
            "hybridPageOcrTimingOcr2RegionPipelineEndpointCount",
        ),
        "hybridPageOcrTimingOcr2RegionPipelineRenderAheadLimit": _max_int_report_value(
            reports,
            "hybridPageOcrTimingOcr2RegionPipelineRenderAheadLimit",
        ),
        "hybridPageOcrTimingOcr2RegionPipelineRenderSpawnCount": (
            _sum_int_report_values(
                reports,
                "hybridPageOcrTimingOcr2RegionPipelineRenderSpawnCount",
            )
        ),
        "hybridPageOcrTimingOcr2RegionPipelineRenderChunkCount": (
            _sum_int_report_values(
                reports,
                "hybridPageOcrTimingOcr2RegionPipelineRenderChunkCount",
            )
        ),
        "hybridPageOcrTimingOcr2RegionPipelineRegionDispatchCount": (
            _sum_int_report_values(
                reports,
                "hybridPageOcrTimingOcr2RegionPipelineRegionDispatchCount",
            )
        ),
        "structureAuthorityPages": _sum_int_report_values(
            reports,
            "structureAuthorityPages",
        ),
        "textShortcutPages": _sum_int_report_values(
            reports,
            "textShortcutPages",
        ),
        "ocrPatchRegions": _sum_int_report_values(
            reports,
            "ocrPatchRegions",
        ),
        "pageRangeDoclingFallbackPages": _sum_int_report_values(
            reports,
            "pageRangeDoclingFallbackPages",
        ),
        "pageRangeDoclingFallbackChunkCount": _sum_int_report_values(
            reports,
            "pageRangeDoclingFallbackChunkCount",
        ),
        "pageRangeDoclingFallbackPlan": _page_range_docling_fallback_plan(reports),
        "pageRangeDoclingFallbackChunks": _page_range_docling_fallback_chunks(
            reports,
        ),
        "pageRangeDoclingFallbackChunkSummary": (
            _page_range_docling_fallback_chunk_summary(
                _page_range_docling_fallback_chunks(reports),
            )
        ),
        "fullDoclingFallbackCount": _sum_int_report_values(
            reports,
            "fullDoclingFallbackCount",
        ),
        "hybridPageOcrTimingSchedulerTrace": _hybrid_page_ocr_scheduler_trace(
            reports,
        ),
        "hybridPageOcrTimingSchedulerTraceSummary": (
            _hybrid_page_ocr_scheduler_trace_summary(
                _hybrid_page_ocr_scheduler_trace(reports),
            )
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


def _hybrid_page_ocr_fallback_reasons(reports: list[dict[str, Any]]) -> list[str]:
    return [
        reason
        for report in reports
        if isinstance((reason := report.get("hybridPageOcrFallbackReason")), str)
        and reason
    ]


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


def _max_int_report_value(reports: list[dict[str, Any]], key: str) -> int:
    return max(
        (value for report in reports if isinstance((value := report.get(key)), int)),
        default=0,
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


def _aggregate_int_report_maps(
    reports: list[dict[str, Any]],
    key: str,
) -> dict[str, int]:
    totals: dict[str, int] = {}
    for report in reports:
        values = report.get(key)
        if not isinstance(values, dict):
            continue
        for item_key, item_value in values.items():
            if isinstance(item_key, str) and isinstance(item_value, int):
                totals[item_key] = totals.get(item_key, 0) + item_value
    return dict(sorted(totals.items()))


def _string_report_counts(reports: list[dict[str, Any]], key: str) -> dict[str, int]:
    counts: dict[str, int] = {}
    for report in reports:
        value = report.get(key)
        if isinstance(value, str) and value:
            counts[value] = counts.get(value, 0) + 1
    return dict(sorted(counts.items()))


def _document_timing_arrow_exists(report: dict[str, Any]) -> bool:
    if bool(report.get("documentTimingArrowExists")):
        return True
    arrow_bytes = report.get("documentTimingArrowBytes")
    row_count = report.get("documentTimingRowCount")
    return (isinstance(arrow_bytes, int) and arrow_bytes > 0) or (
        isinstance(row_count, int) and row_count > 0
    )


def _hybrid_page_ocr_timing_report_exists(report: dict[str, Any]) -> bool:
    if bool(report.get("hybridPageOcrTimingReportExists")):
        return True
    report_bytes = report.get("hybridPageOcrTimingReportBytes")
    total_elapsed_ms = report.get("hybridPageOcrTimingTotalElapsedMs")
    has_report_bytes = isinstance(report_bytes, int) and report_bytes > 0
    has_total_elapsed = (
        isinstance(total_elapsed_ms, int | float) and total_elapsed_ms > 0
    )
    return has_report_bytes or has_total_elapsed


def _hybrid_page_ocr_scheduler_trace(
    reports: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    trace: list[dict[str, Any]] = []
    for report in reports:
        rows = report.get("hybridPageOcrTimingSchedulerTrace")
        if not isinstance(rows, list):
            continue
        trace.extend(row for row in rows if isinstance(row, dict))
    return trace


def _hybrid_page_ocr_scheduler_trace_summary(
    trace: list[dict[str, Any]],
) -> dict[str, Any]:
    source_range = [row for row in trace if row.get("lane") == "source-pdf-page-range"]
    longest = max(
        source_range,
        key=lambda row: float(row.get("latencyMs") or 0.0),
        default={},
    )
    return {
        "sourceRangeChunkCount": len(source_range),
        "sourceRangeShardCount": _sum_numeric_trace_values(
            source_range,
            "shardCount",
        ),
        "sourceRangeTextCharCount": _sum_numeric_trace_values(
            source_range,
            "textCharCount",
        ),
        "sourceRangeLatencyMsMax": _float_or_none(longest.get("latencyMs")),
        "sourceRangeQueueWaitMsMax": _max_float_trace_value(
            source_range,
            "queueWaitMs",
        ),
        "sourceRangeDispatchStartMsMin": _min_float_trace_value(
            source_range,
            "dispatchStartMs",
        ),
        "sourceRangeDispatchEndMsMax": _max_float_trace_value(
            source_range,
            "dispatchEndMs",
        ),
        "sourceRangeLongestPageStart": _int_or_none(longest.get("pageStart")),
        "sourceRangeLongestPageEnd": _int_or_none(longest.get("pageEnd")),
        "sourceRangeLongestShardCount": _int_or_none(longest.get("shardCount")),
        "sourceRangeLongestOcrProfile": _str_or_none(longest.get("ocrProfile")),
        "sourceRangeLongestShardType": _str_or_none(longest.get("shardType")),
        "sourceRangeLongestQueueWaitMs": _float_or_none(longest.get("queueWaitMs")),
        "sourceRangeLongestDispatchStartMs": _float_or_none(
            longest.get("dispatchStartMs"),
        ),
        "sourceRangeLongestDispatchEndMs": _float_or_none(
            longest.get("dispatchEndMs"),
        ),
        "sourceRangeLongestTextCharCount": _int_or_none(
            longest.get("textCharCount"),
        ),
    }


def _page_range_docling_fallback_chunks(
    reports: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    chunks: list[dict[str, Any]] = []
    for report in reports:
        rows = report.get("pageRangeDoclingFallbackChunks")
        if not isinstance(rows, list):
            continue
        chunks.extend(row for row in rows if isinstance(row, dict))
    return chunks


def _page_range_docling_fallback_plan(
    reports: list[dict[str, Any]],
) -> dict[str, Any] | None:
    for report in reports:
        plan = report.get("pageRangeDoclingFallbackPlan")
        if isinstance(plan, dict):
            return plan
    return None


def _page_range_docling_fallback_chunk_summary(
    chunks: list[dict[str, Any]],
) -> dict[str, Any]:
    longest = max(
        chunks,
        key=lambda row: float(row.get("elapsedMs") or 0.0),
        default={},
    )
    elapsed_values = [
        float(value)
        for row in chunks
        if isinstance((value := row.get("elapsedMs")), int | float)
    ]
    elapsed_total = sum(elapsed_values)
    elapsed_max = max(elapsed_values, default=None)
    elapsed_min = min(elapsed_values, default=None)
    elapsed_mean = elapsed_total / len(elapsed_values) if elapsed_values else None
    document_timing_total = sum(
        float(value)
        for row in chunks
        if isinstance((value := row.get("documentTimingTotalElapsedMs")), int | float)
    )
    return {
        "chunkCount": len(chunks),
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
        "documentTimingPhaseElapsedMs": _aggregate_float_report_maps(
            chunks,
            "documentTimingPhaseElapsedMs",
        ),
        "documentExtractProfileCounts": _string_counts_from_rows(
            chunks,
            "documentExtractProfile",
        ),
        "resourceRows": _sum_numeric_trace_values(chunks, "resourceRows"),
        "sourceProfilePageCount": _sum_nested_int_values(
            chunks,
            "sourceProfile",
            "pageCount",
        ),
        "sourceProfileEstimatedWeightTotal": _sum_nested_int_values(
            chunks,
            "sourceProfile",
            "estimatedWeightTotal",
        ),
        "sourceProfileStructureAuthorityRequiredCount": _sum_nested_int_values(
            chunks,
            "sourceProfile",
            "structureAuthorityRequiredCount",
        ),
        "sourceProfileFastProfileRiskCount": _sum_nested_int_values(
            chunks,
            "sourceProfile",
            "fastProfileRiskCount",
        ),
        "sourceProfileBackendTextTopupCount": _sum_nested_int_values(
            chunks,
            "sourceProfile",
            "backendTextTopupCount",
        ),
        "longestPageStart": _int_or_none(longest.get("pageStart")),
        "longestPageEnd": _int_or_none(longest.get("pageEnd")),
        "longestOneBasedStart": _int_or_none(longest.get("oneBasedStart")),
        "longestOneBasedEnd": _int_or_none(longest.get("oneBasedEnd")),
        "longestResourceRows": _int_or_none(longest.get("resourceRows")),
        "longestDocumentTimingTotalElapsedMs": _float_or_none(
            longest.get("documentTimingTotalElapsedMs"),
        ),
        "longestDocumentTimingPhaseElapsedMs": (
            phases
            if isinstance((phases := longest.get("documentTimingPhaseElapsedMs")), dict)
            else None
        ),
        "longestSourceProfile": (
            source_profile
            if isinstance((source_profile := longest.get("sourceProfile")), dict)
            else None
        ),
    }


def _sum_numeric_trace_values(rows: list[dict[str, Any]], key: str) -> int:
    return sum(value for row in rows if isinstance((value := row.get(key)), int))


def _string_counts_from_rows(rows: list[dict[str, Any]], key: str) -> dict[str, int]:
    counts: dict[str, int] = {}
    for row in rows:
        value = row.get(key)
        if isinstance(value, str) and value:
            counts[value] = counts.get(value, 0) + 1
    return dict(sorted(counts.items()))


def _str_or_none(value: Any) -> str | None:
    return value if isinstance(value, str) and value else None


def _sum_nested_int_values(
    rows: list[dict[str, Any]], parent_key: str, key: str
) -> int:
    return sum(
        value
        for row in rows
        if isinstance((parent := row.get(parent_key)), dict)
        and isinstance((value := parent.get(key)), int)
    )


def _max_float_trace_value(rows: list[dict[str, Any]], key: str) -> float | None:
    values = [
        float(value) for row in rows if isinstance((value := row.get(key)), int | float)
    ]
    return max(values, default=None)


def _min_float_trace_value(rows: list[dict[str, Any]], key: str) -> float | None:
    values = [
        float(value) for row in rows if isinstance((value := row.get(key)), int | float)
    ]
    return min(values, default=None)


def _float_or_none(value: Any) -> float | None:
    if isinstance(value, int | float):
        return float(value)
    return None


def _int_or_none(value: Any) -> int | None:
    if isinstance(value, int):
        return value
    return None


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
