---
type: knowledge
kind: readme
title: "xiuxian-wendao-attachments"
category: "package-docs"
status: "active"
author: Xiuxian Artisan Workshop
date: 2026-05-05T00:00-07:00
description: "Package README for Wendao attachment parsing, OCR shard contracts, and polyglot bridge ownership."
tags:
  - attachments
  - wendao
  - ocr
  - polyglot
metadata:
  title: "xiuxian-wendao-attachments"
  retrieval:
    saliency_base: 7.0
    decay_rate: 0.03
---

# xiuxian-wendao-attachments

`xiuxian-wendao-attachments` owns reusable attachment parsing, audit, and
artifact helpers for Wendao document surfaces.

The crate is intentionally not part of Wendao's default feature set. Expensive
or native document tooling stays behind explicit features so the main Wendao
gateway can depend on the crate without pulling PDF accelerators into default,
`studio`, or `performance` builds.

## Features

| Feature            | Purpose                                                              |
| ------------------ | -------------------------------------------------------------------- |
| `archive-audit`    | Enables tar and tar.gz member manifest audits for archive fixtures.  |
| `pdf-source-range` | Enables `lopdf` source-page manifests without PDFium.                |
| `pdf-render`       | Adds PDFium-backed region/page raster proofs on top of source range. |

## Boundaries

- `xiuxian-wendao-attachments` owns optional PDF accelerator dependencies.
  `lopdf` is the source-page intake dependency; `pdfium-render` is limited to
  explicit raster and region proof lanes.
- `xiuxian-wendao` owns the Studio gateway, Flight/REST routes, and production
  document extraction behavior.
- Production extraction still falls back to Python/Docling unless a later
  approved milestone wires a feature-gated fast or hybrid path into the live
  provider.
- The stable document extraction resource table remains Arrow-based. Browser
  JSON is only an edge serialization surface.

## Image Attachment Audit

The crate exposes a lightweight image preflight audit with no OCR dependency
and no image decoding dependency. It reads file metadata and bounded headers for
known Docling image suffixes, records MIME/format hints, and extracts
PNG/JPEG/BMP/GIF/WebP/TIFF dimensions when they are available directly from
the header.

This audit is a Rust control-plane signal. It can identify future candidates
for whole-image OCR cache keys, oversized image preflight, and later
crop/tile planning, but it does not replace Docling OCR or layout authority.
The Wendao performance probe can include these audit fields through the
`document-extract-attachment-audit` feature on `xiuxian-wendao`; default live
extraction still calls the existing Python/Docling path.

## Archive Attachment Audit

The optional `archive-audit` feature adds a non-extracting tar and tar.gz
member manifest audit. It records entry counts, regular file and directory
counts, total member sizes, lowercase suffix counts, image member counts, XML
member counts, the likely METS XML member, and the largest regular member.

This audit targets archive-backed Docling fixtures such as METS GBS. It is a
control-plane signal for future member-level cache keys and selective routing,
not a parser path. Live extraction still calls Python/Docling, and the stable
resource and structure Arrow schemas do not change.

## Source Page-Range Routing

Full-page OCR shards use direct source-PDF page-range manifests before falling
back to raster input. `lopdf` reads the page tree, Rust writes stable OCR shard
v1 rows with whole-page provenance, and Python/Docling performs OCR against the
original source PDF page range.
The crate also owns the lightweight source-page complexity profile used by
Studio's source-range scheduler and profile planner. The cached profile helper
keys entries by canonical path, file length, and modification timestamp so one
provider process can reuse page facts across force-refresh, shard-cache reuse,
and cache-hit probes without persisting transient profile state in Python.
The same profile now exposes conservative page classification facts for the
Docling-centered recovery lane. A page is marked as requiring Docling structure
authority when it has draw-object operations, rectangle operations, or at least
64 path operations. Current fast-profile structural-risk pages are marked as
OCR/VLM patch candidates only when region rendering can supply a patch surface.
Pages without structure authority and with backend text signals are marked as
text-shortcut eligible. Attachments does not schedule work or choose fallback;
it supplies stable PDF facts for Studio's scheduler and analyzer-side Docling
execution. The profile also exposes a conservative structure-cost estimate so
Studio can budget Docling page-range work from structural pressure rather than
fixed worker counts or page counts alone.
The stable OCR shard schema carries `ocrProfile` as the profile-selection
surface. Current profile identifiers include `docling-compatible-page-ocr-v1`,
`docling-fast-text-ocr`, `docling-backend-text-ocr-v1`,
`hosted-vlm-direct-ocr-v1`, and `docling-vlm-deepseek-ocr`. The backend-text
identifier is a source-range canary for PDF-native text extraction. The hosted
VLM identifier is the model-agnostic direct recovery profile; the Docling VLM
DeepSeek identifier is only a comparator profile. Adding those profile IDs
does not change the Arrow input or result column set.
The crate also exposes a small `lopdf` source-text helper for owner-side
backend-text canaries. Studio may use that helper for
`docling-backend-text-ocr-v1` rows, but the helper does not replace Docling's
fast-text top-up path or loosen the frozen benchmark character floor. When
Studio enables its local empty backend-text fail-fast diagnostic, the failure
decision remains Studio-owned; attachments only supplies the source-page text
read. The helper also exposes a per-page result form so one unextractable page
does not poison successful pages in the same source-range run; Studio remains
responsible for deciding whether a page-local failure becomes a failed OCR row,
a Python dispatch, or a precision-preserving full-document fallback.

