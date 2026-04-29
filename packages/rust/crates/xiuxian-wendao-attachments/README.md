# xiuxian-wendao-attachments

`xiuxian-wendao-attachments` owns reusable attachment parsing, audit, and
artifact helpers for Wendao document surfaces.

The crate is intentionally not part of Wendao's default feature set. Expensive
or native document tooling stays behind explicit features so the main Wendao
gateway can depend on the crate without pulling PDF accelerators into default,
`studio`, or `performance` builds.

## Features

| Feature         | Purpose                                                                                   |
| --------------- | ----------------------------------------------------------------------------------------- |
| `pdf-inspector` | Enables the pinned upstream `firecrawl/pdf-inspector` audit and text-layer proof helpers. |
| `pdf-render`    | Enables PDFium-backed page rendering and OCR shard manifest helpers.                      |

## Boundaries

- `xiuxian-wendao-attachments` owns optional PDF accelerator dependencies such
  as `pdf-inspector` and `pdfium-render`.
- `xiuxian-wendao` owns the Studio gateway, Flight/REST routes, and production
  document extraction behavior.
- Production extraction still falls back to Python/Docling unless a later
  approved milestone wires a feature-gated fast or hybrid path into the live
  provider.
- The stable document extraction resource table remains Arrow-based. Browser
  JSON is only an edge serialization surface.

## Routing Diagnostics

PDF audit helpers expose detector confidence separately from direct fast-path
eligibility. `confidence` describes how strongly the inspector classified the
PDF type, while `fastPathScore` and `gateFailures` explain whether Rust text
extraction may bypass Docling. A high-confidence scanned PDF is therefore still
blocked from the direct text fast path and routed toward OCR or Docling
fallback.

Complex layout and OCR-required pages are routed to the hybrid shard fallback
candidate, not to unconditional full-document fallback. The hybrid proof mode
uses `PdfPageRenderSelection::ShardFallbackPages`: it renders only pages that
need raster OCR, renders all pages only for scanned/image PDFs without reliable
page hints, checks per-page markdown image placeholders when explicit OCR hints
are absent, and escalates complex hybrid candidates to page OCR when no
reliable region can be derived yet. Mixed PDFs without reliable page hints also
escalate to page OCR instead of silently selecting zero shards. Full Docling
fallback is reserved for preflight failures, encoding problems, empty
documents, or low-confidence PDFs that have no page-level shard signal.

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

## PDFium Runtime

The `pdf-render` feature uses `pdfium-render`, which binds to a native PDFium
shared library at runtime. Live Wendao extraction does not require this library.
Only the opt-in render proof needs it.

Use `WENDAO_PDFIUM_LIBRARY_PATH` to point at an existing PDFium shared library,
or run the benchmark script with `--prepare-pdfium-runtime` to fetch the pinned
`bblanchon/pdfium-binaries` runtime for the current platform into the project
cache before invoking the ignored cargo-test proof. Add `--require-pdfium` when
the proof must fail instead of recording a Docling fallback.

## OCR Contract

The `pdf-render` feature also exposes an internal Arrow-only OCR worker
contract. Rendered page manifests can be projected into `_ocr_input.arrow`
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

The Rust Studio provider controls Python OCR pressure with a global OCR worker
pool and the internal `x-wendao-pdf-ocr-workers` Flight metadata header. The
pool is sized from available machine parallelism, with
`WENDAO_DOCUMENT_EXTRACT_PDF_OCR_WORKERS` available for deployment override.
Rust splits OCR shard batches into scheduled chunks, sends only the acquired
worker count to Python for each exchange, and Python keeps output rows ordered
by the input Arrow batch so worker completion order cannot become document
order.

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
manifest still carries page geometry, content hash, reading order, and stable
OCR shard v1 fields, but Python reads the original `sourcePath` page range
through Docling and returns one row per page. Region shards still use real
PDFium crop rendering because their OCR input is a raster region.
Rust may split one contiguous source-PDF page range into multiple contiguous
subranges when OCR worker permits are available. The default target is
sublinear in the host worker budget to avoid over-parallelizing Docling PDF
conversion, and `WENDAO_DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS` can
override that source-range target for deployment benchmarking.

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
replace only their selected pages.

When region shards pass through the hybrid provider, `_structure.arrow`
preserves their `readingOrderKey`, PDF-point bbox, confidence, shard identity,
parent shard identity, and raster/image provenance. `_resources.arrow` remains
the stable nine-column result table; structure metadata is kept in the sidecar
so downstream consumers can restore document order without expanding the user
resource schema.

The `pdf-inspector` text helpers can also project native non-OCR pages into
per-page `text_page` rows. The Studio provider uses those rows only for the
explicit hybrid OCR mode and only when page coverage can be proven complete.

This is still opt-in infrastructure. No OCR worker is started by the
production Wendao gateway, and default document extraction does not consume
these rows. The Studio provider may consume them only when explicitly built
with `document-extract-pdf-render` and called with the `hybrid-page-ocr`
document extraction mode.

## Test Policy

This crate depends on `xiuxian-testing` and mounts the shared crate test-policy
harness from `src/lib.rs`. Unit tests live under `tests/unit/` and are mounted
back into the source modules with `#[path]` so focused `cargo test --lib`
commands still run the relevant tests.
