"""Benchmark probe orchestration helpers."""

from __future__ import annotations

from xiuxian_wendao_analyzer.docling_groundtruth import (
    compare_report_artifacts_to_docling_groundtruth,
    summarize_docling_groundtruth_reports,
)

from .artifact_summary import (
    max_rss_kb,
    percentile,
    rows_per_second,
    summarize_artifact_reports,
)
from .attachment_classes import classify_attachment
from .audio_transcript_org import (
    DOCUMENT_RESOURCES_ARROW_CACHE_NAME,
    export_audio_transcript_org,
    export_audio_transcript_reference_drafts,
)
from .common import (
    Any,
    Path,
    argparse,
    json,
)
from .fake_fixtures import distinct_miss_wait_ms
from .features import cargo_features_for_flight_mode
from .http_status import run_command_with_status_sampling
from .providers import apply_rust_pdf_ocr_env
from .runtime import rust_process_env
from .rust_status import (
    combine_rust_jobs_status_summaries,
    summarize_rust_jobs_status_samples,
)
from .structure_consistency import fixture_structure_order_consistency


def run_distinct_miss_probe(
    args: argparse.Namespace,
    fixtures: dict[str, Path],
    output_dir: Path,
) -> dict[str, Any] | None:
    if not fixtures:
        return None
    converter_count_before = read_converter_count(args)
    report = run_cargo_perf_test(
        args,
        next(iter(fixtures.values())),
        output_dir,
        force=False,
        iterations=1,
        concurrency=len(fixtures),
        report_path=output_dir / "distinct-miss.json",
        inputs=fixtures,
        wait_ms=distinct_miss_wait_ms(args),
    )
    converter_count_after = read_converter_count(args)
    converter_calls = None
    if converter_count_before is not None and converter_count_after is not None:
        converter_calls = converter_count_after - converter_count_before
    error_rows = report.get("errorRowCount", 0)
    if args.fail_on_error_rows and error_rows:
        raise SystemExit(
            f"distinct cold-miss burst produced document extraction error rows: {error_rows}"
        )
    if (
        args.fail_on_distinct_miss_conversions
        and converter_calls is not None
        and converter_calls != len(fixtures)
    ):
        raise SystemExit(
            "distinct cold-miss burst converted "
            f"{converter_calls} documents; expected {len(fixtures)}"
        )
    rust_jobs_status_summary = report.get(
        "rustJobsStatusSummary",
        summarize_rust_jobs_status_samples([]),
    )
    return {
        "enabled": True,
        "fixtures": list(fixtures),
        "fixtureCount": len(fixtures),
        "concurrency": len(fixtures),
        "waitMs": distinct_miss_wait_ms(args),
        "requestCount": report.get("requestCount", len(fixtures)),
        "converterCalls": converter_calls,
        "errorRows": error_rows,
        "statusCounts": report.get("statusCounts", {}),
        "wallTimeMs": report.get("wallTimeMs", 0.0),
        "rustJobsStatusSummary": rust_jobs_status_summary,
        "rustJobsStatusSampleCount": rust_jobs_status_summary["sampleCount"],
        "rustJobsMaxQueuedJobs": rust_jobs_status_summary["maxQueuedJobs"],
        "rustJobsMaxRunningJobs": rust_jobs_status_summary["maxRunningJobs"],
        "rustJobsMaxInProcessRunningConversions": rust_jobs_status_summary[
            "maxInProcessRunningConversions"
        ],
        "rustJobsMinAvailableConversionPermits": rust_jobs_status_summary[
            "minAvailableConversionPermits"
        ],
        "rustJobsMaxRunningConversions": rust_jobs_status_summary[
            "maxRunningConversions"
        ],
        "rustJobsMaxConversionDurationMs": rust_jobs_status_summary[
            "maxConversionDurationMs"
        ],
    }


def resolve_structure_baseline_root(
    args: argparse.Namespace,
    report_dir: Path,
) -> Path | None:
    explicit_root = getattr(args, "structure_baseline_root", None)
    if explicit_root is not None:
        return explicit_root.resolve()
    if getattr(args, "generate_structure_baselines", False):
        return (report_dir / "structure-baselines").resolve()
    return None


