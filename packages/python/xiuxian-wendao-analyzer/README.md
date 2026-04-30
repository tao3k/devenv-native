---
type: knowledge
metadata:
  title: "xiuxian-wendao-analyzer"
---

# xiuxian-wendao-analyzer

`xiuxian-wendao-analyzer` is a beta Python analyzer and document parsing
package built on top of `wendao-core-lib`.

Its boundary is intentionally narrow:

1. analyze rows and Arrow tables that already came back from Rust-owned Wendao
   query or exchange surfaces
2. provide one built-in deterministic ranking strategy for score-carrying rows
3. expose lightweight run objects and summary models for downstream callers
4. reuse `wendao-arrow-interface` for offline scripted authoring and testing
5. provide Docling-backed document extraction helpers that return Arrow-shaped
   resource rows
6. expose a Wendao-facing Arrow Flight service for document extraction

It does not own:

1. rerank transport contracts
2. local rerank semantics
3. Flight metadata assembly
4. any Python-hosted shadow of Rust runtime logic

## Current Beta Surface

The current beta exports:

1. `AnalyzerConfig`
2. `ScoreRankAnalyzer`
3. `AnalyzerResultRow`
4. `AnalysisSummary`
5. `RowsAnalysisRun`
6. `TableAnalysisRun`
7. `QueryAnalysisRun`
8. `RepoAnalysisRun`
9. `build_analyzer(config)`
10. `analyze_rows(...)`
11. `analyze_table(...)`
12. `analyze_query(...)`
13. `analyze_repo_search(...)`
14. `analyze_repo_query_text(...)`
15. `run_rows_analysis(...)`
16. `run_table_analysis(...)`
17. `run_query_analysis(...)`
18. `run_repo_analysis(...)`
19. `run_repo_search_analysis(...)`
20. `extract_document_table(...)`
21. `extract_document_resources(...)`
22. `DOCLING_SUPPORTED_DOCUMENT_FORMATS`
23. `DOCLING_COMMON_SOURCE_SUFFIXES`
24. `is_known_docling_source(...)`
25. `extract_pdf_table(...)` and `extract_pdf_resources(...)` compatibility wrappers
26. `DocumentExtractFlightServer`
27. `build_document_extract_table(...)`
28. `/analysis/document-extract` as the primary document extraction route
29. `DoclingPdfOcrShardWorker` for opt-in page-shard OCR
30. summary helpers over the same rows, table, query, and repo-search runs

Docling is optional through the `documents` extra. That extra includes
Docling's XBRL support so the documented XML/XBRL coverage is real, not only a
suffix hint:

```bash
uv sync --extra documents
```

Audio ASR needs heavier model and media dependencies. Use the dedicated audio
extra when running real audio conversion:

```bash
uv sync --extra documents-audio
```

Docling is the parsing authority. The analyzer does not maintain a runtime
allowlist; it exposes known common Docling formats and suffixes for downstream
UX. The current documented set includes PDF, DOCX, XLSX, PPTX, Markdown,
AsciiDoc, HTML/XHTML, CSV, PNG, JPEG, TIFF, BMP, WEBP, USPTO XML, JATS XML,
XBRL XML, METS GBS, WebVTT, LaTeX, plain text, audio, and Docling JSON.

## Wendao Document Service

The package is a real Wendao integration package, not only an example bundle.
The document service entrypoints are:

```bash
uv run wendao-document-extract --host 0.0.0.0 --port 50051
uv run xiuxian-wendao-document-extract --host 0.0.0.0 --port 50051
uv run wendao-document-extract --host 0.0.0.0 --port 50051 --pdf-ocr-worker docling
uv run wendao-document-extract --host 0.0.0.0 --port 50051 --pdf-ocr-worker docling --pdf-ocr-workers auto
```

The Arrow Flight route is `/analysis/document-extract`. Request metadata uses:

1. `x-wendao-schema-version`
2. `x-wendao-document-extract-source-path`
3. `x-wendao-document-extract-output-dir`
4. `x-wendao-document-extract-force`
5. `x-wendao-document-extract-error-row`

The returned Arrow table uses the stable document resource schema:

1. `sourcePath`
2. `resourceType`
3. `resourcePath`
4. `pageIndex`
5. `caption`
6. `content`
7. `mimeType`
8. `status`
9. `elementId`

Each extraction emits a main markdown `document` row and may emit structured
rows when Docling exposes reusable content, including `table`, `image`,
`formula`, `code`, `docling_json`, `audio`, and `subtitle`. Reusable rows are
cached in Arrow IPC as `_resources.arrow`; JSON is only an optional exported
resource row and is not the Python-to-Rust transport contract. The
`docling_json` row points at the exported JSON file without inlining that
payload into the Arrow `content` column, so large XML/PDF conversions do not
force every cache-hit response to carry the full Docling JSON export.

