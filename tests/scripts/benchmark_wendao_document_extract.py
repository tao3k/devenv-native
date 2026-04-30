#!/usr/bin/env python3
"""Benchmark Wendao document extraction across Python Flight and Rust tests."""

from __future__ import annotations

import argparse
import json
import os
import platform
import resource
import signal
import socket
import subprocess
import sys
import tarfile
import tempfile
import textwrap
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any

REPORT_SCHEMA = "xiuxian_wendao.document_extract_perf.v2"
DOCLING_REPO_URL = "https://github.com/docling-project/docling.git"
DOCLING_DEFAULT_GIT_REF = "main"
DOCLING_DATA_RELATIVE_ROOT = Path("tests/data")
PDFIUM_BINARIES_RELEASE = "chromium/7543"
PDFIUM_BINARIES_BASE_URL = (
    "https://github.com/bblanchon/pdfium-binaries/releases/download"
)
DEFAULT_OCR_SHARD_CACHE_MAX_BYTES = 10 * 1024 * 1024 * 1024
OCR_SHARD_CACHE_ROOT_ENV = "WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT"

DOCLING_REAL_FIXTURE_PATHS = {
    "pdf": "tests/data/pdf/2206.01062.pdf",
    "docx": "tests/data/docx/word_sample.docx",
    "xlsx": "tests/data/xlsx/xlsx_01.xlsx",
    "pptx": "tests/data/pptx/powerpoint_sample.pptx",
    "markdown": "tests/data/md/wiki.md",
    "asciidoc": "tests/data/asciidoc/test_01.asciidoc",
    "html": "tests/data/html/wiki_duck.html",
    "csv": "tests/data/csv/csv-comma.csv",
    "image-png": "tests/data/2305.03393v1-pg9-img.png",
    "image-tiff": "tests/data/tiff/2206.01062.tif",
    "image-webp": "tests/data/webp/webp-test.webp",
    "uspto-xml": "tests/data/uspto/ipa20110039701.xml",
    "jats-xml": "tests/data/jats/elife-56337.xml",
    "xbrl-xml": "tests/data/xbrl/mlac-20251231.xml",
    "mets-gbs": "tests/data/mets_gbs/32044009881525_select.tar.gz",
    "docling-json": "tests/data/groundtruth/docling_v2/2206.01062.json",
    "webvtt": "tests/data/webvtt/webvtt_example_01.vtt",
    "latex": "tests/data/latex/example_01.tex",
    "audio": "tests/data/audio/sample_10s.mp3",
}

