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
using `xiuxian_wendao.pdf_ocr_shard_input.v2`; OCR workers return
`xiuxian_wendao.pdf_ocr_shard_result.v1`; successful, failed, or skipped OCR
results can then be projected back into the stable document resource schema.
The crate provides Arrow batch builders plus input and result decoders so the
Studio gateway can roundtrip worker requests and responses without JSON as an
internal contract.

The v2 shard input keeps page shards compatible while carrying region
provenance for later crop rendering: `shardType`, `regionIndex`,
`parentShardElementId`, `readingOrderKey`, and the region's pixel box within
the source page raster. This metadata is internal routing and merge state; it
does not change the stable `_resources.arrow` schema or switch production
extraction away from Docling.

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
