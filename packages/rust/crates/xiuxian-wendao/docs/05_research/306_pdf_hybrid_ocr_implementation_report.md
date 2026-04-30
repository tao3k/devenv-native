# PDF Hybrid OCR Implementation Report

:PROPERTIES:
:ID: wendao-pdf-hybrid-ocr-report
:PARENT: [[../index|Wendao DocOS Kernel: Map of Content]]
:TAGS: research, document-extraction, pdf, ocr, arrow, docling, attachments
:STATUS: UPDATED
:VERSION: 1.3
:END:

## Executive Summary

Wendao evaluated Firecrawl `pdf-inspector` as an early Rust-side PDF detector,
but the active implementation has retired that dependency. The current PDF
acceleration path uses `lopdf` directly for source PDF page-tree intake, PDFium
only for explicit raster and region-render proof lanes, and Python/Docling as
the OCR and rich document-understanding authority.

The retirement is intentional. The useful capability for the current hot path
is PDF page-tree access, and Wendao already uses that lower-level dependency
directly. Keeping both the detector crate and its lower-level dependencies in
the active graph adds build and supply-chain surface without improving the
measured OCR-positive source-range path.

The stable user contract remains the nine-column `_resources.arrow` table:

1. `sourcePath`
2. `resourceType`
3. `resourcePath`
4. `pageIndex`
5. `caption`
6. `content`
7. `mimeType`
8. `status`
9. `elementId`

`_structure.arrow` remains an internal sidecar for merge, cache, benchmark, and
future UI structure order. The OCR shard contracts remain Arrow-only and stay
at `xiuxian_wendao.pdf_ocr_shard_input.v1` and
`xiuxian_wendao.pdf_ocr_shard_result.v1`.

## Current Architecture

The active hybrid PDF path is:

```text
PDF source
  -> Rust source page-range manifest with lopdf
  -> Arrow OCR shard input v1
  -> Python/Docling page-range OCR worker
  -> Arrow OCR shard result v1
  -> Rust order gate, shard cache, and resource projection
  -> _resources.arrow + _structure.arrow
```

Responsibilities:

- `xiuxian-wendao-attachments` owns reusable PDF helpers, OCR shard Arrow
  schemas, source page-range manifests, PDFium-backed region/raster proof
  helpers, and structure sidecar helpers.
- `xiuxian-wendao` owns the Studio document extraction provider, async job
  routing, OCR worker scheduling, shard cache, coverage gates, and benchmark
  lanes.
- `xiuxian-wendao-analyzer` owns Docling conversion and the internal
  `/analysis/pdf-ocr-shards` Arrow Flight exchange.

The default `sync` and `async` document extraction modes still use the existing
Docling extraction path. The hybrid source-page OCR path is explicit and
feature-gated behind `document-extract-pdf-source-range`; the narrower
`document-extract-pdf-render` feature is reserved for PDFium-backed raster and
region proof lanes.

## Retired Detector Evidence

The retired detector was useful as a research probe:

| Fixture / profile | Pages | Decision | Elapsed ms | Observation |
| ----------------- | ----: | -------- | ---------: | ----------- |
| `2206.01062.pdf` detect audit | 9 | `fast_rust_candidate` | 102.814 | Reliable text layer detected. |
| `2206.01062.pdf` analyze audit | 9 | `hybrid_page_ocr_candidate` | 796.432 | Layout complexity blocked direct text fast path. |
| `2604.17337` OCR-positive audit | 21 | zero OCR shards | 1095.312 | Detector signals missed OCR-positive pages. |

The key conclusion is that the detector was not enough for the real OCR-positive
case. Wendao now treats page-level OCR recall as a first-order precision gate:
source page-range fallback selects every page when no finer safe region signal
exists, instead of trusting a detector that may return zero shards.

## Source-Range OCR Milestone

The current source-PDF page-range path avoids high-DPI PNG rendering for
full-page shards. Rust reads the page tree with `lopdf`, emits one stable
whole-page source-range shard row per selected page, and sends the original PDF
page range to Python/Docling. Docling still performs OCR and Markdown export,
so the precision authority does not move into Rust.

Region shards are separate. They still use PDFium crop rendering because a
region shard is a raster OCR input. Region OCR remains opt-in and supplemental;
partial or region coverage falls back to Docling unless native text merge
support is explicitly available and coverage gates prove that no page is
missing.