DOCLING_REAL_PDF_CORPUS_FIXTURE_PATHS = {
    "pdf-2203-paper": "tests/data/pdf/2203.01017v2.pdf",
    "pdf-2305-paper-page9": "tests/data/pdf/2305.03393v1-pg9.pdf",
    "pdf-2305-paper": "tests/data/pdf/2305.03393v1.pdf",
    "pdf-amt-handbook": "tests/data/pdf/amt_handbook_sample.pdf",
    "pdf-code-formula": "tests/data/pdf/code_and_formula.pdf",
    "pdf-multi-page": "tests/data/pdf/multi_page.pdf",
    "pdf-normal-4pages": "tests/data/pdf/normal_4pages.pdf",
    "pdf-picture-classification": "tests/data/pdf/picture_classification.pdf",
    "pdf-redp5110-sampled": "tests/data/pdf/redp5110_sampled.pdf",
    "pdf-rtl-01": "tests/data/pdf/right_to_left_01.pdf",
    "pdf-rtl-02": "tests/data/pdf/right_to_left_02.pdf",
    "pdf-rtl-03": "tests/data/pdf/right_to_left_03.pdf",
    "pdf-skipped-1page": "tests/data/pdf/skipped_1page.pdf",
    "pdf-skipped-2pages": "tests/data/pdf/skipped_2pages.pdf",
    "pdf-latex-llncsdoc": "tests/data/latex/2305.03393/llncsdoc.pdf",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Benchmark Python document extraction Flight service through Rust cargo tests.",
    )
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=50051)
    parser.add_argument("--iterations", type=int, default=3)
    parser.add_argument("--concurrency", type=int, default=1)
    parser.add_argument("--server-start-timeout", type=float, default=30.0)
    parser.add_argument(
        "--external-endpoint",
        action="store_true",
        help="Do not start local workers; benchmark an already-running endpoint.",
    )
    parser.add_argument(
        "--flight-mode",
        choices=("sync", "async", "hybrid-page-ocr"),
        default="sync",
        help="Document extraction Flight mode header sent by the Rust probe.",
    )
    parser.add_argument(
        "--pdf-ocr-worker",
        choices=("skip", "fixture", "docling"),
        default="skip",
        help=(
            "OCR worker started by the local Python service for "
            "/analysis/pdf-ocr-shards. `fixture` is deterministic test OCR; "
            "`docling` requires --real-docling."
        ),
    )
    parser.add_argument(
        "--pdf-ocr-workers",
        default="auto",
        help=(
            "Local Python Docling OCR worker budget when Rust does not send "
            "x-wendao-pdf-ocr-workers. Use `auto` for CPU-adaptive sizing."
        ),
    )
    parser.add_argument(
        "--python-uv-package",
        default="xiuxian-wendao-analyzer",
        help=(
            "Workspace package used when the benchmark starts its local Python "
            "document worker through `uv run --package`."
        ),
    )
    parser.add_argument(
        "--python-uv-extra",
        action="append",
        default=[],
        metavar="EXTRA",
        help=(
            "Optional uv extra passed to the local Python worker. Use "
            "`--python-uv-extra documents` for real Docling OCR and "
            "`--python-uv-extra documents-audio` for audio ASR runs."
        ),
    )
    parser.add_argument(
        "--rust-pdf-ocr-workers",
        help=(
            "Optional Rust provider override for WENDAO_DOCUMENT_EXTRACT_PDF_OCR_WORKERS. "
            "When omitted, Rust sizes OCR worker budgets from available parallelism."
        ),
    )
    parser.add_argument(
        "--ocr-shard-cache-root",
        type=Path,
        help=(
            "Optional OCR shard cache root for local Rust provider/gateway runs. "
            "When omitted, local benchmark runs use an isolated temporary cache "
            f"unless {OCR_SHARD_CACHE_ROOT_ENV} is already set."
        ),
    )
    parser.add_argument(
        "--wait-ms",
        type=int,
        default=0,
        help="Async wait budget header in milliseconds.",
    )
    parser.add_argument(
        "--rust-provider-host",
        help="Host for the local Rust provider endpoint in async non-external mode.",
    )
    parser.add_argument(
        "--rust-provider-port",
        type=int,
        help="Port for the local Rust provider endpoint in async non-external mode.",
    )
    parser.add_argument(
        "--rust-provider-mode",
        choices=("flight", "gateway"),
        default="flight",
        help=(
            "Local Rust service to start in non-external mode. `flight` starts "
            "the focused Flight provider; `gateway` starts the real Wendao HTTP "
            "gateway and enables REST status sampling by default."
        ),
    )
    parser.add_argument(
        "--rust-provider-features",
        default="studio,zhenfa-router,duckdb,builtin-plugins",
        help="Cargo feature set used to start the local Rust provider.",
    )
    parser.add_argument(
        "--gateway-features",
        default="studio,zhenfa-router,duckdb,builtin-plugins",
        help="Cargo feature set used to start the local Wendao gateway.",
    )
    parser.add_argument(
        "--gateway-valkey-port",
        type=int,
        help=(
            "Port for the temporary Valkey instance used by "
            "--rust-provider-mode gateway. Defaults to an available local port."
        ),
    )
    parser.add_argument(
        "--rust-rest-endpoint",
        default=os.environ.get("WENDAO_DOCUMENT_EXTRACT_REST_ENDPOINT"),
        help=(
            "Optional Rust gateway REST base URL used to sample "
            "/api/document-extract-jobs during benchmark probes."
        ),
    )
    parser.add_argument(
        "--rust-rest-status-sample-interval-ms",
        type=int,
        default=250,
        help="Sampling interval for --rust-rest-endpoint during cargo probes.",
    )
    parser.add_argument(
        "--require-rust-rest-status",
        action="store_true",
        help="Fail if --rust-rest-endpoint cannot be sampled.",
    )
    parser.add_argument(
        "--cargo-features",
        default="performance,studio,zhenfa-router,duckdb",
        help="Cargo feature set used by the Rust benchmark probe.",
    )
    parser.add_argument(
        "--duplicate-miss-concurrency",
        type=int,
        default=0,
        help="Run one cold async cache-miss burst at this concurrency before cache-hit timing.",
    )
    parser.add_argument(
        "--distinct-miss-concurrency",
        type=int,
        default=0,
        help=(
            "Run one cold async cache-miss burst across this many different "
            "documents and record suite-level queue/capacity metrics."
        ),
    )
    parser.add_argument(
        "--distinct-miss-wait-ms",
        type=int,
        help=(
            "Async wait budget for --distinct-miss-concurrency. Defaults to "
            "max(--wait-ms, 60000) so cold conversions can finish locally."
        ),
    )
    parser.add_argument(
        "--converter-count-path",
        type=Path,
        help=(
            "Optional converter count file to read in external-endpoint benchmark mode."
        ),
    )
    parser.add_argument(
        "--fail-on-duplicate-conversions",
        action="store_true",
        help="Fail when fake duplicate-miss benchmarking observes more than one conversion.",
    )
    parser.add_argument(
        "--shard-cache-reuse-probe",
        action="store_true",
        help=(
            "After the force run, run a second forced hybrid-page-ocr extraction "
            "into a fresh output directory to measure OCR shard cache reuse "
            "without relying on the whole-document _resources.arrow cache."
        ),
    )
    parser.add_argument(
        "--fail-on-distinct-miss-conversions",
        action="store_true",
        help=(
            "Fail when counted distinct cold-miss benchmarking does not "
            "observe one conversion per distinct document."
        ),
    )
    parser.add_argument(
        "--report-dir",
        default=".run/reports/xiuxian-wendao/document-extract-perf",
    )
    parser.add_argument(
        "--pdf-render-shard-audit",
        action="store_true",
        help=(
            "Run the feature-gated Rust PDF render shard manifest audit "
            "against selected PDF fixtures and exit without starting extraction workers."
        ),
    )
    parser.add_argument(
        "--pdf-render-selection",
        choices=("all-pages", "shard-fallback-pages", "region-shards"),
        default="all-pages",
        help=(
            "Page selection mode for --pdf-render-shard-audit. "
            "`all-pages` proves renderer capacity; `shard-fallback-pages` "
            "uses the current high-recall raster fallback; `region-shards` "
            "renders explicit PDF-point regions supplied by "
            "--pdf-render-region."
        ),
    )
    parser.add_argument(
        "--pdf-render-region",
        action="append",
        default=[],
        metavar="NAME=PAGE,REGION,LEFT,BOTTOM,RIGHT,TOP[,ORDER]",
        help=(
            "Explicit region fixture for --pdf-render-selection region-shards "
            "or --hybrid-pdf-render-selection region-shards. NAME must match "
            "the selected fixture alias; coordinates are PDF points in the "
            "source page coordinate space. May be passed more than once."
        ),
    )
    parser.add_argument(
        "--hybrid-pdf-render-selection",
        choices=("all-pages", "shard-fallback-pages", "region-shards"),
        default="shard-fallback-pages",
        help=(
            "Page selection mode used by the live hybrid-page-ocr provider "
            "during benchmark runs. Defaults to shard-fallback-pages so live "
            "benchmarks keep routing behavior unless OCR worker proof explicitly "
            "forces all pages. `region-shards` uses explicit regions from "
            "--pdf-render-region."
        ),
    )
    parser.add_argument(
        "--pdfium-library-path",
        type=Path,
        help=("Path to a libpdfium shared library used by --pdf-render-shard-audit."),
    )
    parser.add_argument(
        "--prepare-pdfium-runtime",
        action="store_true",
        help=(
            "Download the pinned pdfium-binaries runtime for the current platform "
            "into the project cache before --pdf-render-shard-audit."
        ),
    )
    parser.add_argument(
        "--require-pdfium",
        action="store_true",
        help=("Fail --pdf-render-shard-audit when no PDF pages are actually rendered."),
    )
    parser.add_argument("--cargo", default="cargo")
    parser.add_argument("--real-docling", action="store_true")
    parser.add_argument(
        "--fixture-suite",
        choices=("fake", "docling-real"),
        default="fake",
    )
    parser.add_argument(
        "--docling-source-root",
        type=Path,
        help=(
            "Docling fixture checkout root used for docling-real fixtures. "
            "Defaults to $PRJ_DATA_HOME/docling-real-fixtures or "
            ".data/docling-real-fixtures."
        ),
    )
    parser.add_argument(
        "--prepare-docling-fixtures",
        action="store_true",
        help="Sparse clone or refresh Docling tests/data fixtures under the fixture root.",
    )
    parser.add_argument(
        "--prepare-only",
        action="store_true",
        help="Prepare Docling real fixtures and exit without running the benchmark.",
    )
    parser.add_argument(
        "--docling-repo-url",
        default=DOCLING_REPO_URL,
        help="Git repository URL used by --prepare-docling-fixtures.",
    )
    parser.add_argument(
        "--docling-git-ref",
        default=DOCLING_DEFAULT_GIT_REF,
        help="Git ref used by --prepare-docling-fixtures.",
    )
    parser.add_argument(
        "--skip-audio",
        action="store_true",
        help="Skip the real Docling ASR fixture in docling-real mode.",
    )
    parser.add_argument(
        "--include-docling-pdf-corpus",
        action="store_true",
        help=(
            "Add extra Docling tests/data PDF corpus fixtures to docling-real "
            "benchmark selection. This is opt-in so the default real suite stays small."
        ),
    )
    parser.add_argument(
        "--only-fixture",
        action="append",
        default=[],
        help="Run only the named fixture. May be passed more than once.",
    )
    parser.add_argument(
        "--extra-fixture",
        action="append",
        default=[],
        metavar="NAME=PATH",
        help=(
            "Add an explicit fixture file by alias. May be passed more than once; "
            "useful for opt-in real PDFs under the project data directory."
        ),
    )
    parser.add_argument(
        "--fail-on-error-rows",
        action="store_true",
        help="Fail when the Rust perf report receives any document extraction error rows.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
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
        print(
            f"prepared {len(fixtures)} Docling real fixtures under {real_fixture_root}"
        )
        return 0

    report_dir = Path(args.report_dir)
    report_dir.mkdir(parents=True, exist_ok=True)

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
        server = None
        rust_server = None
        valkey_server = None
        ocr_shard_cache_summary = None
        if not args.external_endpoint:
            converter_count_path = None
            if (
                args.duplicate_miss_concurrency > 0
                or args.distinct_miss_concurrency > 0
            ):
                converter_count_path = temp_root / "converter-count.txt"
                converter_count_path.write_text("0", encoding="utf-8")
                args.converter_count_path = converter_count_path
            server = start_server(
                args.host,
                args.port,
                real_docling=args.real_docling,
                real_fixture_root=real_fixture_root,
                include_audio=not args.skip_audio,
                converter_count_path=converter_count_path,
                pdf_ocr_worker=args.pdf_ocr_worker,
                pdf_ocr_workers=args.pdf_ocr_workers,
                python_uv_package=args.python_uv_package,
                python_uv_extras=args.python_uv_extra,
                log_dir=process_log_dir,
            )
        try:
            if server is not None:
                wait_for_port(
                    args.host,
                    args.port,
                    server,
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
            elif (
                args.flight_mode in {"async", "hybrid-page-ocr"}
                and not args.external_endpoint
            ):
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
            terminate_server(server)

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
        "rustPdfOcrWorkers": args.rust_pdf_ocr_workers,
        "pdfOcrProfile": pdf_ocr_profile_label(args),
        "shardCacheReuseProbe": args.shard_cache_reuse_probe,
        "ocrShardCache": ocr_shard_cache_summary
        or summarize_ocr_shard_cache(args.ocr_shard_cache_root),
        "distinctMiss": distinct_miss_report,
        "doclingFixtureRoot": str(real_fixture_root) if real_fixture_root else None,
        "results": results,
        "summary": summarize_results(results, distinct_miss_report),
    }
    (report_dir / "document_extract_perf.json").write_text(
        json.dumps(payload, indent=2, sort_keys=True),
        encoding="utf-8",
    )
    (report_dir / "document_extract_perf.md").write_text(
        render_markdown(payload),
        encoding="utf-8",
    )
    print(f"document extract perf report: {report_dir / 'document_extract_perf.json'}")
    return 0


def resolve_fixtures(
    args: argparse.Namespace,
    fixture_dir: Path,
) -> tuple[dict[str, Path], Path | None]:
    if args.fixture_suite == "fake":
        if args.real_docling:
            raise SystemExit(
                "--real-docling requires --fixture-suite docling-real and "
                "--docling-source-root so benchmark inputs are valid documents"
            )
        return (
            merge_extra_fixtures(
                write_fake_fixtures(fixture_dir),
                getattr(args, "extra_fixture", []),
            ),
            None,
        )

    if not args.real_docling:
        raise SystemExit("--fixture-suite docling-real requires --real-docling")
    real_fixture_root = resolve_docling_source_root(args.docling_source_root)
    if args.prepare_docling_fixtures:
        prepare_docling_fixtures(
            real_fixture_root,
            repo_url=args.docling_repo_url,
            git_ref=args.docling_git_ref,
        )
    require_docling_source_root(real_fixture_root)
    return (
        merge_extra_fixtures(
            docling_real_fixtures(
                real_fixture_root,
                include_audio=not args.skip_audio,
                include_pdf_corpus=args.include_docling_pdf_corpus,
            ),
            getattr(args, "extra_fixture", []),
        ),
        real_fixture_root,
    )


def run_pdf_render_shard_audit(args: argparse.Namespace, report_dir: Path) -> int:
    with tempfile.TemporaryDirectory(
        prefix="wendao-pdf-render-shard-audit-"
    ) as temp_root_text:
        fixture_dir = Path(temp_root_text) / "fixtures"
        fixture_dir.mkdir()
        fixtures, _real_fixture_root = resolve_fixtures(args, fixture_dir)
        fixtures = select_fixtures(fixtures, args.only_fixture)
        if not args.only_fixture:
            fixtures = {
                name: path
                for name, path in fixtures.items()
                if path.suffix.lower() == ".pdf"
            }
        if not fixtures:
            raise SystemExit(
                "PDF render shard audit requires at least one selected PDF fixture"
            )
        command, env_update = build_pdf_render_shard_audit_command(
            args,
            fixtures,
            report_dir.resolve(),
        )
        env = rust_process_env()
        env.update(env_update)
        subprocess.run(command, check=True, env=env)
    print(
        "PDF render shard reports: "
        f"{report_dir / 'pdf_page_render_shard_manifest.json'}, "
        f"{report_dir / 'pdf_page_render_shard_manifest.md'}"
    )
    return 0


def build_pdf_render_shard_audit_command(
    args: argparse.Namespace,
    fixtures: dict[str, Path],
    report_dir: Path,
) -> tuple[list[str], dict[str, str]]:
    inputs = [
        {
            "name": name,
            "source": str(path),
        }
        for name, path in fixtures.items()
    ]
    command = [
        args.cargo,
        "test",
        "-p",
        "xiuxian-wendao",
        "--test",
        "xiuxian-testing-gate",
        "--features",
        cargo_features_with_pdf_render(args.cargo_features),
        "pdf_render_page_render_shard_manifest",
        "--",
        "--ignored",
        "--nocapture",
    ]
    env = {
        "WENDAO_PDF_RENDER_SHARD_INPUTS_JSON": json.dumps(inputs),
        "WENDAO_PDF_RENDER_SHARD_REPORT_DIR": str(report_dir),
        "WENDAO_PDF_RENDER_SELECTION": normalize_render_selection(
            args.pdf_render_selection
        ),
    }
    env.update(build_pdf_render_region_env(args, fixtures))
    pdfium_library_path = resolve_pdfium_library_path(args)
    if pdfium_library_path is not None:
        env["WENDAO_PDFIUM_LIBRARY_PATH"] = str(pdfium_library_path)
    if getattr(args, "require_pdfium", False):
        env["WENDAO_PDF_RENDER_REQUIRE_PDFIUM"] = "1"
    return command, env


def build_pdf_render_region_env(
    args: argparse.Namespace,
    fixtures: dict[str, Path],
) -> dict[str, str]:
    region_specs = getattr(args, "pdf_render_region", [])
    selection = normalize_render_selection(args.pdf_render_selection)
    if selection != "region_shards":
        if region_specs:
            raise SystemExit(
                "--pdf-render-region requires --pdf-render-selection region-shards"
            )
        return {}
    return {
        "WENDAO_PDF_RENDER_REGIONS_JSON": json.dumps(
            parse_pdf_render_regions(region_specs, fixtures)
        )
    }


def build_hybrid_pdf_render_region_env(args: argparse.Namespace) -> dict[str, str]:
    selection = normalize_render_selection(
        getattr(args, "hybrid_pdf_render_selection", "shard-fallback-pages")
    )
    region_specs = getattr(args, "pdf_render_region", [])
    if selection != "region_shards":
        return {}
    fixtures = getattr(args, "benchmark_fixtures", {})
    if not fixtures:
        raise SystemExit(
            "--hybrid-pdf-render-selection region-shards requires selected fixtures"
        )
    return {
        "WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_JSON": json.dumps(
            parse_pdf_render_regions(region_specs, fixtures)
        )
    }


def parse_pdf_render_regions(
    region_specs: list[str],
    fixtures: dict[str, Path],
) -> list[dict[str, Any]]:
    if not region_specs:
        raise SystemExit(
            "--pdf-render-selection region-shards requires at least one "
            "--pdf-render-region"
        )
    regions_by_fixture: dict[str, list[dict[str, Any]]] = {
        name: [] for name in fixtures
    }
    seen_regions: set[tuple[str, int, int]] = set()
    for region_spec in region_specs:
        fixture_name, region = parse_pdf_render_region(region_spec)
        if fixture_name not in fixtures:
            available = ", ".join(sorted(fixtures))
            raise SystemExit(
                f"Unknown --pdf-render-region fixture alias: {fixture_name}\n"
                f"Available fixtures: {available}"
            )
        region_key = (
            fixture_name,
            int(region["pageIndex"]),
            int(region["regionIndex"]),
        )
        if region_key in seen_regions:
            raise SystemExit(
                "Duplicate --pdf-render-region page/region for fixture: "
                f"{fixture_name} page={region_key[1]} region={region_key[2]}"
            )
        seen_regions.add(region_key)
        regions_by_fixture[fixture_name].append(region)

    missing = sorted(
        fixture_name
        for fixture_name, regions in regions_by_fixture.items()
        if not regions
    )
    if missing:
        raise SystemExit(
            "Missing --pdf-render-region for selected fixture(s): " + ", ".join(missing)
        )
    return [
        {
            "source": str(fixtures[fixture_name]),
            "regions": regions_by_fixture[fixture_name],
        }
        for fixture_name in fixtures
    ]


def parse_pdf_render_region(region_spec: str) -> tuple[str, dict[str, Any]]:
    if "=" not in region_spec:
        raise SystemExit(
            "--pdf-render-region must use "
            "NAME=PAGE,REGION,LEFT,BOTTOM,RIGHT,TOP[,ORDER] syntax: " + region_spec
        )
    fixture_name, raw_region = region_spec.split("=", maxsplit=1)
    fixture_name = fixture_name.strip()
    if not fixture_name:
        raise SystemExit("--pdf-render-region fixture alias must not be empty")
    parts = [part.strip() for part in raw_region.split(",")]
    if len(parts) not in {6, 7}:
        raise SystemExit(
            "--pdf-render-region requires 6 or 7 comma-separated values after "
            f"NAME=: {region_spec}"
        )
    try:
        page_index = int(parts[0])
        region_index = int(parts[1])
        left = float(parts[2])
        bottom = float(parts[3])
        right = float(parts[4])
        top = float(parts[5])
    except ValueError as error:
        raise SystemExit(
            f"Invalid --pdf-render-region numeric value: {region_spec}"
        ) from error
    if page_index < 0 or region_index < 0:
        raise SystemExit(
            "--pdf-render-region page and region indexes must be non-negative: "
            + region_spec
        )
    if right <= left or top <= bottom:
        raise SystemExit(
            "--pdf-render-region bbox must satisfy right > left and top > bottom: "
            + region_spec
        )
    region: dict[str, Any] = {
        "pageIndex": page_index,
        "regionIndex": region_index,
        "regionBox": {
            "left": left,
            "bottom": bottom,
            "right": right,
            "top": top,
        },
    }
    if len(parts) == 7 and parts[6]:
        region["readingOrderKey"] = parts[6]
    return fixture_name, region


def resolve_pdfium_library_path(args: argparse.Namespace) -> Path | None:
    explicit_path = getattr(args, "pdfium_library_path", None)
    if explicit_path is not None:
        return validate_pdfium_library_path(explicit_path)
    if getattr(args, "prepare_pdfium_runtime", False):
        return prepare_pdfium_runtime()
    return None


def validate_pdfium_library_path(path: Path) -> Path:
    resolved = path.resolve()
    if not resolved.is_file():
        raise SystemExit(f"PDFium library path does not exist: {resolved}")
    return resolved


def prepare_pdfium_runtime() -> Path:
    asset_name = pdfium_asset_name()
    expected_library_name = pdfium_library_filename()
    cache_root = resolve_project_cache_home()
    release_dir = (
        cache_root
        / "wendao-document-extract"
        / "pdfium"
        / ("chromium-" + PDFIUM_BINARIES_RELEASE.split("/", maxsplit=1)[1])
    )
    target_dir = release_dir / asset_name.removesuffix(".tgz")
    existing_library = find_pdfium_library(target_dir, expected_library_name)
    if existing_library is not None:
        return existing_library

    target_dir.mkdir(parents=True, exist_ok=True)
    archive_path = release_dir / asset_name
    if not archive_path.is_file():
        download_pdfium_archive(asset_name, archive_path)
    safe_extract_tgz(archive_path, target_dir)
    library_path = find_pdfium_library(target_dir, expected_library_name)
    if library_path is None:
        raise SystemExit(
            "Downloaded PDFium archive did not contain "
            f"{expected_library_name}: {archive_path}"
        )
    return library_path


def resolve_project_cache_home() -> Path:
    cache_home = Path(os.environ.get("PRJ_CACHE_HOME", ".cache"))
    return cache_home.resolve()


def benchmark_ocr_shard_cache_root(args: argparse.Namespace, temp_root: Path) -> Path:
    explicit_root = getattr(args, "ocr_shard_cache_root", None)
    if explicit_root is not None:
        return explicit_root.resolve()
    configured = os.environ.get(OCR_SHARD_CACHE_ROOT_ENV)
    if configured:
        return Path(configured).resolve()
    if getattr(args, "external_endpoint", False):
        return resolve_ocr_shard_cache_root()
    return (temp_root / "ocr-shard-cache").resolve()


def resolve_ocr_shard_cache_root() -> Path:
    configured = os.environ.get(OCR_SHARD_CACHE_ROOT_ENV)
    if configured:
        return Path(configured).resolve()
    return resolve_project_cache_home() / "wendao-document-extract" / "ocr-shards"


def summarize_ocr_shard_cache(root: Path | None = None) -> dict[str, Any]:
    root = root.resolve() if root is not None else resolve_ocr_shard_cache_root()
    file_count = 0
    total_bytes = 0
    if root.exists():
        for path in root.rglob("*.arrow"):
            if not path.is_file():
                continue
            file_count += 1
            total_bytes += path.stat().st_size
    return {
        "root": str(root),
        "fileCount": file_count,
        "totalBytes": total_bytes,
        "maxBytes": optional_positive_int_env(
            "WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_BYTES"
        )
        or DEFAULT_OCR_SHARD_CACHE_MAX_BYTES,
        "maxEntries": optional_positive_int_env(
            "WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_ENTRIES"
        ),
        "maxAgeSecs": optional_positive_int_env(
            "WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_AGE_SECS"
        ),
    }


def optional_positive_int_env(key: str) -> int | None:
    value = os.environ.get(key)
    if value is None:
        return None
    try:
        parsed = int(value)
    except ValueError:
        return None
    return parsed if parsed > 0 else None


def pdfium_asset_name(
    *,
    sys_platform: str | None = None,
    machine: str | None = None,
) -> str:
    sys_platform = sys_platform or sys.platform
    machine = normalize_machine(machine or platform.machine())
    if sys_platform == "darwin":
        if machine in {"arm64", "aarch64"}:
            return "pdfium-mac-arm64.tgz"
        if machine in {"x86_64", "amd64"}:
            return "pdfium-mac-x64.tgz"
    if sys_platform.startswith("linux"):
        if machine in {"arm64", "aarch64"}:
            return "pdfium-linux-arm64.tgz"
        if machine in {"x86_64", "amd64"}:
            return "pdfium-linux-x64.tgz"
    if sys_platform.startswith("win"):
        if machine in {"arm64", "aarch64"}:
            return "pdfium-win-arm64.tgz"
        if machine in {"x86_64", "amd64"}:
            return "pdfium-win-x64.tgz"
        if machine in {"x86", "i386", "i686"}:
            return "pdfium-win-x86.tgz"
    raise SystemExit(
        "No pinned PDFium binary is configured for "
        f"platform={sys_platform} machine={machine}"
    )


def normalize_machine(machine: str) -> str:
    return machine.strip().lower().replace("-", "_")


def pdfium_library_filename(*, sys_platform: str | None = None) -> str:
    sys_platform = sys_platform or sys.platform
    if sys_platform == "darwin":
        return "libpdfium.dylib"
    if sys_platform.startswith("win"):
        return "pdfium.dll"
    return "libpdfium.so"


def download_pdfium_archive(asset_name: str, archive_path: Path) -> None:
    release = PDFIUM_BINARIES_RELEASE.replace("/", "%2F")
    url = f"{PDFIUM_BINARIES_BASE_URL}/{release}/{asset_name}"
    archive_path.parent.mkdir(parents=True, exist_ok=True)
    temporary_path = archive_path.with_suffix(archive_path.suffix + ".download")
    with (
        urllib.request.urlopen(url, timeout=60.0) as response,
        temporary_path.open("wb") as output,
    ):
        while True:
            chunk = response.read(1024 * 1024)
            if not chunk:
                break
            output.write(chunk)
    temporary_path.replace(archive_path)


def safe_extract_tgz(archive_path: Path, target_dir: Path) -> None:
    root = target_dir.resolve()
    with tarfile.open(archive_path, "r:gz") as archive:
        members = archive.getmembers()
        for member in members:
            member_target = (root / member.name).resolve()
            if root != member_target and root not in member_target.parents:
                raise SystemExit(
                    f"PDFium archive member escapes target directory: {member.name}"
                )
        try:
            archive.extractall(root, members=members, filter="data")
        except TypeError:
            archive.extractall(root, members=members)


def find_pdfium_library(root: Path, library_name: str) -> Path | None:
    if not root.exists():
        return None
    preferred = root / "lib" / library_name
    if preferred.is_file():
        return preferred.resolve()
    matches = sorted(path for path in root.rglob(library_name) if path.is_file())
    if not matches:
        return None
    return matches[0].resolve()


def cargo_features_with_pdf_render(features: str) -> str:
    return cargo_features_with_pdf_feature(features, "document-extract-pdf-render")


def cargo_features_with_pdf_source_range(features: str) -> str:
    return cargo_features_with_pdf_feature(
        features, "document-extract-pdf-source-range"
    )


def cargo_features_for_flight_mode(features: str, flight_mode: str) -> str:
    if flight_mode == "hybrid-page-ocr":
        return cargo_features_with_pdf_source_range(features)
    return features


def cargo_features_for_provider_mode(features: str, args: argparse.Namespace) -> str:
    flight_mode = getattr(args, "flight_mode", "sync")
    if flight_mode != "hybrid-page-ocr":
        return features
    selection = normalize_render_selection(
        getattr(args, "hybrid_pdf_render_selection", "shard-fallback-pages")
    )
    if selection == "region_shards":
        return cargo_features_with_pdf_render(features)
    return cargo_features_with_pdf_source_range(features)


def cargo_features_with_pdf_feature(features: str, feature: str) -> str:
    parts = [
        part.strip()
        for chunk in features.split(",")
        for part in chunk.split()
        if part.strip()
    ]
    if feature not in parts:
        parts.append(feature)
    if "performance" not in parts:
        parts.insert(0, "performance")
    return ",".join(parts)


def select_fixtures(
    fixtures: dict[str, Path],
    fixture_names: list[str],
) -> dict[str, Path]:
    if not fixture_names:
        return fixtures

    missing = sorted(set(fixture_names).difference(fixtures))
    if missing:
        available = ", ".join(sorted(fixtures))
        raise SystemExit(
            "Unknown fixture(s): "
            + ", ".join(missing)
            + f"\nAvailable fixtures: {available}"
        )
    return {fixture_name: fixtures[fixture_name] for fixture_name in fixture_names}


def merge_extra_fixtures(
    fixtures: dict[str, Path],
    fixture_specs: list[str],
) -> dict[str, Path]:
    extra_fixtures = parse_extra_fixtures(fixture_specs)
    collisions = sorted(set(fixtures).intersection(extra_fixtures))
    if collisions:
        raise SystemExit(
            "Extra fixture alias collides with existing fixture(s): "
            + ", ".join(collisions)
        )
    return {**fixtures, **extra_fixtures}


def parse_extra_fixtures(fixture_specs: list[str]) -> dict[str, Path]:
    fixtures: dict[str, Path] = {}
    for fixture_spec in fixture_specs:
        fixture_name, fixture_path = parse_extra_fixture(fixture_spec)
        if fixture_name in fixtures:
            raise SystemExit(f"Duplicate extra fixture alias: {fixture_name}")
        fixtures[fixture_name] = fixture_path
    return fixtures


def parse_extra_fixture(fixture_spec: str) -> tuple[str, Path]:
    if "=" not in fixture_spec:
        raise SystemExit("--extra-fixture must use NAME=PATH syntax: " + fixture_spec)
    fixture_name, raw_path = fixture_spec.split("=", maxsplit=1)
    fixture_name = fixture_name.strip()
    raw_path = raw_path.strip()
    if not fixture_name:
        raise SystemExit("--extra-fixture alias must not be empty")
    if not raw_path:
        raise SystemExit(f"--extra-fixture path must not be empty: {fixture_name}")
    fixture_path = Path(raw_path).expanduser().resolve()
    if not fixture_path.is_file():
        raise SystemExit(f"Extra fixture path does not exist: {fixture_path}")
    return fixture_name, fixture_path


def resolve_docling_source_root(source_root: Path | None) -> Path:
    if source_root is not None:
        return source_root.resolve()
    data_home = Path(os.environ.get("PRJ_DATA_HOME", ".data"))
    return (data_home / "docling-real-fixtures").resolve()


def require_docling_source_root(root: Path) -> None:
    if not (root / DOCLING_DATA_RELATIVE_ROOT).exists():
        raise SystemExit(
            "Docling real fixture root does not contain tests/data: "
            f"{root}\nRun with --prepare-docling-fixtures to sparse clone "
            "Docling's real test attachments into the data directory."
        )


def prepare_docling_fixtures(root: Path, *, repo_url: str, git_ref: str) -> None:
    root.parent.mkdir(parents=True, exist_ok=True)
    if (root / ".git").exists():
        subprocess.run(
            ["git", "-C", str(root), "fetch", "--depth", "1", "origin", git_ref],
            check=True,
        )
        subprocess.run(["git", "-C", str(root), "checkout", "FETCH_HEAD"], check=True)
    else:
        subprocess.run(
            [
                "git",
                "clone",
                "--depth",
                "1",
                "--filter=blob:none",
                "--sparse",
                repo_url,
                str(root),
            ],
            check=True,
        )
        if git_ref != DOCLING_DEFAULT_GIT_REF:
            subprocess.run(
                ["git", "-C", str(root), "fetch", "--depth", "1", "origin", git_ref],
                check=True,
            )
            subprocess.run(
                ["git", "-C", str(root), "checkout", "FETCH_HEAD"], check=True
            )
    subprocess.run(
        [
            "git",
            "-C",
            str(root),
            "sparse-checkout",
            "set",
            "--skip-checks",
            str(DOCLING_DATA_RELATIVE_ROOT),
        ],
        check=True,
    )


def docling_real_fixtures(
    root: Path,
    *,
    include_audio: bool,
    include_pdf_corpus: bool = False,
) -> dict[str, Path]:
    selected_paths = dict(DOCLING_REAL_FIXTURE_PATHS)
    if include_pdf_corpus:
        selected_paths.update(DOCLING_REAL_PDF_CORPUS_FIXTURE_PATHS)
    if not include_audio:
        selected_paths.pop("audio", None)

    fixtures = {
        fixture_name: root / relative_path
        for fixture_name, relative_path in selected_paths.items()
    }
    missing = [
        f"{fixture_name}: {fixture_path}"
        for fixture_name, fixture_path in fixtures.items()
        if not fixture_path.exists()
    ]
    if missing:
        raise SystemExit("Missing Docling real fixtures:\n" + "\n".join(missing))
    return fixtures


def start_server(
    host: str,
    port: int,
    *,
    real_docling: bool,
    real_fixture_root: Path | None,
    include_audio: bool,
    converter_count_path: Path | None,
    pdf_ocr_worker: str = "skip",
    pdf_ocr_workers: str = "auto",
    python_uv_package: str | None = "xiuxian-wendao-analyzer",
    python_uv_extras: list[str] | None = None,
    log_dir: Path | None = None,
) -> subprocess.Popen[str]:
    if pdf_ocr_worker == "docling" and not real_docling:
        raise SystemExit("--pdf-ocr-worker docling requires --real-docling")
    if real_docling:
        command = python_worker_command(
            real_docling_server_code(
                host,
                port,
                real_fixture_root,
                include_audio,
                converter_count_path,
                pdf_ocr_worker,
                pdf_ocr_workers,
            ),
            uv_package=python_uv_package,
            uv_extras=python_uv_extras,
        )
    else:
        command = python_worker_command(
            fixture_server_code(
                host,
                port,
                converter_count_path,
                pdf_ocr_worker,
                pdf_ocr_workers,
            ),
            uv_package=python_uv_package,
            uv_extras=python_uv_extras,
        )
    effective_log_dir = log_dir or (
        Path(os.environ.get("PRJ_RUNTIME_DIR", ".run"))
        / "document-extract-perf-process-logs"
    )
    return start_logged_process(
        command, log_dir=effective_log_dir, name="python-worker"
    )


def python_worker_command(
    code: str,
    *,
    uv_package: str | None,
    uv_extras: list[str] | None,
) -> list[str]:
    command = ["uv", "run"]
    if uv_package:
        command.extend(["--package", uv_package])
    for extra in uv_extras or []:
        command.extend(["--extra", extra])
    command.extend(["python", "-c", code])
    return command


def start_rust_provider_server(
    args: argparse.Namespace,
    *,
    rust_host: str,
    rust_port: int,
    python_host: str,
    python_port: int,
    temp_root: Path,
    log_dir: Path | None = None,
) -> subprocess.Popen[str]:
    provider_root = temp_root / "rust-provider"
    provider_root.mkdir(parents=True, exist_ok=True)
    env = rust_process_env()
    pdfium_library_path = resolve_pdfium_library_path(args)
    ocr_shard_cache_root = getattr(
        args,
        "ocr_shard_cache_root",
        (temp_root / "ocr-shard-cache").resolve(),
    )
    env.update(
        {
            "WENDAO_DOCUMENT_EXTRACT_ENDPOINT": f"http://{python_host}:{python_port}",
            "WENDAO_DOCUMENT_EXTRACT_JOB_DB": str(provider_root / "jobs.duckdb"),
            "WENDAO_DOCUMENT_EXTRACT_ARTIFACT_ROOT": str(provider_root / "artifacts"),
            OCR_SHARD_CACHE_ROOT_ENV: str(ocr_shard_cache_root),
            "WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_SELECTION": normalize_render_selection(
                getattr(args, "hybrid_pdf_render_selection", "shard-fallback-pages")
            ),
        }
    )
    env.update(build_hybrid_pdf_render_region_env(args))
    if pdfium_library_path is not None:
        env["WENDAO_PDFIUM_LIBRARY_PATH"] = str(pdfium_library_path)
    rust_pdf_ocr_workers = getattr(args, "rust_pdf_ocr_workers", None)
    if rust_pdf_ocr_workers:
        env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_WORKERS"] = str(rust_pdf_ocr_workers)
    command = [
        args.cargo,
        "run",
        "-p",
        "xiuxian-wendao",
        "--no-default-features",
        "--features",
        cargo_features_for_provider_mode(args.rust_provider_features, args),
        "--bin",
        "wendao_search_flight_server",
        "--",
        f"{rust_host}:{rust_port}",
        "alpha/repo",
        str(resolve_project_root()),
        "--schema-version=v2",
    ]
    return start_logged_process(
        command,
        log_dir=log_dir or temp_root / "process-logs",
        name="rust-provider",
        env=env,
    )


def start_valkey_server(
    *,
    host: str,
    port: int,
    temp_root: Path,
    log_dir: Path | None = None,
) -> subprocess.Popen[str]:
    valkey_root = temp_root / "valkey"
    valkey_root.mkdir(parents=True, exist_ok=True)
    command = [
        "valkey-server",
        "--bind",
        host,
        "--port",
        str(port),
        "--dir",
        str(valkey_root),
        "--save",
        "",
        "--appendonly",
        "no",
        "--daemonize",
        "no",
        "--protected-mode",
        "no",
    ]
    return start_logged_process(
        command, log_dir=log_dir or temp_root / "process-logs", name="valkey"
    )


def start_gateway_server(
    args: argparse.Namespace,
    *,
    gateway_port: int,
    python_host: str,
    python_port: int,
    valkey_url: str,
    temp_root: Path,
    log_dir: Path | None = None,
) -> subprocess.Popen[str]:
    gateway_root = temp_root / "gateway"
    gateway_root.mkdir(parents=True, exist_ok=True)
    config_path = write_gateway_benchmark_config(gateway_root, valkey_url=valkey_url)
    env = rust_process_env()
    pdfium_library_path = resolve_pdfium_library_path(args)
    ocr_shard_cache_root = getattr(
        args,
        "ocr_shard_cache_root",
        (temp_root / "ocr-shard-cache").resolve(),
    )
    env.update(
        {
            "WENDAO_DOCUMENT_EXTRACT_ENDPOINT": f"http://{python_host}:{python_port}",
            "WENDAO_DOCUMENT_EXTRACT_JOB_DB": str(gateway_root / "jobs.duckdb"),
            "WENDAO_DOCUMENT_EXTRACT_ARTIFACT_ROOT": str(gateway_root / "artifacts"),
            OCR_SHARD_CACHE_ROOT_ENV: str(ocr_shard_cache_root),
            "WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_SELECTION": normalize_render_selection(
                getattr(args, "hybrid_pdf_render_selection", "shard-fallback-pages")
            ),
            "VALKEY_URL": valkey_url,
            "REDIS_URL": valkey_url,
            "XIUXIAN_WENDAO_SEARCH_PLANE_VALKEY_URL": valkey_url,
            "XIUXIAN_WENDAO_KNOWLEDGE_VALKEY_URL": valkey_url,
            "XIUXIAN_WENDAO_GATEWAY_BOOTSTRAP_BACKGROUND_INDEXING": "false",
        }
    )
    env.update(build_hybrid_pdf_render_region_env(args))
    if pdfium_library_path is not None:
        env["WENDAO_PDFIUM_LIBRARY_PATH"] = str(pdfium_library_path)
    rust_pdf_ocr_workers = getattr(args, "rust_pdf_ocr_workers", None)
    if rust_pdf_ocr_workers:
        env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_WORKERS"] = str(rust_pdf_ocr_workers)
    command = [
        args.cargo,
        "run",
        "-p",
        "xiuxian-wendao",
        "--no-default-features",
        "--features",
        cargo_features_for_provider_mode(args.gateway_features, args),
        "--bin",
        "wendao",
        "--",
        "--conf",
        str(config_path),
        "--root",
        str(resolve_project_root()),
        "gateway",
        "start",
        "--port",
        str(gateway_port),
    ]
    return start_logged_process(
        command,
        log_dir=log_dir or temp_root / "process-logs",
        name="gateway",
        env=env,
    )


def start_logged_process(
    command: list[str],
    *,
    log_dir: Path,
    name: str,
    env: dict[str, str] | None = None,
) -> subprocess.Popen[str]:
    log_dir.mkdir(parents=True, exist_ok=True)
    stdout_path = log_dir / f"{name}.stdout.log"
    stderr_path = log_dir / f"{name}.stderr.log"
    stdout_file = stdout_path.open("w", encoding="utf-8")
    stderr_file = stderr_path.open("w", encoding="utf-8")
    try:
        process = subprocess.Popen(
            command,
            stdout=stdout_file,
            stderr=stderr_file,
            text=True,
            env=env,
            start_new_session=True,
        )
    finally:
        stdout_file.close()
        stderr_file.close()
    process.wendao_stdout_log = stdout_path
    process.wendao_stderr_log = stderr_path
    return process


def normalize_render_selection(selection: str) -> str:
    return selection.strip().replace("-", "_")


def write_gateway_benchmark_config(config_root: Path, *, valkey_url: str) -> Path:
    config_path = config_root / "wendao.toml"
    quoted_valkey_url = json.dumps(valkey_url)
    config_path.write_text(
        textwrap.dedent(
            f"""
            [gateway]
            bind = "127.0.0.1"
            webhook_enabled = false

            [gateway.runtime]
            studio_request_timeout_secs = 300

            [search.cache]
            valkey_url = {quoted_valkey_url}

            [link_graph.cache]
            valkey_url = {quoted_valkey_url}
            key_prefix = "xiuxian_wendao:document_extract_perf"
            """
        ).lstrip(),
        encoding="utf-8",
    )
    return config_path


def terminate_server(server: subprocess.Popen[str] | None) -> None:
    if server is None:
        return
    if server.poll() is not None:
        return
    terminate_process_group(server, signal.SIGTERM)
    try:
        server.wait(timeout=10)
    except subprocess.TimeoutExpired:
        terminate_process_group(server, signal.SIGKILL)
        server.wait(timeout=10)


def terminate_process_group(server: subprocess.Popen[str], sig: signal.Signals) -> None:
    try:
        os.killpg(server.pid, sig)
    except ProcessLookupError:
        pass
    except OSError:
        if sig == signal.SIGTERM:
            server.terminate()
        else:
            server.kill()


def normalize_rest_endpoint(endpoint: str | None) -> str | None:
    if endpoint is None:
        return None
    endpoint = endpoint.strip().rstrip("/")
    return endpoint or None


def pick_free_port(host: str) -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        listener.bind((host, 0))
        return int(listener.getsockname()[1])


def wait_for_http_endpoint(
    url: str,
    server: subprocess.Popen[str],
    *,
    timeout_seconds: float,
) -> None:
    deadline = time.monotonic() + timeout_seconds
    while time.monotonic() < deadline:
        if server.poll() is not None:
            stderr = server.stderr.read() if server.stderr is not None else ""
            raise RuntimeError(
                f"server exited before HTTP endpoint was ready:\n{stderr}"
            )
        try:
            with urllib.request.urlopen(url, timeout=1.0) as response:
                if 200 <= response.status < 500:
                    return
        except (OSError, TimeoutError, urllib.error.URLError):
            time.sleep(0.2)
    raise TimeoutError(f"HTTP endpoint did not become ready: {url}")


def fetch_rust_jobs_status(
    endpoint: str | None,
    *,
    require_status: bool,
) -> dict[str, Any] | None:
    endpoint = normalize_rest_endpoint(endpoint)
    if endpoint is None:
        return None
    url = f"{endpoint}/api/document-extract-jobs"
    try:
        with urllib.request.urlopen(url, timeout=1.0) as response:
            payload = json.loads(response.read().decode("utf-8"))
    except (
        OSError,
        TimeoutError,
        urllib.error.URLError,
        json.JSONDecodeError,
    ) as error:
        if require_status:
            raise RuntimeError(
                f"failed to sample Rust document extract jobs status: {error}"
            ) from error
        return None
    payload["sampledAtMs"] = int(time.time() * 1000)
    return payload


def run_command_with_status_sampling(
    command: list[str],
    *,
    env: dict[str, str],
    rest_endpoint: str | None,
    sample_interval_ms: int,
    require_status: bool,
) -> list[dict[str, Any]]:
    endpoint = normalize_rest_endpoint(rest_endpoint)
    if endpoint is None:
        subprocess.run(command, check=True, env=env)
        return []

    samples: list[dict[str, Any]] = []
    before = fetch_rust_jobs_status(endpoint, require_status=require_status)
    if before is not None:
        samples.append(before)

    process = subprocess.Popen(command, env=env)
    interval = max(sample_interval_ms, 25) / 1000
    while process.poll() is None:
        sample = fetch_rust_jobs_status(endpoint, require_status=require_status)
        if sample is not None:
            samples.append(sample)
        time.sleep(interval)

    after = fetch_rust_jobs_status(endpoint, require_status=require_status)
    if after is not None:
        samples.append(after)

    if process.returncode != 0:
        raise subprocess.CalledProcessError(process.returncode, command)
    return samples


def resolve_project_root() -> Path:
    return Path(os.environ.get("PRJ_ROOT", Path.cwd())).resolve()


def rust_process_env() -> dict[str, str]:
    env = dict(os.environ)
    if sys.platform == "darwin" and ("SDKROOT" not in env or "LIBRARY_PATH" not in env):
        try:
            sdk_path = subprocess.run(
                ["xcrun", "--show-sdk-path"],
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()
        except (OSError, subprocess.CalledProcessError):
            sdk_path = ""
        if sdk_path:
            env.setdefault("SDKROOT", sdk_path)
            env.setdefault("LIBRARY_PATH", str(Path(sdk_path) / "usr/lib"))
    return env


def real_docling_server_code(
    host: str,
    port: int,
    fixture_root: Path | None,
    include_audio: bool,
    converter_count_path: Path | None,
    pdf_ocr_worker: str = "skip",
    pdf_ocr_workers: str = "auto",
) -> str:
    fixture_root_text = str(fixture_root) if fixture_root is not None else ""
    count_path_text = (
        str(converter_count_path) if converter_count_path is not None else ""
    )
    return textwrap.dedent(
        f"""
        from pathlib import Path
        from threading import Lock

        from docling.datamodel.backend_options import XBRLBackendOptions
        from docling.datamodel.base_models import InputFormat
        from docling.document_converter import DocumentConverter, XBRLFormatOption
        from xiuxian_wendao_analyzer.document_service import DocumentExtractFlightServer
        from xiuxian_wendao_analyzer.pdf_ocr import DoclingPdfOcrShardWorker

        fixture_root = Path({fixture_root_text!r}) if {bool(fixture_root_text)!r} else None
        CONVERTER_COUNT_PATH = Path({count_path_text!r}) if {bool(count_path_text)!r} else None
        if CONVERTER_COUNT_PATH is not None:
            CONVERTER_COUNT_PATH.parent.mkdir(parents=True, exist_ok=True)
            CONVERTER_COUNT_PATH.write_text("0", encoding="utf-8")

        class CountingConverter:
            def __init__(self, inner):
                self.inner = inner
                self.calls = 0
                self.lock = Lock()

            def convert(self, source, **kwargs):
                with self.lock:
                    self.calls += 1
                    if CONVERTER_COUNT_PATH is not None:
                        CONVERTER_COUNT_PATH.write_text(str(self.calls), encoding="utf-8")
                return self.inner.convert(source, **kwargs)

        format_options = {{}}
        if fixture_root is not None:
            taxonomy = fixture_root / "tests" / "data" / "xbrl" / "mlac-taxonomy"
            if taxonomy.exists():
                format_options[InputFormat.XML_XBRL] = XBRLFormatOption(
                    backend_options=XBRLBackendOptions(
                        enable_local_fetch=True,
                        taxonomy=taxonomy,
                    )
                )

        if {include_audio!r}:
            import os
            import shutil
            import tempfile

            try:
                import imageio_ffmpeg

                ffmpeg_path = Path(imageio_ffmpeg.get_ffmpeg_exe())
                ffmpeg_bin_dir = Path(tempfile.mkdtemp(prefix="wendao-docling-ffmpeg-"))
                ffmpeg_link = ffmpeg_bin_dir / "ffmpeg"
                try:
                    ffmpeg_link.symlink_to(ffmpeg_path)
                except OSError:
                    shutil.copy2(ffmpeg_path, ffmpeg_link)
                    ffmpeg_link.chmod(0o755)
                os.environ["PATH"] = (
                    str(ffmpeg_bin_dir)
                    + os.pathsep
                    + os.environ.get("PATH", "")
                )
            except ImportError:
                pass

            from docling.datamodel import asr_model_specs
            from docling.datamodel.pipeline_options import AsrPipelineOptions
            from docling.document_converter import AudioFormatOption
            from docling.pipeline.asr_pipeline import AsrPipeline

            audio_options = AsrPipelineOptions()
            audio_options.asr_options = asr_model_specs.WHISPER_TINY
            format_options[InputFormat.AUDIO] = AudioFormatOption(
                pipeline_cls=AsrPipeline,
                pipeline_options=audio_options,
            )

        def make_converter():
            converter = DocumentConverter(format_options=format_options)
            if CONVERTER_COUNT_PATH is not None:
                return CountingConverter(converter)
            return converter

        converter = make_converter()
        ocr_worker = None
        if {pdf_ocr_worker!r} == "docling":
            ocr_worker = DoclingPdfOcrShardWorker(
                converter_factory=make_converter,
                max_workers={pdf_ocr_workers!r},
            )
        server = DocumentExtractFlightServer(
            "grpc://{host}:{port}",
            converter=converter,
            ocr_worker=ocr_worker,
        )
        server.serve()
        """
    )


def fixture_server_code(
    host: str,
    port: int,
    converter_count_path: Path | None,
    pdf_ocr_worker: str = "skip",
    pdf_ocr_workers: str = "auto",
) -> str:
    count_path_text = (
        str(converter_count_path) if converter_count_path is not None else ""
    )
    return textwrap.dedent(
        f"""
        from pathlib import Path
        from threading import Lock
        import time
        from xiuxian_wendao_analyzer.document_service import DocumentExtractFlightServer
        from xiuxian_wendao_analyzer.pdf_ocr import succeeded_pdf_ocr_shard_result

        CONVERTER_COUNT_PATH = Path({count_path_text!r}) if {bool(count_path_text)!r} else None
        if CONVERTER_COUNT_PATH is not None:
            CONVERTER_COUNT_PATH.parent.mkdir(parents=True, exist_ok=True)
            CONVERTER_COUNT_PATH.write_text("0", encoding="utf-8")

        class Element:
            def __init__(self, text, self_ref, page_no=1):
                self.text = text
                self.self_ref = self_ref
                self.page_no = page_no

        class Document:
            def __init__(self, source):
                name = Path(source).name
                self.tables = [Element("| k | v |\\n| - | - |\\n| file | " + name + " |", "#/tables/0", 1)]
                self.pictures = [Element("fixture image " + name, "#/pictures/0", 1)]
                self.audio_segments = [Element("fixture transcript " + name, "#/audio/0", 1)]
                self.subtitles = [Element("00:00.000 --> 00:01.000\\n" + name, "#/cues/0", 1)]
            def export_to_markdown(self):
                return "# Fixture\\n\\nParsed by fake Docling converter.\\n"
            def export_to_dict(self):
                return {{"schema_name": "DoclingDocument", "fixture": True}}

        class Result:
            def __init__(self, source):
                self.document = Document(source)

        class Converter:
            def __init__(self):
                self.calls = 0
                self.lock = Lock()
            def convert(self, source, **kwargs):
                _ = kwargs
                with self.lock:
                    self.calls += 1
                    if CONVERTER_COUNT_PATH is not None:
                        CONVERTER_COUNT_PATH.write_text(str(self.calls), encoding="utf-8")
                time.sleep(0.025)
                return Result(source)

        class FixtureOcrWorker:
            def recognize(self, inputs, *, max_workers=None):
                _ = max_workers
                return [
                    succeeded_pdf_ocr_shard_result(
                        input_row,
                        "fixture OCR page " + str(input_row["pageIndex"]),
                        0.99,
                    )
                    for input_row in inputs
                ]

        ocr_worker = FixtureOcrWorker() if {pdf_ocr_worker!r} == "fixture" else None
        server = DocumentExtractFlightServer(
            "grpc://{host}:{port}",
            converter=Converter(),
            ocr_worker=ocr_worker,
        )
        server.serve()
        """
    )


def wait_for_port(
    host: str,
    port: int,
    server: subprocess.Popen[str],
    *,
    timeout_seconds: float,
) -> None:
    deadline = time.monotonic() + timeout_seconds
    while time.monotonic() < deadline:
        if server.poll() is not None:
            raise RuntimeError(
                "document extract service exited before listening:\n"
                + process_log_tail(server)
            )
        try:
            with socket.create_connection((host, port), timeout=1):
                return
        except OSError:
            time.sleep(0.2)
    raise TimeoutError(
        f"document extract service did not listen on {host}:{port} "
        f"within {timeout_seconds:.1f}s\n{process_log_tail(server)}"
    )


def process_log_tail(server: subprocess.Popen[str]) -> str:
    stderr_log = getattr(server, "wendao_stderr_log", None)
    stdout_log = getattr(server, "wendao_stdout_log", None)
    parts = []
    if stderr_log is not None:
        parts.append(f"stderr log: {stderr_log}\n{tail_file(Path(stderr_log))}")
    elif server.stderr is not None:
        parts.append(server.stderr.read())
    if stdout_log is not None:
        parts.append(f"stdout log: {stdout_log}\n{tail_file(Path(stdout_log))}")
    elif server.stdout is not None:
        parts.append(server.stdout.read())
    return "\n".join(part for part in parts if part).strip()


def tail_file(path: Path, limit: int = 4000) -> str:
    if not path.exists():
        return ""
    text = path.read_text(encoding="utf-8", errors="replace")
    return text[-limit:]


def write_fake_fixtures(fixture_dir: Path) -> dict[str, Path]:
    fixtures = {
        "small-md": ("sample.md", b"# Sample\n\nHello\n"),
        "docx-like": ("sample.docx", b"docx fixture"),
        "image": ("scan.png", b"\x89PNG\r\n\x1a\n"),
        "audio": ("lecture.mp3", b"ID3 fixture"),
    }
    paths = {}
    for name, (filename, content) in fixtures.items():
        path = fixture_dir / filename
        path.write_bytes(content)
        paths[name] = path
    return paths


def write_distinct_fake_fixtures(fixture_dir: Path, count: int) -> dict[str, Path]:
    fixture_dir.mkdir(parents=True, exist_ok=True)
    templates = [
        ("markdown", ".md", b"# Distinct fixture\n\n"),
        ("docx", ".docx", b"distinct docx-like fixture\n"),
        ("image", ".png", b"\x89PNG\r\n\x1a\ndistinct image fixture\n"),
        ("audio", ".mp3", b"ID3 distinct audio fixture\n"),
        ("webvtt", ".vtt", b"WEBVTT\n\n00:00.000 --> 00:01.000\nfixture\n"),
        ("xml", ".xml", b'<?xml version="1.0"?><fixture/>'),
    ]
    paths = {}
    for index in range(count):
        kind, suffix, content = templates[index % len(templates)]
        name = f"distinct-{index + 1:02d}-{kind}"
        path = fixture_dir / f"{name}{suffix}"
        path.write_bytes(content + f"\ninstance={index + 1}\n".encode())
        paths[name] = path
    return paths


def prepare_distinct_miss_fixtures(
    args: argparse.Namespace,
    fixtures: dict[str, Path],
    fixture_dir: Path,
) -> dict[str, Path]:
    count = args.distinct_miss_concurrency
    if count <= 0:
        return {}
    if args.flight_mode != "async":
        raise SystemExit("--distinct-miss-concurrency requires --flight-mode async")
    if args.fixture_suite == "fake":
        return write_distinct_fake_fixtures(fixture_dir, count)
    if args.duplicate_miss_concurrency > 0:
        raise SystemExit(
            "--distinct-miss-concurrency and --duplicate-miss-concurrency should "
            "be run separately with real Docling fixtures so both remain true "
            "cold-miss probes"
        )
    if count > len(fixtures):
        raise SystemExit(
            f"--distinct-miss-concurrency requested {count} real fixtures, "
            f"but only {len(fixtures)} selected fixtures are available"
        )
    return dict(list(fixtures.items())[:count])


def distinct_miss_wait_ms(args: argparse.Namespace) -> int:
    if args.distinct_miss_wait_ms is not None:
        return max(args.distinct_miss_wait_ms, 0)
    return max(args.wait_ms, 60_000)


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
    cached_report = run_cargo_perf_test(
        args,
        fixture_path,
        output_dir,
        force=False,
        iterations=args.iterations,
        concurrency=args.concurrency,
        report_path=output_dir / "cache.json",
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
    cache_error_rows = cached_report.get("errorRowCount", 0)
    artifact_summary = summarize_artifact_reports(
        cached_report.get("artifactReports", [])
    )
    if args.fail_on_error_rows and (
        force_error_rows or shard_cache_reuse_error_rows or cache_error_rows
    ):
        raise SystemExit(
            f"fixture `{fixture_name}` produced document extraction error rows: "
            f"force={force_error_rows}, "
            f"shard_cache_reuse={shard_cache_reuse_error_rows}, "
            f"cache={cache_error_rows}"
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
            cached_report.get("rustJobsStatusSummary", {}),
        ]
    )
    return {
        "fixture": fixture_name,
        "source": str(fixture_path),
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
        "forceRefreshMs": force_report["latenciesMs"][0],
        "forceErrorRows": force_error_rows,
        "forceStatusCounts": force_report.get("statusCounts", {}),
        "forceMaxRssKb": force_report.get("maxRssKb"),
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
        "structureArrowExists": artifact_summary["structureArrowExists"],
        "structureRows": artifact_summary["structureRows"],
        "structureOcrPageBlocks": artifact_summary["structureOcrPageBlocks"],
        "structureOcrRegionBlocks": artifact_summary["structureOcrRegionBlocks"],
        "structureBboxBlocks": artifact_summary["structureBboxBlocks"],
        "structureReadingOrderSorted": artifact_summary["structureReadingOrderSorted"],
        "metricsArrowExists": artifact_summary["metricsArrowExists"],
        "metricsRows": artifact_summary["metricsRows"],
        "metricsResultChars": artifact_summary["metricsResultChars"],
        "metricsBboxCount": artifact_summary["metricsBboxCount"],
        "metricsRustSchedulerElapsedMs": artifact_summary[
            "metricsRustSchedulerElapsedMs"
        ],
        "artifactErrorCount": artifact_summary["artifactErrorCount"],
        "artifactReports": cached_report.get("artifactReports", []),
        "rowsPerSecond": rows_per_second(total_rows, cached_report["wallTimeMs"]),
        "cacheSpeedup": force_report["latenciesMs"][0]
        / max(percentile(cached_latencies, 50), 0.001),
    }


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
) -> dict[str, Any]:
    output_dir.mkdir(parents=True, exist_ok=True)
    env = rust_process_env()
    effective_wait_ms = args.wait_ms if wait_ms is None else wait_ms
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
            "WENDAO_DOCUMENT_EXTRACT_PERF_MODE": args.flight_mode,
            "WENDAO_DOCUMENT_EXTRACT_PERF_WAIT_MS": str(effective_wait_ms),
            "WENDAO_DOCUMENT_EXTRACT_PERF_REPORT": str(report_path),
        }
    )
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
    command = [
        args.cargo,
        "test",
        "-p",
        "xiuxian-wendao",
        "--no-default-features",
        "--features",
        cargo_features_for_flight_mode(args.cargo_features, args.flight_mode),
        "--test",
        "xiuxian-testing-gate",
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
    text = path.read_text(encoding="utf-8").strip()
    if not text:
        return 0
    return int(text)


def summarize_artifact_reports(reports: list[dict[str, Any]]) -> dict[str, Any]:
    structure_sorted_values = [
        report.get("structureReadingOrderSorted")
        for report in reports
        if report.get("structureReadingOrderSorted") is not None
    ]
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
        "artifactErrorCount": sum(
            1 for report in reports if report.get("artifactError")
        ),
    }


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