def run_structure_baseline_probe(
    args: argparse.Namespace,
    fixtures: dict[str, Path],
    baseline_root: Path | None,
) -> dict[str, Any] | None:
    if not getattr(args, "generate_structure_baselines", False):
        return None
    if baseline_root is None:
        raise SystemExit("--generate-structure-baselines requires a baseline root")
    if not fixtures:
        return {
            "enabled": True,
            "root": str(baseline_root),
            "fixtureCount": 0,
            "totalErrorRows": 0,
            "totalStructureRows": 0,
            "allStructureReadingOrderSorted": None,
            "fixtures": [],
        }

    baseline_root.mkdir(parents=True, exist_ok=True)
    fixture_reports = []
    for fixture_name, fixture_path in fixtures.items():
        output_dir = baseline_root / fixture_name
        report = run_cargo_perf_test(
            args,
            fixture_path,
            output_dir,
            force=True,
            iterations=1,
            concurrency=1,
            report_path=output_dir / "baseline.json",
            flight_mode="sync",
            include_structure_baseline_root=False,
        )
        error_rows = report.get("errorRowCount", 0)
        if args.fail_on_error_rows and error_rows:
            raise SystemExit(
                f"structure baseline `{fixture_name}` produced error rows: {error_rows}"
            )
        artifact_summary = summarize_artifact_reports(report.get("artifactReports", []))
        docling_groundtruth_reports = _compare_docling_groundtruth(args, report)
        fixture_reports.append(
            {
                "fixture": fixture_name,
                "source": str(fixture_path),
                "outputDir": str(output_dir),
                "reportPath": str(output_dir / "baseline.json"),
                "errorRows": error_rows,
                "resourcesRows": artifact_summary["resourcesRows"],
                "structureRows": artifact_summary["structureRows"],
                "structureReadingOrderSorted": artifact_summary[
                    "structureReadingOrderSorted"
                ],
                "doclingGroundtruth": summarize_docling_groundtruth_reports(
                    docling_groundtruth_reports,
                ),
                "doclingGroundtruthReports": docling_groundtruth_reports,
            }
        )

    sorted_values = [
        report["structureReadingOrderSorted"]
        for report in fixture_reports
        if report["structureReadingOrderSorted"] is not None
    ]
    return {
        "enabled": True,
        "root": str(baseline_root),
        "fixtureCount": len(fixture_reports),
        "totalErrorRows": sum(report["errorRows"] for report in fixture_reports),
        "totalStructureRows": sum(
            report["structureRows"] for report in fixture_reports
        ),
        "allStructureReadingOrderSorted": (
            all(bool(value) for value in sorted_values) if sorted_values else None
        ),
        "fixtures": fixture_reports,
    }