`PdfPageRenderSelection::ShardFallbackPages` is intentionally high-recall in
the current source-range path. When no narrower safe region signal exists, it
selects every page instead of silently producing zero OCR shards. Preflight
failures, empty PDFs, region requests without configured regions, and partial
coverage still fall back to the Docling document path unless the opt-in route
can prove complete coverage.

## Structure Sidecar

The stable user-facing extraction result remains `_resources.arrow` with the
nine-column document resource schema. Internal merge, debug, benchmark, and
future UI code may also use `_structure.arrow`, an Arrow sidecar with schema
version `xiuxian_wendao.document_structure.v1`.

The sidecar records source content hash, block id, parent block id, page index,
block index, reading order key, block type, linked resource element id, content,
MIME type, status, engine, optional confidence, optional bounding box, and
provenance. Structure rows are sorted by page index, reading order key, block
index, and block id. This makes document order explicit and prevents OCR shard
completion order from becoming the reconstructed document order.

## Benchmark Reporting

The document extraction performance probe records both stable resource artifact
health and internal structure sidecar health. Each run reports `_resources.arrow`
and `_structure.arrow` row counts, resource/block type counts, OCR page and
region block counts, bbox block counts, reading-order sortedness, and artifact
read errors. These metrics are benchmark evidence only; they do not change the
stable user-facing `_resources.arrow` schema.

Mixed-format benchmark reports additionally group precision and speed signals
by attachment class. PDF, Office, image, structured text, web, table-data, XML,
subtitle, audio, Docling JSON, archive-backed, and unknown custom fixtures each
receive class-level error, structure, order, force-latency, cache-latency, and
speedup summaries. Image class summaries may also include Rust image audit
candidate counts, such as whole-image OCR cache candidates or oversized image
preflight candidates. Archive-backed class summaries may include Rust archive
audit counts for archive formats, member counts, XML/image member counts,
member suffixes, and member-manifest routing candidates. These remain
benchmark evidence only, not live parser authority over Docling-owned
semantics.

## PDFium Runtime

The `pdf-render` feature uses `pdfium-render`, which binds to a native PDFium
shared library at runtime. Live Wendao extraction does not require this library.
The source-page OCR path uses `pdf-source-range` and does not pull PDFium. Only
the opt-in raster or region render proof needs PDFium.

Use `WENDAO_PDFIUM_LIBRARY_PATH` to point at an existing PDFium shared library,
or run the benchmark script with `--prepare-pdfium-runtime` to fetch the pinned
`bblanchon/pdfium-binaries` runtime for the current platform into the project
cache before invoking the ignored cargo-test proof. Add `--require-pdfium` when
the proof must fail instead of recording a Docling fallback.
The native PDFium binding is process-global in `pdfium-render`, so this package
serializes the binding step before any PDFium-backed page or region render
work. Render callers may still schedule independent region chunks concurrently;
the one-time library binding itself remains guarded to avoid concurrent
initialization panics. Page and region render entrypoints retry up to three
transient `PdfiumLibraryInternalError` attempts with a short backoff by
reopening the document before reporting a fallback; parse failures and
unsupported inputs still fail fast into the existing Docling fallback path.

## OCR Contract

The `pdf-source-range` feature exposes the internal Arrow-only OCR worker
contract. Source page-range manifests can be projected into `_ocr_input.arrow`
using `xiuxian_wendao.pdf_ocr_shard_input.v1`; OCR workers return
`xiuxian_wendao.pdf_ocr_shard_result.v1`; successful, failed, or skipped OCR
results can then be projected back into the stable document resource schema.
The crate provides Arrow batch builders plus input and result decoders so the
Studio gateway can roundtrip worker requests and responses without JSON as an
internal contract.

