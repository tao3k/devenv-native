"""CLI orchestration for Wendao document extraction benchmarks."""

from __future__ import annotations

import shutil

from xiuxian_wendao_analyzer.docling_groundtruth import resolve_docling_groundtruth_root

from .args import parse_args
from .audio_trace import summarize_hosted_audio_request_traces
from .cache import benchmark_ocr_shard_cache_root, summarize_ocr_shard_cache
from .common import (
    Any,
    Path,
    json,
    os,
    sys,
    tempfile,
)
from .constants import OPENROUTER_API_KEY_ENVS, REPORT_SCHEMA
from .fake_fixtures import prepare_distinct_miss_fixtures
from .fixtures import (
    docling_real_fixtures,
    prepare_docling_fixtures,
    require_docling_source_root,
    resolve_docling_source_root,
    resolve_fixtures,
    select_fixtures,
)
from .http_status import normalize_rest_endpoint, pick_free_port, wait_for_http_endpoint
from .ocr2_trace import summarize_hosted_vlm_ocr_request_traces
from .pdf_render import run_pdf_render_shard_audit
from .precision_speed import candidate_taxonomy, hosted_vlm_promotion_gate
from .probes import (
    resolve_structure_baseline_root,
    run_distinct_miss_probe,
    run_fixture_probe,
    run_structure_baseline_probe,
)
from .processes import terminate_server
from .providers import (
    start_gateway_server,
    start_rust_provider_server,
    start_valkey_server,
)
from .reporting import pdf_ocr_profile_label, render_markdown, summarize_results
from .runtime import (
    wait_for_document_extract_flight_endpoint,
    wait_for_port,
    wait_for_process_stdout_contains,
)
from .workers import (
    audio_worker_process_env,
    hosted_vlm_ocr_process_env,
    resolve_document_extract_full_threads,
    resolve_document_extract_prewarm_page_ranges,
    resolve_local_python_ocr_endpoint_count,
    start_server_pool,
)