This dependency boundary is strict: no-OCR and source-page OCR acceleration
must not pull PDFium. A Rust dependency is accepted only when it either reduces
Python-side work while preserving Docling precision, improves structural
provenance, or creates reusable shard/cache artifacts for later high-performance
query paths.

The provider restores OCR rows by original shard identity before projection.
It does not trust Python worker completion order as document order. Successful
OCR shard rows are cached as Arrow IPC entries below the whole-document cache,
so a forced extraction into a new output directory can reuse page or region
results without repeating Docling OCR.

## Performance Evidence

Real `2604.17337` OCR-positive PDF evidence from the current lane:

| Path class | Force / path ms | Cache p95 ms | Resource rows | Structure rows | OCR blocks | Reading order | Error rows |
| ---------- | --------------: | -----------: | ------------: | -------------: | ---------: | ------------- | ---------: |
| Original Docling baseline | 256037.271 | 6.645 | 21 | 21 | 21 | sorted | 0 |
| Rust scheduled per-page Docling OCR | 141100.000 | 7.850 | 21 | 21 | 21 | sorted | 0 |
| Source page-range OCR | 53850.000 | 7.850 | 21 | 21 | 21 | sorted | 0 |
| Adaptive source subranges | 48978.562 | 7.008 | 21 | 21 | 21 | sorted | 0 |
| Direct `lopdf` source selector | 49117.947 | 21.698 | 21 | 21 | 21 | sorted | 0 |
| Empty shard-cache fill | 62290.000 | 4.280 | 21 | 21 | 21 | sorted | 0 |
| Shard-cache forced reuse | 119.052 | 5.583 | 21 | 21 | 21 | sorted | 0 |
| Whole-document cache hit | 5.583 | 5.583 | 21 | 21 | 21 | sorted | 0 |

Interpretation:

- The original 256 s OCR-positive cold miss no longer repeats after page shards
  are known. Fresh output directories can rebuild from the shard cache in the
  low hundreds of milliseconds, and whole-document cache hits stay in the
  low-millisecond class.
- Unique OCR-heavy content is still dominated by Docling OCR. Source page-range
  OCR reduced the cold path to roughly the 49-54 s class without losing the 21
  ordered OCR blocks, but the next large gain requires safe region discovery or
  lower-cost Docling OCR profiles that keep precision parity.
- Retiring the detector removes the previous direct native text materialization
  proof from the active path. Text-only PDF acceleration should return only
  after Wendao has a direct, tested `lopdf`-owned text/page structure extractor
  or another stable Rust-native source that passes Docling parity gates.

## Precision Rules

The implementation must preserve or improve extraction quality:

1. Rust must not implement OCR.
2. Rust must not replace Docling's semantic understanding for complex tables,
   formulas, or uncertain layouts.
3. Source page-range OCR may send page shards to Python/Docling, but the final
   merge must use shard identity and reading-order keys, not completion order.
4. OCR must not overwrite reliable native text. Until native text merge support
   is present without the retired detector, partial and region coverage falls
   back conservatively.
5. Any uncertain route that cannot prove coverage emits a Docling fallback
   rather than a partial `_resources.arrow` artifact.

## Active Risks

| Risk | Impact | Mitigation |
| ---- | ------ | ---------- |
| Unique OCR-heavy cold miss remains expensive | First-time extraction can still take tens of seconds | Continue source-range batching, safe region discovery, shard cache reuse, and Docling profile measurement |
| Native text fast path was retired with the detector dependency | Some text-only PDFs lose the previous Rust-only proof path | Rebuild native text extraction directly over owned PDF primitives only after parity tests exist |
| Region OCR without native text merge can produce partial coverage | User-visible document order or coverage could degrade | Keep region mode opt-in and fallback on partial coverage |
| PDFium runtime mismatch | Raster proof output could differ across hosts | Keep PDFium confined to opt-in raster/region proof lanes and validate geometry |
| Shard cache growth | Project cache can grow under large OCR workloads | Keep oldest-first sweep and report cache size, entry count, and limits in benchmarks |

## Recommendation

Do not keep the retired detector crate in the active Wendao PDF extraction
dependency graph. The current hot path should stay on direct `lopdf` page-tree
intake, Arrow shard contracts, Rust-owned ordering and cache gates, and
Python/Docling OCR authority.

The next optimization milestone should focus on precision-safe region discovery
and native text structure extraction over dependencies Wendao already owns or
can justify independently. The acceptance bar remains the same: `totalErrorRows
= 0`, sorted `_structure.arrow`, stable `_resources.arrow`, and no precision
regression against Docling baselines.
