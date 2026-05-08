"""Command-line parsing for the benchmark harness."""

from __future__ import annotations

from .common import (
    Path,
    argparse,
    os,
)
from .constants import (
    DOCLING_DEFAULT_GIT_REF,
    DOCLING_REPO_URL,
    OCR_SHARD_CACHE_ROOT_ENV,
)


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
        "--pdf-ocr-prewarm-profile",
        action="append",
        default=[],
        choices=(
            "docling-compatible-page-ocr-v1",
            "docling-fast-text-ocr",
            "docling-backend-text-ocr-v1",
        ),
        help=(
            "Pre-initialize a local Python Docling OCR converter before the worker "
            "listens. This shifts daemon readiness cost out of force-refresh timing "
            "without changing OCR output."
        ),
    )
    parser.add_argument(
        "--pdf-ocr-prewarm-source-path",
        default=None,
        help=(
            "Optional PDF path converted once during Docling OCR worker startup "
            "for each --pdf-ocr-prewarm-profile. This triggers lazy model warmup "
            "before force-refresh timing."
        ),
    )
    parser.add_argument(
        "--pdf-ocr-prewarm-page-index",
        type=int,
        default=None,
        help=(
            "Zero-based page index used with --pdf-ocr-prewarm-source-path. "
            "Defaults to page 0 when a source path is supplied."
        ),
    )
    parser.add_argument(
        "--pdf-ocr-prewarm-page-indices",
        default=None,
        help=(
            "Comma-separated zero-based page indices used with "
            "--pdf-ocr-prewarm-source-path. This supersedes "
            "--pdf-ocr-prewarm-page-index and lets benchmark readiness warm "
            "multiple known slow Docling OCR pages before force-refresh timing."
        ),
    )
    parser.add_argument(
        "--pdf-ocr-prewarm-endpoint-count",
        type=int,
        default=None,
        help=(
            "Optional number of local Python OCR endpoints that receive the Docling "
            "OCR prewarm environment. When omitted, all local OCR endpoints keep the "
            "existing prewarm behavior."
        ),
    )
    parser.add_argument(
        "--pdf-ocr-backend-text-page-fallback",
        choices=("disabled", "compatible-page"),
        default="disabled",
        help=(
            "Opt-in Python OCR worker canary for backend-text page failures. "
            "`compatible-page` retries only failed or empty backend-text source pages "
            "through docling-compatible-page-ocr-v1 before Rust full-document fallback "
            "is allowed to run."
        ),
    )
    parser.add_argument(
        "--local-python-ocr-endpoint-count",
        default="auto",
        help=(
            "Number of local Python OCR Flight endpoints to start for Rust "
            "endpoint-pool benchmarks, including the primary document worker. "
            "Use `auto` to fan out real hybrid Docling OCR by machine profile."
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
        "--rust-pdf-ocr-source-range-workers",
        help=(
            "Optional Rust provider override for "
            "WENDAO_DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS. Use this "
            "only for source-PDF page-range benchmark profiling."
        ),
    )
    parser.add_argument(
        "--rust-pdf-local-backend-text",
        choices=("disabled", "rust-lopdf"),
        default="disabled",
        help=(
            "Opt-in Rust source-PDF text extraction for backend-text OCR shards. "
            "`rust-lopdf` forwards WENDAO_DOCUMENT_EXTRACT_PDF_LOCAL_BACKEND_TEXT "
            "and bypasses Python Docling only for docling-backend-text-ocr-v1 page shards."
        ),
    )
    parser.add_argument(
        "--rust-pdf-local-fast-text",
        choices=("disabled", "rust-lopdf"),
        default="disabled",
        help=(
            "Opt-in Rust source-PDF text extraction for fast-text OCR parent/top-up shards. "
            "`rust-lopdf` forwards WENDAO_DOCUMENT_EXTRACT_PDF_LOCAL_FAST_TEXT and remains "
            "bench-gated because it replaces Python Docling fast-text rows."
        ),
    )
    parser.add_argument(
        "--rust-pdf-fast-text-source-range-split",
        choices=("disabled", "single-page"),
        default="disabled",
        help=(
            "Opt-in scheduler canary for Docling fast-text source-PDF rows. "
            "`single-page` splits fast-text source ranges into one page per request "
            "while preserving Docling as the OCR authority."
        ),
    )
    parser.add_argument(
        "--rust-pdf-fast-text-endpoint-affinity",
        choices=("disabled", "single-page-first"),
        default="disabled",
        help=(
            "Opt-in scheduler canary for Docling fast-text source-PDF rows. "
            "`single-page-first` routes single-page fast-text chunks to the first "
            "configured OCR endpoint so benchmark prewarm can target that lane."
        ),
    )
    parser.add_argument(
        "--rust-pdf-backend-text-topup",
        choices=("profile", "disabled", "hosted-vlm"),
        default="profile",
        help=(
            "Controls extra fast-text top-up pages inside the hosted-vlm-risk-window-backend-text "
            "Rust profile planner. `profile` preserves the existing dense-page top-up heuristic; "
            "`disabled` routes non-recovery top-up pages to backend-text while region parent pages "
            "can still be protected by the region recovery flow; `hosted-vlm` routes dense "
            "top-up pages to hosted VLM full-page OCR."
        ),
    )
    parser.add_argument(
        "--rust-pdf-failed-page-recovery",
        choices=("disabled", "hosted-vlm-page"),
        default="disabled",
        help=(
            "Opt-in Rust precision-preserving failed-page recovery forwarded to "
            "WENDAO_DOCUMENT_EXTRACT_PDF_FAILED_PAGE_RECOVERY. "
            "`hosted-vlm-page` retries failed or empty non-hosted page shards through "
            "Hosted VLM/OCR before the existing full-document Docling fallback is used."
        ),
    )
    parser.add_argument(
        "--rust-pdf-ocr-profile-planner",
        choices=(
            "disabled",
            "fast-all",
            "fast-risk-window",
            "hosted-vlm-all",
            "hosted-vlm-risk-window",
            "hosted-vlm-risk-window-backend-text",
        ),
        help=(
            "Optional Rust provider override for "
            "WENDAO_DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER. Use "
            "`fast-*` or `hosted-vlm-*` modes only when profiling mixed "
            "candidate/accurate source-range OCR."
        ),
    )
    parser.add_argument(
        "--rust-pdf-hosted-vlm-render-dpi",
        type=int,
        help=(
            "Hosted VLM/OCR rendered-page DPI forwarded to "
            "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_RENDER_DPI for Rust provider "
            "page-image payload experiments. Values below the default OCR DPI "
            "are ignored by the Rust provider."
        ),
    )
    parser.add_argument(
        "--rust-pdf-ocr-region-context-ratio",
        type=float,
        help=(
            "Semantic padding ratio forwarded to "
            "WENDAO_DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO for hybrid "
            "region-shard Hosted VLM/OCR recovery. Use 0 to disable padding."
        ),
    )
    parser.add_argument(
        "--rust-pdf-hosted-vlm-region-planner",
        choices=(
            "disabled",
            "profile-risk-window",
            "profile-risk-window-slices",
            "profile-risk-window-adaptive",
        ),
        help=(
            "Optional Rust provider override for "
            "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER. "
            "`profile-risk-window` builds conservative Hosted VLM/OCR content-band "
            "regions for pages already selected by the Hosted VLM/OCR risk-window "
            "profile planner when no explicit region JSON is configured. "
            "`profile-risk-window-slices` splits that content band into "
            "top/middle/bottom regions for same-page Hosted VLM/OCR composite tests. "
            "`profile-risk-window-adaptive` chooses one, two, or three slices "
            "from the estimated region pixel area."
        ),
    )
    parser.add_argument(
        "--rust-pdf-hosted-vlm-region-pipeline",
        choices=("disabled", "render-dispatch"),
        default="disabled",
        help=(
            "Opt-in Rust Hosted VLM/OCR region pipeline forwarded to "
            "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PIPELINE. "
            "`render-dispatch` overlaps source-range OCR scheduling with "
            "region rendering and dispatches region OCR as each render chunk "
            "becomes available."
        ),
    )
    parser.add_argument(
        "--rust-pdf-hosted-vlm-region-render-ahead",
        type=int,
        help=(
            "Opt-in Rust Hosted VLM/OCR region render-ahead limit forwarded "
            "to WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_AHEAD. "
            "Values above 1 let the render-dispatch pipeline pre-render "
            "multiple page-region chunks while preserving deterministic result "
            "ordering."
        ),
    )
    parser.add_argument(
        "--rust-pdf-hosted-vlm-region-render-chunk",
        choices=("page", "all", "region", "page-area-desc", "page-max-area-desc"),
        default="page",
        help=(
            "Opt-in Rust Hosted VLM/OCR region render chunking forwarded to "
            "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_CHUNK. "
            "`page` preserves the existing page-grouped render chunks; "
            "`all` renders every recovery region in one PDF pass before dispatch; "
            "`region` renders each recovery region as an independent chunk "
            "so OpenRouter dispatch can start as soon as the first region is ready; "
            "`page-area-desc` keeps page chunks but renders pages with the "
            "largest total recovery-region area first; `page-max-area-desc` keeps "
            "page chunks but renders pages with the largest single recovery region first."
        ),
    )
    parser.add_argument(
        "--hosted-vlm-ocr-base-url",
        help=(
            "OpenAI-compatible Hosted VLM/OCR base URL forwarded to "
            "WENDAO_HOSTED_VLM_OCR_BASE_URL for local Python OCR workers."
        ),
    )
    parser.add_argument(
        "--hosted-vlm-ocr-provider",
        choices=("openai-compatible", "openrouter"),
        help=(
            "Direct Hosted VLM/OCR provider preset forwarded to "
            "WENDAO_HOSTED_VLM_OCR_PROVIDER. Use `openrouter` to call a "
            "hosted OpenRouter chat/completions endpoint instead of a local "
            "model server."
        ),
    )
    parser.add_argument(
        "--hosted-vlm-ocr-model",
        help=(
            "Hosted VLM/OCR model id forwarded to WENDAO_HOSTED_VLM_OCR_MODEL. "
            "Use the served vLLM model id or community AWQ/GPTQ artifact id."
        ),
    )
    parser.add_argument(
        "--hosted-vlm-ocr-prompt",
        help="Prompt forwarded to WENDAO_HOSTED_VLM_OCR_PROMPT.",
    )
    parser.add_argument(
        "--hosted-vlm-ocr-max-tokens",
        type=int,
        help="Max tokens forwarded to WENDAO_HOSTED_VLM_OCR_MAX_TOKENS.",
    )
    parser.add_argument(
        "--hosted-vlm-ocr-region-max-tokens",
        type=int,
        help=(
            "Region-shard max tokens forwarded to "
            "WENDAO_HOSTED_VLM_OCR_REGION_MAX_TOKENS. The analyzer clamps this "
            "by WENDAO_HOSTED_VLM_OCR_MAX_TOKENS and applies it only to Hosted VLM/OCR "
            "region rows."
        ),
    )
    parser.add_argument(
        "--hosted-vlm-ocr-region-composite-size",
        type=int,
        help=(
            "Direct Hosted VLM/OCR same-page region composite size forwarded to "
            "WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE. Values above 1 batch "
            "same-page, same-parent region images in one request and fall back "
            "to individual region requests when the response cannot be split "
            "back into rows."
        ),
    )
    parser.add_argument(
        "--hosted-vlm-ocr-region-atlas-mode",
        choices=("disabled", "same-page-json"),
        default="disabled",
        help=(
            "Opt-in direct Hosted VLM/OCR same-page region atlas mode forwarded to "
            "WENDAO_HOSTED_VLM_OCR_REGION_ATLAS_MODE. same-page-json packs "
            "same-page region crops into one labeled PNG atlas and requires "
            "JSON output keyed by exact shard markers."
        ),
    )
    parser.add_argument(
        "--hosted-vlm-ocr-scaffold-mode",
        choices=("disabled", "region-table-json"),
        default="disabled",
        help=(
            "Opt-in structural scaffold mode forwarded to both Rust and Python "
            "Hosted VLM/OCR region recovery. `region-table-json` writes Rust region "
            "scaffold sidecars and asks the Hosted VLM/OCR worker for strict JSON that "
            "is canonicalized back into Markdown."
        ),
    )
    parser.add_argument(
        "--hosted-vlm-ocr-image-optimization",
        choices=("disabled", "region-whitespace-trim"),
        default="disabled",
        help=(
            "Opt-in Hosted VLM/OCR request-image payload optimization forwarded to "
            "WENDAO_HOSTED_VLM_OCR_IMAGE_OPTIMIZATION. `region-whitespace-trim` "
            "trims near-white margins from region PNG request payloads only; "
            "it does not change OCR shard rows, DPI, or result schema."
        ),
    )
    parser.add_argument(
        "--hosted-vlm-ocr-timeout-seconds",
        type=float,
        help="Request timeout forwarded to WENDAO_HOSTED_VLM_OCR_TIMEOUT_SECONDS.",
    )
    parser.add_argument(
        "--hosted-vlm-ocr-request-concurrency",
        type=int,
        help=(
            "Direct Hosted VLM/OCR request concurrency forwarded to WENDAO_HOSTED_VLM_OCR_REQUEST_CONCURRENCY."
        ),
    )
    parser.add_argument(
        "--hosted-vlm-ocr-speculative-retry-delay-seconds",
        type=float,
        help=(
            "Opt-in Hosted VLM/OCR region hedge delay forwarded to "
            "WENDAO_HOSTED_VLM_OCR_SPECULATIVE_RETRY_DELAY_SECONDS. Values above "
            "0 start a second request for a direct region shard when the first "
            "request has not produced a valid result by this delay."
        ),
    )
    parser.add_argument(
        "--hosted-vlm-ocr-page-window-size",
        type=int,
        help=(
            "Direct Hosted VLM/OCR contiguous page-window size forwarded to "
            "WENDAO_HOSTED_VLM_OCR_PAGE_WINDOW_SIZE. Values above 1 batch "
            "adjacent page images in one request and fall back to page-level "
            "requests when the response cannot be split back into rows."
        ),
    )
    parser.add_argument(
        "--openrouter-model",
        help=(
            "OpenRouter model id forwarded to WENDAO_OPENROUTER_MODEL when "
            "WENDAO_HOSTED_VLM_OCR_MODEL is not set."
        ),
    )
    parser.add_argument(
        "--openrouter-http-referer",
        help="Optional OpenRouter HTTP-Referer attribution header.",
    )
    parser.add_argument(
        "--openrouter-title",
        help="Optional OpenRouter X-OpenRouter-Title attribution header.",
    )
    parser.add_argument(
        "--rust-pdf-ocr-endpoint",
        action="append",
        default=[],
        metavar="ENDPOINT",
        help=(
            "Optional Python OCR Flight endpoint forwarded to "
            "WENDAO_DOCUMENT_EXTRACT_PDF_OCR_ENDPOINTS. May be repeated to "
            "exercise Rust-side OCR endpoint-pool scheduling."
        ),
    )
    parser.add_argument(
        "--rust-document-extract-endpoint",
        action="append",
        default=[],
        metavar="ENDPOINT",
        help=(
            "Optional Python document extraction Flight endpoint forwarded to "
            "WENDAO_DOCUMENT_EXTRACT_ENDPOINTS. May be repeated to exercise "
            "Rust-side full-document endpoint-pool scheduling."
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
        "--structure-baseline-root",
        type=Path,
        help=(
            "Optional artifact root containing per-fixture Docling baseline "
            "_structure.arrow files for strict structure parity reporting."
        ),
    )
    parser.add_argument(
        "--generate-structure-baselines",
        action="store_true",
        help=(
            "Generate sync/full-Docling baseline artifacts before candidate "
            "probes, then reuse that baseline root for strict structure parity."
        ),
    )
    parser.add_argument(
        "--compare-docling-groundtruth",
        action="store_true",
        help=(
            "Compare force-run artifacts with upstream Docling "
            "tests/data/groundtruth/docling_v2 files when available."
        ),
    )
    parser.add_argument(
        "--docling-groundtruth-root",
        type=Path,
        help=(
            "Explicit upstream Docling groundtruth root. Defaults to "
            "<docling-source-root>/tests/data/groundtruth/docling_v2 when "
            "--compare-docling-groundtruth is enabled."
        ),
    )
    parser.add_argument(
        "--docling-groundtruth-min-char-coverage",
        type=float,
        default=0.98,
        help="Minimum candidate/groundtruth Markdown character coverage.",
    )
    parser.add_argument(
        "--docling-groundtruth-min-similarity",
        type=float,
        default=0.98,
        help="Minimum normalized candidate/groundtruth Markdown similarity.",
    )
    parser.add_argument(
        "--fail-on-docling-groundtruth-mismatch",
        action="store_true",
        help=(
            "Fail when an available upstream Docling groundtruth comparison "
            "does not pass the configured coverage and similarity thresholds."
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
        default="cli-bin-support,zhenfa-router,duckdb",
        help="Cargo feature set used to start the local Rust provider.",
    )
    parser.add_argument(
        "--gateway-features",
        default="cli-bin-support,zhenfa-router,duckdb",
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
        default=(
            "performance,studio,zhenfa-router,duckdb,document-extract-attachment-audit"
        ),
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
        "--artifact-registry-reuse-probe",
        action="store_true",
        help=(
            "After the force run, run a force=false extraction into a fresh "
            "output directory to measure Rust content-hash artifact registry reuse."
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
    parser.add_argument(
        "--rust-provider-bin",
        type=Path,
        help=(
            "Run a prebuilt Rust provider binary instead of starting the "
            "selected Rust provider through cargo run. Use the "
            "wendao_search_flight_server binary with --rust-provider-mode "
            "flight and the wendao binary with --rust-provider-mode gateway."
        ),
    )
    parser.add_argument("--real-docling", action="store_true")
    parser.add_argument(
        "--fixture-suite",
        choices=("fake", "docling-real", "explicit", "milestone"),
        default="fake",
    )
    parser.add_argument(
        "--docling-source-root",
        type=Path,
        help=(
            "Docling fixture checkout root used for docling-real fixtures. This "
            "is a cache/download surface, not a canonical milestone fixture "
            "authority. Defaults to $PRJ_DATA_HOME/docling-real-fixtures or "
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
    parser.add_argument(
        "--fail-on-structure-order-mismatch",
        action="store_true",
        help=(
            "Fail when force, shard-cache rebuild, and cache-hit runs produce "
            "different structure order signatures."
        ),
    )
    parser.add_argument(
        "--fail-on-missing-ocr-metrics",
        action="store_true",
        help=(
            "Fail when any measured run expected to exercise OCR shards "
            "produces no OCR metrics sidecar rows."
        ),
    )
    parser.add_argument(
        "--fail-on-pdf-milestone-regression",
        action="store_true",
        help=(
            "Fail when an OCR-positive milestone run is missing or regresses "
            "below the stored 2604.17337 precision/speed envelope. Milestone "
            "inputs should be supplied from repo-tracked or explicit auditable "
            "fixture paths, not transient .data downloads."
        ),
    )
    return parser.parse_args()