def main() -> int:
    args = parse_args()
    args.local_python_ocr_endpoint_count = resolve_local_python_ocr_endpoint_count(args)
    args.document_extract_prewarm_page_ranges_resolved = (
        resolve_document_extract_prewarm_page_ranges(args)
    )
    if args.external_endpoint and args.local_python_ocr_endpoint_count != 1:
        raise SystemExit(
            "--local-python-ocr-endpoint-count cannot start workers in --external-endpoint mode"
        )
    if args.shard_cache_reuse_probe and args.flight_mode != "hybrid-page-ocr":
        raise SystemExit("--shard-cache-reuse-probe requires --flight-mode hybrid-page-ocr")
    if args.prepare_only:
        real_fixture_root = resolve_docling_source_root(args.docling_source_root)
        prepare_docling_fixtures(
            real_fixture_root,
            repo_url=args.docling_repo_url,
            git_ref=args.docling_git_ref,
        )
        require_docling_source_root(real_fixture_root)
        fixtures = docling_real_fixtures(
            real_fixture_root,
            include_audio=not args.skip_audio,
            include_pdf_corpus=args.include_docling_pdf_corpus,
        )
        sys.stdout.write(
            f"prepared {len(fixtures)} Docling real fixtures under {real_fixture_root}\n"
        )
        return 0

    report_dir = Path(args.report_dir)
    report_dir.mkdir(parents=True, exist_ok=True)
    args.report_dir_path = report_dir
    args.structure_baseline_root = resolve_structure_baseline_root(args, report_dir)

    if args.pdf_render_shard_audit:
        return run_pdf_render_shard_audit(args, report_dir / "pdf-render-shard-manifest")

    with tempfile.TemporaryDirectory(prefix="wendao-doc-extract-perf-") as temp_root_text:
        temp_root = Path(temp_root_text)
        fixture_dir = temp_root / "fixtures"
        output_dir = temp_root / "outputs"
        process_log_dir = report_dir / "process-logs"
        reset_process_log_dir(process_log_dir)
        args.hosted_vlm_ocr_request_trace_log_dir = process_log_dir
        args.hosted_audio_request_trace_log_dir = process_log_dir
        fixture_dir.mkdir()
        output_dir.mkdir()
        args.ocr_shard_cache_root = benchmark_ocr_shard_cache_root(args, temp_root)
        fixtures, real_fixture_root = resolve_fixtures(args, fixture_dir)
        args.docling_groundtruth_root = resolve_docling_groundtruth_root(
            explicit_root=args.docling_groundtruth_root,
            compare_enabled=args.compare_docling_groundtruth,
            real_fixture_root=real_fixture_root,
        )
        fixtures = select_fixtures(fixtures, args.only_fixture)
        args.benchmark_fixtures = fixtures
        distinct_miss_fixtures = prepare_distinct_miss_fixtures(
            args,
            fixtures,
            temp_root / "distinct-fixtures",
        )

        args.benchmark_host = args.host
        args.benchmark_port = args.port
        args.converter_count_path = args.converter_count_path
        python_workers = []
        rust_server = None
        valkey_server = None
        ocr_shard_cache_summary = None
        args.rust_document_extract_endpoint = list(args.rust_document_extract_endpoint)
        args.rust_pdf_ocr_endpoint = list(args.rust_pdf_ocr_endpoint)
        if not args.external_endpoint:
            converter_count_path = None
            if args.duplicate_miss_concurrency > 0 or args.distinct_miss_concurrency > 0:
                converter_count_path = (
                    temp_root / "converter-counts"
                    if args.local_python_ocr_endpoint_count > 1
                    else temp_root / "converter-count.txt"
                )
                if args.local_python_ocr_endpoint_count == 1:
                    converter_count_path.write_text("0", encoding="utf-8")
                args.converter_count_path = converter_count_path
            python_workers = start_server_pool(
                args.host,
                args.port,
                endpoint_count=args.local_python_ocr_endpoint_count,
                real_docling=args.real_docling,
                real_fixture_root=real_fixture_root,
                include_audio=not args.skip_audio,
                converter_count_path=converter_count_path,
                pdf_ocr_worker=args.pdf_ocr_worker,
                pdf_ocr_workers=args.pdf_ocr_workers,
                audio_worker=args.audio_worker,
                audio_workers=args.audio_workers,
                python_uv_package=args.python_uv_package,
                python_uv_extras=args.python_uv_extra,
                hosted_vlm_ocr_env=hosted_vlm_ocr_process_env(args),
                audio_worker_env=audio_worker_process_env(args),
                pdf_ocr_prewarm_endpoint_count=args.pdf_ocr_prewarm_endpoint_count,
                log_dir=process_log_dir,
                allow_base_port_fallback=not getattr(args, "port_was_explicit", False),
            )
            if python_workers:
                args.port = python_workers[0].port
                args.benchmark_port = args.port
            if args.local_python_ocr_endpoint_count > 1:
                args.rust_document_extract_endpoint.extend(
                    worker.endpoint_url for worker in python_workers
                )
                args.rust_pdf_ocr_endpoint.extend(worker.endpoint_url for worker in python_workers)
        try:
            for worker in python_workers:
                wait_for_document_extract_flight_endpoint(
                    worker.host,
                    worker.port,
                    worker.process,
                    timeout_seconds=args.server_start_timeout,
                )
            if args.rust_provider_mode == "gateway" and not args.external_endpoint:
                gateway_host = args.rust_provider_host or args.host
                gateway_port = resolve_local_rust_provider_port(args)
                valkey_port = args.gateway_valkey_port or pick_free_port(args.host)
                valkey_server = start_valkey_server(
                    host=args.host,
                    port=valkey_port,
                    temp_root=temp_root,
                    log_dir=process_log_dir,
                )
                wait_for_port(
                    args.host,
                    valkey_port,
                    valkey_server,
                    timeout_seconds=args.server_start_timeout,
                )
                args.benchmark_host = gateway_host
                args.benchmark_port = gateway_port
                if normalize_rest_endpoint(args.rust_rest_endpoint) is None:
                    args.rust_rest_endpoint = f"http://{gateway_host}:{gateway_port}"
                rust_server = start_gateway_server(
                    args,
                    gateway_port=gateway_port,
                    python_host=args.host,
                    python_port=args.port,
                    valkey_url=f"redis://{args.host}:{valkey_port}/0",
                    temp_root=temp_root,
                    log_dir=process_log_dir,
                )
                wait_for_http_endpoint(
                    f"http://{gateway_host}:{gateway_port}/api/health",
                    rust_server,
                    timeout_seconds=args.server_start_timeout,
                )
            elif should_start_local_rust_provider(args) and not args.external_endpoint:
                rust_host = args.rust_provider_host or args.host
                rust_port = resolve_local_rust_provider_port(args)
                args.benchmark_host = rust_host
                args.benchmark_port = rust_port
                rust_server = start_rust_provider_server(
                    args,
                    rust_host=rust_host,
                    rust_port=rust_port,
                    python_host=args.host,
                    python_port=args.port,
                    temp_root=temp_root,
                    log_dir=process_log_dir,
                )
                wait_for_rust_provider_ready(
                    rust_host,
                    rust_port,
                    rust_server,
                    timeout_seconds=args.server_start_timeout,
                )
            structure_baseline_report = run_structure_baseline_probe(
                args,
                {**fixtures, **distinct_miss_fixtures},
                args.structure_baseline_root,
            )
            distinct_miss_report = run_distinct_miss_probe(
                args,
                distinct_miss_fixtures,
                output_dir / "distinct-miss",
            )
            results = [
                run_fixture_probe(
                    args,
                    fixture_name,
                    fixture_path,
                    output_dir / fixture_name,
                )
                for fixture_name, fixture_path in fixtures.items()
            ]
            ocr_shard_cache_summary = summarize_ocr_shard_cache(args.ocr_shard_cache_root)
        finally:
            terminate_server(rust_server)
            terminate_server(valkey_server)
            for worker in reversed(python_workers):
                terminate_server(worker.process)

    payload = build_report_payload(
        args,
        real_fixture_root=real_fixture_root,
        results=results,
        distinct_miss_report=distinct_miss_report,
        structure_baseline_report=structure_baseline_report,
        ocr_shard_cache_summary=ocr_shard_cache_summary,
    )
    (report_dir / "document_extract_perf.json").write_text(
        json.dumps(payload, indent=2, sort_keys=True),
        encoding="utf-8",
    )
    (report_dir / "document_extract_perf.md").write_text(
        render_markdown(payload),
        encoding="utf-8",
    )
    sys.stdout.write(f"document extract perf report: {report_dir / 'document_extract_perf.json'}\n")
    enforce_report_gates(args, payload)
    return 0