def run_fixture_probe(
    args: argparse.Namespace,
    fixture_name: str,
    fixture_path: Path,
    output_dir: Path,
) -> dict[str, Any]:
    duplicate_report = None
    duplicate_miss_converter_calls = None
    if args.duplicate_miss_concurrency > 0:
        converter_count_before = read_converter_count(args)
        duplicate_report = run_cargo_perf_test(
            args,
            fixture_path,
            output_dir / "duplicate-miss",
            force=False,
            iterations=1,
            concurrency=args.duplicate_miss_concurrency,
            report_path=output_dir / "duplicate-miss.json",
        )
        converter_count_after = read_converter_count(args)
        if converter_count_before is not None and converter_count_after is not None:
            duplicate_miss_converter_calls = (
                converter_count_after - converter_count_before
            )
        duplicate_error_rows = duplicate_report.get("errorRowCount", 0)
        if args.fail_on_error_rows and duplicate_error_rows:
            raise SystemExit(
                f"fixture `{fixture_name}` duplicate miss produced error rows: "
                f"{duplicate_error_rows}"
            )
        if (
            args.fail_on_duplicate_conversions
            and duplicate_miss_converter_calls is not None
            and duplicate_miss_converter_calls != 1
        ):
            raise SystemExit(
                f"fixture `{fixture_name}` duplicate miss converted "
                f"{duplicate_miss_converter_calls} times; expected 1"
            )

    force_report = run_cargo_perf_test(
        args,
        fixture_path,
        output_dir,
        force=True,
        iterations=1,
        concurrency=1,
        report_path=output_dir / "force.json",
    )
    force_docling_groundtruth_reports = _compare_docling_groundtruth(args, force_report)
    force_docling_groundtruth = summarize_docling_groundtruth_reports(
        force_docling_groundtruth_reports,
    )
    shard_cache_reuse_report = None
    if args.shard_cache_reuse_probe:
        shard_cache_reuse_report = run_cargo_perf_test(
            args,
            fixture_path,
            output_dir / "shard-cache-reuse",
            force=True,
            iterations=1,
            concurrency=1,
            report_path=output_dir / "shard-cache-reuse.json",
        )
    artifact_registry_reuse_report = None
    if args.artifact_registry_reuse_probe:
        artifact_registry_reuse_report = run_cargo_perf_test(
            args,
            fixture_path,
            output_dir / "artifact-registry-reuse",
            force=False,
            iterations=1,
            concurrency=1,
            report_path=output_dir / "artifact-registry-reuse.json",
        )
    cached_report = run_cargo_perf_test(
        args,
        fixture_path,
        output_dir,
        force=False,
        iterations=args.iterations,
        concurrency=args.concurrency,
        report_path=output_dir / "cache.json",
    )
    audio_transcript_org = export_audio_transcript_org_for_fixture(
        args,
        fixture_name,
        output_dir,
    )
    audio_transcript_reference_draft = (
        export_audio_transcript_reference_draft_for_fixture(
            args,
            fixture_name,
            output_dir,
        )
    )
    cached_latencies = cached_report["latenciesMs"]
    request_count = cached_report["requestCount"]
    row_count = cached_report["rowCount"]
    total_rows = row_count * request_count
    force_error_rows = force_report.get("errorRowCount", 0)
    shard_cache_reuse_error_rows = (
        shard_cache_reuse_report.get("errorRowCount", 0)
        if shard_cache_reuse_report
        else 0
    )
    artifact_registry_reuse_error_rows = (
        artifact_registry_reuse_report.get("errorRowCount", 0)
        if artifact_registry_reuse_report
        else 0
    )
    cache_error_rows = cached_report.get("errorRowCount", 0)
    force_artifact_summary = summarize_artifact_reports(
        force_report.get("artifactReports", [])
    )
    artifact_summary = summarize_artifact_reports(
        cached_report.get("artifactReports", [])
    )
    shard_cache_reuse_artifact_summary = (
        summarize_artifact_reports(shard_cache_reuse_report.get("artifactReports", []))
        if shard_cache_reuse_report
        else None
    )
    artifact_registry_reuse_artifact_summary = (
        summarize_artifact_reports(
            artifact_registry_reuse_report.get("artifactReports", [])
        )
        if artifact_registry_reuse_report
        else None
    )
    metrics_rows_by_run = {
        "force": force_artifact_summary["metricsRows"],
        "cache": artifact_summary["metricsRows"],
    }
    if shard_cache_reuse_artifact_summary:
        metrics_rows_by_run["shard_cache_reuse"] = shard_cache_reuse_artifact_summary[
            "metricsRows"
        ]
    if artifact_registry_reuse_artifact_summary:
        metrics_rows_by_run["artifact_registry_reuse"] = (
            artifact_registry_reuse_artifact_summary["metricsRows"]
        )
    structure_order_consistency = fixture_structure_order_consistency(
        force_report,
        cached_report,
        shard_cache_reuse_report,
        artifact_registry_reuse_report,
    )
    if args.fail_on_error_rows and (
        force_error_rows
        or shard_cache_reuse_error_rows
        or artifact_registry_reuse_error_rows
        or cache_error_rows
    ):
        raise SystemExit(
            f"fixture `{fixture_name}` produced document extraction error rows: "
            f"force={force_error_rows}, "
            f"shard_cache_reuse={shard_cache_reuse_error_rows}, "
            f"artifact_registry_reuse={artifact_registry_reuse_error_rows}, "
            f"cache={cache_error_rows}"
        )
    if getattr(args, "fail_on_missing_ocr_metrics", False):
        missing_metrics_runs = [
            name
            for name, metrics_rows in metrics_rows_by_run.items()
            if metrics_rows <= 0
        ]
        if missing_metrics_runs:
            fallback_reasons = hybrid_page_ocr_fallback_reasons_by_run(
                force_report,
                cached_report,
                shard_cache_reuse_report,
                artifact_registry_reuse_report,
            )
            reason_suffix = (
                "; hybrid fallback reasons: " + "; ".join(fallback_reasons)
                if fallback_reasons
                else ""
            )
            raise SystemExit(
                f"fixture `{fixture_name}` produced no OCR metrics rows for: "
                f"{', '.join(missing_metrics_runs)}{reason_suffix}"
            )
    if (
        getattr(args, "fail_on_structure_order_mismatch", False)
        and structure_order_consistency["structureOrderStable"] is False
    ):
        raise SystemExit(
            f"fixture `{fixture_name}` produced unstable structure order across runs: "
            f"mismatches={structure_order_consistency['structureOrderMismatchCount']}"
        )
    if (
        getattr(args, "fail_on_docling_groundtruth_mismatch", False)
        and force_docling_groundtruth["passed"] is False
    ):
        raise SystemExit(
            f"fixture `{fixture_name}` did not match upstream Docling groundtruth: "
            f"{force_docling_groundtruth['failures']}"
        )
    rust_jobs_status_summary = combine_rust_jobs_status_summaries(
        [
            (
                duplicate_report.get("rustJobsStatusSummary", {})
                if duplicate_report
                else {}
            ),
            force_report.get("rustJobsStatusSummary", {}),
            (
                shard_cache_reuse_report.get("rustJobsStatusSummary", {})
                if shard_cache_reuse_report
                else {}
            ),
            (
                artifact_registry_reuse_report.get("rustJobsStatusSummary", {})
                if artifact_registry_reuse_report
                else {}
            ),
            cached_report.get("rustJobsStatusSummary", {}),
        ]
    )
    force_refresh_ms = force_report["latenciesMs"][0]
    document_timing_overhead_ms = document_timing_overhead(
        force_refresh_ms,
        artifact_summary,
    )
    return {
        "fixture": fixture_name,
        "source": str(fixture_path),
        "attachmentClass": classify_attachment(fixture_name, fixture_path),
        "duplicateMissConcurrency": args.duplicate_miss_concurrency,
        "duplicateMissConverterCalls": duplicate_miss_converter_calls,
        "duplicateMissErrorRows": (
            duplicate_report.get("errorRowCount", 0) if duplicate_report else 0
        ),
        "duplicateMissStatusCounts": (
            duplicate_report.get("statusCounts", {}) if duplicate_report else {}
        ),
        "duplicateMissWallTimeMs": (
            duplicate_report.get("wallTimeMs", 0.0) if duplicate_report else 0.0
        ),
        "forceRefreshMs": force_refresh_ms,
        "forceErrorRows": force_error_rows,
        "forceStatusCounts": force_report.get("statusCounts", {}),
        "forceMaxRssKb": force_report.get("maxRssKb"),
        "doclingGroundtruth": force_docling_groundtruth,
        "doclingGroundtruthReports": force_docling_groundtruth_reports,
        "doclingGroundtruthChecked": force_docling_groundtruth["checked"],
        "doclingGroundtruthPassed": force_docling_groundtruth["passed"],
        "doclingGroundtruthMissingCount": force_docling_groundtruth["missingCount"],
        "doclingGroundtruthFailureCount": force_docling_groundtruth["failureCount"],
        "doclingGroundtruthMinMarkdownSimilarity": force_docling_groundtruth[
            "minMarkdownSimilarity"
        ],
        "doclingGroundtruthMinCharCoverageRatio": force_docling_groundtruth[
            "minCharCoverageRatio"
        ],
        "shardCacheReuseEnabled": args.shard_cache_reuse_probe,
        "shardCacheReuseForceMs": (
            shard_cache_reuse_report["latenciesMs"][0]
            if shard_cache_reuse_report
            else None
        ),
        "shardCacheReuseErrorRows": shard_cache_reuse_error_rows,
        "shardCacheReuseStatusCounts": (
            shard_cache_reuse_report.get("statusCounts", {})
            if shard_cache_reuse_report
            else {}
        ),
        "artifactRegistryReuseEnabled": args.artifact_registry_reuse_probe,
        "artifactRegistryReuseForceMs": (
            artifact_registry_reuse_report["latenciesMs"][0]
            if artifact_registry_reuse_report
            else None
        ),
        "artifactRegistryReuseErrorRows": artifact_registry_reuse_error_rows,
        "artifactRegistryReuseStatusCounts": (
            artifact_registry_reuse_report.get("statusCounts", {})
            if artifact_registry_reuse_report
            else {}
        ),
        "concurrency": cached_report["concurrency"],
        "requestCount": request_count,
        "wallTimeMs": cached_report["wallTimeMs"],
        "cacheHitP50Ms": percentile(cached_latencies, 50),
        "cacheHitP95Ms": percentile(cached_latencies, 95),
        "cacheHitMaxMs": max(cached_latencies),
        "cacheErrorRows": cache_error_rows,
        "cacheStatusCounts": cached_report.get("statusCounts", {}),
        "cacheMaxRssKb": cached_report.get("maxRssKb"),
        "rustJobsStatusSummary": rust_jobs_status_summary,
        "rustJobsStatusSampleCount": rust_jobs_status_summary["sampleCount"],
        "rustJobsMaxQueuedJobs": rust_jobs_status_summary["maxQueuedJobs"],
        "rustJobsMaxRunningJobs": rust_jobs_status_summary["maxRunningJobs"],
        "rustJobsMaxInProcessRunningConversions": rust_jobs_status_summary[
            "maxInProcessRunningConversions"
        ],
        "rustJobsMaxInProcessScheduledJobs": rust_jobs_status_summary[
            "maxInProcessScheduledJobs"
        ],
        "rustJobsMinAvailableConversionPermits": rust_jobs_status_summary[
            "minAvailableConversionPermits"
        ],
        "rustJobsMaxConversionDurationMs": rust_jobs_status_summary[
            "maxConversionDurationMs"
        ],
        "rows": row_count,
        "totalRows": total_rows,
        "batches": cached_report["batchCount"],
        "arrowIpcBytes": cached_report["arrowIpcBytes"],
        "resourcesArrowExists": artifact_summary["resourcesArrowExists"],
        "resourcesRows": artifact_summary["resourcesRows"],
        "audioTranscriptChars": artifact_summary["audioTranscriptChars"],
        "audioTranscriptTimelineMarkerCount": artifact_summary[
            "audioTranscriptTimelineMarkerCount"
        ],
        "audioTranscriptTimelineMarkedRows": artifact_summary[
            "audioTranscriptTimelineMarkedRows"
        ],
        "audioTranscriptOrgPath": audio_transcript_org["path"],
        "audioTranscriptOrgRows": audio_transcript_org["rows"],
        "audioTranscriptOrgChars": audio_transcript_org["chars"],
        "audioTranscriptOrgTimelineMarkerCount": audio_transcript_org[
            "timelineMarkerCount"
        ],
        "audioTranscriptReferenceDraftJsonlPath": audio_transcript_reference_draft[
            "jsonlPath"
        ],
        "audioTranscriptReferenceDraftTsvPath": audio_transcript_reference_draft[
            "tsvPath"
        ],
        "audioTranscriptReferenceDraftRows": audio_transcript_reference_draft["rows"],
        "audioTranscriptReferenceDraftChars": audio_transcript_reference_draft["chars"],
        "structureArrowExists": artifact_summary["structureArrowExists"],
        "structureRows": artifact_summary["structureRows"],
        "structureOcrPageBlocks": artifact_summary["structureOcrPageBlocks"],
        "structureOcrRegionBlocks": artifact_summary["structureOcrRegionBlocks"],
        "structureBboxBlocks": artifact_summary["structureBboxBlocks"],
        "structureReadingOrderSorted": artifact_summary["structureReadingOrderSorted"],
        **structure_order_consistency,
        "structureParityChecked": artifact_summary["structureParityChecked"],
        "structureParityPassed": artifact_summary["structureParityPassed"],
        "structureParityErrorCount": artifact_summary["structureParityErrorCount"],
        "metricsArrowExists": artifact_summary["metricsArrowExists"],
        "metricsRows": artifact_summary["metricsRows"],
        "metricsResultChars": artifact_summary["metricsResultChars"],
        "metricsBboxCount": artifact_summary["metricsBboxCount"],
        "metricsRustSchedulerElapsedMs": artifact_summary[
            "metricsRustSchedulerElapsedMs"
        ],
        "forceHybridPageOcrTimingTotalElapsedMs": force_artifact_summary[
            "hybridPageOcrTimingTotalElapsedMs"
        ],
        "forceHybridPageOcrTimingPhaseElapsedMs": force_artifact_summary[
            "hybridPageOcrTimingPhaseElapsedMs"
        ],
        "forceHybridPageOcrTimingOcr2RegionShardCount": force_artifact_summary[
            "hybridPageOcrTimingOcr2RegionShardCount"
        ],
        "forceHybridPageOcrTimingOcr2RegionRequestCount": force_artifact_summary[
            "hybridPageOcrTimingOcr2RegionRequestCount"
        ],
        "forceHybridPageOcrTimingOcr2RegionRenderedShardCount": force_artifact_summary[
            "hybridPageOcrTimingOcr2RegionRenderedShardCount"
        ],
        "forceHybridPageOcrTimingOcr2RegionRenderCacheHitCount": force_artifact_summary[
            "hybridPageOcrTimingOcr2RegionRenderCacheHitCount"
        ],
        "forceHybridPageOcrTimingOcr2RegionRenderCacheMissCount": force_artifact_summary[
            "hybridPageOcrTimingOcr2RegionRenderCacheMissCount"
        ],
        "structureAuthorityPages": force_artifact_summary["structureAuthorityPages"],
        "textShortcutPages": force_artifact_summary["textShortcutPages"],
        "ocrPatchRegions": force_artifact_summary["ocrPatchRegions"],
        "pageRangeDoclingFallbackPages": force_artifact_summary[
            "pageRangeDoclingFallbackPages"
        ],
        "pageRangeDoclingFallbackChunkCount": force_artifact_summary[
            "pageRangeDoclingFallbackChunkCount"
        ],
        "forceHybridPageOcrTimingPageRangeDoclingFallbackPlan": force_artifact_summary[
            "pageRangeDoclingFallbackPlan"
        ],
        "forceHybridPageOcrTimingPageRangeDoclingFallbackChunkSummary": (
            force_artifact_summary["pageRangeDoclingFallbackChunkSummary"]
        ),
        "fullDoclingFallbackCount": force_artifact_summary["fullDoclingFallbackCount"],
        "forceHybridPageOcrTimingSchedulerTraceSummary": force_artifact_summary[
            "hybridPageOcrTimingSchedulerTraceSummary"
        ],
        "shardCacheReuseMetricsRustSchedulerElapsedMs": (
            shard_cache_reuse_artifact_summary["metricsRustSchedulerElapsedMs"]
            if shard_cache_reuse_artifact_summary
            else None
        ),
        "shardCacheReuseHybridPageOcrTimingTotalElapsedMs": (
            shard_cache_reuse_artifact_summary["hybridPageOcrTimingTotalElapsedMs"]
            if shard_cache_reuse_artifact_summary
            else None
        ),
        "shardCacheReuseHybridPageOcrTimingPhaseElapsedMs": (
            shard_cache_reuse_artifact_summary["hybridPageOcrTimingPhaseElapsedMs"]
            if shard_cache_reuse_artifact_summary
            else {}
        ),
        "shardCacheReuseHybridPageOcrTimingOcr2RegionShardCount": (
            shard_cache_reuse_artifact_summary[
                "hybridPageOcrTimingOcr2RegionShardCount"
            ]
            if shard_cache_reuse_artifact_summary
            else None
        ),
        "shardCacheReuseHybridPageOcrTimingOcr2RegionRequestCount": (
            shard_cache_reuse_artifact_summary[
                "hybridPageOcrTimingOcr2RegionRequestCount"
            ]
            if shard_cache_reuse_artifact_summary
            else None
        ),
        "shardCacheReuseHybridPageOcrTimingOcr2RegionRenderedShardCount": (
            shard_cache_reuse_artifact_summary[
                "hybridPageOcrTimingOcr2RegionRenderedShardCount"
            ]
            if shard_cache_reuse_artifact_summary
            else None
        ),
        "shardCacheReuseHybridPageOcrTimingOcr2RegionRenderCacheHitCount": (
            shard_cache_reuse_artifact_summary[
                "hybridPageOcrTimingOcr2RegionRenderCacheHitCount"
            ]
            if shard_cache_reuse_artifact_summary
            else None
        ),
        "shardCacheReuseHybridPageOcrTimingOcr2RegionRenderCacheMissCount": (
            shard_cache_reuse_artifact_summary[
                "hybridPageOcrTimingOcr2RegionRenderCacheMissCount"
            ]
            if shard_cache_reuse_artifact_summary
            else None
        ),
        "shardCacheReuseHybridPageOcrTimingSchedulerTraceSummary": (
            shard_cache_reuse_artifact_summary[
                "hybridPageOcrTimingSchedulerTraceSummary"
            ]
            if shard_cache_reuse_artifact_summary
            else {}
        ),
        "documentTimingArrowExists": artifact_summary["documentTimingArrowExists"],
        "documentTimingRows": artifact_summary["documentTimingRows"],
        "documentTimingTotalElapsedMs": artifact_summary[
            "documentTimingTotalElapsedMs"
        ],
        "documentTimingOverheadMs": document_timing_overhead_ms,
        "documentTimingPhaseElapsedMs": artifact_summary[
            "documentTimingPhaseElapsedMs"
        ],
        "hybridPageOcrFallbackReasons": artifact_summary[
            "hybridPageOcrFallbackReasons"
        ],
        "imageAttachmentAuditCount": artifact_summary["imageAttachmentAuditCount"],
        "imageKnownDimensionCount": artifact_summary["imageKnownDimensionCount"],
        "imageFormatCounts": artifact_summary["imageFormatCounts"],
        "imageDimensionSourceCounts": artifact_summary["imageDimensionSourceCounts"],
        "imageAccelerationCandidates": artifact_summary["imageAccelerationCandidates"],
        "maxImageWidthPx": artifact_summary["maxImageWidthPx"],
        "maxImageHeightPx": artifact_summary["maxImageHeightPx"],
        "maxImagePixelCount": artifact_summary["maxImagePixelCount"],
        "archiveAttachmentAuditCount": artifact_summary["archiveAttachmentAuditCount"],
        "archiveMemberCount": artifact_summary["archiveMemberCount"],
        "archiveRegularFileCount": artifact_summary["archiveRegularFileCount"],
        "archiveXmlMemberCount": artifact_summary["archiveXmlMemberCount"],
        "archiveImageMemberCount": artifact_summary["archiveImageMemberCount"],
        "archiveTotalMemberSizeBytes": artifact_summary["archiveTotalMemberSizeBytes"],
        "archiveFormatCounts": artifact_summary["archiveFormatCounts"],
        "archiveAccelerationCandidates": artifact_summary[
            "archiveAccelerationCandidates"
        ],
        "archiveExtensionCounts": artifact_summary["archiveExtensionCounts"],
        "maxArchiveLargestMemberSizeBytes": artifact_summary[
            "maxArchiveLargestMemberSizeBytes"
        ],
        "artifactErrorCount": artifact_summary["artifactErrorCount"],
        "artifactReports": cached_report.get("artifactReports", []),
        "rowsPerSecond": rows_per_second(total_rows, cached_report["wallTimeMs"]),
        "cacheSpeedup": force_refresh_ms / max(percentile(cached_latencies, 50), 0.001),
    }