def summarize_rust_jobs_status_samples(samples: list[dict[str, Any]]) -> dict[str, Any]:
    if not samples:
        return {
            "sampleCount": 0,
            "maxQueuedJobs": None,
            "maxRunningJobs": None,
            "maxInProcessRunningConversions": None,
            "maxInProcessScheduledJobs": None,
            "minAvailableConversionPermits": None,
            "maxRunningConversions": None,
            "maxPdfOcrWorkers": None,
            "maxCurrentPdfOcrWorkerBudget": None,
            "minAvailablePdfOcrWorkerPermits": None,
            "maxInProcessPdfOcrWorkers": None,
            "maxInFlightPdfOcrShards": None,
            "maxPdfOcrCacheHits": None,
            "maxPdfOcrCacheMisses": None,
            "maxPdfOcrLiveRequests": None,
            "maxPdfOcrQueueWaitP95Ms": None,
            "maxPdfOcrLatencyP95Ms": None,
            "maxPdfOcrSourcePdfPageRangeShards": None,
            "maxPdfOcrRenderedPageShards": None,
            "maxPdfOcrRenderedRegionShards": None,
            "maxPdfOcrBudgetIncreaseEvents": None,
            "maxPdfOcrBudgetDecreaseEvents": None,
            "lastConversionDurationMs": None,
            "maxConversionDurationMs": None,
        }
    return {
        "sampleCount": len(samples),
        "maxQueuedJobs": max_int_sample(samples, "queuedJobs"),
        "maxRunningJobs": max_int_sample(samples, "runningJobs"),
        "maxInProcessRunningConversions": max_int_sample(
            samples,
            "inProcessRunningConversions",
        ),
        "maxInProcessScheduledJobs": max_int_sample(samples, "inProcessScheduledJobs"),
        "minAvailableConversionPermits": min_int_sample(
            samples,
            "availableConversionPermits",
        ),
        "maxRunningConversions": max_int_sample(samples, "maxRunningConversions"),
        "maxPdfOcrWorkers": max_int_sample(samples, "maxPdfOcrWorkers"),
        "maxCurrentPdfOcrWorkerBudget": max_int_sample(
            samples,
            "currentPdfOcrWorkerBudget",
        ),
        "minAvailablePdfOcrWorkerPermits": min_int_sample(
            samples,
            "availablePdfOcrWorkerPermits",
        ),
        "maxInProcessPdfOcrWorkers": max_int_sample(samples, "inProcessPdfOcrWorkers"),
        "maxInFlightPdfOcrShards": max_int_sample(samples, "inFlightPdfOcrShards"),
        "maxPdfOcrCacheHits": max_int_sample(samples, "pdfOcrCacheHits"),
        "maxPdfOcrCacheMisses": max_int_sample(samples, "pdfOcrCacheMisses"),
        "maxPdfOcrLiveRequests": max_int_sample(samples, "pdfOcrLiveRequests"),
        "maxPdfOcrQueueWaitP95Ms": max_int_sample(samples, "pdfOcrQueueWaitP95Ms"),
        "maxPdfOcrLatencyP95Ms": max_int_sample(samples, "pdfOcrLatencyP95Ms"),
        "maxPdfOcrSourcePdfPageRangeShards": max_int_sample(
            samples,
            "pdfOcrSourcePdfPageRangeShards",
        ),
        "maxPdfOcrRenderedPageShards": max_int_sample(
            samples,
            "pdfOcrRenderedPageShards",
        ),
        "maxPdfOcrRenderedRegionShards": max_int_sample(
            samples,
            "pdfOcrRenderedRegionShards",
        ),
        "maxPdfOcrBudgetIncreaseEvents": max_int_sample(
            samples,
            "pdfOcrBudgetIncreaseEvents",
        ),
        "maxPdfOcrBudgetDecreaseEvents": max_int_sample(
            samples,
            "pdfOcrBudgetDecreaseEvents",
        ),
        "lastConversionDurationMs": last_present_sample(
            samples,
            "lastConversionDurationMs",
        ),
        "maxConversionDurationMs": max_int_sample(samples, "maxConversionDurationMs"),
    }


