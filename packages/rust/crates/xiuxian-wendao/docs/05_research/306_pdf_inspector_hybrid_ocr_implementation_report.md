# PDF Inspector Hybrid OCR Implementation Report

:PROPERTIES:
:ID: wendao-pdf-inspector-hybrid-ocr-report
:PARENT: [[../index|Wendao DocOS Kernel: Map of Content]]
:TAGS: research, document-extraction, pdf, ocr, arrow, docling, attachments
:STATUS: PROPOSED
:VERSION: 1.0
:END:

## Executive Summary

Wendao should use [Firecrawl `pdf-inspector`](https://github.com/firecrawl/pdf-inspector)
as a Rust-side PDF accelerator and routing layer, not as a replacement for
Docling.

The target architecture is precision-preserving:

1. Rust performs deterministic PDF intake, classification, page/object
   inventory, page rendering, content hashing, cache lookup, and Arrow
   materialization.
2. Python/Docling remains the high-precision OCR and rich document
   understanding fallback.
3. Fast or hybrid extraction is enabled only when measurable quality gates prove
   that it does not reduce output quality against the Docling baseline.
4. OCR-required and complex-layout PDFs route to hybrid shard fallback first.
   Full Docling conversion remains the fallback for encoding problems, empty
   documents, preflight failures, or low-confidence PDFs without a safe
   page-level shard signal.

This keeps the existing Arrow Flight contract intact while moving expensive
all-or-nothing Python PDF conversion toward a Rust-owned page routing pipeline.

## Current Baseline

The current document extraction lane already solved duplicate cold misses with
the Rust-owned async queue, content-hash deduplication, cache reuse, and bounded
conversion permits. The remaining production risk is first-time conversion for
different cold documents, especially PDF, OCR-heavy image input, and complex XML
formats.

Observed real Docling no-audio strict benchmark summary:

| Fixture      |  Force ms | Cache p95 ms | Rows/request | Error rows |
| ------------ | --------: | -----------: | -----------: | ---------: |
| `pdf`        | 23357.738 |        2.870 |           13 |          0 |
| `image-png`  |  3116.928 |        2.728 |            3 |          0 |
| `image-tiff` |  4215.292 |        2.657 |            4 |          0 |
| `image-webp` |  2300.449 |        3.737 |            2 |          0 |
| `mets-gbs`   | 12205.210 |        5.041 |            5 |          0 |

Observed distinct cold-miss pressure summary:

| Metric                               | Value |
| ------------------------------------ | ----: |
| Distinct miss fixture count          |     4 |
| Distinct miss converter calls        |     4 |
| Total error rows                     |     0 |
| Max running conversions              |     4 |
| Max in-process running conversions   |     4 |
| Minimum available conversion permits |     0 |
| Max conversion duration ms           | 40743 |

Interpretation:

- Cache-hit latency is already in the low-millisecond range and is not the
  primary bottleneck.
- Duplicate cold miss risk is closed for identical content.
- Different cold documents can still saturate conversion permits when PDF,
  OCR, or complex layout conversion occupies Python workers for seconds to tens
  of seconds.

## Initial Detect-Audit Evidence

Milestone 1 added a feature-gated `pdf-inspector` detect/analyze audit lane and
ran it against the real Docling `2206.01062.pdf` fixture.

| Profile        | PDF type     | Pages | Confidence | Complex | Decision                    | Elapsed ms |
| -------------- | ------------ | ----: | ---------: | ------- | --------------------------- | ---------: |
| `detect_full`  | `text_based` |     9 |      1.000 | false   | `fast_rust_candidate`       |    102.814 |
| `analyze_full` | `text_based` |     9 |      1.000 | true    | `hybrid_page_ocr_candidate` |    796.432 |

This is the intended safety behavior. The cheaper detect-only pass shows that
the file has a reliable text layer, while the analyze pass finds multi-column
and table-heavy layout complexity and blocks the direct Rust text fast path.
The current router sends that case to the hybrid shard fallback candidate
instead of unconditional full-document fallback.

Milestone 2 now has an internal, feature-gated Rust text fast-path artifact
builder. It is disabled by default and is not wired into production
Flight/REST extraction. The builder writes the same stable document resource
Arrow schema and only produces `_resources.arrow` for PDFs that pass every
quality gate.

| Input                      | Gate result | Arrow rows | Markdown bytes | Decision                    | Elapsed ms |
| -------------------------- | ----------- | ---------: | -------------: | --------------------------- | ---------: |
| Minimal generated text PDF | `ok`        |          1 |            > 0 | `fast_rust_candidate`       |  unit test |
| Real Docling PDF fixture   | `fallback`  |          0 |              0 | `hybrid_page_ocr_candidate` |    711.710 |

The real fixture result is deliberately conservative: the document has a valid
text layer, but full analysis marks it as complex, so direct Rust text
extraction is not allowed. The router now keeps the file in the hybrid shard
fallback lane so later OCR/text merge work can operate at page or shard
granularity.

Milestone 3 moved the reusable PDF accelerator boundary into
`xiuxian-wendao-attachments`. The optional `pdf-inspector`, `pdfium-render`, and
`xiuxian-testing` integration now live there. `xiuxian-wendao` only exposes
feature-gated re-exports for Studio provider tests and performance lanes.

The render proof is still non-production. It builds typed page shard manifests,
projects internal `ocr_pending` rows using the stable nine-column resource
schema, and treats missing native PDFium runtime support as a Docling fallback
condition. Production sync and async document extraction still call the existing
Python/Docling worker.

The real render proof now has an explicit runtime preparation path in the
benchmark script. `--prepare-pdfium-runtime` downloads the pinned
`bblanchon/pdfium-binaries` runtime matching `pdfium-render`'s default Pdfium
API release for the current platform into the project cache, and
`--require-pdfium` turns the ignored proof lane into a hard failure when no page
shards are rendered. This is benchmark-only plumbing; it does not add PDFium to
default Wendao features or to production extraction.

All-pages render proof on the Docling `2206.01062.pdf` fixture:

| Status     | Decision                    | Pages | Shards | Elapsed ms |
| ---------- | --------------------------- | ----: | -----: | ---------: |
| `rendered` | `hybrid_page_ocr_candidate` |     9 |      9 |  24869.386 |

This result is a renderer capacity proof, not the optimized path. It deliberately
uses all-page 300 DPI PNG rendering to prove that the feature-gated Rust path can
write the internal shard manifest, OCR worker input, and pending OCR Arrow
resource rows when a native PDFium runtime is explicitly provided.

Shard-fallback routing proof on the same fixture:

| Status    | Decision                    | Selection              | Pages | Shards | Elapsed ms |
| --------- | --------------------------- | ---------------------- | ----: | -----: | ---------: |
| `skipped` | `hybrid_page_ocr_candidate` | `shard_fallback_pages` |     9 |      0 |    504.064 |

This is the optimized routing behavior for the current fixture. The file is
complex enough to block direct fast-path extraction, but `pdf-inspector` reports
no raster OCR pages, so Rust does not render all pages. Later hybrid extraction
can preserve the native text layer and invoke Python/Docling only for explicit
OCR or semantic shard work.

The benchmark harness now has an opt-in Docling PDF corpus mode for real
shard-fallback audits. It expands the default real Docling PDF fixture to the
additional PDFs under Docling's test corpus without changing the default real
suite or production extraction behavior.

Shard-fallback audit across 16 real Docling PDF corpus inputs:

| Metric                       | Value |
| ---------------------------- | ----: |
| PDF inputs                   |    16 |
| Total pages                  |    89 |
| Fast Rust candidates         |     5 |
| Hybrid page OCR candidates   |    11 |
| Rendered OCR shards          |     0 |
| Max per-file routing time ms |   623 |
| Error rows                   |     0 |

All 16 inputs completed as `skipped` for raster rendering. This is a useful
capacity signal: the broader Docling corpus still did not require page raster
OCR according to the current routing signals, so Rust avoided the previous
all-page render cost. It is not a positive scanned-PDF proof. The next real
coverage gap is to add opt-in scanned/image PDF fixtures under project data and
prove that `shard_fallback_pages` renders only the pages that actually need OCR.

The first external OCR-positive fixture was then added as an opt-in benchmark
input from project data, using arXiv `2604.17337`. It exposed an important
split between route recall and OCR execution:

| Profile                   | Pages | Shards | Elapsed ms | Status      |
| ------------------------- | ----: | -----: | ---------: | ----------- |
| `shard_fallback_pages`    |    21 |      0 |   1095.312 | `skipped`   |
| forced `all_pages` render |    21 |     21 |  61244.427 | `rendered`  |
| forced Docling shard OCR  |    21 |     21 | 241718.175 | `succeeded` |
| cache hit after OCR       |    21 |     21 |     12.515 | `succeeded` |

The OCR worker contract is therefore proven on a real PDF: Rust can render page
images, send Arrow OCR shard inputs through the Python analyzer, receive
Docling OCR shard results, and materialize 21 successful stable resource rows
with `totalErrorRows=0`. The routing gap is also clear: current
`pdf-inspector` signals did not mark any page as needing raster OCR, so the
optimized `shard_fallback_pages` mode would not invoke OCR for this fixture.
The next routing slice should improve OCR-page recall for image-bearing text
PDFs before enabling this path by default.

Milestone 4 starts that routing slice without changing production defaults.
Python/Docling extraction now writes an internal `_structure.arrow` sidecar
next to the stable `_resources.arrow` table. The sidecar uses
`xiuxian_wendao.document_structure.v1` and records page index, block index,
reading order key, block type, resource element id, content, engine,
confidence, bounding boxes when Docling exposes them, and provenance. The
stable nine-column `_resources.arrow` contract is unchanged.

The explicit Rust `hybrid-page-ocr` route also writes `_structure.arrow` after
successful hybrid extraction. Rust projects the stable resource rows into
ordered structure blocks and sorts by page index, reading order key, block
index, and block id before writing the sidecar. This keeps the user-facing
resource table stable while giving merge, debug, benchmark, and future UI code
a deterministic structural order source that does not depend on OCR shard
completion order.

The page router now has a high-recall OCR page hint pass for image-bearing
markdown emitted by `pdf-inspector`. If a hybrid candidate has no explicit
`pages_needing_ocr` hints, Rust checks per-page markdown for image placeholders
before deciding that no raster OCR pages are required. Scanned, image-based,
and mixed PDFs with no reliable page hints now escalate to page OCR rather than
silently selecting zero shards. Complex hybrid candidates also escalate to page
OCR when no reliable region can be derived yet; Docling remains the precision
fallback for cases the hybrid merge cannot prove complete.

Post-change rerun on arXiv `2604.17337`:

| Profile                | Pages | Shards | Elapsed ms | Status     |
| ---------------------- | ----: | -----: | ---------: | ---------- |
| `shard_fallback_pages` |    21 |     21 |  63889.030 | `rendered` |

This closes the immediate precision risk exposed by the previous `0` shard
selection: the router no longer misses OCR-bearing complex candidates. It does
not yet deliver the desired speedup for this fixture, because the safe fallback
is still full page OCR when no region can be derived. The next optimization
milestone is therefore region discovery and crop rendering, with Docling still
serving as the accuracy oracle.

The current proof slices add an Arrow-only OCR worker contract under
`xiuxian-wendao-attachments` plus a feature-gated Studio-side Flight client.
Rendered page manifests are projected into
`xiuxian_wendao.pdf_ocr_shard_input.v1`, while OCR outputs use
`xiuxian_wendao.pdf_ocr_shard_result.v1`. Rust can now send those batches to
the Python analyzer's internal `/analysis/pdf-ocr-shards` exchange route,
decode the returned OCR result rows, and project them back into the stable
document resource schema as `ocr_text`, `ocr_error`, or `ocr_skipped` rows. The
render proof still writes `_ocr_input.arrow` next to `_ocr_shards.arrow` and
`_ocr_pending.arrow`. The production-default `sync` and `async` providers still
ignore those rows, while the explicit feature-gated `hybrid-page-ocr` mode may
consume them.

The current region-shard slice stabilizes the internal OCR input contract.
`xiuxian_wendao.pdf_ocr_shard_input.v1` includes `shardType`, `regionIndex`,
`parentShardElementId`, `readingOrderKey`, and source-page pixel coordinates so
later crop rendering and structure-aware merge code can insert OCR blocks by
document order rather than worker completion order. Page shards still emit
`shardType=page` with full-page source pixel bounds, and the stable
`_resources.arrow` schema remains unchanged.

The next proof slice adds configured region crop rendering under
`xiuxian-wendao-attachments`. It accepts explicit PDF-point regions, renders the
owning page, crops each region into a real OCR shard PNG, and writes the same
manifest/input/pending Arrow artifacts used by page shards. This is not yet
automatic region discovery and does not switch production routing; it proves
that a future router can reduce OCR payload size while preserving page,
reading-order, and bbox provenance.

The audit report now records two additional routing diagnostics:

- `fastPathScore`: a conservative score for direct Rust text fast-path
  eligibility, not a general document quality score.
- `gateFailures`: machine-readable reasons that block the direct fast path,
  such as low confidence, non-text PDF type, OCR-required pages, complex
  layout, or encoding issues.

This distinction matters: a scanned PDF can have high detector confidence
because the classifier is sure it needs OCR, while still having a low fast-path
score because Rust text extraction must not bypass Docling/OCR for that file.

## `pdf-inspector` Fit

`pdf-inspector` exposes capabilities that match Wendao's next optimization
boundary:

- PDF type classification: `TextBased`, `Scanned`, `ImageBased`, and `Mixed`.
- Detection confidence.
- Per-page OCR routing through `pages_needing_ocr`.
- Detect-only, analyze, and full processing modes.
- Position-aware text extraction with page, coordinate, font, size, bold, and
  italic metadata.
- Layout complexity signals for tables and multi-column content.
- Encoding issue detection for fallback decisions.
- Markdown conversion for text-layer PDFs.
- Region-based text extraction APIs suitable for future hybrid OCR pipelines.

These capabilities map to a Rust routing layer:

```text
PDF bytes
  -> Rust classifier
  -> Rust quality gate
  -> Rust page/object inventory
  -> fast text extraction OR page-image OCR shards OR Docling fallback
  -> Arrow resource table
```

The important boundary is that `pdf-inspector` is not an OCR engine. It should
not be asked to understand scanned images, formulas, visual tables, or uncertain
layouts by itself.

## Precision Rules

The implementation must preserve or improve extraction quality. Rust must not
replace Docling's semantic understanding with approximate slicing.

Mandatory rules:

1. Rust must not infer semantic blocks for complex PDFs.
   It may split by page, PDF object, embedded image, text item, or explicit
   coordinate region only.
2. Every page shard must keep reversible provenance:
   source path, content hash, page index, page size, media box, crop box,
   rotation, render DPI, raster dimensions, raster hash, and coordinate mapping
   from raster pixels back to PDF points.
3. Text-layer extraction must keep native PDF text when the confidence gate
   allows it. OCR must not overwrite reliable text-layer content.
4. OCR input can be generated by Rust, but OCR output quality is judged against
   Docling baseline output before fast or hybrid paths are enabled by default.
5. Any uncertain routing decision without a safe page-level shard signal must
   fall back to full Docling conversion.

Recommended quality gates for an initial production profile:

| Signal            | Fast path condition                                       |
| ----------------- | --------------------------------------------------------- |
| PDF type          | `TextBased`                                               |
| Confidence        | `>= 0.90`                                                 |
| Encoding issues   | false                                                     |
| Pages needing OCR | empty                                                     |
| Complex layout    | false for direct fast path; hybrid shard fallback allowed |
| Tables/formulas   | fallback until table/formula parity is proven             |
| Renderer parity   | required for image/scanned hybrid paths                   |

## Target Architecture

### 1. Rust PDF Intake

Ownership:

- Package boundary: `xiuxian-wendao-attachments`.
- Gateway boundary: `xiuxian-wendao` depends on the attachments crate only
  through explicit PDF accelerator features and re-exports the internal test
  helpers needed by Studio provider and performance lanes.

Responsibilities:

- Read PDF bytes once.
- Compute or reuse content hash.
- Run `pdf-inspector` detect-only or analyze mode.
- Create a `PdfIntakeRecord` with page count, classification, confidence,
  pages needing OCR, layout complexity, and encoding issue flags.
- Include the inspector version and routing profile in the document extraction
  cache key.

### 2. Routing Decision

Decision matrix:

| Input class              | Rust action                                              | Python/Docling action                  | Default result         |
| ------------------------ | -------------------------------------------------------- | -------------------------------------- | ---------------------- |
| High-confidence text PDF | Extract markdown and page text in Rust                   | None                                   | Rust Arrow rows        |
| Mixed PDF                | Extract text-layer pages in Rust; shard OCR pages        | OCR selected pages only                | Hybrid Arrow rows      |
| Scanned/image PDF        | Render page images; shard pages                          | OCR page shards                        | OCR Arrow rows         |
| Low confidence PDF       | Diagnostics; shard only when OCR/page signal is explicit | Full Docling otherwise                 | Hybrid or Docling rows |
| Complex tables/formulas  | Page/object diagnostics; no direct text fast path        | Shard OCR or Docling semantic fallback | Hybrid or Docling rows |
| Encoding issues          | None beyond diagnostics                                  | Full Docling or OCR                    | Docling/OCR Arrow rows |

The routing result should be represented as a small Rust enum rather than hidden
stringly metadata:

```text
FastRustText
HybridPageOcr
FullDoclingFallback
FailedPreflight
```

### 3. Page Shard Stage

For scanned, image-based, and mixed PDFs, Rust should create page-level OCR
shards instead of forwarding the entire PDF to Python.

Each shard must include:

- page index
- source content hash
- raster artifact path or buffer handle
- raster MIME type
- raster hash
- render profile
- PDF-to-raster coordinate transform
- output row target identity

This stage should initially support page-level rendering only. Region-level
cropping can come after page-level parity is proven.

Renderer selection is separate from `pdf-inspector`. Candidate backends include
PDFium, MuPDF, or Poppler bindings. The selected backend must pass pixel
orientation, crop box, media box, rotation, transparency, and DPI tests before
OCR sharding becomes default.

### 4. OCR Worker Contract

The OCR worker should receive page-image shard manifests, not whole PDFs.

Input contract:

- Arrow table or Flight stream of OCR shard rows.
- Page image artifact reference or inline page image buffer.
- Page provenance fields.
- OCR profile id.

Output contract:

- Arrow table with page index, text, optional bounding boxes, confidence,
  status, error message, and provenance id.
- No JSON contract between Python and Rust for the primary extraction result.

Python can still host Docling OCR or another OCR engine. The key change is that
Python becomes an OCR/rich-understanding worker, not the owner of all PDF
preprocessing, queueing, deduplication, and result assembly.

### 5. Merge Stage

Rust merges:

- text-layer rows from `pdf-inspector`
- OCR rows from page shards
- Docling fallback rows when required
- resource artifacts
- status/error rows

The stable `_resources.arrow` schema remains the browser and downstream
contract:

- `sourcePath`
- `resourceType`
- `resourcePath`
- `pageIndex`
- `caption`
- `content`
- `mimeType`
- `status`
- `elementId`

Because the stable schema does not include bounding boxes, the first
implementation should either encode only minimal provenance in `elementId` or
write a sidecar provenance Arrow file for internal use. A future schema upgrade
can promote bounding boxes and engine confidence to first-class columns.

## Expected Benefits

### Latency

Expected effect by input class:

| Class             | Current behavior              | Target behavior                           | Expected result                                                            |
| ----------------- | ----------------------------- | ----------------------------------------- | -------------------------------------------------------------------------- |
| Text-layer PDF    | Full Docling PDF conversion   | Rust detect + Rust text extraction        | Seconds to sub-second for eligible PDFs                                    |
| Mixed PDF         | Full document conversion      | Rust text pages + OCR only selected pages | Proportional reduction by skipped OCR pages                                |
| Scanned/image PDF | Full PDF conversion in Python | Rust page rendering + page OCR shards     | Similar OCR cost, lower orchestration overhead and better parallel control |
| Cache hit         | Rust cache reuse              | Same                                      | No regression expected                                                     |

The strongest candidate benefit is high-confidence text PDFs. The current
measured PDF cold path is about 23.36 seconds. `pdf-inspector` is designed for
local text-PDF routing and extraction in the sub-second range, but Wendao should
accept this only after running local quality and latency benchmarks.

### Capacity

The current async dedup queue prevents identical documents from causing
duplicate conversion. The remaining capacity problem is many different cold
documents.

The hybrid architecture reduces Python worker pressure in three ways:

1. Text PDFs bypass Python entirely after quality gates pass.
2. Mixed PDFs send only OCR-required pages to Python.
3. Scanned PDFs are represented as independent page shards, enabling bounded
   worker scheduling, page-level retry, and page-level cache reuse.

The direct production effect is fewer long-running Python conversion permits
occupied by avoidable PDF work.

### Precision

The architecture can improve precision if implemented conservatively:

- Native text-layer content is preserved instead of being re-OCRed.
- OCR is focused on pages or regions that actually need visual recognition.
- Page shards carry exact coordinate transforms, which improves merge order and
  provenance.
- Low-confidence cases preserve current Docling behavior unless there is an
  explicit OCR/page-level shard signal that can be processed without losing
  provenance.
- Multiple OCR engines can later be merged by confidence and bounding box
  without changing the Arrow Flight contract.

### Cost

The architecture reduces:

- OCR calls for text-layer pages.
- Python process CPU time for simple PDFs.
- duplicated page processing through page-hash caching.
- user-facing queue delay during cold bursts.

It may add:

- Rust PDF renderer dependency complexity.
- raster artifact storage.
- additional parity tests for rendering and coordinate mapping.

## Implementation Milestones

### Milestone 1: Detect-Only Audit

Scope:

- Add optional `pdf-inspector` dependency behind a feature flag.
- Build a Rust wrapper for detect-only and analyze modes.
- Record version, profile, PDF type, confidence, pages needing OCR, layout
  complexity, and encoding issue flags.
- Run local benchmark comparison against existing real PDF fixtures and Docling
  baseline output.

Acceptance:

- No extraction behavior changes.
- No default dependency exposure until supply-chain policy is accepted.
- Report includes detect latency, routing decision, and fallback reason.

### Milestone 2: Text PDF Fast Path

Scope:

- Enable Rust markdown extraction for high-confidence text PDFs only.
- Convert output into existing `_resources.arrow` rows.
- Keep Docling fallback on every failed quality gate.

Acceptance:

- Simple generated text PDF extraction writes one stable document row and an
  Arrow IPC `_resources.arrow` artifact.
- Complex real fixture analysis falls back before writing any Rust extraction
  artifact.
- Cache profile material records the inspector fork branch pin and routing
  profile.
- Cache-hit latency does not regress.
- Fast path can be disabled by config.

### Milestone 3: Page Rendering and OCR Shard Manifest

Scope:

- Select and integrate one Rust PDF renderer.
- Render page images with explicit render profile.
- Produce page shard manifests and `ocr_pending` Arrow rows.
- Do not connect real OCR yet.
- Keep renderer and inspector dependencies in `xiuxian-wendao-attachments`, not
  in the main Wendao gateway crate.

Acceptance:

- Rotation, crop box, media box, DPI, and raster dimensions are proven by tests.
- Shards are content-addressed.
- Rendering errors emit a fallback report and keep live extraction on the
  existing Python/Docling path.
- Default Wendao, `studio`, and `performance` features do not pull renderer
  dependencies.

### Milestone 4: Page-Level OCR Worker

Scope:

- Send page shard manifests to the Python OCR worker over Arrow Flight.
- Receive OCR result rows over Arrow.
- Merge OCR rows into `_resources.arrow`.

Acceptance:

- Python does not receive whole PDFs for eligible page-shard OCR mode.
- Failed pages can retry without reconverting successful pages.
- OCR output is page-order stable.

Current implementation status:

- The Python analyzer document service now exposes an internal
  `/analysis/pdf-ocr-shards` Flight `do_exchange` route.
- The route accepts `xiuxian_wendao.pdf_ocr_shard_input.v1` Arrow batches and
  returns `xiuxian_wendao.pdf_ocr_shard_result.v1` Arrow batches.
- The default worker returns deterministic `skipped` rows until a real OCR
  worker is injected. This keeps the production `/analysis/document-extract`
  sync and async paths unchanged while proving the shard-level Arrow handoff.
- Feature-gated Rust code now proves the reverse side of that handoff: it sends
  OCR shard input batches through Flight `do_exchange`, decodes the returned
  OCR result rows, and materializes stable resource rows.
- The Studio document extraction provider now has an explicit
  `hybrid-page-ocr` Flight metadata mode behind
  `document-extract-pdf-render`. Default `sync` and `async` extraction remain
  unchanged. The opt-in route renders eligible PDF OCR shards, sends
  `_ocr_input.arrow` rows to the Python OCR shard exchange, writes the returned
  OCR rows to `_resources.arrow`, and marks the cache complete only when every
  page is represented by OCR shard output.
- Milestone 4 did not mark partial-page OCR output complete. Mixed or complex
  PDFs that would produce only a subset of page OCR rows fell back to full
  Docling so the stable resource table was never marked complete with missing
  non-OCR pages.
- The Milestone 5 merge primitive is now underway: `pdf-inspector` can project
  native non-OCR pages into per-page `text_page` rows, and the explicit
  `hybrid-page-ocr` provider route can merge those rows with successful OCR
  shard rows into one page-sorted stable resource batch. Any skipped, failed,
  duplicate, out-of-range, or incomplete page coverage path still falls back to
  full Docling.
- Milestone 6 adds the first real shard OCR worker option on the Python side:
  the document service keeps skipped shard rows as the default, but
  `--pdf-ocr-worker docling` converts Rust-rendered page images through
  Docling and returns `succeeded` or `failed` OCR shard rows over the existing
  Arrow Flight exchange. The benchmark harness can now drive
  `hybrid-page-ocr` through the Rust provider with the PDF render feature
  selected explicitly.
- The cold-miss worker scheduling slice keeps the same Arrow v1 OCR shard
  contracts but changes the execution shape: Rust owns a global PDF OCR worker
  pool, splits OCR shard batches into scheduled chunks, and sends
  `x-wendao-pdf-ocr-workers` with the acquired worker count for each Python
  exchange. The pool is sized from available parallelism, with
  `WENDAO_DOCUMENT_EXTRACT_PDF_OCR_WORKERS` available as a deployment override.
  Python keeps result rows in input order and isolates failures per shard.
- Full-page PDF OCR shards now try Docling `sourcePath` page ranges before
  rendered images. This preserves Docling's native PDF backend behavior for
  page shards while keeping rendered images as the fallback for region shards,
  page-range failures, and raster tests.
- The source-PDF page range path now avoids high-DPI PNG rendering for
  full-page shards. Rust prepares stable OCR shard v1 rows from PDF page
  geometry and sends the original `sourcePath` to Python; Python converts one
  contiguous Docling page range and exports page-scoped Markdown back into one
  result row per shard. Region shards still use PDFium crop rendering.
- Real `2604.17337` proof after this change:
  - previous full OCR-positive cold miss baseline: about `256s`
  - Rust scheduled per-page Docling OCR: `141.1s`
  - source page-range batch without page PNG rendering: `53.85s`
  - cache hit: `7.85ms`
  - resource rows: `21`, error rows: `0`
  - `_structure.arrow`: `21` rows, `21` OCR page blocks, reading order sorted
- The follow-up shard cache and order-gate slice makes that source page-range
  path reusable at page or region granularity. Rust now validates OCR result
  rows against the original shard input identity and reorders them to the input
  shard order before `_resources.arrow` projection. Successful OCR rows are
  stored as Arrow IPC shard cache entries under the document extraction cache
  root, and cache hits are merged with live misses in the same input order.
  This keeps `_structure.arrow` ordering independent from Python worker
  completion order and gives future Agent page/region lookups a precise cache
  layer below the whole-document `_resources.arrow` artifact.
- Real `2604.17337` shard-cache proof after the cache was empty:
  - shard-cache population force run: `62.29s`
  - persistent OCR shard Arrow cache entries: `21`
  - new-output forced run using shard cache: `286ms`
  - whole-document cache hit after that run: `4.28ms`
  - resource rows: `21`, error rows: `0`
  - `_structure.arrow`: `21` rows, `21` OCR page blocks, reading order sorted
- The benchmark harness now has `--shard-cache-reuse-probe` so JSON and
  Markdown reports carry `shardCacheReuseForceMs` directly. With the populated
  `2604.17337` shard cache, the report-field proof measured force `319ms`,
  shard-cache forced reuse `162ms`, whole-document cache hit `4.67ms`, 21 OCR
  rows, zero error rows, and sorted structure order.
- Shard cache capacity is now governed by an oldest-first Rust sweep. The
  default OCR shard cache budget is 10 GiB under the document extraction cache
  root, with optional max-bytes, max-entries, max-age, and sweep-interval
  deployment overrides. Benchmark JSON and Markdown reports include
  `ocrShardCache` file count, total bytes, and configured limits so capacity
  decisions are visible next to latency and structure metrics.
- The real hybrid benchmark proof then exposed a text-only hybrid candidate:
  `pdf-inspector` classified the real `2206.01062.pdf` fixture as a
  `hybrid_page_ocr_candidate`, but the shard selector found no raster OCR
  pages. The provider now materializes complete native `text_page` rows for
  that case instead of falling back to full Docling. On the same real PDF, the
  force path improved from about 41.9 seconds to about 1.38 seconds, cache-hit
  latency stayed around 3-5 ms, and `totalErrorRows` stayed 0.
- Region crop proof infrastructure is available behind the existing render
  audit lane. The benchmark script can pass explicit PDF-point region fixtures
  into the ignored cargo test, which calls the Rust region crop helper and
  writes the same OCR shard Arrow artifacts as page rendering. This keeps
  region OCR evidence reproducible without changing production sync or async
  extraction.
- Explicit region fixtures can now also drive the opt-in
  `hybrid-page-ocr` benchmark provider. `region_shards` remains disabled unless
  requested by benchmark configuration, and region OCR is validated as
  supplemental to native text page coverage rather than as a full-page
  replacement. Full-page shard coverage gates remain strict.
- Hybrid region OCR now carries ordering and provenance into `_structure.arrow`.
  The sidecar records the shard `readingOrderKey`, region bbox, confidence,
  parent shard id, raster hash, and image path while keeping `_resources.arrow`
  on the stable nine-column schema.
- The performance probe now reports artifact health for both `_resources.arrow`
  and `_structure.arrow`. Reports include resource row counts, structure row
  counts, OCR page and region block counts, bbox block counts, reading-order
  sortedness, and artifact read errors. This makes structure precision part of
  the benchmark output instead of a manual post-run inspection.

Final current-branch benchmark evidence:

| Fixture / path class                  |   Force ms | Cache p95 ms | Resource rows      | Structure rows | OCR blocks | Reading order | Error rows |
| ------------------------------------- | ---------: | -----------: | ------------------ | -------------: | ---------: | ------------- | ---------: |
| `pdf-multi-page` fast text candidate  |    367.796 |        8.055 | 5 `text_page`      |              5 |          0 | sorted        |          0 |
| `normal_4pages` complex fallback      |   7204.978 |        4.793 | Docling rich rows  |              5 |          0 | sorted        |          0 |
| arXiv `2604.17337` OCR-positive proof | 256037.271 |        6.645 | 21 `ocr_text` rows |             21 |         21 | sorted        |          0 |

Milestone interpretation:

- The Rust fast path now meets the expected performance class for eligible text
  PDFs: `pdf-multi-page` materializes five native `text_page` rows in about
  368 ms and cache hits stay below 10 ms p95.
- Precision fallback is still working: `normal_4pages` contains rich image and
  table structure, so the route preserves Docling output instead of emitting an
  approximate Rust text-only result.
- OCR recall and structure order are proven on `2604.17337`: the route emits 21
  page OCR blocks, every block has bbox provenance, `_structure.arrow` is sorted
  by document order, and `totalErrorRows=0`.
- The remaining unmet performance target is OCR-heavy cold miss latency when no
  reliable region signal exists. That path still performs full-page shard OCR
  and measured about 256 seconds for the 21-page external arXiv fixture. The
  next optimization milestone must derive safe region/crop shards, then compare
  the cropped result against the Docling page OCR baseline before enabling it.

### Milestone 5: Hybrid Mixed-PDF Pipeline

Scope:

- Combine Rust text-layer extraction and selected-page OCR.
- Preserve page order and provenance.
- Keep full Docling fallback available for complex tables, formulas, and
  uncertain layout, but prefer shard fallback whenever routing can preserve
  page-level provenance.

Acceptance:

- Mixed-PDF output is not worse than Docling baseline for coverage and ordering.
- Page OCR count is lower than page count for eligible mixed PDFs.
- Concurrent cold-miss pressure shows lower Python permit occupancy.

Current implementation status:

- Native per-page text rows are available as stable `text_page` resource rows.
- The explicit `hybrid-page-ocr` route can merge native text pages with OCR
  shard rows only when every page is covered and every OCR row succeeded.
- Text-only hybrid candidates with zero OCR shards now complete directly from
  Rust-native text-page rows when page coverage is complete; coverage failure
  still falls back to full Docling.
- The Python analyzer service can provide real Docling image OCR for those
  shard rows when explicitly started with `--pdf-ocr-worker docling`; the
  default worker still returns `skipped` rows.
- Default `sync` and `async` extraction are unchanged.

## Test and Benchmark Plan

Unit tests:

- Fake inspector routes text, scanned, image, mixed, low-confidence, encoding
  issue, and complex-layout cases.
- Fake renderer proves page index, dimensions, rotation, and hash propagation.
- Fake OCR proves page-level retry, failure row, and merge ordering.
- Fake and Docling-injected shard workers prove skipped, succeeded, failed, and
  empty-output paths without requiring live OCR models in unit tests.
- Arrow schema tests prove `_resources.arrow` remains stable.

Real tests:

- Text PDF benchmark against Docling output.
- Scanned/image PDF page-rendering parity.
- Mixed PDF page routing and OCR shard count.
- Opt-in `hybrid-page-ocr` benchmark with Docling shard OCR enabled.
- Table-heavy and formula-heavy fallback tests.
- Large PDF detect-only latency test.

Stress tests:

- Duplicate same-PDF cold miss: conversion or OCR shard generation count remains
  one.
- Distinct text-PDF cold miss burst: Python conversion permit usage stays low.
- Distinct mixed-PDF cold miss burst: OCR page count and permit occupancy are
  reported.
- Cache-hit path remains in the low-millisecond range.

Quality metrics:

- text coverage ratio versus Docling baseline
- page order agreement
- OCR page routing precision
- fallback rate
- rendering parity failures
- table/formula fallback correctness
- p50/p95/max latency
- rows/sec
- Arrow IPC bytes
- Python conversion permit occupancy

## Risks

| Risk                            | Impact                     | Mitigation                                                             |
| ------------------------------- | -------------------------- | ---------------------------------------------------------------------- |
| `pdf-inspector` API instability | Integration churn          | Keep behind optional feature and narrow wrapper                        |
| Git dependency policy           | Build reproducibility risk | Pin, mirror, or vendor only after license and lint review              |
| Renderer mismatch               | OCR quality loss           | Pixel and coordinate parity tests before default enablement            |
| Complex layout degradation      | Precision loss             | Conservative fallback to Docling                                       |
| Table/formula loss              | Retrieval quality loss     | Keep full Docling for table/formula-heavy PDFs until parity is proven  |
| Extra artifacts                 | Storage growth             | Content-addressed artifact root and cleanup policy                     |
| OCR worker variability          | Non-deterministic output   | Record OCR profile and model version in cache key                      |
| Shard completion order drift    | Wrong document order       | Rust restores OCR rows to shard input order before resource projection |
| Repeated shard OCR              | Cold retry latency         | Successful OCR results are cached as Arrow IPC shard rows              |

## Recommendation

Adopt `pdf-inspector` as a gated Rust PDF intake and routing accelerator.

Do not adopt it as the document understanding authority. The authority model
should remain:

- Rust owns transport, cache, deduplication, scheduling, PDF preflight, page
  sharding, Arrow materialization, and merge.
- Python/Docling owns high-precision OCR and rich understanding when required.
- Docling remains the fallback oracle until local quality benchmarks prove a
  narrower fast or hybrid route is safe.

The first implementation slice should be detect-only and non-mutating. It
should produce a benchmark report and routing audit before any extraction path
changes.