def export_audio_transcript_org_for_fixture(
    args: argparse.Namespace,
    fixture_name: str,
    output_dir: Path,
) -> dict[str, Any]:
    if not getattr(args, "export_audio_transcript_org", False):
        return {
            "path": None,
            "rows": 0,
            "chars": 0,
            "timelineMarkerCount": 0,
        }
    report_dir = Path(
        getattr(args, "report_dir_path", getattr(args, "report_dir", "."))
    )
    org_path = report_dir / "audio-transcripts" / f"{fixture_name}.org"
    return export_audio_transcript_org(
        output_dir / DOCUMENT_RESOURCES_ARROW_CACHE_NAME,
        org_path,
    )


def export_audio_transcript_reference_draft_for_fixture(
    args: argparse.Namespace,
    fixture_name: str,
    output_dir: Path,
) -> dict[str, Any]:
    if not getattr(args, "export_audio_transcript_org", False):
        return {
            "jsonlPath": None,
            "tsvPath": None,
            "rows": 0,
            "chars": 0,
        }
    report_dir = Path(
        getattr(args, "report_dir_path", getattr(args, "report_dir", "."))
    )
    draft_dir = report_dir / "audio-transcripts"
    return export_audio_transcript_reference_drafts(
        output_dir / DOCUMENT_RESOURCES_ARROW_CACHE_NAME,
        draft_dir / f"{fixture_name}.reference_draft.jsonl",
        draft_dir / f"{fixture_name}.reference_draft.tsv",
    )