def max_int_sample(samples: list[dict[str, Any]], key: str) -> int | None:
    values = [
        value for sample in samples if isinstance((value := sample.get(key)), int)
    ]
    return max(values, default=None)


def min_int_sample(samples: list[dict[str, Any]], key: str) -> int | None:
    values = [
        value for sample in samples if isinstance((value := sample.get(key)), int)
    ]
    return min(values, default=None)


def last_present_sample(samples: list[dict[str, Any]], key: str) -> Any:
    for sample in reversed(samples):
        value = sample.get(key)
        if value is not None:
            return value
    return None


def combine_rust_jobs_status_summaries(
    summaries: list[dict[str, Any]],
) -> dict[str, Any]:
    samples = [
        summary
        for summary in summaries
        if summary and summary.get("sampleCount", 0) > 0
    ]
    if not samples:
        return summarize_rust_jobs_status_samples([])
    return {
        "sampleCount": sum_int_values(samples, "sampleCount"),
        "maxQueuedJobs": max_optional_int(samples, "maxQueuedJobs"),
        "maxRunningJobs": max_optional_int(samples, "maxRunningJobs"),
        "maxInProcessRunningConversions": max_optional_int(
            samples,
            "maxInProcessRunningConversions",
        ),
        "maxInProcessScheduledJobs": max_optional_int(
            samples,
            "maxInProcessScheduledJobs",
        ),
        "minAvailableConversionPermits": min_optional_int(
            samples,
            "minAvailableConversionPermits",
        ),
        "maxRunningConversions": max_optional_int(samples, "maxRunningConversions"),
        "maxPdfOcrWorkers": max_optional_int(samples, "maxPdfOcrWorkers"),
        "maxCurrentPdfOcrWorkerBudget": max_optional_int(
            samples,
            "maxCurrentPdfOcrWorkerBudget",
        ),
        "minAvailablePdfOcrWorkerPermits": min_optional_int(
            samples,
            "minAvailablePdfOcrWorkerPermits",
        ),
        "maxInProcessPdfOcrWorkers": max_optional_int(
            samples,
            "maxInProcessPdfOcrWorkers",
        ),
        "maxInFlightPdfOcrShards": max_optional_int(samples, "maxInFlightPdfOcrShards"),
        "maxPdfOcrCacheHits": max_optional_int(samples, "maxPdfOcrCacheHits"),
        "maxPdfOcrCacheMisses": max_optional_int(samples, "maxPdfOcrCacheMisses"),
        "maxPdfOcrLiveRequests": max_optional_int(samples, "maxPdfOcrLiveRequests"),
        "maxPdfOcrQueueWaitP95Ms": max_optional_int(
            samples,
            "maxPdfOcrQueueWaitP95Ms",
        ),
        "maxPdfOcrLatencyP95Ms": max_optional_int(samples, "maxPdfOcrLatencyP95Ms"),
        "maxPdfOcrSourcePdfPageRangeShards": max_optional_int(
            samples,
            "maxPdfOcrSourcePdfPageRangeShards",
        ),
        "maxPdfOcrRenderedPageShards": max_optional_int(
            samples,
            "maxPdfOcrRenderedPageShards",
        ),
        "maxPdfOcrRenderedRegionShards": max_optional_int(
            samples,
            "maxPdfOcrRenderedRegionShards",
        ),
        "maxPdfOcrBudgetIncreaseEvents": max_optional_int(
            samples,
            "maxPdfOcrBudgetIncreaseEvents",
        ),
        "maxPdfOcrBudgetDecreaseEvents": max_optional_int(
            samples,
            "maxPdfOcrBudgetDecreaseEvents",
        ),
        "lastConversionDurationMs": last_present_sample(
            samples,
            "lastConversionDurationMs",
        ),
        "maxConversionDurationMs": max_optional_int(samples, "maxConversionDurationMs"),
    }