The stable v1 shard input keeps page shards compatible while carrying region
provenance for later crop rendering: `shardType`, `regionIndex`,
`parentShardElementId`, `readingOrderKey`, and the region's pixel box within
the source page raster. This metadata is internal routing and merge state; it
does not change the stable `_resources.arrow` schema or switch production
extraction away from Docling.

The structure sidecar boundary also provides a strict Docling-baseline parity
helper for future golden benchmark lanes. A candidate structure must preserve
baseline page coverage, per-page text coverage, protected `table`, `formula`,
`image`, and `code` block counts, and sorted reading order before a faster
path can be accepted by a benchmark or later routing gate.

The Rust Studio provider controls Python OCR pressure with an adaptive OCR
scheduler and the internal `x-wendao-pdf-ocr-workers` Flight metadata header.
The scheduler derives its upper bound from available machine parallelism, while
`WENDAO_DOCUMENT_EXTRACT_PDF_OCR_WORKERS` is a deployment ceiling rather than a
fixed worker truth. Rust adjusts the active budget from queue wait, OCR latency,
errors, and timeout pressure, sends only the selected worker count to Python for
each exchange, and Python keeps output rows ordered by the input Arrow batch so
worker completion order cannot become document order.
The final worker/shard clamp for that scheduler is now routed through the
`xiuxian-polyglot-orchestrator` Docling schedule plan via the attachment
polyglot bridge. Studio still owns live permits, queue wait observation,
endpoint dispatch, and the Flight metadata header, while the orchestrator owns
the source-range auto worker sizing policy from attachment and Studio facts.

This is also the attachment-side ownership boundary for
[RFC: Polyglot Compute Orchestrator](../../../../docs/rfcs/2026-05-04-polyglot-compute-orchestrator-rfc.md)
Phase 1.1. `xiuxian-wendao-attachments` owns OCR shard scheduling evidence,
cache reuse, ordering validation, Docling fallback policy, and the stable OCR
shard input/result contracts. It does not own Python worker lifecycle,
runtime-level Flight admission, Julia profile readiness, or a standalone
polyglot scheduler. The approved `xiuxian-polyglot-orchestrator` crate owns the
pure Docling scheduling-plan contract, including the source-range worker sizing
policy. Attachments may call it through `pdf_ocr_shard_schedule_plan` or
`pdf_ocr_source_range_shard_schedule_plan` with attachment-owned pressure facts, then
translate the inert plan into attachment-local batch sizing, cache reuse,
ordering validation, and fallback behavior. The orchestrator must not duplicate
cache ownership, shard ordering authority, or Docling fallback policy.

The active `rust-lang-project-harness` profile marks `src/polyglot.rs` as the
attachment-side polyglot bridge. That profile records OCR shard evidence and
schedule-plan projection ownership without transferring cache, ordering, or
fallback authority to the orchestrator crate.

The Studio provider does not rely on Python response order for correctness.
Before projecting OCR rows into `_resources.arrow`, Rust validates every
result against the original shard id, source hash, page index, raster hash,
render profile, and OCR profile, then restores rows to the input shard order.
Successful OCR shard results are reusable through provider-local Arrow IPC
cache files under the document extraction cache root. The default root is
`$PRJ_CACHE_HOME/wendao-document-extract/ocr-shards`, with
`WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT` available for isolated
benchmarks or deployments. The cache has an oldest-first sweep policy with
`WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_BYTES` defaulting to 10 GiB, plus
optional `WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_ENTRIES`,
`WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_AGE_SECS`, and
`WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_SWEEP_INTERVAL_SECS` overrides. Cache
hits and misses are merged in the same input order, so whole-document retries,
forced runs, and future page or region Agent lookups can avoid repeating
Docling OCR for unchanged shards without changing the stable OCR shard v1
contract.