The same service also exposes an internal OCR shard exchange route at
`/analysis/pdf-ocr-shards`. This route uses Arrow Flight `do_exchange`, not
JSON metadata: callers upload `xiuxian_wendao.pdf_ocr_shard_input.v1` Arrow
batches and receive `xiuxian_wendao.pdf_ocr_shard_result.v1` Arrow batches.
It is a worker contract for Rust-rendered page or region shards and does not
change the primary `/analysis/document-extract` sync or async extraction path.
The default worker returns explicit `skipped` rows so deployments never load OCR
models by accident. Passing `--pdf-ocr-worker docling` enables the opt-in
Docling image worker for rendered shards; failed or empty shard OCR rows remain
table-shaped failures so the Rust hybrid provider can fall back to full Docling
when coverage is incomplete.

Docling shard OCR is bounded and adaptive. The service accepts
`--pdf-ocr-workers auto|N` as a direct local default, but the Rust provider may
override it per exchange through the internal `x-wendao-pdf-ocr-workers` Flight
metadata header. Rust owns the global OCR worker budget, splits shard batches
into scheduled chunks, and sends only the acquired worker count to Python for
that exchange. Python remains the Docling OCR execution boundary. Full-page PDF
shards from the same source are selected by Rust, grouped into Docling
`sourcePath` page ranges, and then split back into one OCR result row per page.
The full-page source-range selector avoids `pdf-inspector`, PDFium, and
high-DPI rendering on the hot OCR path: Rust reads the PDF page tree with
`lopdf`, emits stable whole-page shard rows, and lets Docling convert the
original `sourcePath` page range. Python first asks Docling for a single
page-break-separated Markdown export for the converted range; if that shape is
unavailable or incomplete, it falls back to Docling's `page_no`-scoped Markdown
export for each page. Rendered images remain the fallback path for failed page
ranges, region shards, and explicit raster tests.
Result rows are still returned in input shard order. The Rust provider also
validates and restores result order against the original shard input rows before
merge, so document order is not coupled to Python worker completion order.

The built-in strategy is intentionally small:

1. `score_rank`
2. consumes rows that already carry a numeric `score`
3. emits the same rows with a stable integer `rank`

## Boundary Reading

The package split is:

1. `wendao-core-lib` owns Arrow Flight transport and typed contracts
2. `wendao-arrow-interface` owns downstream session ergonomics and scripted
   fixtures
3. `xiuxian-wendao-analyzer` owns analysis over already materialized results
   and local document-to-Arrow resource shaping

That means rerank stays transport-owned. If you need to analyze a rerank result,
fetch it through `wendao-core-lib` or `wendao-arrow-interface`, then hand the
returned rows or table into `analyze_rows(...)` or `analyze_table(...)`.

## Workflow Selection Guide

| Workflow                                              | Recommended entrypoint                                                                                     | Analyzer ownership                         | Host involvement                       | Validation status |
| ----------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- | ------------------------------------------ | -------------------------------------- | ----------------- |
| Offline repo-search authoring with scripted results   | `WendaoArrowSession.for_repo_search_testing(...)` + `run_repo_analysis(session.client, ...)`               | downstream user code                       | none                                   | local covered     |
| PDF attachment search then analyze the returned table | `attachment_search_request(...)` + `WendaoArrowSession.attachment_search(...)` + `run_table_analysis(...)` | downstream user code                       | scripted by default, endpoint optional | local covered     |
| Wendao document extraction service                    | `wendao-document-extract` + `/analysis/document-extract`                                                   | analyzer package service adapter           | Arrow Flight                           | local covered     |
| Local multi-format document parsing into Arrow rows   | `extract_document_table(...)` or `extract_document_resources(...)` with the optional `documents` extra     | Docling-backed document extraction helpers | none                                   | local covered     |

| Repo search with built-in ranking | `run_repo_analysis(...)` + `summarize_repo_analysis(...)` | built-in `score_rank` | real `wendao_search_flight_server` | real-host covered |
| Repo search with a custom Python analyzer | `run_repo_analysis(...)` + `summarize_repo_analysis(...)` + `analyzer=<your analyzer object>` | downstream user code | real `wendao_search_flight_server` | real-host covered |
| Analyze an already materialized Rust query result | `analyze_rows(...)` or `analyze_table(...)` | built-in `score_rank` or downstream user code | depends on who fetched the data | local covered |

## Documentation Set

This README is the intended v1 public docs surface index:

1. [`docs/first_analyzer_author_tutorial.md`](docs/first_analyzer_author_tutorial.md)
2. [`docs/write_your_first_custom_analyzer.md`](docs/write_your_first_custom_analyzer.md)
3. [`docs/release_and_compatibility_policy.md`](docs/release_and_compatibility_policy.md)
4. [`docs/external_consumer_checklist.md`](docs/external_consumer_checklist.md)

