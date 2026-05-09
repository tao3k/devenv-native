---
type: knowledge
kind: package-readme
title: "xiuxian-wendao-analyzer"
category: "package"
status: "active"
author: Xiuxian Artisan Workshop
date: 2026-05-05T00:00-07:00
description: "Python analyzer and Docling Flight worker boundary for Wendao document extraction and table analysis."
tags:
  - python
  - analyzer
  - docling
  - arrow-flight
metadata:
  title: "xiuxian-wendao-analyzer"
  retrieval:
    saliency_base: 6.4
    decay_rate: 0.04
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
For benchmark canaries, Rust can also retry failed or empty non-hosted,
non-backend-text page OCR rows through the existing Hosted VLM/OCR page profile
before allowing the full-document Docling fallback to run. Enable this with
`--rust-pdf-failed-page-recovery hosted-vlm-page` in the benchmark harness or
`WENDAO_DOCUMENT_EXTRACT_PDF_FAILED_PAGE_RECOVERY=hosted-vlm-page` in the Rust
provider environment. The default remains disabled. Backend-text page failures
and invalid hosted recovery rows still fall through to the existing precision
fallback.
For benchmark readiness only, the worker can pre-initialize selected Docling OCR
profiles and optionally convert one or more real source pages before the worker
listens. `WENDAO_PDF_OCR_PREWARM_PAGE_INDICES` accepts comma-separated
zero-based page indices and supersedes the legacy single
`WENDAO_PDF_OCR_PREWARM_PAGE_INDEX` value. This does not change OCR output
schemas or live routing. The current milestone evidence rejects broad
multi-page prewarm for pages `5,11` because it preserved precision but
regressed force refresh to `18784.875625 ms`.

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
uv run python tests/scripts/benchmark_wendao_document_extract.py
```

Reports are written to the script's configured report directory. The default
benchmark uses a fake converter for deterministic fixture coverage. Real
Docling coverage is opt-in. The benchmark can prepare a sparse local fixture
checkout so the real run uses Docling's own `tests/data` attachments without
adding those files to this repository:

```bash
uv sync --extra documents
uv run python tests/scripts/benchmark_wendao_document_extract.py --prepare-only
uv run python tests/scripts/benchmark_wendao_document_extract.py \
  --real-docling \
  --fixture-suite docling-real \
  --prepare-docling-fixtures \
  --python-uv-extra documents \
  --fail-on-error-rows