def enforce_report_gates(args, payload: dict[str, Any]) -> None:
    if getattr(args, "fail_on_precision_gate_failure", False):
        precision_speed = payload["summary"]["precisionSpeedSummary"]
        if precision_speed.get("precisionGatePassed") is not True:
            raise SystemExit("precision gate failed")
    if getattr(args, "fail_on_structure_parity_mismatch", False):
        summary = payload["summary"]
        checked = int(summary.get("structureParityCheckedFixtures") or 0)
        errors = int(summary.get("totalStructureParityErrors") or 0)
        passed = summary.get("allStructureParityPassed")
        if checked == 0:
            raise SystemExit("structure parity gate failed: no fixtures checked")
        if errors > 0 or passed is False:
            raise SystemExit(f"structure parity gate failed: {errors} parity error(s)")
    if args.fail_on_pdf_milestone_regression:
        guard = payload["summary"]["precisionSpeedSummary"]["pdfOcrMilestoneGuard"]
        if not guard["passed"]:
            reason = guard["reason"] or "; ".join(guard["regressions"])
            raise SystemExit(f"PDF OCR milestone regression guard failed: {reason}")


def build_report_payload(
    args,
    *,
    real_fixture_root: Path | None,
    results: list[dict[str, Any]],
    distinct_miss_report: dict[str, Any] | None,
    structure_baseline_report: dict[str, Any] | None,
    ocr_shard_cache_summary: dict[str, Any] | None,
) -> dict[str, Any]:
    summary = summarize_results(results, distinct_miss_report)
    payload = {
        "schema": REPORT_SCHEMA,
        "mode": "real-docling" if args.real_docling else "fixture",
        "endpoint": f"http://{args.benchmark_host}:{args.benchmark_port}",
        "rustRestEndpoint": normalize_rest_endpoint(args.rust_rest_endpoint),
        "iterations": args.iterations,
        "concurrency": args.concurrency,
        "flightMode": args.flight_mode,
        "waitMs": args.wait_ms,
        "pdfOcrWorker": args.pdf_ocr_worker,
        "pdfOcrWorkers": args.pdf_ocr_workers,
        "audioWorker": getattr(args, "audio_worker", "skip"),
        "audioWorkers": getattr(args, "audio_workers", "auto"),
        "pdfOcrPrewarmProfiles": list(getattr(args, "pdf_ocr_prewarm_profile", [])),
        "pdfOcrPrewarmSourcePath": getattr(args, "pdf_ocr_prewarm_source_path", None),
        "pdfOcrPrewarmPageIndex": getattr(args, "pdf_ocr_prewarm_page_index", None),
        "pdfOcrPrewarmPageIndices": getattr(args, "pdf_ocr_prewarm_page_indices", None),
        "pdfOcrPrewarmEndpointCount": getattr(args, "pdf_ocr_prewarm_endpoint_count", None),
        "documentExtractConverterCache": getattr(
            args, "document_extract_converter_cache", "disabled"
        ),
        "documentExtractFullThreads": getattr(args, "document_extract_full_threads", "auto"),
        "documentExtractFullThreadsResolved": resolve_document_extract_full_threads(args),
        "documentExtractPrewarmSourcePath": getattr(
            args, "document_extract_prewarm_source_path", None
        ),
        "documentExtractPrewarmPageRanges": getattr(
            args, "document_extract_prewarm_page_ranges", None
        ),
        "documentExtractPrewarmPageRangesResolved": getattr(
            args, "document_extract_prewarm_page_ranges_resolved", None
        ),
        "pdfOcrBackendTextPageFallback": getattr(
            args,
            "pdf_ocr_backend_text_page_fallback",
            "disabled",
        ),
        "pdfOcrBackendTextEmptyPage": getattr(
            args,
            "pdf_ocr_backend_text_empty_page",
            "disabled",
        ),
        "localPythonOcrEndpointCount": args.local_python_ocr_endpoint_count,
        "rustPdfOcrWorkers": args.rust_pdf_ocr_workers,
        "rustPdfOcrSourceRangeWorkers": args.rust_pdf_ocr_source_range_workers,
        "rustAudioBackendProfile": getattr(args, "rust_audio_backend_profile", None),
        "rustAudioChunkMs": getattr(args, "rust_audio_chunk_ms", None),
        "rustAudioContextBeforeMs": getattr(args, "rust_audio_context_before_ms", None),
        "rustAudioContextAfterMs": getattr(args, "rust_audio_context_after_ms", None),
        "rustAudioRecoverySplitMs": getattr(args, "rust_audio_recovery_split_ms", None),
        "rustAudioSampleRateHz": getattr(args, "rust_audio_sample_rate_hz", None),
        "rustAudioChannels": getattr(args, "rust_audio_channels", None),
        "rustAudioFormat": getattr(args, "rust_audio_format", None),
        "rustAudioArtifactCacheDir": (
            str(path)
            if (path := getattr(args, "rust_audio_artifact_cache_dir", None)) is not None
            else None
        ),
        "rustAudioBaseWorkers": getattr(args, "rust_audio_base_workers", None),
        "rustAudioRecoveryWorkers": getattr(args, "rust_audio_recovery_workers", None),
        "rustAudioSpeechSegmentsJsonl": (
            str(path)
            if (path := getattr(args, "rust_audio_speech_segments_jsonl", None)) is not None
            else None
        ),
        "rustAudioSpeechMergeGapMs": getattr(args, "rust_audio_speech_merge_gap_ms", None),
        "rustAudioSpeechMinWindowMs": getattr(
            args,
            "rust_audio_speech_min_window_ms",
            None,
        ),
        "rustAudioSpeechLimitChunks": getattr(
            args,
            "rust_audio_speech_limit_chunks",
            None,
        ),
        "rustPdfDoclingPageRangeChunkPlan": getattr(
            args,
            "rust_pdf_docling_page_range_chunk_plan",
            None,
        ),
        "rustPdfDoclingPageRangeProfile": getattr(
            args,
            "rust_pdf_docling_page_range_profile",
            "full",
        ),
        "rustPdfDoclingPageRangeHedgeDelayMs": getattr(
            args,
            "rust_pdf_docling_page_range_hedge_delay_ms",
            None,
        ),
        "rustPdfDoclingPageRangeStructureCostBudget": getattr(
            args,
            "rust_pdf_docling_page_range_structure_cost_budget",
            None,
        ),
        "rustPdfDoclingTextShortcutPromotion": getattr(
            args,
            "rust_pdf_docling_text_shortcut_promotion",
            "range-fill",
        ),
        "rustPdfLocalBackendText": getattr(args, "rust_pdf_local_backend_text", "disabled"),
        "rustPdfLocalBackendTextEmpty": getattr(
            args,
            "rust_pdf_local_backend_text_empty",
            "dispatch-python",
        ),
        "rustPdfLocalFastText": getattr(args, "rust_pdf_local_fast_text", "disabled"),
        "rustPdfFastTextSourceRangeSplit": getattr(
            args,
            "rust_pdf_fast_text_source_range_split",
            "disabled",
        ),
        "rustPdfFastTextEndpointAffinity": getattr(
            args,
            "rust_pdf_fast_text_endpoint_affinity",
            "disabled",
        ),
        "rustPdfOcrSchedulerLaneFairness": getattr(
            args,
            "rust_pdf_ocr_scheduler_lane_fairness",
            "disabled",
        ),
        "rustPdfBackendTextTopup": getattr(args, "rust_pdf_backend_text_topup", "profile"),
        "rustPdfFailedPageRecovery": getattr(
            args,
            "rust_pdf_failed_page_recovery",
            "disabled",
        ),
        "rustPdfOcrProfilePlanner": getattr(args, "rust_pdf_ocr_profile_planner", None),
        "rustPdfHostedVlmRenderDpi": getattr(args, "rust_pdf_hosted_vlm_render_dpi", None),
        "rustPdfHostedVlmRegionPlanner": getattr(args, "rust_pdf_hosted_vlm_region_planner", None),
        "rustPdfHostedVlmRegionTargetPixels": getattr(
            args,
            "rust_pdf_hosted_vlm_region_target_pixels",
            None,
        ),
        "rustPdfHostedVlmRegionMaxSlices": getattr(
            args,
            "rust_pdf_hosted_vlm_region_max_slices",
            None,
        ),
        "rustPdfHostedVlmRegionPipeline": getattr(
            args, "rust_pdf_hosted_vlm_region_pipeline", "disabled"
        ),
        "rustPdfHostedVlmRegionRenderAhead": getattr(
            args, "rust_pdf_hosted_vlm_region_render_ahead", None
        ),
        "rustPdfHostedVlmRegionRenderChunk": getattr(
            args, "rust_pdf_hosted_vlm_region_render_chunk", "page"
        ),
        "rustPdfRegionRenderMode": getattr(args, "rust_pdf_region_render_mode", "default"),
        "rustPdfHostedVlmRegionDispatchChunkSize": getattr(
            args,
            "rust_pdf_hosted_vlm_region_dispatch_chunk_size",
            None,
        ),
        "rustDocumentExtractEndpoints": args.rust_document_extract_endpoint,
        "rustPdfOcrEndpoints": args.rust_pdf_ocr_endpoint,
        "structureBaselineRoot": (
            str(args.structure_baseline_root) if args.structure_baseline_root else None
        ),
        "doclingGroundtruthRoot": (
            str(docling_groundtruth_root)
            if (docling_groundtruth_root := getattr(args, "docling_groundtruth_root", None))
            else None
        ),
        "compareDoclingGroundtruth": getattr(args, "compare_docling_groundtruth", False),
        "doclingGroundtruthMinCharCoverage": getattr(
            args,
            "docling_groundtruth_min_char_coverage",
            None,
        ),
        "doclingGroundtruthMinSimilarity": getattr(
            args,
            "docling_groundtruth_min_similarity",
            None,
        ),
        "pdfOcrProfile": pdf_ocr_profile_label(args),
        "pdfOcrFastTextSourceConverter": getattr(
            args,
            "pdf_ocr_fast_text_source_converter",
            "default",
        ),
        "hostedVlmOcr": {
            "backend": "vllm-openai-compatible",
            "provider": getattr(args, "hosted_vlm_ocr_provider", None),
            "baseUrl": getattr(args, "hosted_vlm_ocr_base_url", None),
            "model": getattr(args, "hosted_vlm_ocr_model", None),
            "openRouterModel": getattr(args, "openrouter_model", None),
            "openRouterHttpReferer": getattr(args, "openrouter_http_referer", None),
            "openRouterTitle": getattr(args, "openrouter_title", None),
            "openRouterProvider": _json_object_arg(
                getattr(args, "hosted_vlm_ocr_openrouter_provider_json", None),
            ),
            "openRouterApiKeyConfigured": _openrouter_key_configured(),
            "prompt": getattr(args, "hosted_vlm_ocr_prompt", None),
            "maxTokens": getattr(args, "hosted_vlm_ocr_max_tokens", None),
            "regionMaxTokens": getattr(args, "hosted_vlm_ocr_region_max_tokens", None),
            "regionPromptMode": getattr(
                args,
                "hosted_vlm_ocr_region_prompt_mode",
                "default",
            ),
            "regionCompositeSize": getattr(args, "hosted_vlm_ocr_region_composite_size", None),
            "regionCompositeMode": getattr(args, "hosted_vlm_ocr_region_composite_mode", "fixed"),
            "regionCompositeMaxSourcePixels": getattr(
                args, "hosted_vlm_ocr_region_composite_max_source_pixels", None
            ),
            "regionCompositeMaxImageBytes": getattr(
                args, "hosted_vlm_ocr_region_composite_max_image_bytes", None
            ),
            "regionAtlasMode": getattr(args, "hosted_vlm_ocr_region_atlas_mode", "disabled"),
            "scaffoldMode": getattr(args, "hosted_vlm_ocr_scaffold_mode", "disabled"),
            "imageOptimizationMode": getattr(args, "hosted_vlm_ocr_image_optimization", "disabled"),
            "timeoutSeconds": getattr(args, "hosted_vlm_ocr_timeout_seconds", None),
            "requestConcurrency": getattr(args, "hosted_vlm_ocr_request_concurrency", None),
            "speculativeRetryDelaySeconds": getattr(
                args, "hosted_vlm_ocr_speculative_retry_delay_seconds", None
            ),
            "speculativeRetryMinSourcePixels": getattr(
                args, "hosted_vlm_ocr_speculative_retry_min_source_pixels", None
            ),
            "speculativeRetryMinImageBytes": getattr(
                args, "hosted_vlm_ocr_speculative_retry_min_image_bytes", None
            ),
            "pageWindowSize": getattr(args, "hosted_vlm_ocr_page_window_size", None),
            "requestSummary": summarize_hosted_vlm_ocr_request_traces(
                getattr(args, "hosted_vlm_ocr_request_trace_log_dir", None)
            ),
        },
        "hostedAudio": {
            "backend": "openai-compatible-audio",
            "provider": getattr(args, "audio_hosted_provider", None),
            "baseUrl": getattr(args, "audio_hosted_base_url", None),
            "model": getattr(args, "audio_hosted_model", None),
            "apiKeyConfigured": bool(getattr(args, "audio_hosted_api_key", None))
            or bool(os.environ.get("OPENROUTER_API_KEY")),
            "timeoutSeconds": getattr(args, "audio_hosted_timeout_seconds", None),
            "requestConcurrency": getattr(args, "audio_hosted_request_concurrency", None),
            "requestSummary": summarize_hosted_audio_request_traces(
                getattr(args, "hosted_audio_request_trace_log_dir", None)
            ),
        },
        "shardCacheReuseProbe": args.shard_cache_reuse_probe,
        "artifactRegistryReuseProbe": args.artifact_registry_reuse_probe,
        "ocrShardCache": ocr_shard_cache_summary
        or summarize_ocr_shard_cache(args.ocr_shard_cache_root),
        "distinctMiss": distinct_miss_report,
        "structureBaseline": structure_baseline_report,
        "doclingFixtureRoot": str(real_fixture_root) if real_fixture_root else None,
        "results": results,
        "summary": summary,
        "precisionSpeedSummary": summary.get("precisionSpeedSummary"),
    }
    payload["hostedVlmPromotionGate"] = hosted_vlm_promotion_gate(payload)
    payload["candidateTaxonomy"] = candidate_taxonomy(payload)
    return payload