## Examples

The shipped example set is now:

1. [`examples/scripted_repo_search_workflow.py`](examples/scripted_repo_search_workflow.py)
   - offline analyzer authoring with `WendaoArrowSession.for_repo_search_testing(...)`
2. [`examples/attachment_pdf_analyzer_workflow.py`](examples/attachment_pdf_analyzer_workflow.py)
   - scripted-by-default PDF attachment search over Rust-returned rows, with optional endpoint mode
3. [`examples/document_extraction_workflow.py`](examples/document_extraction_workflow.py)
   - Docling-backed multi-format document extraction into Arrow resource rows, fixture mode by default
4. [`examples/repo_search_workflow.py`](examples/repo_search_workflow.py)
   - host-backed repo-search analysis with built-in `score_rank`
5. [`examples/custom_repo_analyzer_workflow.py`](examples/custom_repo_analyzer_workflow.py)
   - host-backed repo-search analysis with a custom analyzer object
6. [`examples/host_backed_repo_search_beta_smoke.py`](examples/host_backed_repo_search_beta_smoke.py)
   - one-shot beta smoke for the full host-backed repo-search path

Example commands:

```bash
uv run python examples/scripted_repo_search_workflow.py
uv run python examples/attachment_pdf_analyzer_workflow.py
uv run python examples/document_extraction_workflow.py
uv run python examples/repo_search_workflow.py --help
uv run python examples/custom_repo_analyzer_workflow.py --help
uv run python examples/host_backed_repo_search_beta_smoke.py --mode custom --port 0
```

Document extraction performance can be measured through the Rust ignored test
harness, driven by the local benchmark script:

```bash
uv run python scripts/benchmark_wendao_document_extract.py
```

Reports are written to the script's configured report directory. The default
benchmark uses a fake converter for deterministic fixture coverage. Real
Docling coverage is opt-in. The benchmark can prepare a sparse local fixture
checkout so the real run uses Docling's own `tests/data` attachments without
adding those files to this repository:

```bash
uv sync --extra documents
uv run python scripts/benchmark_wendao_document_extract.py --prepare-only
uv run python scripts/benchmark_wendao_document_extract.py \
  --real-docling \
  --fixture-suite docling-real \
  --prepare-docling-fixtures \
  --fail-on-error-rows
```

Use `--skip-audio` when ASR model artifacts should not be loaded. For real
audio ASR, install `documents-audio` and run without `--skip-audio`; the
benchmark configures the bundled `imageio-ffmpeg` executable for Whisper.
Use `--only-fixture audio` or another fixture name for targeted real fixture
diagnostics. Use `--docling-source-root` only when you already have a prepared
Docling fixture checkout. Use `--concurrency` to stress the Rust-to-Python
cache-hit path and `--server-start-timeout` for cold Docling starts. The report
captures request counts, wall-clock timing, Arrow IPC bytes, status counts, and
error-row counts; `--fail-on-error-rows` makes table-shaped conversion failures
fail the benchmark run.
For `hybrid-page-ocr`, pass `--shard-cache-reuse-probe` when you need explicit
evidence that OCR shard cache reuse works independently from the
whole-document `_resources.arrow` cache. The probe runs a second forced
extraction into a fresh output directory after the first force run and reports
`shardCacheReuseForceMs` in JSON and Markdown output. Reports also include an
`ocrShardCache` summary with the shard cache root, Arrow file count, total
bytes, and configured limits. The Rust provider defaults the OCR shard cache
limit to 10 GiB and supports
`WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_BYTES`,
`WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_ENTRIES`, and
`WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_AGE_SECS` for deployment-specific
capacity policy.
For source-PDF page-range OCR, Rust owns the outer scheduling policy. It may
split one contiguous source-PDF OCR range into several contiguous subranges and
send those subranges concurrently to Python/Docling. The default source-range
target is sublinear in the Rust worker budget because real Docling conversion
can regress when too many page-range conversions run at once. Use
`--rust-pdf-ocr-workers` for the global Rust OCR budget and
`WENDAO_DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS` when a deployment needs
a source-range-specific override.
For async provider validation, run with `--flight-mode async`; the driver starts
the synchronous Python worker plus the existing Rust Flight provider and can
verify cold duplicate-miss deduplication with `--duplicate-miss-concurrency`
and `--fail-on-duplicate-conversions`. Use `--distinct-miss-concurrency` to
add a suite-level burst where different documents cold-miss at the same time;
that report section records queue depth, running conversions, permit pressure,
and one-conversion-per-document counts when a converter counter is available.
For real Docling fixtures, run distinct-miss and duplicate-miss probes as
separate benchmark invocations so both remain true cold-miss measurements.
When a Rust gateway REST endpoint is already running, pass
`--rust-rest-endpoint http://127.0.0.1:<gateway-port>` to sample
`GET /api/document-extract-jobs` during each cargo probe. The JSON and Markdown
reports then include queue depth, running job count, in-process scheduled jobs,
conversion capacity, permit pressure, and last/max conversion duration.
For a fully local gateway-observed run, use `--rust-provider-mode gateway`.
The benchmark starts a temporary Valkey process, the synchronous Python worker,
and `wendao gateway start`, then samples the gateway REST status endpoint while
the Rust Flight probes run:

