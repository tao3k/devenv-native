"""Benchmark summary and Markdown rendering helpers."""

from __future__ import annotations

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


def summarize_results(
    results: list[dict[str, Any]],
    distinct_miss_report: dict[str, Any] | None = None,
) -> dict[str, Any]:
    rust_jobs_status = combine_rust_jobs_status_summaries(
        [result.get("rustJobsStatusSummary", {}) for result in results]
        + [
            (
                distinct_miss_report.get("rustJobsStatusSummary", {})
                if distinct_miss_report
                else {}
            )
        ]
    )
    distinct_error_rows = (
        distinct_miss_report.get("errorRows", 0) if distinct_miss_report else 0
    )
    total_error_rows = (
        sum(
            result["forceErrorRows"]
            + result.get("shardCacheReuseErrorRows", 0)
            + result.get("artifactRegistryReuseErrorRows", 0)
            + result["cacheErrorRows"]
            for result in results
        )
        + distinct_error_rows
    )
    artifact_error_count = sum(
        result.get("artifactErrorCount", 0) for result in results
    )
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
        "totalStructureRows": sum(result.get("structureRows", 0) for result in results),
        "totalStructureOcrPageBlocks": sum(
            result.get("structureOcrPageBlocks", 0) for result in results
        ),
        "totalStructureOcrRegionBlocks": sum(
            result.get("structureOcrRegionBlocks", 0) for result in results
        ),
        "totalStructureBboxBlocks": sum(
            result.get("structureBboxBlocks", 0) for result in results
        ),
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
        "totalMetricsResultChars": sum(
            result.get("metricsResultChars", 0) for result in results
        ),
        "totalMetricsBboxCount": sum(
            result.get("metricsBboxCount", 0) for result in results
        ),
        "totalMetricsRustSchedulerElapsedMs": sum(
            result.get("metricsRustSchedulerElapsedMs", 0.0) for result in results
        ),
        "totalDocumentTimingRows": sum(
            result.get("documentTimingRows", 0) for result in results
        ),
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
            result.get("forceHybridPageOcrTimingPhaseElapsedMs", {})
            for result in results
        ),
        "shardCacheReuseHybridPageOcrTimingPhaseElapsedMs": _combine_float_counts(
            result.get("shardCacheReuseHybridPageOcrTimingPhaseElapsedMs", {})
            for result in results
        ),
        "maxShardCacheReuseMetricsRustSchedulerElapsedMs": max(
            (
                value
                for result in results
                if isinstance(
                    (
                        value := result.get(
                            "shardCacheReuseMetricsRustSchedulerElapsedMs"
                        )
                    ),
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
        "archiveFormatCounts": aggregate_archive_audit_strings(
            results, "archiveFormat"
        ),
        "archiveAccelerationCandidates": aggregate_archive_acceleration_candidates(
            results,
        ),
        "archiveExtensionCounts": aggregate_archive_extension_counts(results),
        "maxArchiveLargestMemberSizeBytes": max_archive_largest_member_size(results),
        "artifactErrorCount": artifact_error_count,
        "minCacheSpeedup": min(
            (result["cacheSpeedup"] for result in results), default=0.0
        ),
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


def render_markdown(payload: dict[str, Any]) -> str:
    rust_status = payload["summary"]["rustJobsStatusSummary"]
    ocr_shard_cache = payload.get("ocrShardCache", {})
    structure_baseline = payload.get("structureBaseline") or {}
    precision_speed = payload["summary"].get("precisionSpeedSummary", {})
    pdf_milestone = precision_speed.get("pdfOcrMilestoneGuard", {})
    hosted_vlm_promotion = payload.get("hostedVlmPromotionGate") or {}
    hosted_vlm_ocr = payload.get("hostedVlmOcr") or {}
    hosted_vlm_ocr_requests = hosted_vlm_ocr.get("requestSummary") or {}
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
        f"- PDF OCR prewarm profiles: `{payload.get('pdfOcrPrewarmProfiles')}`",
        f"- PDF OCR prewarm source path: `{payload.get('pdfOcrPrewarmSourcePath')}`",
        f"- PDF OCR prewarm page index: `{payload.get('pdfOcrPrewarmPageIndex')}`",
        f"- PDF OCR prewarm page indices: `{payload.get('pdfOcrPrewarmPageIndices')}`",
        f"- PDF OCR prewarm endpoint count: `{payload.get('pdfOcrPrewarmEndpointCount')}`",
        f"- PDF OCR backend-text page fallback: `{payload.get('pdfOcrBackendTextPageFallback')}`",
        f"- Local Python OCR endpoints: `{payload.get('localPythonOcrEndpointCount', 1)}`",
        f"- Rust PDF OCR worker pool: `{payload['rustPdfOcrWorkers']}`",
        f"- Rust PDF OCR source-range workers: `{payload['rustPdfOcrSourceRangeWorkers']}`",
        f"- Rust PDF local backend text: `{payload.get('rustPdfLocalBackendText')}`",
        "- Rust PDF local backend-text empty mode: "
        f"`{payload.get('rustPdfLocalBackendTextEmpty')}`",
        f"- Rust PDF local fast text: `{payload.get('rustPdfLocalFastText')}`",
        "- Rust PDF fast-text source-range split: "
        f"`{payload.get('rustPdfFastTextSourceRangeSplit')}`",
        "- Rust PDF fast-text endpoint affinity: "
        f"`{payload.get('rustPdfFastTextEndpointAffinity')}`",
        f"- Rust PDF backend-text top-up: `{payload.get('rustPdfBackendTextTopup')}`",
        f"- Rust PDF failed-page recovery: `{payload.get('rustPdfFailedPageRecovery')}`",
        f"- Rust PDF OCR profile planner: `{payload.get('rustPdfOcrProfilePlanner')}`",
        f"- Rust PDF Hosted VLM/OCR render DPI: `{payload.get('rustPdfHostedVlmRenderDpi')}`",
        f"- Rust PDF Hosted VLM/OCR region planner: `{payload.get('rustPdfHostedVlmRegionPlanner')}`",
        f"- Rust PDF Hosted VLM/OCR region pipeline: `{payload.get('rustPdfHostedVlmRegionPipeline')}`",
        f"- Rust PDF Hosted VLM/OCR region render ahead: `{payload.get('rustPdfHostedVlmRegionRenderAhead')}`",
        f"- Rust PDF Hosted VLM/OCR region render chunk: `{payload.get('rustPdfHostedVlmRegionRenderChunk')}`",
        f"- Hosted VLM/OCR backend: `{hosted_vlm_ocr.get('backend')}`",
        f"- Hosted VLM/OCR provider: `{hosted_vlm_ocr.get('provider')}`",
        f"- Hosted VLM/OCR base URL: `{hosted_vlm_ocr.get('baseUrl')}`",
        f"- Hosted VLM/OCR model: `{hosted_vlm_ocr.get('model')}`",
        f"- OpenRouter model: `{hosted_vlm_ocr.get('openRouterModel')}`",
        f"- OpenRouter key configured: `{hosted_vlm_ocr.get('openRouterApiKeyConfigured')}`",
        f"- Hosted VLM/OCR max tokens: `{hosted_vlm_ocr.get('maxTokens')}`",
        f"- Hosted VLM/OCR region max tokens: `{hosted_vlm_ocr.get('regionMaxTokens')}`",
        f"- Hosted VLM/OCR region composite size: `{hosted_vlm_ocr.get('regionCompositeSize')}`",
        f"- Hosted VLM/OCR region atlas mode: `{hosted_vlm_ocr.get('regionAtlasMode')}`",
        "- Hosted VLM/OCR image optimization mode: "
        f"`{hosted_vlm_ocr.get('imageOptimizationMode')}`",
        f"- Hosted VLM/OCR timeout seconds: `{hosted_vlm_ocr.get('timeoutSeconds')}`",
        f"- Hosted VLM/OCR request concurrency: `{hosted_vlm_ocr.get('requestConcurrency')}`",
        "- Hosted VLM/OCR speculative retry delay seconds: "
        f"`{hosted_vlm_ocr.get('speculativeRetryDelaySeconds')}`",
        f"- Hosted VLM/OCR page window size: `{hosted_vlm_ocr.get('pageWindowSize')}`",
        "- Hosted VLM/OCR requests: "
        f"`count={hosted_vlm_ocr_requests.get('requestCount')}, "
        f"httpAttempts={hosted_vlm_ocr_requests.get('httpAttemptCountTotal')}, "
        f"pages={hosted_vlm_ocr_requests.get('pageCountTotal')}, "
        f"shards={hosted_vlm_ocr_requests.get('shardCountTotal')}, "
        f"regions={hosted_vlm_ocr_requests.get('regionShardCount')}, "
        f"sourcePixels={hosted_vlm_ocr_requests.get('sourcePixelAreaTotal')}, "
        f"success={hosted_vlm_ocr_requests.get('successCount')}, "
        f"failed={hosted_vlm_ocr_requests.get('failureCount')}, "
        f"p50Ms={_format_optional_float(hosted_vlm_ocr_requests.get('latencyMsP50'))}, "
        f"p95Ms={_format_optional_float(hosted_vlm_ocr_requests.get('latencyMsP95'))}, "
        f"maxMs={_format_optional_float(hosted_vlm_ocr_requests.get('latencyMsMax'))}, "
        f"wallSpanMs={_format_optional_float(hosted_vlm_ocr_requests.get('requestWallSpanMs'))}, "
        f"overlapRatio={_format_optional_float(hosted_vlm_ocr_requests.get('requestLatencyOverlapRatio'))}, "
        f"chars={hosted_vlm_ocr_requests.get('charCountTotal')}, "
        f"kinds={_format_counts(hosted_vlm_ocr_requests.get('requestKindCounts'))}, "
        f"imageModes={_format_counts(hosted_vlm_ocr_requests.get('imageOptimizationModeCounts'))}, "
        f"http={_format_counts(hosted_vlm_ocr_requests.get('httpStatusCounts'))}`",
        f"- Rust document extract endpoints: `{payload.get('rustDocumentExtractEndpoints', [])}`",
        f"- Rust PDF OCR endpoints: `{payload.get('rustPdfOcrEndpoints', [])}`",
        f"- Structure baseline root: `{payload.get('structureBaselineRoot')}`",
        f"- PDF OCR profile: `{payload['pdfOcrProfile']}`",
        "- Shard-cache reuse probe: "
        f"`{any(result.get('shardCacheReuseEnabled') for result in payload['results'])}`",
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
        f"chars={precision_speed.get('totalForceHybridPageOcrSourceRangeTraceChars')}`",
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
        "shardReuseSchedulerMs="
        f"`{_format_optional_float(payload['summary'].get('maxShardCacheReuseMetricsRustSchedulerElapsedMs'))}`, "
        "hostedLocalGapMs="
        f"`{_format_optional_float(hosted_vlm_promotion.get('observed', {}).get('forceHostedVlmLocalOverheadMs'))}`, "
        "schedulerNonHostedMs="
        f"`{_format_optional_float(hosted_vlm_promotion.get('observed', {}).get('forceHostedVlmSchedulerNonRequestMs'))}`, "
        "baseResultMs="
        f"`{_format_optional_float(hosted_vlm_promotion.get('observed', {}).get('forceHostedVlmRegionPipelineLastBaseResultMs'))}`, "
        "regionResultMs="
        f"`{_format_optional_float(hosted_vlm_promotion.get('observed', {}).get('forceHostedVlmRegionPipelineLastRegionResultMs'))}`",
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
        f"- Artifact errors: `{payload['summary']['artifactErrorCount']}`",
        "",
        "| Fixture | Requests | Rows/request | Error rows | Duplicate conversions | Queue max | Running max | Permits min | Total rows | Structure rows | OCR blocks | Order sorted | IPC bytes | Force ms | Artifact reuse ms | Shard reuse force ms | Cache p50 ms | Cache p95 ms | Wall ms | Max RSS KB | Speedup |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for result in payload["results"]:
        error_rows = (
            result["forceErrorRows"]
            + result.get("shardCacheReuseErrorRows", 0)
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
            "shardCacheReuseForceMs": _format_optional_float(
                result.get("shardCacheReuseForceMs")
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
            "{shardCacheReuseForceMs} | "
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
                    resourceTypes=_format_counts(
                        class_summary.get("resourceTypeCounts")
                    ),
                    blockTypes=_format_counts(
                        class_summary.get("structureBlockTypeCounts")
                    ),
                    bboxBlocks=class_summary.get("structureBboxBlocks", 0),
                    imageFormats=_format_counts(class_summary.get("imageFormatCounts")),
                    imageDimensionSources=_format_counts(
                        class_summary.get("imageDimensionSourceCounts")
                    ),
                    imageDimensions=class_summary.get("imageKnownDimensionCount", 0),
                    imageCandidates=_format_counts(
                        class_summary.get("imageAccelerationCandidates")
                    ),
                    archiveFormats=_format_counts(
                        class_summary.get("archiveFormatCounts")
                    ),
                    archiveMembers=class_summary.get("archiveMemberCount", 0),
                    archiveCandidates=_format_counts(
                        class_summary.get("archiveAccelerationCandidates")
                    ),
                    orderSorted=precision_speed.get("structureReadingOrderSorted"),
                    orderStable=precision_speed.get("structureOrderStable"),
                    slowestForce=_format_fixture_latency(
                        class_summary.get("slowestForceFixture")
                    ),
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
                    minCacheSpeedup=_format_optional_float(
                        precision_speed.get("minCacheSpeedup")
                    ),
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
                    minAvailablePermits=distinct_status[
                        "minAvailableConversionPermits"
                    ],
                    maxRunningConversions=distinct_status["maxRunningConversions"],
                    maxConversionDurationMs=distinct_status["maxConversionDurationMs"],
                ),
                "",
                "Fixtures: "
                + ", ".join(f"`{fixture}`" for fixture in distinct_miss["fixtures"]),
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