```

Use `--skip-audio` when ASR model artifacts should not be loaded. For real
audio ASR, install `documents-audio` and run without `--skip-audio`; the
benchmark configures the bundled `imageio-ffmpeg` executable for Whisper. Pass
`--python-uv-extra documents` for real Docling document OCR and
`--python-uv-extra documents-audio` for real audio ASR worker starts.
Use `--only-fixture audio` or another fixture name for targeted real fixture
diagnostics. Use `--docling-source-root` only when you already have a prepared
Docling fixture checkout. Use `--concurrency` to stress the Rust-to-Python
cache-hit path and `--server-start-timeout` for cold Docling starts. The report
captures request counts, wall-clock timing, Arrow IPC bytes, status counts, and
error-row counts; `--fail-on-error-rows` makes table-shaped conversion failures
fail the benchmark run.
Use `--fail-on-structure-order-mismatch` when a real OCR benchmark must fail
if force, shard-cache rebuild, and cache-hit artifacts produce different
structure order signatures.
Use `--fail-on-structure-parity-mismatch` when a real OCR benchmark has a
Docling baseline and must fail if the candidate loses baseline text coverage,
protected block counts, or any other structure parity guard.
Use `--fail-on-precision-gate-failure` when a benchmark candidate should fail
on the aggregate precision gate instead of only recording the failure in the
report. This combines error rows, artifact errors, structure order, structure
parity, and Docling groundtruth status from `precisionSpeedSummary`.
Each report also includes a `precisionSpeedSummary` section that keeps the
quality and latency signals together: error rows, artifact errors, structure
order, force/cache/shard-reuse order stability, parity status, OCR/bbox block
counts, force latency, cache p95, shard-cache rebuild latency, Rust scheduler
elapsed time, Docling convert time, Docling convert share, and request-boundary
overhead share. These fields make it explicit whether a cold miss is dominated
by Docling parsing or by Rust/Flight/Arrow overhead.
Full-document Docling cache misses also write an internal
`_document_metrics.arrow` sidecar. It records phase timings for conversion,
Markdown export, resource row construction, structure sidecar construction,
and Arrow cache writes. The sidecar is benchmark evidence only; it does not
change the returned resource table, `_resources.arrow`, `_structure.arrow`,
OCR `_metrics.arrow`, or the Flight/REST extraction contracts.
The document Flight service pre-initializes the Arrow table and IPC writers at
startup so the first user request does not pay the one-time PyArrow writer
initialization cost. Reports also compute document timing overhead as
`forceRefreshMs - documentTimingTotalElapsedMs` when a timing sidecar exists,
which separates Python extraction work from Flight/Rust request-boundary cost.
`precisionSpeedSummary.maxDoclingConvertShare` and
`precisionSpeedSummary.maxDocumentTimingOverheadShare` keep those two costs
visible in the same precision gate used for PDF, image, Office, XML, audio, and
structured-text regressions.
Mixed Docling fixture runs also include `summary.attachmentClassSummary` plus
an `Attachment Class Summary` Markdown table. The class summary groups the same
precision and speed signals by attachment class, including PDF, Office,
images, structured text, web documents, table data, XML, subtitles, audio,
Docling JSON, archive-backed documents, and unknown custom inputs. This keeps
non-PDF attachment regressions visible without changing the Arrow Flight
document extraction contract. Class summaries also aggregate resource type
counts, structure block type counts, bbox block counts, Rust image audit
format counts, dimension-source counts, known-dimension counts, acceleration
candidate counts, and the slowest force/cache fixture in each class so image
OCR and XML/table-heavy hotspots can be diagnosed before adding a new fast
path. Rust image audit is a preflight/control-plane signal only; Docling
remains the image OCR and layout authority until a later parity-gated fast path
is proven.
For `hybrid-page-ocr`, pass `--shard-cache-reuse-probe` when you need explicit
evidence that OCR shard cache reuse works independently from the
whole-document `_resources.arrow` cache. The probe runs a second forced
extraction into a fresh output directory after the first force run and reports
`shardCacheReuseForceMs` in JSON and Markdown output. Reports also include an
`ocrShardCache` summary with the shard cache root, Arrow file count, total
bytes, and configured limits. Hybrid OCR reports also summarize the internal
`_metrics.arrow` sidecar: shard metric row count, OCR result characters, bbox
coverage count, and Rust scheduler elapsed time. Hosted VLM/OCR benchmark
reports include `hostedVlmPromotionGate`, which checks precision, row/order
stability, character floor, hosted request success, hosted key presence,
force-refresh latency, and shard-cache reuse against the locked
`fast-risk-window` promotion baseline. This is a reporting gate only; it does
not change runtime routing.
The Rust provider defaults the OCR shard cache limit to 10 GiB and supports
`WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_BYTES`,
`WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_ENTRIES`, and
`WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_AGE_SECS` for deployment-specific
capacity policy. Local benchmark runs isolate the OCR shard cache by default
so fake and real OCR evidence do not cross-contaminate; use
`--ocr-shard-cache-root` when a run intentionally targets a persistent cache.
For non-PDF and full-document cache validation, pass
`--artifact-registry-reuse-probe`. That probe runs `force=false` into a fresh
output directory after the force run and reports `artifactRegistryReuseForceMs`,
which measures Rust content-hash artifact mirroring separately from the normal
same-output cache-hit path. In local mode this probe starts the Rust Flight
provider even when `--flight-mode sync` is selected, because Python-direct sync
benchmarks do not exercise the Rust artifact registry.
Use `--structure-baseline-root` when a golden Docling baseline artifact root is
available. The Rust ignored benchmark reads each candidate `_structure.arrow`,
matches it to the baseline artifact directory by fixture output name, and
reports strict structure parity pass/error fields without changing the
document extraction Flight contract. Pass `--generate-structure-baselines` to
run a sync/full-Docling baseline pass before candidate probes; when no explicit
baseline root is provided, the script writes those baseline artifacts below the
configured report directory.
For source-PDF page-range OCR, Rust owns the outer scheduling policy. It may
split one contiguous source-PDF OCR range into several contiguous subranges and
send those subranges concurrently to Python/Docling. The default source-range
target uses the current adaptive Rust OCR budget, capped by a machine-derived
source-range ceiling and page count because real Docling conversion can regress
when too many page-range conversions run at once. Milestone and regression
gates should leave `--rust-pdf-ocr-source-range-workers` unset so the Rust
scheduler's system-aware auto policy is exercised. Use `--rust-pdf-ocr-workers`
only for the global Rust OCR budget ceiling and
`--rust-pdf-ocr-source-range-workers` only for diagnostic profile sweeps.
For `docling-structure-recovery`, the benchmark auto profile caps Docling
full-profile PDF accelerator threads to one per Python document worker through
`WENDAO_DOCUMENT_EXTRACT_FULL_THREADS=1`. Rust keeps outer parallelism and
endpoint-pool scheduling authority, while each Python worker avoids nested
Docling thread contention. Use `--document-extract-full-threads` only for
diagnostic sweeps or platform-specific validation. The May 8, 2026 DocLayNet
fixture run with this auto policy preserved zero errors, stable order, and
Docling structure parity while reducing cold force refresh to `10127.429667 ms`
against the locked `12856.546292 ms` baseline.
For explicit benchmark readiness, use `--document-extract-prewarm-source-path`
with `--document-extract-prewarm-page-ranges` to run selected Docling page-range
conversions before the worker advertises readiness. This warms the converter
and Docling lazy state only; it does not publish candidate output artifacts or
replace the force-refresh precision gate. The default is disabled. On May 8,
2026, prewarming page range `1:1` for the same DocLayNet PDF fixture preserved
zero errors, stable order, Docling structure parity, `13` resource rows, and
`12` structure blocks while reducing force refresh to a best sample of
`8715.070334 ms`. A repeat of the same shape preserved the same correctness
gates but measured `11627.203583 ms`, so treat prewarm as an explicit readiness
control with visible variance rather than a stable sub-10s default.
The current auto policy targets seven source PDF pages per worker before
clamping to the adaptive Rust budget, machine cap, remaining permits, and shard
count. Within that bounded chunk budget, Rust reads a lightweight source-PDF
page complexity profile and forms contiguous, reading-order-preserving
subranges. The planner must not cross cache-miss gaps, and it keeps every
selected page on the source-PDF OCR lane so the Python worker continues to use
Docling over original PDF page ranges rather than lower-precision raster or
table-fast shortcuts.
Python also recognizes the existing `docling-fast-text-ocr` OCR profile as a
separate Docling converter profile. Rows with different `ocrProfile` values are
not merged into one source-PDF page range, so Rust can mix fast and accurate
Docling ranges without changing the Arrow schema. The benchmark exposes this as
`--rust-pdf-ocr-profile-planner fast-risk-window`, which forwards
`WENDAO_DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER=fast-risk-window` to the Rust
provider. The local real-Docling benchmark server passes `ocrProfile` through
its converter factory, so `docling-fast-text-ocr` uses Docling's FAST table
mode even when benchmark fixture options also configure other formats. Because
Rust owns the outer OCR worker budget, the fast-text Docling converter uses one
internal accelerator thread by default to avoid multiplying Python-side Docling
threads across Rust-scheduled source-range chunks. Set
`WENDAO_PDF_OCR_FAST_TEXT_THREADS` only for a host-specific diagnostic sweep.
Python also recognizes `docling-backend-text-ocr-v1`, a PDF-native backend-text
source-range profile that disables Docling OCR and table-structure work while
forcing backend text extraction. Rust can combine that profile with
`docling-fast-text-ocr` top-up pages in the benchmark-only
`hosted-vlm-risk-window-backend-text` planner without changing the Arrow shard
schema. In that mixed source-range mode, Rust schedules contiguous profile
runs as the dispatch unit, prioritizes fast-text top-up runs before backend
runs, and automatically expands the requested dispatch budget to the run count
before clamping it through live worker permits. When the hosted recovery
region pipeline is also in `render-dispatch` mode, Rust keeps the run-count
floor for the remaining source top-up ranges after local backend-text rows are
handled. This lets non-contiguous precision top-up ranges run in parallel with
hosted region recovery while the live Rust scheduler still owns worker
admission. The mode remains opt-in and is promoted only for benchmark evidence
that preserves the frozen character floor and beats the locked force-refresh
baseline.
The benchmark can additionally pass
`--rust-pdf-local-backend-text rust-lopdf` to let the Rust provider satisfy
`docling-backend-text-ocr-v1` rows with the attachment-owned `lopdf`
source-text helper. The current promoted OpenRouter canary does not require
that helper: it uses `mistralai/ministral-3b-2512`, render-dispatch with
render-ahead `3`, region trim, and an explicit `2s` hosted hedge. Two May 9,
2026 milestone runs preserved zero error rows, stable `27` rows, `21/6`
page/region OCR blocks, and the frozen character floor: best force refresh
`7338.796584 ms` with `metricsResultChars=115735`, and repeat force refresh
`8322.027792 ms` with `metricsResultChars=115925`. The hosted request wall
span was `5166 ms` and `6140 ms`; both runs used `12` HTTP attempts for `6`
logical hosted region requests, so this remains an explicit benchmark profile
decision rather than a global default. The older endpoint-local `4s` hedge
sample at `8201.568417 ms` and the 2026-05-07 r59/r60 evidence remain
historical regression controls. A same-shape `1s` hedge canary stayed
precision-valid at `8562.0245 ms` but did not beat the `2s` envelope.
The latest follow-up canaries keep the same active envelope. Disabling
fast-text top-up is rejected because it preserved row/order checks but dropped
`metricsResultChars` to `100981`, below the frozen `103984` floor. Page `5`
prewarm plus `single-page-first` affinity measured `8516.511291 ms` with the
precision gate intact, but did not beat the `8322.027792 ms` repeat. Composite
size `3` reduced hosted requests to `4`, but hosted p95 reached `8528.296 ms`
and force refresh regressed to `10797.20775 ms`. Region render chunk mode
`region` improved first region readiness to about `0.70-0.72s`, but the two
precision-valid repeats measured `8202.969708 ms` and `8927.807167 ms`, so it
remains diagnostic instead of replacing page-grouped region chunks.
`region-seed-page` is the next opt-in chunk-shape canary: Rust renders the
smallest recovery region first, then renders the remaining regions grouped by
page. The analyzer sees the same OCR shard rows and request schema; promotion
still depends on the normal row/order, character-floor, hosted-tail, and
precision gates.
`--rust-pdf-local-backend-text-empty fail-fast` is a diagnostic scheduler
canary for source-page-range placeholder rows. When Rust `lopdf` proves a
`docling-backend-text-ocr-v1` source page has empty backend text, or cannot
produce the requested source-page text vector, the provider returns a failed
OCR row immediately so the precision-preserving
full-document fallback can run without sending the non-image placeholder
through the Python raster OCR path. The default remains `dispatch-python`.
`--pdf-ocr-backend-text-empty-page verified-empty` is a narrower recovery
canary for true empty source-page-range rows. The benchmark forwards it to the
Python worker as `WENDAO_PDF_OCR_BACKEND_TEXT_EMPTY_PAGE=verified-empty` and
to the Rust provider as
`WENDAO_DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_EMPTY_PAGE=verified-empty`. When a
`docling-backend-text-ocr-v1` source-page placeholder has no image path and
backend-text plus the optional compatible-page retry both produce no text, the
worker may return a successful empty Markdown row. Rust accepts that empty text
only for backend-text page shards backed by source-page-range placeholders, so
ordinary empty OCR output still triggers the existing precision fallback.
The 2026-05-08 r108c real Docling canary rejected this mode as a promotion
candidate for the `pdf-redp5110-sampled`, `pdf-skipped-1page`, and
`pdf-skipped-2pages` fixtures: force refresh improved, but all three structure
parity checks failed because backend-text rows did not preserve baseline text
coverage. Use the structure parity gate for any future run that enables this
canary.
`--pdf-ocr-backend-text-page-fallback compatible-page` is a separate Python
worker canary for backend-text page failures. It retries only failed or empty
backend-text source pages through `docling-compatible-page-ocr-v1` before the
Rust provider escalates to full-document fallback. The default remains
`disabled`; promotion requires structure parity and Docling groundtruth gates
because compatible page Markdown may not preserve full-document structure.
`--rust-pdf-backend-text-topup disabled` is a separate character-floor
diagnostic. It is rejected for the current milestone fixture because disabling
top-up drops `metricsResultChars` below the frozen floor; the current-rev
canary measured `100981` result characters at `9169.448167 ms`. The default
remains `profile`.
`--rust-pdf-backend-text-topup hosted-vlm` is a full-page hosted VLM/OCR
top-up replacement canary. It is rejected for the milestone fixture because
force refresh regressed to `35374.309 ms` and `metricsResultChars=91265` fell
below the frozen floor; the hosted model did not preserve the dense page `5`
text coverage that Docling fast-text top-up currently supplies.
`--rust-pdf-local-fast-text rust-lopdf` is also diagnostic-only. It improves
force refresh on the milestone fixture but is rejected because it drops
`metricsResultChars` below the frozen floor. The converter-only prewarm
diagnostic `--pdf-ocr-prewarm-profile docling-fast-text-ocr` is
precision-valid but did not improve the promoted r59/r60 envelope. Pair it
with `--pdf-ocr-prewarm-source-path` and `--pdf-ocr-prewarm-page-index` when a
long-lived local worker should perform one real source-page conversion before
listening, which triggers Docling's lazy table-structure warmup outside the
force-refresh request path. On the milestone fixture, source page `0` warmup
reduced page `5` fast-text top-up to about `7.2-7.4s` and passed repeat
promotion at `11990.357708 ms` and `11537.015125 ms`, but it is accepted as a
stability diagnostic rather than the current best evidence because it is slower
than the promoted `8201.568417 ms` OpenRouter sample.
Use `--pdf-ocr-prewarm-endpoint-count N` when only the first `N` local OCR
endpoints should receive that source-page prewarm. This avoids the invalid
full-pool page `5` prewarm failure while still allowing Rust endpoint affinity
experiments. The paired
`--rust-pdf-fast-text-endpoint-affinity single-page-first` canary routes
single-page fast-text source-PDF chunks to the first endpoint. On the milestone
fixture, r70 preserved precision and completed force refresh at
`9636.47725 ms`, with page `5` fast-text reduced to `5274.754916 ms`. It is
accepted as endpoint-locality evidence. A 2026-05-08 control without
endpoint-local prewarm and affinity regressed to `22329.780375 ms`, with page
`5` fast-text source-range work tailing at `20193.906625 ms`; restoring the r70
shape brought the canary back to `10164.795292 ms` with a `5s` hosted hedge,
and tightening only the hosted hedge to `4s` produced the previous promoted
`8201.568417 ms` sample. Current-rev `2s` hedge repeats now supersede it as
the active OpenRouter region-recovery envelope. r71 prewarmed endpoints `0-3`
and reduced the page
`11-13` fast-text chunk to `5972.05625 ms`, but force refresh regressed to
`10336.721667 ms` because the hosted region tail dominated.
The later Ministral same-page region composite size `3` diagnostics preserved
precision but remain rejected. The older page `12` three-region composite
request tailed at `14430.981 ms` and force refresh regressed to
`17806.492208 ms`; the current-rev repeat reduced hosted requests to `4` but
still measured `10797.20775 ms` force refresh with hosted p95 `8528.296 ms`.
The r83 composite repeat with source split and endpoint affinity failed before
OCR metrics with a Flight `BrokenPipe`, so region composite remains a
provider-stability canary rather than a promoted OpenRouter optimization. The
worker now traces unexpected composite exceptions as failed canary attempts
and falls back to individual region requests, preserving the existing row
contract. The r84 fallback-guard rerun completed with valid OCR metrics and
passed the locked baseline at `12658.151 ms`, with `metricsResultChars=106410`,
zero error rows, stable `27` rows, and `21/6` page/region OCR blocks. It
remains diagnostic only because it is slower than the promoted `8201.568417 ms`
OpenRouter envelope.
`--rust-pdf-fast-text-source-range-split single-page` is a source-range
chunk-shape diagnostic. It keeps Rust scheduler permits as the admission
owner and preserves precision on the milestone fixture. The older r64b run is
rejected because page `5` single-page Docling fast-text conversion regressed
force refresh to `23629.474667 ms`. The later r82b run, paired with
endpoint-local fast-text affinity, passed the locked promotion gate at
`12133.964875 ms` with `metricsResultChars=108788`, but it remains
diagnostic-only because it did not beat the current promoted OpenRouter sample
and page `5` still dominated the source-range tail.
The fast
profile is still not the global default because broader corpus promotion needs
separate evidence, but the arXiv `2604.17337` guard proves this opt-in
risk-window profile on the current machine: 12,856.546 ms force-refresh
latency, 92.084 ms shard-cache reuse latency, 2.309 ms cache p95, zero error
rows, 21 OCR page blocks, 21 bbox blocks, 21 metrics rows, 103,985 OCR result
characters, and stable structure order.
Python recognizes `hosted-vlm-direct-ocr-v1` as the model-agnostic direct VLM
OCR profile over the same shard rows. That path calls an externally managed
OpenAI-compatible backend, with vLLM as the preferred runtime, using
`WENDAO_HOSTED_VLM_OCR_BASE_URL`, `WENDAO_HOSTED_VLM_OCR_MODEL`,
`WENDAO_HOSTED_VLM_OCR_API_KEY`, `WENDAO_HOSTED_VLM_OCR_PROMPT`,
`WENDAO_HOSTED_VLM_OCR_MAX_TOKENS`,
`WENDAO_HOSTED_VLM_OCR_REGION_MAX_TOKENS`, and
`WENDAO_HOSTED_VLM_OCR_TIMEOUT_SECONDS`. Set
`WENDAO_HOSTED_VLM_OCR_REQUEST_CONCURRENCY` to tune direct remote request
fan-out; keep it provider-gated by benchmark evidence because some hosted
models lose stability when concurrency is too high. The direct client retries
transient hosted HTTP failures such as `429`, `500`, `502`, `503`, and `504`
with a short bounded backoff before returning a failed row, so Rust precision
and fallback gates still own the final decision. Set
`WENDAO_HOSTED_VLM_OCR_PAGE_WINDOW_SIZE` above `1` to let the direct worker
combine contiguous page images into one request as a provider capability
canary. Current hosted optimization keeps this disabled by default. The
`hosted-vlm-risk-window` planner sends only the source-profile risk window to
the hosted VLM/OCR backend while leaving ordinary pages on the fast profile.
`hosted-vlm-risk-window-backend-text` is a benchmark-only source-range
optimization canary that keeps hosted recovery semantics but routes ordinary
low-risk pages to `docling-backend-text-ocr-v1` and dense text top-up pages to
`docling-fast-text-ocr`.
The Docling-centered recovery lane makes this package the structure execution
owner, not just a full-document fallback. The internal Flight metadata header
`x-wendao-document-extract-page-range` requests a 1-based inclusive Docling
conversion range and writes uniquely prefixed page-range resource rows plus a
matching `_structure.arrow` sidecar. Studio may merge those rows back by
`pageIndex` to replace failed or empty page OCR rows while preserving Docling
reading order. Hosted OpenRouter, local OCR2, backend text, and fast text are
therefore text patch or acceleration paths over Docling structure, not
replacement structure pipelines.
Rust may request multiple contiguous page ranges for one document when a
benchmark enables page-range chunking. The analyzer treats each range as an
independent Docling conversion request with stable page-range element ids;
Studio is responsible for wrapper-row normalization, structure parity checks,
and escalation to full-document fallback when any range is incomplete. The
benchmark report now keeps per-range Docling fallback timing and the slowest
chunk summary visible so analyzer conversion cost can be separated from Rust
scheduler and Flight overhead.
For tail-latency diagnostics, `--rust-pdf-docling-page-range-chunk-plan`
forwards an exact 1-based fallback chunk plan such as `1:3,4:4,5:6,7:9` to the
Rust provider. The Rust side rejects plans that omit, duplicate, or include
pages outside the Docling fallback set, so this remains a precision-preserving
benchmark control rather than a new routing default. The first May 8, 2026
tail-splitting canary with that plan preserved Docling structure parity but
regressed to `18912.534209 ms`, so the accepted default remains three-page
chunks.
For automatic high-cost diagnostics,
`--rust-pdf-docling-page-range-structure-cost-budget` forwards
`WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_STRUCTURE_COST_BUDGET` to Rust.
When set, Rust may split automatic `docling-structure-recovery` fallback ranges
whose source-profile structure cost exceeds the budget. The split may spend
spare document-extract endpoint capacity, but it must remain a single Docling
execution wave; without spare capacity Rust keeps the capped range shape. The
flag is disabled by default and is only a benchmark-visible diagnostic control;
structure parity, row order, and precision gates still decide promotion.
Set `WENDAO_DOCUMENT_EXTRACT_CONVERTER_CACHE=profile` only for explicit
page-range benchmark probes that need to test whether reusing a Docling
converter across Flight document-extract calls reduces conversion setup cost.
The benchmark flag is `--document-extract-converter-cache profile`. The default
remains disabled so ordinary document extraction keeps the existing converter
lifecycle.
Set `WENDAO_DOCUMENT_EXTRACT_PREWARM_SOURCE_PATH` and
`WENDAO_DOCUMENT_EXTRACT_PREWARM_PAGE_RANGES` only through benchmark-controlled
readiness probes. These controls are intentionally separate from converter
reuse: prewarm validates daemon readiness and Docling lazy initialization,
while converter-cache probes isolate repeated page-range setup cost after the
worker is already serving requests.
Explicit `region-shards` benchmarks are the narrower
recovery-surface proof before automatic region discovery is promoted. Narrow
exact-risk-only page routing is not the promotion path because the real
milestone run lost the frozen character floor. In the current mixed-render
route, Rust keeps ordinary pages as source-range rows and renders 300 DPI page
images only for hosted VLM recovery pages, preserving the existing Arrow input
and result schemas. When the benchmark supplies explicit region shards, Rust
keeps the parent page on the fast source-range profile and appends recovery
region rows as supplemental recovery inputs. Rust binds each hosted recovery
region row to the retained fast parent page and records a `sentinel-sidecar-v1`
structure provenance marker so region recovery remains a validated sidecar
patch rather than an implicit string splice. When no explicit region JSON is configured,
`--rust-pdf-hosted-vlm-region-planner profile-risk-window` lets Rust derive a
conservative content-band region for pages already selected by
`hosted-vlm-risk-window`; this remains benchmark-only until region precision and
stitching pass the promotion gate. The adjacent benchmark-only
`profile-risk-window-slices` planner splits that same content band into
top/middle/bottom same-page regions so hosted VLM/OCR tests can exercise
same-page composite requests without changing the Arrow shard schema.
`profile-risk-window-adaptive` keeps the same benchmark-only source selection
but chooses one, two, or three slices from Rust's source-page structure profile
and estimated content-band pixel area. Exact structure-risk pages may receive
more slices, while low-complexity risk-window neighbor pages can stay as one
region. It is the preferred next hosted benchmark profile because it targets
the measured trade-off between single-band provider tail latency and blanket
three-slice request overhead without lowering DPI.
The benchmark can also pass
`--rust-pdf-hosted-vlm-region-pipeline render-dispatch`, forwarded to
`WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PIPELINE`, to let Rust overlap
ordinary source-range OCR scheduling with hosted recovery-region rendering.
This mode is a local-overhead optimization probe for the measured gap between
hosted request wall span and force-refresh latency; it stays disabled by
default and does not change the Arrow shard schema or precision gates.
Pass `--rust-pdf-hosted-vlm-region-render-ahead N` when that opt-in pipeline
should pre-render more than one page-region chunk while hosted requests are in
flight. Rust still sorts the final OCR shard inputs back to deterministic
reading order before validating rows and writing result artifacts.
Pass `--rust-pdf-hosted-vlm-region-render-chunk region` only as a chunk-shape
diagnostic. It asks Rust to render each recovery region as an independent
chunk rather than grouping regions by page, so the first hosted request can be
dispatched as soon as the first region is ready. Current-rev milestone repeats
preserved zero errors, stable order, `27` rows, `21/6` page/region blocks, and
the frozen character floor while moving first region readiness to about
`0.70-0.72s`; force refresh still measured `8202.969708 ms` and
`8927.807167 ms`, so the default remains page-grouped chunks. The all-region
render chunk diagnostic is also rejected because it delayed first hosted
dispatch to `12528.588583 ms` and regressed force refresh to `21670.3075 ms`.
Pass `--rust-pdf-hosted-vlm-region-render-chunk region-seed-page` for the
middle-ground canary: one smallest-area region becomes an early seed request,
and all remaining recovery regions stay page-grouped. This is intended to test
whether early hosted dispatch can be recovered without the full tail cost of
splitting every region. The first explicit PDFium OpenRouter gate passed the
precision gate with zero error rows, stable `27` rows, `21/6` page/region OCR
blocks, and the frozen character floor. The measured pair is
`8250.492790999999 ms` with `metricsResultChars=116286`, then
`8445.105417 ms` with `metricsResultChars=116270`. It remains a canary because
it has not beaten the active `7338.796584/8322.027792 ms` envelope and cannot
replace page-grouped chunks yet.
Region rows use
`WENDAO_HOSTED_VLM_OCR_REGION_MAX_TOKENS`,
defaulting to 2048 and clamped by `WENDAO_HOSTED_VLM_OCR_MAX_TOKENS`, so a
single hosted region response cannot silently consume the full page-token
budget unless the benchmark explicitly raises the region cap through
`--hosted-vlm-ocr-region-max-tokens`. Set
`WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE` above `1`, or pass
`--hosted-vlm-ocr-region-composite-size`, to let the direct worker combine
same-page, same-parent hosted recovery region rows into one multi-image request. Region
composite output must split back into one non-empty Markdown result per region
sentinel marker; otherwise the worker falls back to individual region requests
so the existing row/order contract is preserved. Batched page-window responses
follow the same marker-split rule for page markers. The benchmark can also set
`WENDAO_HOSTED_VLM_OCR_REGION_ATLAS_MODE=same-page-json`, or pass
`--hosted-vlm-ocr-region-atlas-mode same-page-json`, to pack a same-page region
composite group into one labeled PNG atlas and request strict JSON keyed by
exact shard markers. Atlas mode is an opt-in request-surface canary: valid JSON
is canonicalized back into Markdown rows, while invalid JSON, row-count
mismatches, empty text, and HTTP failures fall back to individual region
requests so the existing Arrow shard result contract and precision fallback
remain unchanged. Set
`WENDAO_HOSTED_VLM_OCR_SCAFFOLD_MODE=region-table-json`, or pass
`--hosted-vlm-ocr-scaffold-mode region-table-json` in the benchmark harness, to
enable structural scaffold recovery for hosted recovery region rows. In that mode the
worker loads Studio's `_hosted_vlm_region_scaffolds.json` sidecar beside the rendered
region images, validates the shard id, parent shard id, source content hash,
and raster hash, asks the hosted VLM/OCR provider for JSON-only output keyed by exact region markers,
and canonicalizes valid table/text JSON back into Markdown result rows. Missing
sidecars, fingerprint mismatches, malformed JSON, marker or row-count
mismatches, empty canonical text, and invalid table cell shapes return failed
rows so the existing Rust precision fallback protects correctness.
The current OpenRouter/Qianfan fast benchmark rejects scaffold composite as a
promotion path because the provider failed strict row-count and canonical-text
validation in real runs. Keep scaffold mode disabled unless the selected
provider has fresh evidence with zero scaffold validation failures. Set
`WENDAO_HOSTED_VLM_OCR_SPECULATIVE_RETRY_DELAY_SECONDS` above `0`, or pass
`--hosted-vlm-ocr-speculative-retry-delay-seconds`, to enable an opt-in
hedged request for direct hosted region rows: if the first hosted request has
not produced a valid result by that delay, the worker starts a second request
and returns the first valid Markdown response. This is a tail-latency guard for
cloud OCR providers; it does not change the Arrow schema or validation gates,
and benchmark traces report both logical requests and HTTP attempt counts so
request-cost evidence stays visible. The current promoted OpenRouter region
canary pins this delay at `2s`; this is benchmark evidence, not a global
default change. Set
`WENDAO_HOSTED_VLM_OCR_TRACE_PATH` to a JSONL file when a benchmark needs
request-level latency, HTTP status, image-byte, Markdown character,
shard-type, region-count, render-DPI, source-pixel-area, scaffold mode,
scaffold applied count, scaffold validation failure count, JSON character
count, and canonical Markdown character telemetry. Trace records intentionally
omit API keys and image payloads. Hosted VLM/OCR recovery
benchmarks must not lower render DPI to gain speed; region shrinkage and
provider capability gates are the accepted optimization levers. Set
`WENDAO_HOSTED_VLM_OCR_IMAGE_OPTIMIZATION=region-whitespace-trim`, or benchmark
with `--hosted-vlm-ocr-image-optimization region-whitespace-trim`, to trim
near-white margins from Hosted VLM/OCR region PNG request payloads. The default
is `disabled`; the optimization applies only to region rows, keeps render DPI
intact, and does not change Arrow shard rows, result schema, row order, or
validation gates. Benchmark trace summaries report
`imageOptimizationModeCounts`, and the top-level Hosted VLM/OCR report records
the selected mode. Analyzer declares `Pillow` as a runtime dependency because
region trimming and same-page atlas packing both need deterministic image
decoding outside Docling's optional dependency set.
Scheduler trace summaries also expose source-range queue wait and dispatch
start/end timing when the Rust scheduler provides those fields. These are
diagnostic-only report fields used to separate request latency from scheduler
admission and wave ordering; they do not change the Arrow OCR shard input or
result schemas.
A May 8, 2026 real-fixture diagnostic separated fixture coverage from hosted
OCR evidence. The Docling `2305.03393` page and full-paper probes validated
the real fixture path but routed only to backend-text source-page shards under
the risk-window planner. A forced DocLayNet region through
`baidu/qianfan-ocr-fast:free` preserved zero errors and stable order, but the
single hosted region request tailed at about `16.7s` while scheduler queue wait
was effectively zero. For that provider, the next latency lever is model or
provider choice, region payload shape, or scaffolded output constraints rather
than Rust queue admission.
The analyzer normalizes wrapping quotes from Hosted VLM/OCR environment values
before building OpenRouter headers. This protects local `.env` loaders that
materialize a value such as `"sk-or-v1-..."` with literal quotes. With that
normalization in place, the same real DocLayNet region completed through
`mistralai/ministral-3b-2512` with zero errors, stable order, one hosted
region request at about `7.4s`, and a force-refresh time around `9.3s`.
`qwen/qwen3-vl-8b-instruct` also preserved correctness on the same probe but
tailed around `27.3s`, so it remains a rejected candidate for this region.
Set
`WENDAO_HOSTED_VLM_OCR_PROVIDER=openrouter` to use OpenRouter's
OpenAI-compatible `/chat/completions` API instead of a local model server. The
OpenRouter preset defaults the base URL to `https://openrouter.ai/api/v1`,
reads `WENDAO_OPENROUTER_API_KEY` or `OPENROUTER_API_KEY`, accepts
`WENDAO_OPENROUTER_MODEL` when `WENDAO_HOSTED_VLM_OCR_MODEL` is not set, and
forwards optional
`WENDAO_OPENROUTER_HTTP_REFERER` and
`WENDAO_OPENROUTER_TITLE` attribution headers. The selected OpenRouter model
must support image URL chat content. When no OpenRouter model is configured,
the hosted smoke default is `baidu/qianfan-ocr-fast:free`, which is used only
to validate the cloud OCR path; current promotion evidence should pin the
faster validated `mistralai/ministral-3b-2512` candidate explicitly. Use the
[OpenRouter quickstart](https://openrouter.ai/docs/quickstart) when configuring
the hosted provider:

```bash
export WENDAO_HOSTED_VLM_OCR_PROVIDER=openrouter
export WENDAO_OPENROUTER_API_KEY=...
export WENDAO_OPENROUTER_MODEL=baidu/qianfan-ocr-fast:free
```

The analyzer does not load or quantize OCR2 weights in process. Use the
repository `fetch-models` and `start-ocr-backend` Justfile recipes only when a
local backend is desired; they fetch prebuilt community or official artifacts
and expose them through a vLLM OpenAI-compatible service:

```bash
just fetch-models
just start-ocr-backend
```

`fetch-models` also auto-selects the local artifact flavor. On macOS Apple
Silicon it defaults to the MLX-converted
`mlx-community/DeepSeek-OCR-2-bf16` artifact and links it to
`deepseek-ocr2-current`; on non-Mac hosts it defaults to the community FP8
vLLM artifact. Set `WENDAO_DEEPSEEK_OCR2_MODEL_FLAVOR=generic-vllm` or
`metal-mlx`, set `WENDAO_DEEPSEEK_OCR2_HF_REPO`, or pass `repo_id=...` when
validating a newer AWQ, GPTQ, GGUF, or MLX artifact. The Justfile recipes are
thin entrypoints over the analyzer CLI (`wendao-document-extract --ocr2-*`),
so backend automation stays package-owned and directly testable.
`start-ocr-backend` auto-selects the local platform runner. On macOS Apple
Silicon it selects `mlx-vlm`, which wraps the MLX-converted OCR2 model in a
repository OpenAI-compatible adapter. Install the shared local Metal/MLX
runtime and probe it with:

```bash
just install-vllm-metal
just probe-vllm-metal
```

The Metal runner exports the MLX/Metal defaults (`VLLM_METAL_USE_MLX=1`,
`VLLM_MLX_DEVICE=gpu`, and
`VLLM_METAL_MULTIMODAL_MODE=multimodal-native`) before invoking the vLLM CLI.
The `mlx-vlm` runner is the default on macOS because a direct local probe with
`mlx-community/DeepSeek-OCR-2-bf16` successfully returns OCR text through the
OpenAI-compatible `/v1/chat/completions` shape expected by the analyzer. The
lower-level `metal-vllm` runner remains available for explicit vLLM Metal
experiments, but it still gates DeepSeek-OCR-2 by default because current vLLM
Metal documentation lists multimodal vision/audio models as unsupported and
focuses the plugin on text-only language models. Set
`WENDAO_DEEPSEEK_OCR2_VLLM_METAL_ALLOW_UNSUPPORTED_VLM=1` only for explicit
frontier probes after validating the local vLLM Metal build. The current local
frontier probe reaches Metal engine initialization and resolves
`DeepseekOCR2ForCausalLM`, but the vLLM Metal path loads VLMs in text-only mode
and fails inside the `mlx-vlm` DeepSeek vision tower before serving an
endpoint. The immediate compatibility mismatch is that the OCR2 artifact uses a
DeepEncoderV2-style nested `vision_config.width`, while the current
`mlx-vlm` DeepSeek-VL2 loader expects a scalar vision width.
`vllm-omni` is not selected as the default OCR2 backend in this slice: its
current supported-models surface does not list DeepSeek-OCR-2, and it does not
address the macOS Metal `mlx-vlm` vision-loader mismatch above.
On non-Mac deployment hosts, `start-ocr-backend` selects `generic-vllm` and
starts the current vLLM OpenAI-compatible server shape for OCR2. It adds the
OCR2 n-gram logits processor and disables prefix caching by default, matching
the current vLLM OCR2 serving guidance. `WENDAO_DEEPSEEK_OCR2_VLLM_PACKAGE`
defaults to `vllm>=0.20.1`, so repository automation requires a vLLM line with
first-class OCR2 serving support instead of silently resolving to older
packages. Deployment hosts that need a known CUDA wheel or exact compatibility
set can still pin `WENDAO_DEEPSEEK_OCR2_VLLM_PACKAGE`, add helper packages
through `WENDAO_DEEPSEEK_OCR2_VLLM_WITH`, or override all server flags with
`WENDAO_DEEPSEEK_OCR2_VLLM_EXTRA_ARGS`.
`WENDAO_DEEPSEEK_OCR2_BACKEND_RUNNER=official-vllm` remains available as a
diagnostic compatibility runner for the official repository adapter, but it is
not the default because that adapter follows older vLLM internal APIs.
Default analyzer installs stay free of vLLM and model-runtime dependencies. The
`docling-vlm-deepseek-ocr` profile remains a Docling VLM comparator path, not
the default direct OCR2 implementation.
Production deployments can set
`WENDAO_DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS` directly when local
evidence shows a fixed override is appropriate for that machine profile, but
that override is not the default correctness or performance gate.
Pass `--fixture-suite milestone --only-fixture autosearch-2604.17337` for the
repo-owned OCR milestone input. Use `--fixture-suite explicit` with
`--extra-fixture NAME=PATH` only when benchmarking another audited real input
without preparing the full Docling fixture checkout. Milestone inputs that
define regression gates must come from a repo-tracked fixture path or another
explicit, auditable source path; `.data` downloads are cache material only. Add
`--fail-on-pdf-milestone-regression` to fail an OCR-positive milestone run when
it drops below the stored `2604.17337` precision/speed envelope. The guard is
evaluated after the JSON and Markdown reports are written so a failing run
still leaves evidence for diagnosis. Low `metricsResultChars` is recorded as a
milestone regression, not as a missing milestone observation.
The May 5, 2026 source-range endpoint-fanout profile kept the default Rust
source-range worker policy unset, used the default local endpoint auto fanout,
and observed a 21-page force latency of 18,969.021 ms with zero error rows,
21 OCR page blocks, 21 bbox blocks, and 103,984 OCR result characters. A
same-machine diagnostic four-endpoint run observed 15,811.373 ms, so endpoint
pool fanout is the current larger optimization lever; fixed source-range worker
counts remain diagnostic only. A later May 5, 2026 structure-order weighted
multi-shard run kept the same defaults and observed 16,364.335 ms force
latency, 92.843 ms shard-cache rebuild latency, 2.261 ms cache p95 latency,
zero error rows, 21 OCR page blocks, 21 bbox blocks, 21 metrics rows, 103,984
OCR result characters, and stable structure order. That run is still above the
sub-15 target, but it preserves the precision envelope while improving the
accepted default fanout baseline. The follow-up profile-aware fast-risk-window
run stayed on automatic endpoint fanout, kept the Rust source-range worker
override unset, and passed the stored milestone guard with 12,856.546 ms force
latency, 92.084 ms shard-cache reuse latency, 2.309 ms cache p95 latency, zero
error rows, 21 OCR page blocks, 21 bbox blocks, 21 metrics rows, 103,985 OCR
result characters, and stable structure order.
Use `--local-python-ocr-endpoint-count N` when a local benchmark should start
`N` Python Flight executors, including the primary document worker, and expose
that pool to the Rust scheduler for both full-document conversion and PDF OCR
shards. The default `auto` value keeps ordinary modes at one local endpoint and
fans out real `hybrid-page-ocr` Docling OCR by the machine profile so the Rust
source-range endpoint-pool scheduler is exercised without pinning a fixed worker
count. Use `--rust-document-extract-endpoint` or `--rust-pdf-ocr-endpoint` more
than once when a benchmark should target already-running Python Flight
executors. The script forwards those endpoints through
`WENDAO_DOCUMENT_EXTRACT_ENDPOINTS` and
`WENDAO_DOCUMENT_EXTRACT_PDF_OCR_ENDPOINTS`; when these flags are omitted,
Rust keeps using the normal document extraction endpoint as the single Python
executor.
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
uv run python tests/scripts/benchmark_wendao_document_extract.py \
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
uv run python tests/scripts/benchmark_wendao_document_extract.py \
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
endpoint. Sync `force=false` extraction also checks the Rust content-hash
artifact registry before calling Python; when a complete artifact already
exists, Rust mirrors it into the requested output directory and returns the
Arrow table without re-running Docling. Successful sync conversions are mirrored
back into that artifact registry for future reuse. First-time Docling
conversion is still CPU/model bound and should be handled with queueing and
worker-pool sizing in production. The Rust provider limits concurrently
running cold conversions with host `available_parallelism()` by default.
Full-profile Docling conversion runs in a child Python process by default, so
native crashes in heavyweight model paths fail the current extraction instead
of terminating the Arrow Flight worker. Set
`WENDAO_DOCUMENT_EXTRACT_FULL_ISOLATION=false` only for local diagnostics that
need inline stack traces. Set `WENDAO_DOCUMENT_EXTRACT_FULL_TIMEOUT_SECONDS`
when a deployment needs a stricter wall-clock bound than the default 900
seconds. Attachment-oriented `fast-text` extraction remains inline and uses the
lighter profile selected by the Rust or frontend request header.
`WENDAO_DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS` is a deployment upper bound:
set it when a host should run fewer simultaneous Docling conversions than its
CPU budget would otherwise allow. Jobs waiting for this Rust-owned capacity
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
Deployments can run multiple Python document workers and set
`WENDAO_DOCUMENT_EXTRACT_ENDPOINTS` to a comma- or whitespace-separated pool.
Rust round-robins full-document cache misses across that pool while keeping
content-hash deduplication, queue state, and artifact mirroring in the Rust
control plane. `WENDAO_DOCUMENT_EXTRACT_ENDPOINT` remains the single-endpoint
fallback and the default endpoint used when the pool is not configured.

For
[RFC: Polyglot Compute Orchestrator](../../../docs/rfcs/2026-05-04-polyglot-compute-orchestrator-rfc.md)
Phase 1.1, this package owns only the Python Docling Flight worker surface:
document conversion, OCR shard execution, Arrow resource rows, and the existing
`/analysis/document-extract` and `/analysis/pdf-ocr-shards` service routes.
Rust continues to own queueing, worker-budget selection, status metadata,
cache policy, and fallback coordination. The approved
`xiuxian-polyglot-orchestrator` crate may model Python-lane admission and
pressure evidence, but it does not add a second Docling wrapper service or
change the analyzer public route/schema contract.
The Rust-side harness profile verifies the owner bridges and Studio adoption
point; this Python package remains covered by its package-local Python project
harness and pytest suites.

## Beta Readiness

ready now:

1. offline scripted repo-search authoring
2. scripted PDF attachment analysis over Rust-returned attachment-search tables
3. Docling-backed local multi-format document extraction into Arrow resource rows
4. Wendao-facing Arrow Flight service entrypoint for document extraction
5. host-backed repo-search analysis with built-in or custom analyzers
6. generic rows/table/query analysis over Rust-returned data
7. package-local Python project harness coverage for syntax, project shape,
   modularity, pytest layout, and agent-policy drift

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
