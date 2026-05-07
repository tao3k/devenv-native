"""CLI orchestration for Wendao document extraction benchmarks."""

from __future__ import annotations

from .args import parse_args
from .cache import benchmark_ocr_shard_cache_root, summarize_ocr_shard_cache
from .common import (
    Any,
    Path,
    json,
    os,
    sys,
    tempfile,
)
from .constants import REPORT_SCHEMA
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
from .ocr2_trace import summarize_deepseek_ocr2_request_traces
from .pdf_render import run_pdf_render_shard_audit
from .precision_speed import ocr2_promotion_gate
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
from .runtime import wait_for_port
from .workers import (
    deepseek_ocr2_process_env,
    resolve_local_python_ocr_endpoint_count,
    start_server_pool,
)


def main() -> int:
    args = parse_args()
    args.local_python_ocr_endpoint_count = resolve_local_python_ocr_endpoint_count(args)
    if args.external_endpoint and args.local_python_ocr_endpoint_count != 1:
        raise SystemExit(
            "--local-python-ocr-endpoint-count cannot start workers in --external-endpoint mode"
        )
    if args.shard_cache_reuse_probe and args.flight_mode != "hybrid-page-ocr":
        raise SystemExit(
            "--shard-cache-reuse-probe requires --flight-mode hybrid-page-ocr"
        )
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
    args.structure_baseline_root = resolve_structure_baseline_root(args, report_dir)

    if args.pdf_render_shard_audit:
        return run_pdf_render_shard_audit(
            args, report_dir / "pdf-render-shard-manifest"
        )

    with tempfile.TemporaryDirectory(
        prefix="wendao-doc-extract-perf-"
    ) as temp_root_text:
        temp_root = Path(temp_root_text)
        fixture_dir = temp_root / "fixtures"
        output_dir = temp_root / "outputs"
        process_log_dir = report_dir / "process-logs"
        args.deepseek_ocr2_request_trace_log_dir = process_log_dir
        fixture_dir.mkdir()
        output_dir.mkdir()
        args.ocr_shard_cache_root = benchmark_ocr_shard_cache_root(args, temp_root)
        fixtures, real_fixture_root = resolve_fixtures(args, fixture_dir)
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
            if (
                args.duplicate_miss_concurrency > 0
                or args.distinct_miss_concurrency > 0
            ):
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
                python_uv_package=args.python_uv_package,
                python_uv_extras=args.python_uv_extra,
                deepseek_ocr2_env=deepseek_ocr2_process_env(args),
                log_dir=process_log_dir,
            )
            if args.local_python_ocr_endpoint_count > 1:
                args.rust_document_extract_endpoint.extend(
                    worker.endpoint_url for worker in python_workers
                )
                args.rust_pdf_ocr_endpoint.extend(
                    worker.endpoint_url for worker in python_workers
                )
        try:
            for worker in python_workers:
                wait_for_port(
                    worker.host,
                    worker.port,
                    worker.process,
                    timeout_seconds=args.server_start_timeout,
                )
            if args.rust_provider_mode == "gateway" and not args.external_endpoint:
                gateway_host = args.rust_provider_host or args.host
                gateway_port = args.rust_provider_port or (args.port + 1)
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
                rust_port = args.rust_provider_port or (args.port + 1)
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
                wait_for_port(
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
            ocr_shard_cache_summary = summarize_ocr_shard_cache(
                args.ocr_shard_cache_root
            )
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
    sys.stdout.write(
        f"document extract perf report: {report_dir / 'document_extract_perf.json'}\n"
    )
    if args.fail_on_pdf_milestone_regression:
        guard = payload["summary"]["precisionSpeedSummary"]["pdfOcrMilestoneGuard"]
        if not guard["passed"]:
            reason = guard["reason"] or "; ".join(guard["regressions"])
            raise SystemExit(f"PDF OCR milestone regression guard failed: {reason}")
    return 0


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
        "localPythonOcrEndpointCount": args.local_python_ocr_endpoint_count,
        "rustPdfOcrWorkers": args.rust_pdf_ocr_workers,
        "rustPdfOcrSourceRangeWorkers": args.rust_pdf_ocr_source_range_workers,
        "rustPdfOcrProfilePlanner": getattr(args, "rust_pdf_ocr_profile_planner", None),
        "rustPdfOcr2RenderDpi": getattr(args, "rust_pdf_ocr2_render_dpi", None),
        "rustPdfOcr2RegionPlanner": getattr(args, "rust_pdf_ocr2_region_planner", None),
        "rustDocumentExtractEndpoints": args.rust_document_extract_endpoint,
        "rustPdfOcrEndpoints": args.rust_pdf_ocr_endpoint,
        "structureBaselineRoot": (
            str(args.structure_baseline_root) if args.structure_baseline_root else None
        ),
        "pdfOcrProfile": pdf_ocr_profile_label(args),
        "deepseekOcr2": {
            "backend": "vllm-openai-compatible",
            "provider": getattr(args, "deepseek_ocr2_provider", None),
            "baseUrl": getattr(args, "deepseek_ocr2_base_url", None),
            "model": getattr(args, "deepseek_ocr2_model", None),
            "openRouterModel": getattr(args, "openrouter_model", None),
            "openRouterHttpReferer": getattr(args, "openrouter_http_referer", None),
            "openRouterTitle": getattr(args, "openrouter_title", None),
            "openRouterApiKeyConfigured": _openrouter_key_configured(),
            "prompt": getattr(args, "deepseek_ocr2_prompt", None),
            "maxTokens": getattr(args, "deepseek_ocr2_max_tokens", None),
            "regionMaxTokens": getattr(args, "deepseek_ocr2_region_max_tokens", None),
            "regionCompositeSize": getattr(
                args, "deepseek_ocr2_region_composite_size", None
            ),
            "regionAtlasMode": getattr(
                args, "deepseek_ocr2_region_atlas_mode", "disabled"
            ),
            "scaffoldMode": getattr(args, "deepseek_ocr2_scaffold_mode", "disabled"),
            "timeoutSeconds": getattr(args, "deepseek_ocr2_timeout_seconds", None),
            "requestConcurrency": getattr(
                args, "deepseek_ocr2_request_concurrency", None
            ),
            "pageWindowSize": getattr(args, "deepseek_ocr2_page_window_size", None),
            "requestSummary": summarize_deepseek_ocr2_request_traces(
                getattr(args, "deepseek_ocr2_request_trace_log_dir", None)
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
    payload["ocr2PromotionGate"] = ocr2_promotion_gate(payload)
    return payload


def should_start_local_rust_provider(args) -> bool:
    return args.flight_mode in {"async", "hybrid-page-ocr"} or bool(
        args.artifact_registry_reuse_probe
    )


def _openrouter_key_configured() -> bool:
    return any(
        bool(os.environ.get(key))
        for key in (
            "WENDAO_OPENROUTER_API_KEY",
            "OPENROUTER_API_KEY",
            "OPENROUTE_API_KEY",
            "WENDAO_DEEPSEEK_OCR2_API_KEY",
        )
    )