For full-page PDF OCR shards, the hybrid provider can prepare source-PDF
page-range shard manifests without rendering high-DPI PNG files first. The
manifest still carries content hash, reading order, a whole-page provenance
envelope, and stable OCR shard v1 fields, but Python reads the original
`sourcePath` page range through Docling and returns one row per page. Region
shards still use real PDFium crop rendering because their OCR input is a raster
region.
The source-PDF page-range selector is separate from the older render audit
selector. It avoids detector middleware, PDFium, and high-DPI raster work before
OCR: `lopdf` reads the page tree, Rust emits one source-range shard row per
selected page, and Docling remains the OCR authority over the original PDF page
range. PDFium-backed rendering remains available for region/raster proofs, but
it is not required for the full-page source-range hot path.
When Studio asks for hosted VLM risk-window recovery, attachments can
materialize a bounded set of selected page indices through the same PDFium page-shard
contract. This lets the provider keep ordinary pages on source-range rows while
rendering only the recovery pages that require VLM image input.
Rust may split one contiguous source-PDF page range into multiple contiguous
subranges when OCR worker permits are available. The source-range lane uses a
current adaptive OCR budget, machine-derived worker bound, and page count to let
the orchestrator select a conservative worker recommendation for Docling PDF
conversion. The current auto policy targets seven source pages per worker before
clamping that recommendation to live owner facts. Within that requested chunk
budget, Rust can profile the source PDF's decoded page content streams and
balance contiguous subranges by lightweight page complexity while preserving
reading order and cache-miss gaps.
When an opt-in profile planner creates mixed source-range profiles, such as
backend-text ordinary pages plus fast-text top-up pages, contiguous profile
runs remain hard chunk boundaries. Studio may expand the requested dispatch
budget to the run count, but the final worker admission still belongs to the
live Rust scheduler permits. During the hosted region `render-dispatch`
pipeline, Studio keeps that run-count floor for source top-up ranges even after
backend-text rows are satisfied locally, so non-contiguous precision top-up
ranges do not serialize behind hosted region recovery. Attachments supplies the
profile/run facts; Studio owns the final admission decision.
Attachments also owns deterministic hosted recovery region chunk ordering.
The default page-grouped chunking preserves page locality and stable reading
order. A benchmark-only region-grouped helper lets Studio test whether
single-region render chunks improve first-request latency, but Studio must
still normalize final rows through the same OCR shard order and precision
gates before accepting any result.
The benchmark-only all-region chunk helper is also rejected for the milestone
fixture: r80e preserved precision but delayed first hosted region dispatch to
`12528.588583 ms` and regressed force refresh to `21670.3075 ms`. Page-grouped
chunks remain the accepted default because they can dispatch each page's
regions without waiting for every recovery region in the document to render.
Two additional benchmark-only helpers sort page chunks by total recovery-region
area or by the largest single recovery region. They are diagnostic controls
only: the milestone r73/r74 probes preserved precision but failed to beat the
accepted OpenRouter force-refresh evidence, and r74 showed large-region render
completion, not page sort order, gating the hosted tail.
Studio also has an opt-in fast-text single-page split diagnostic, but it is not
promoted for the milestone fixture. The older r64b split probe preserved
precision but regressed force refresh to `23629.474667 ms`; the later r82b
split plus endpoint-affinity probe passed the locked promotion gate at
`12133.964875 ms` with `metricsResultChars=108788`, but it did not beat the
current `8201.568417 ms` OpenRouter evidence and page `5` still tailed at
`6817.215834 ms`.
Studio also owns endpoint-locality canaries for Docling fast-text top-up. The
attachments-owned shard facts still only describe source profile runs and page
identity; Studio decides whether benchmark prewarm should be limited to the
first `N` OCR endpoints and whether a single-page fast-text source-PDF chunk
should be sent to the first endpoint. The r70 canary preserved precision and
finished at `9636.47725 ms`; the later r96 canary reused the same
endpoint-local shape with a `4s` hosted hedge and finished at
`8201.568417 ms`. This does not change the attachments schema or make endpoint
selection an attachments responsibility.
The analyzer benchmark can prewarm multiple source pages before workers listen,
but this is also a Studio/analyzer readiness canary rather than an attachment
contract. The r75 page `5,11` endpoint `0-3` probe preserved precision but
regressed force refresh to `18784.875625 ms`, so broad multi-page prewarm is
not promoted for the milestone fixture.
Attachments also hardens PDFium page geometry for the live region path. If
PDFium reports `UnknownBitmapRotation` while reading page rotation metadata,
the package treats that single anomaly as rotation `0` so hybrid OCR can keep
its region-render contract; other rotation read errors still fail. This
hardening was added after r77b fell back before OCR metrics and was validated
by r78, which stayed on the hybrid path with zero error rows.
Studio can also route dense source-range top-up pages to full-page hosted
VLM/OCR with `WENDAO_DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_TOPUP=hosted-vlm`, but
that diagnostic is not promoted for the milestone fixture: force refresh
regressed to `35374.309 ms` and `metricsResultChars=91265` fell below the
frozen floor. The attachments-owned page facts therefore still mark those
top-up pages as requiring the Docling fast-text source-range path for this
fixture.
Those same page structure facts may be consumed by Studio's opt-in OCR profile
planner to choose fast versus accurate Docling source-page ranges, but
attachments remains the fact/shard contract owner only. Live worker dispatch,
cache reuse, and precision fallback decisions stay in Studio and the Python
Docling worker.
Structure parity treats `docling_json` and document wrapper rows as transport
metadata for text-coverage purposes, while still enforcing protected table,
formula, image, and code block counts. This lets full-document and page-range
Docling structure rows compare on real document structure instead of wrapper
payload placement.
Attachments also exposes the source-PDF page count helper used by Studio's
direct Docling structure recovery path. The helper is a source-page fact only:
Studio still owns page-range fallback chunking, dispatch timing, merge, cache,
and full-fallback promotion decisions.
`WENDAO_DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS` remains an explicit
benchmark override for source-range chunk count experiments, not a production
default.