```bash
uv run python scripts/benchmark_wendao_document_extract.py \
  --real-docling \
  --fixture-suite docling-real \
  --skip-audio \
  --only-fixture pdf \
  --only-fixture image-png \
  --only-fixture uspto-xml \
  --only-fixture xbrl-xml \
  --flight-mode async \
  --rust-provider-mode gateway \
  --wait-ms 70000 \
  --duplicate-miss-concurrency 4 \
  --fail-on-duplicate-conversions \
  --fail-on-error-rows
```

For the distinct-document cold-miss capacity slice, run a separate invocation
with `--distinct-miss-concurrency` and no duplicate-miss flag:

```bash
uv run python scripts/benchmark_wendao_document_extract.py \
  --real-docling \
  --fixture-suite docling-real \
  --skip-audio \
  --flight-mode async \
  --rust-provider-mode gateway \
  --wait-ms 70000 \
  --distinct-miss-concurrency 4 \
  --fail-on-distinct-miss-conversions \
  --fail-on-error-rows
```

The cache-hit path is optimized for service use: cached `_resources.arrow`
files are returned as Arrow tables without a Python row roundtrip, and the Rust
document extraction provider reuses its Tonic channel for the configured
endpoint. First-time Docling conversion is still CPU/model bound and should be
handled with queueing and worker-pool sizing in production. The Rust provider
limits concurrently running cold conversions with
`WENDAO_DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS`; the default is bounded to
the host parallelism with a maximum of four. Jobs waiting for this capacity
remain in `queued` status and do not occupy Python-side SQL or registry work.
The browser-facing `GET /api/document-extract-jobs` endpoint returns the Rust
provider runtime snapshot, including queue counts, running counts, conversion
capacity, scheduled in-process jobs, and last/max conversion duration. Use it
with the single-job `GET /api/document-extract-job?job_id=...` status endpoint
when tuning real PDF/OCR/audio pressure runs.
The Python service stays a synchronous Arrow Flight worker. Async queueing,
content-hash deduplication, job status, and DuckDB-backed durability are owned
by the Rust Wendao provider/gateway so Python does not spend cycles on SQL
registry work before returning Arrow data.

## Beta Readiness

ready now:

1. offline scripted repo-search authoring
2. scripted PDF attachment analysis over Rust-returned attachment-search tables
3. Docling-backed local multi-format document extraction into Arrow resource rows
4. Wendao-facing Arrow Flight service entrypoint for document extraction
5. host-backed repo-search analysis with built-in or custom analyzers
6. generic rows/table/query analysis over Rust-returned data

known gaps before broader adoption:

1. no GA-level release promise yet
2. no analyzer-owned rerank helper surface
3. callers still need `uv run python ...` from the package directory for the
   shipped examples

## Beta Freeze Audit

The current package boundary is now intentionally lockable as `0.2.1`.

frozen for this beta trial:

1. the document extraction service route and returned Arrow resource schema
2. the six shipped examples above
3. `WendaoArrowSession.for_repo_search_testing(...)` as the documented offline
   author workflow
4. `WendaoArrowSession.attachment_search(...)` plus `run_table_analysis(...)`
   as the documented PDF attachment workflow seam
5. `extract_document_table(...)` as the documented multi-format document parsing workflow seam
6. `wendao-document-extract` as the Wendao-facing service command
7. `run_repo_analysis(...)` and `run_query_analysis(...)` as the host-backed
   analyzer entrypoints
8. the rule that analyzer-owned rerank helpers are out of scope

not frozen for this beta trial:

1. helper symmetry for every possible Wendao route
2. future analyzer strategies beyond `score_rank`
3. additional convenience wrappers over already materialized transport results

current freeze rule:

1. workflow-frozen, not helper-frozen
2. transport-owned rerank remains in `wendao-core-lib` and
   `wendao-arrow-interface`

## Beta Exit Audit

exit-ready now:

1. real-host repo-search coverage exists through `wendao_search_flight_server`
2. offline authoring is available through the scripted session surface
3. docs and examples align with the Rust-query-first analyzer boundary

not exit-ready yet:

1. no GA-level versioning promise yet
2. no broader downstream feedback cycle yet
3. no committed compatibility window beyond this beta baseline