def reset_process_log_dir(process_log_dir: Path) -> None:
    if process_log_dir.exists():
        shutil.rmtree(process_log_dir)
    process_log_dir.mkdir(parents=True, exist_ok=True)


def should_start_local_rust_provider(args) -> bool:
    return args.flight_mode in {"async", "hybrid-page-ocr", "audio-shards"} or bool(
        args.artifact_registry_reuse_probe
    )


def resolve_local_rust_provider_port(args: object) -> int:
    explicit_port = getattr(args, "rust_provider_port", None)
    if explicit_port is not None:
        return explicit_port
    return pick_free_port(getattr(args, "host", "127.0.0.1"))


def wait_for_rust_provider_ready(
    host: str,
    port: int,
    server: Any,
    *,
    timeout_seconds: float,
) -> None:
    """Wait until the Rust Flight provider has bound and emitted its ready marker."""
    wait_for_port(
        host,
        port,
        server,
        timeout_seconds=timeout_seconds,
    )
    wait_for_process_stdout_contains(
        server,
        f"READY http://{host}:{port}",
        timeout_seconds=timeout_seconds,
    )


def _openrouter_key_configured() -> bool:
    return any(bool(os.environ.get(key)) for key in OPENROUTER_API_KEY_ENVS)


def _json_object_arg(value: str | None) -> dict[str, Any] | None:
    if value is None or not value.strip():
        return None
    parsed = json.loads(value)
    if not isinstance(parsed, dict):
        raise SystemExit("--hosted-vlm-ocr-openrouter-provider-json must be a JSON object")
    return parsed