def hybrid_page_ocr_fallback_reasons_by_run(
    force_report: dict[str, Any],
    cached_report: dict[str, Any],
    shard_cache_reuse_report: dict[str, Any] | None,
    artifact_registry_reuse_report: dict[str, Any] | None,
) -> list[str]:
    reports_by_run = {
        "force": force_report,
        "cache": cached_report,
    }
    if shard_cache_reuse_report is not None:
        reports_by_run["shard_cache_reuse"] = shard_cache_reuse_report
    if artifact_registry_reuse_report is not None:
        reports_by_run["artifact_registry_reuse"] = artifact_registry_reuse_report
    reasons = []
    for run_name, report in reports_by_run.items():
        summary = summarize_artifact_reports(report.get("artifactReports", []))
        for reason in summary.get("hybridPageOcrFallbackReasons", []):
            reasons.append(f"{run_name}: {reason}")
    return reasons


def document_timing_overhead(
    force_refresh_ms: float,
    artifact_summary: dict[str, Any],
) -> float | None:
    timing_rows = artifact_summary.get("documentTimingRows", 0)
    if not isinstance(timing_rows, int) or timing_rows <= 0:
        return None
    timing_ms = artifact_summary.get("documentTimingTotalElapsedMs")
    if not isinstance(timing_ms, int | float):
        return None
    return max(float(force_refresh_ms) - float(timing_ms), 0.0)