def sum_int_values(items: list[dict[str, Any]], key: str) -> int:
    return sum(value for item in items if isinstance((value := item.get(key)), int))


def max_optional_int(items: list[dict[str, Any]], key: str) -> int | None:
    values = [value for item in items if isinstance((value := item.get(key)), int)]
    return max(values, default=None)


def min_optional_int(items: list[dict[str, Any]], key: str) -> int | None:
    values = [value for item in items if isinstance((value := item.get(key)), int)]
    return min(values, default=None)


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
    return {
        "fixtureCount": len(results),
        "totalRows": sum(result["totalRows"] for result in results),
        "totalErrorRows": sum(
            result["forceErrorRows"]
            + result.get("shardCacheReuseErrorRows", 0)
            + result["cacheErrorRows"]
            for result in results
        )
        + distinct_error_rows,
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
        "allStructureReadingOrderSorted": all_structure_reading_order_sorted(results),
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
        "artifactErrorCount": sum(
            result.get("artifactErrorCount", 0) for result in results
        ),
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
    }


def pdf_ocr_profile_label(args: argparse.Namespace) -> str:
    if args.pdf_ocr_worker == "skip":
        return "skip"
    if args.pdf_ocr_worker == "fixture":
        return "fixture"
    if args.flight_mode != "hybrid-page-ocr":
        return "docling-full-document"
    return "source-page-range-or-parallel-image"


def all_structure_reading_order_sorted(results: list[dict[str, Any]]) -> bool | None:
    values = [
        result.get("structureReadingOrderSorted")
        for result in results
        if result.get("structureReadingOrderSorted") is not None
    ]
    return all(bool(value) for value in values) if values else None


def format_optional_float(value: Any) -> str:
    if isinstance(value, (int, float)):
        return f"{float(value):.3f}"
    return ""


def render_markdown(payload: dict[str, Any]) -> str:
    rust_status = payload["summary"]["rustJobsStatusSummary"]
    ocr_shard_cache = payload.get("ocrShardCache", {})
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
        f"- Rust PDF OCR worker pool: `{payload['rustPdfOcrWorkers']}`",
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
        "- Metrics sidecar: "
        f"`rows={payload['summary'].get('totalMetricsRows')}, "
        f"chars={payload['summary'].get('totalMetricsResultChars')}, "
        f"bbox={payload['summary'].get('totalMetricsBboxCount')}, "
        "rustSchedulerElapsedMs="
        f"{format_optional_float(payload['summary'].get('totalMetricsRustSchedulerElapsedMs'))}`",
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


if __name__ == "__main__":
    sys.exit(main())