The region crop proof accepts explicit PDF-point region requests and emits real
region shard PNGs plus the same `_ocr_shards.arrow`, `_ocr_input.arrow`, and
`_ocr_pending.arrow` artifacts used by page shards. The first proof renders a
page and crops the requested region from that raster, which reduces OCR payload
size without relying on unproven PDFium clip semantics. Automatic region
discovery and production routing are later milestones.

Use the benchmark script's `--pdf-render-shard-audit` lane with
`--pdf-render-selection region-shards` and one or more `--pdf-render-region`
arguments to drive the ignored cargo-test proof against explicit real-PDF
regions. This stays an audit surface; it does not change default extraction.

The same explicit region fixture syntax can drive the opt-in live
`hybrid-page-ocr` benchmark through `--hybrid-pdf-render-selection
region-shards`. In that mode, region OCR is supplemental: native text page
coverage remains required for the page, and full-page OCR shards continue to
replace only their selected pages. Studio may deescalate the parent page back
to the fast source-range profile and append the rendered region rows as hosted
VLM/OCR recovery inputs, but the Arrow OCR shard input/result schema remains
unchanged.
The Studio hybrid provider applies bounded semantic context padding to those
region requests before hosted VLM/OCR recovery so table captions, formula
surroundings, and nearby headers are not cropped away. The benchmark can override
that padding with `--rust-pdf-ocr-region-context-ratio`; `0` disables padding for
controlled experiments. If no explicit region fixture is configured, Studio may
opt into a benchmark-only `profile-risk-window` hosted VLM region planner that
builds a conservative content-band region from the parent page crop box for pages
already selected by the hosted VLM risk-window planner. The adjacent
`profile-risk-window-slices` and `profile-risk-window-adaptive` variants keep
the same source selection while splitting that content band for hosted VLM/OCR
tail-latency probes; adaptive splitting chooses the slice count from
attachment-owned source-page structure profiles plus estimated region pixel
area instead of applying a fixed three-way split. Hosted VLM/OCR render DPI is a
fidelity floor, not a speed knob: values below the default OCR DPI are ignored
by the provider.

When region shards pass through the hybrid provider, `_structure.arrow`
preserves their `readingOrderKey`, PDF-point bbox, confidence, shard identity,
parent shard identity, and raster/image provenance. Hosted VLM/OCR recovery
region rows use a `sentinel-sidecar-v1` patch protocol in structure provenance
after Studio binds each region back to the retained fast parent page.
`_resources.arrow` remains the stable nine-column result table; structure
metadata is kept in the sidecar so downstream consumers can restore document
order without expanding the user resource schema.
When Studio enables hosted VLM structural scaffold mode, its separate
`_hosted_vlm_region_scaffolds.json` sidecar remains Studio-owned and does not
change attachment render output columns or the OCR shard Arrow schema.
Attachments continue to own the raster, crop, source-pixel-box, source-page
profile, and order facts that Studio copies into that scaffold for analyzer-side
validation.

This is still opt-in infrastructure. No OCR worker is started by the
production Wendao gateway, and default document extraction does not consume
these rows. The Studio provider may consume them only when explicitly built
with `document-extract-pdf-source-range` and called with the `hybrid-page-ocr`
document extraction mode. Region raster shards additionally require
`document-extract-pdf-render`.

## Test Policy

This crate uses `rust-lang-project-harness` for the shared crate test-policy
gate from `src/lib.rs`. Unit tests live under `tests/unit/` and are mounted back
into the source modules with `#[path]` so focused `cargo test --lib` commands
still run the relevant tests. The current profile includes the polyglot bridge
surface used by the Studio OCR scheduler adoption path and records the
feature-gated regression command
`cargo test -p xiuxian-wendao-attachments --features pdf-source-range --lib polyglot`.
