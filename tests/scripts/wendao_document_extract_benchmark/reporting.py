"""Benchmark summary and Markdown rendering helpers."""

from __future__ import annotations

from typing import TYPE_CHECKING

from .attachment_classes import attachment_class_summaries
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
        "documentTimingPhaseElapsedMs": combine_float_counts(
            result.get("documentTimingPhaseElapsedMs", {}) for result in results
        ),
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


def format_optional_float(value: Any) -> str:
    if isinstance(value, (int, float)):
        return f"{float(value):.3f}"
    return ""


def format_counts(value: Any) -> str:
    if not isinstance(value, dict) or not value:
        return ""
    return ", ".join(
        f"{key}={count}"
        for key, count in sorted(value.items())
        if isinstance(key, str) and isinstance(count, int)
    )


def format_float_counts(value: Any) -> str:
    if not isinstance(value, dict) or not value:
        return ""
    return ", ".join(
        f"{key}={float(count):.3f}"
        for key, count in sorted(value.items())
        if isinstance(key, str) and isinstance(count, int | float)
    )


def format_fixture_latency(value: Any) -> str:
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
        f"- Local Python OCR endpoints: `{payload.get('localPythonOcrEndpointCount', 1)}`",
        f"- Rust PDF OCR worker pool: `{payload['rustPdfOcrWorkers']}`",
        f"- Rust PDF OCR source-range workers: `{payload['rustPdfOcrSourceRangeWorkers']}`",
        f"- Rust PDF OCR endpoints: `{payload.get('rustPdfOcrEndpoints', [])}`",
        f"- Structure baseline root: `{payload.get('structureBaselineRoot')}`",
        f"- PDF OCR profile: `{payload['pdfOcrProfile']}`",
        "- Shard-cache reuse probe: "
        f"`{any(result.get('shardCacheReuseEnabled') for result in payload['results'])}`",
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
        "- Metrics sidecar: "
        f"`rows={payload['summary'].get('totalMetricsRows')}, "
        f"chars={payload['summary'].get('totalMetricsResultChars')}, "
        f"bbox={payload['summary'].get('totalMetricsBboxCount')}, "
        "rustSchedulerElapsedMs="
        f"{format_optional_float(payload['summary'].get('totalMetricsRustSchedulerElapsedMs'))}`",
        "- Document timing sidecar: "
        f"`rows={payload['summary'].get('totalDocumentTimingRows')}, "
        "totalElapsedMs="
        f"{format_optional_float(payload['summary'].get('totalDocumentTimingElapsedMs'))}, "
        "overheadMs="
        f"{format_optional_float(payload['summary'].get('totalDocumentTimingOverheadMs'))}, "
        "phases="
        f"{format_float_counts(payload['summary'].get('documentTimingPhaseElapsedMs'))}`",
        "- Precision-speed summary: "
        f"`precisionPassed={precision_speed.get('precisionGatePassed')}, "
        f"errorRows={precision_speed.get('errorRows')}, "
        f"orderSorted={precision_speed.get('structureReadingOrderSorted')}, "
        f"orderStable={precision_speed.get('structureOrderStable')}, "
        f"parityPassed={precision_speed.get('structureParityPassed')}, "
        f"maxForceMs={format_optional_float(precision_speed.get('maxForceRefreshMs'))}, "
        "maxTimingOverheadMs="
        f"{format_optional_float(precision_speed.get('maxDocumentTimingOverheadMs'))}, "
        f"maxCacheP95Ms={format_optional_float(precision_speed.get('maxCacheHitP95Ms'))}`",
        f"- Artifact errors: `{payload['summary']['artifactErrorCount']}`",
        "",
        "| Fixture | Requests | Rows/request | Error rows | Duplicate conversions | Queue max | Running max | Permits min | Total rows | Structure rows | OCR blocks | Order sorted | IPC bytes | Force ms | Shard reuse force ms | Cache p50 ms | Cache p95 ms | Wall ms | Max RSS KB | Speedup |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for result in payload["results"]:
        error_rows = (
            result["forceErrorRows"]
            + result.get("shardCacheReuseErrorRows", 0)
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
            "shardCacheReuseForceMs": format_optional_float(
                result.get("shardCacheReuseForceMs")
            ),
        }
        lines.append(
            "| {fixture} | {requestCount} | {rows} | {errorRows} | "
            "{duplicateConversions} | {rustJobsMaxQueuedJobs} | "
            "{rustJobsMaxRunningJobs} | {rustJobsMinAvailableConversionPermits} | "
            "{totalRows} | {structureRows} | {ocrBlocks} | {orderSorted} | "
            "{arrowIpcBytes} | "
            "{forceRefreshMs:.3f} | {shardCacheReuseForceMs} | "
            "{cacheHitP50Ms:.3f} | {cacheHitP95Ms:.3f} | "
            "{wallTimeMs:.3f} | {cacheMaxRssKb} | {cacheSpeedup:.2f} |".format(**row)
        )
    if payload["summary"].get("attachmentClassSummary"):
        lines.extend(
            [
                "",
                "## Attachment Class Summary",
                "",
                "| Class | Fixtures | Error rows | Resource types | Block types | BBox blocks | Rust image candidates | Order sorted | Order stable | Slowest force | Slowest cache p95 | Speedup min |",
                "| --- | ---: | ---: | --- | --- | ---: | --- | --- | --- | --- | --- | ---: |",
            ]
        )
        for class_summary in payload["summary"]["attachmentClassSummary"]:
            precision_speed = class_summary["precisionSpeedSummary"]
            lines.append(
                "| {attachmentClass} | {fixtureCount} | {totalErrorRows} | "
                "{resourceTypes} | {blockTypes} | {bboxBlocks} | "
                "{imageCandidates} | "
                "{orderSorted} | {orderStable} | {slowestForce} | "
                "{slowestCacheP95} | {minCacheSpeedup} |".format(
                    **class_summary,
                    resourceTypes=format_counts(
                        class_summary.get("resourceTypeCounts")
                    ),
                    blockTypes=format_counts(
                        class_summary.get("structureBlockTypeCounts")
                    ),
                    bboxBlocks=class_summary.get("structureBboxBlocks", 0),
                    imageCandidates=format_counts(
                        class_summary.get("imageAccelerationCandidates")
                    ),
                    orderSorted=precision_speed.get("structureReadingOrderSorted"),
                    orderStable=precision_speed.get("structureOrderStable"),
                    slowestForce=format_fixture_latency(
                        class_summary.get("slowestForceFixture")
                    ),
                    slowestCacheP95=format_fixture_latency(
                        class_summary.get("slowestCacheP95Fixture")
                    ),
                    minCacheSpeedup=format_optional_float(
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


def combine_float_counts(values: Any) -> dict[str, float]:
    totals: dict[str, float] = {}
    for value in values:
        if not isinstance(value, dict):
            continue
        for key, count in value.items():
            if isinstance(key, str) and isinstance(count, int | float):
                totals[key] = totals.get(key, 0.0) + float(count)
    return dict(sorted(totals.items()))