def _compare_docling_groundtruth(
    args: argparse.Namespace,
    report: dict[str, Any],
) -> list[dict[str, Any]]:
    return compare_report_artifacts_to_docling_groundtruth(
        enabled=getattr(args, "compare_docling_groundtruth", False),
        groundtruth_root=getattr(args, "docling_groundtruth_root", None),
        report=report,
        min_char_coverage=getattr(
            args,
            "docling_groundtruth_min_char_coverage",
            0.98,
        ),
        min_similarity=getattr(args, "docling_groundtruth_min_similarity", 0.98),
    )


def run_cargo_perf_test(
    args: argparse.Namespace,
    source: Path,
    output_dir: Path,
    *,
    force: bool,
    iterations: int,
    concurrency: int,
    report_path: Path,
    inputs: dict[str, Path] | None = None,
    wait_ms: int | None = None,
    flight_mode: str | None = None,
    include_structure_baseline_root: bool = True,
) -> dict[str, Any]:
    output_dir.mkdir(parents=True, exist_ok=True)
    env = rust_process_env()
    effective_wait_ms = args.wait_ms if wait_ms is None else wait_ms
    effective_flight_mode = flight_mode or args.flight_mode
    env.update(
        {
            "WENDAO_DOCUMENT_EXTRACT_PERF_ENDPOINT": (
                f"http://{args.benchmark_host}:{args.benchmark_port}"
            ),
            "WENDAO_DOCUMENT_EXTRACT_PERF_SOURCE": str(source),
            "WENDAO_DOCUMENT_EXTRACT_PERF_OUTPUT_DIR": str(output_dir),
            "WENDAO_DOCUMENT_EXTRACT_PERF_ITERATIONS": str(iterations),
            "WENDAO_DOCUMENT_EXTRACT_PERF_CONCURRENCY": str(max(concurrency, 1)),
            "WENDAO_DOCUMENT_EXTRACT_PERF_FORCE_FIRST": "true" if force else "false",
            "WENDAO_DOCUMENT_EXTRACT_PERF_MODE": effective_flight_mode,
            "WENDAO_DOCUMENT_EXTRACT_PERF_WAIT_MS": str(effective_wait_ms),
            "WENDAO_DOCUMENT_EXTRACT_PERF_REPORT": str(report_path),
        }
    )
    apply_rust_pdf_ocr_env(args, env)
    if inputs is not None:
        env["WENDAO_DOCUMENT_EXTRACT_PERF_INPUTS_JSON"] = json.dumps(
            [
                {
                    "name": name,
                    "source": str(input_source),
                    "outputDir": str(output_dir / name),
                }
                for name, input_source in inputs.items()
            ]
        )
    structure_baseline_root = getattr(args, "structure_baseline_root", None)
    if include_structure_baseline_root and structure_baseline_root is not None:
        env["WENDAO_DOCUMENT_EXTRACT_PERF_STRUCTURE_BASELINE_ROOT"] = str(
            structure_baseline_root
        )
    command = [
        args.cargo,
        "test",
        "-p",
        "xiuxian-wendao",
        "--no-default-features",
        "--features",
        cargo_features_for_flight_mode(args.cargo_features, effective_flight_mode),
        "--test",
        "wendao-validation-gate",
        "document_extract_python_flight_perf_smoke",
        "--",
        "--ignored",
        "--nocapture",
    ]
    status_samples = run_command_with_status_sampling(
        command,
        env=env,
        rest_endpoint=getattr(args, "rust_rest_endpoint", None),
        sample_interval_ms=getattr(args, "rust_rest_status_sample_interval_ms", 250),
        require_status=getattr(args, "require_rust_rest_status", False),
    )
    report = json.loads(report_path.read_text(encoding="utf-8"))
    report["maxRssKb"] = max_rss_kb()
    report["rustJobsStatusSamples"] = status_samples
    report["rustJobsStatusSummary"] = summarize_rust_jobs_status_samples(status_samples)
    report_path.write_text(
        json.dumps(report, indent=2, sort_keys=True), encoding="utf-8"
    )
    return report


def read_converter_count(args: argparse.Namespace) -> int | None:
    count_path = getattr(args, "converter_count_path", None)
    if count_path is None:
        return None
    path = Path(count_path)
    if not path.exists():
        return 0
    if path.is_dir():
        return sum(
            int(child.read_text(encoding="utf-8").strip() or "0")
            for child in sorted(path.glob("*.txt"))
        )
    text = path.read_text(encoding="utf-8").strip()
    if not text:
        return 0
    return int(text)
