# PDF Hybrid OCR Implementation Report

:PROPERTIES:
:ID: wendao-pdf-hybrid-ocr-report
:PARENT: [[../index|Wendao DocOS Kernel: Map of Content]]
:TAGS: research, document-extraction, pdf, ocr, arrow, docling, attachments
:STATUS: UPDATED
:VERSION: 1.12
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

The structure sidecar has a reusable strict baseline parity helper for golden
benchmark lanes. Before a future region or native-text fast path can claim
parity with Docling, its candidate structure must preserve baseline page
coverage, per-page text coverage, protected table/formula/image/code block
counts, and sorted reading order. The benchmark harness can now point at a
golden baseline artifact root and report the parity summary or exact parity
error alongside the usual `_resources.arrow`, `_structure.arrow`, and
`_metrics.arrow` fields. The same harness can generate sync/full-Docling
baseline artifacts before candidate probes so parity comparisons are tied to
the exact fixture set being measured.

The next implementation slice is verification infrastructure, not a broader
parser replacement. Rust stays the deterministic control plane: acceleration,
scheduling, caching, slicing, ordering, merging, and validation. Docling
remains the OCR, layout, and semantic document-understanding authority. Future
VLM integration is limited to hard-region enhancement after Docling baseline
comparison exists.

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

The hardened acceptance shape adds one more internal artifact:

```text
OCR shard result/projection
  -> Rust precision gate
  -> _metrics.arrow + _resources.arrow + _structure.arrow
```

`_metrics.arrow` is internal observability. It does not change the user-facing
resource table, the structure sidecar, or the OCR shard input/result v1
contracts.

Responsibilities:

- `xiuxian-wendao-attachments` owns reusable PDF helpers, OCR shard Arrow
  schemas, source page-range manifests, PDFium-backed region/raster proof
  helpers, and structure sidecar helpers.
- `xiuxian-wendao` owns the Studio document extraction provider, async job
  routing, adaptive OCR scheduling, shard-level in-flight deduplication, shard
  cache, coverage gates, and benchmark lanes.
- `xiuxian-wendao-analyzer` owns Docling conversion and the internal
  `/analysis/pdf-ocr-shards` Arrow Flight exchange.

The default `sync` and `async` document extraction modes still use the existing
Docling extraction path. The hybrid source-page OCR path is explicit and
feature-gated behind `document-extract-pdf-source-range`; the narrower
`document-extract-pdf-render` feature is reserved for PDFium-backed raster and
region proof lanes.

## Retired Detector Evidence

The retired detector was useful as a research probe:

| Fixture / profile               | Pages | Decision                    | Elapsed ms | Observation                                      |
| ------------------------------- | ----: | --------------------------- | ---------: | ------------------------------------------------ |
| `2206.01062.pdf` detect audit   |     9 | `fast_rust_candidate`       |    102.814 | Reliable text layer detected.                    |
| `2206.01062.pdf` analyze audit  |     9 | `hybrid_page_ocr_candidate` |    796.432 | Layout complexity blocked direct text fast path. |
| `2604.17337` OCR-positive audit |    21 | zero OCR shards             |   1095.312 | Detector signals missed OCR-positive pages.      |

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
results without repeating Docling OCR. The Rust scheduler also keeps
process-local in-flight shard ownership, so concurrent misses for the same OCR
shard wait on one live Python request instead of issuing duplicate Docling OCR.
The scheduler can also target an explicit Python OCR endpoint pool through
`WENDAO_DOCUMENT_EXTRACT_PDF_OCR_ENDPOINTS`. The default remains the existing
`WENDAO_DOCUMENT_EXTRACT_ENDPOINT`, so current deployments keep the same single
Python worker behavior. When a deployment or benchmark supplies multiple OCR
Flight endpoints, Rust round-robins live OCR requests and source-range chunks
across that pool while retaining the same Arrow shard input/result contracts,
ordering gate, shard cache, and Docling OCR authority.

## Performance and Precision Assessment

The current benchmark evidence separates four questions that should not be
mixed:

1. How fast is the first OCR-heavy cold miss?
2. How fast is a repeated document or repeated page shard?
3. Does the optimized path preserve Docling-quality OCR coverage and document
   order?
4. Which Rust dependency is justified by the measured gain?

### OCR-Positive PDF Latency

The real OCR-positive fixture is arXiv `2604.17337`, with 21 OCR-positive
pages. The original full-Docling cold path was observed in the 241-256 second
range across benchmark runs. The latest comparable full-Docling run recorded
241718.175 ms; the earlier archived proof recorded 256037.271 ms.

| Benchmark class                |   Force ms | Cache p95 ms | Shard-cache rebuild ms | Resource rows | Structure rows | OCR page blocks | Bbox blocks | Order  | Error rows |
| ------------------------------ | ---------: | -----------: | ---------------------: | ------------: | -------------: | --------------: | ----------: | ------ | ---------: |
| Full Docling baseline          | 241718.175 |       12.515 |                    n/a |            21 |            n/a |             n/a |         n/a | n/a    |          0 |
| Rust scheduled per-page OCR    | 141110.984 |       10.577 |                    n/a |            21 |             21 |              21 |          21 | sorted |          0 |
| Source page-range manifest     |  53848.876 |        7.850 |                    n/a |            21 |             21 |              21 |          21 | sorted |          0 |
| Parallel source page-range     |  45094.143 |       15.219 |                200.460 |            21 |             21 |              21 |          21 | sorted |          0 |
| Adaptive source subranges      |  48978.562 |        7.008 |                130.841 |            21 |             21 |              21 |          21 | sorted |          0 |
| Direct `lopdf` source selector |  49117.947 |       21.698 |                197.023 |            21 |             21 |              21 |          21 | sorted |          0 |
| Warm shard-cache reuse         |    286.376 |        4.280 |                    n/a |            21 |             21 |              21 |          21 | sorted |          0 |

The table above keeps the historical milestone progression. A follow-up
source-range worker sweep on the same fixture used isolated force-only runs and
kept the same precision shape:

| Source-range override |  Force ms | Cache p95 ms | Rust scheduler elapsed ms | Resource rows | Structure rows | OCR page blocks | Bbox blocks | Order  | Error rows |
| --------------------: | --------: | -----------: | ------------------------: | ------------: | -------------: | --------------: | ----------: | ------ | ---------: |
|                     1 | 21373.000 |        2.646 |                 21248.243 |            21 |             21 |              21 |          21 | sorted |          0 |
|                     2 | 20183.691 |        2.457 |                 20085.701 |            21 |             21 |              21 |          21 | sorted |          0 |
|                     4 | 19442.132 |        2.363 |                 19334.474 |            21 |             21 |              21 |          21 | sorted |          0 |

The mainline scheduler now reflects this evidence without hardcoding a fixed
worker count: source-range OCR uses the current adaptive Rust budget directly,
capped by a machine-derived source-range ceiling and page count. The explicit
source-range worker override remains available for benchmark and deployment
profile experiments, but the default path no longer applies a second square
root to the current adaptive budget at cold start.

Interpretation:

- The current source-range optimization reduces the OCR-positive cold miss from
  roughly 241-256 s to a measured 19-21 s envelope on the best current runs.
  This is about 11.3x-12.4x faster against the latest comparable 241 s
  baseline, or up to 13.2x faster against the earlier 256 s proof. The
  cold-path reduction is roughly 91-92 percent.
- The optimized path keeps the precision-critical shape: 21 resource rows, 21
  structure rows, 21 OCR page blocks, 21 bbox-bearing blocks, sorted reading
  order, and zero error rows.
- Once page shards are cached, forced extraction into a fresh output directory
  rebuilds in the low hundreds of milliseconds instead of repeating Docling OCR.
  Whole-document cache hits remain in the low-millisecond class.
- Unique OCR-heavy cold misses are still dominated by Docling OCR. Rust has
  removed avoidable render and orchestration waste, and now owns adaptive OCR
  pressure control for a single provider process, but it has not removed the
  fundamental OCR cost. The next major reduction requires precision-safe region
  discovery, horizontal Python executor pools, or a Docling OCR profile that
  proves parity on the same structure gates.

### Real Docling Format Coverage

The real non-audio Docling fixture suite covers 18 document classes: PDF, DOCX,
XLSX, PPTX, Markdown, AsciiDoc, HTML, CSV, PNG, TIFF, WebP, USPTO XML, JATS
XML, XBRL XML, METS GBS, Docling JSON, WebVTT, and LaTeX. The strict run
produced 304 rows across 36 force/cache requests, 1640656 Arrow IPC bytes, and
zero error rows.

Selected cold timings from that suite:

| Fixture          |  Force ms | Cache p95 ms | Rows | Error rows |
| ---------------- | --------: | -----------: | ---: | ---------: |
| PDF `2206.01062` | 23357.739 |        2.870 |   13 |          0 |
| DOCX sample      |    44.563 |        3.368 |    4 |          0 |
| XLSX sample      |    37.506 |        2.480 |   10 |          0 |
| HTML sample      |   330.743 |        2.522 |   31 |          0 |
| PNG image        |  3116.929 |        2.728 |    3 |          0 |
| TIFF image       |  4215.292 |        2.657 |    4 |          0 |
| USPTO XML        |  1270.865 |       13.623 |   19 |          0 |
| XBRL XML         |  1274.086 |        2.613 |   25 |          0 |
| METS GBS         | 12205.210 |        5.041 |    5 |          0 |
| WebVTT           |    12.361 |        4.126 |    2 |          0 |
| LaTeX            |    10.721 |        3.852 |    2 |          0 |

This confirms the broader Docling alignment surface remains functional while
the PDF OCR path is being optimized. The report does not claim equal semantic
depth for every format; it claims the Arrow transport, resource projection, and
cache path return stable rows with zero error rows across the represented
format set.

### Deduplication and Capacity Evidence

The same-content cold-miss production risk is materially closed for the tested
classes:

| Fixture group                          | Duplicate converter calls | Error rows | Observation                                                                   |
| -------------------------------------- | ------------------------: | ---------: | ----------------------------------------------------------------------------- |
| Real PDF duplicate miss                |                         1 |          0 | Repeated cold requests reused the same job/cache lineage.                     |
| Real mixed PDF/image/XML/XBRL pressure |             1 per fixture |          0 | Four different fixtures each converted once.                                  |
| Real audio pressure                    |                         1 |          0 | Audio fixture deduplicated under the same async job path.                     |
| Fake distinct cold miss smoke          | 4 for 4 distinct fixtures |          0 | The conversion cap reached four in-process conversions and did not exceed it. |

Capacity interpretation:

- For 10000 users requesting the same unchanged OCR-heavy PDF, the important
  number is not 10000 Docling conversions. The tested duplicate path converges
  to one conversion, then job/status/cache reuse. This is the production risk
  that the async dedup milestone was designed to close.
- For 10000 different OCR-heavy PDFs, the risk is still real capacity, not
  correctness. The adaptive Rust scheduler prevents one static worker setting
  from underusing or overwhelming the host, and it reports current OCR budget,
  queue wait, latency, in-flight shards, cache hits/misses, lane counts, and
  AIMD budget changes. A single instance still remains bounded by Docling OCR
  time. The next capacity slice now has a concrete execution surface:
  horizontal Python OCR executor pools exposed as multiple Flight endpoints and
  scheduled by Rust, plus finer region/text fast paths after parity gates.
- For already-cached documents, cache p95 remains in the low-millisecond class
  in the current evidence, so query-time reuse is compatible with
  high-concurrency user traffic.

### Dependency and Precision Judgment

The latest source-range feature split makes the dependency decision explicit:
`document-extract-pdf-source-range` powers default `hybrid-page-ocr` source
page-range OCR, while `document-extract-pdf-render` is required only for
PDFium-backed raster or region proof lanes. A `cargo tree` proof over the
source-range feature set produced no `pdfium` or `pdfium-render` matches.

That boundary matches the precision policy:

- `lopdf` is justified because it removes avoidable page selection and
  high-DPI raster work before Docling OCR while preserving the original source
  PDF as the OCR input.
- PDFium is not justified for no-OCR or source-page OCR acceleration. It is
  reserved for explicit region/raster proofs where a crop image is the actual
  OCR input.
- Rust does not replace Docling OCR. Rust schedules work, emits Arrow shard
  contracts, validates coverage, restores input shard order, writes structure
  sidecars, and caches reusable page results.
- The precision gate is structural, not just latency-based: zero error rows,
  complete page coverage, stable resource rows, sorted `_structure.arrow`, and
  bbox/provenance preservation for OCR blocks.

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

## Precision Gate And Metrics Slice

The current slice centralizes the fast-path acceptance rules before any default
region OCR work. A hybrid result is accepted only after the Rust precision gate
proves:

1. OCR worker results match the input shard identity and page index.
2. Every required page has native or OCR coverage.
3. Resource rows contain no error, failed, or skipped OCR rows.
4. OCR structure blocks keep bbox and provenance.
5. Structure rows remain sortable by page, reading-order key, block index, and
   block id.

The metrics sidecar records one row per OCR shard output. Initial Rust-owned
fields include source path, page index, shard id, OCR profile, page count, bbox
count, result characters, status, and available Rust scheduler/provenance
timing. Docling phase timings remain nullable until Python can expose them
without changing the stable OCR result v1 contract.

The strict structure parity helper is the first reusable building block for the
golden baseline suite. It intentionally stays outside the default extraction
route in this slice; benchmark lanes and later candidate fast paths can use it
to reject candidates that lose baseline pages, text coverage, protected
semantic block types, or reading order before falling back to full Docling.

The benchmark harness now lives under `tests/scripts/` and reads
`_metrics.arrow` from each extraction artifact directory. It includes the
sidecar in JSON and Markdown reports with metrics row count, OCR result
characters, bbox count, and total Rust scheduler elapsed time. This keeps
performance and precision evidence attached to the same Arrow artifact set as
`_resources.arrow` and `_structure.arrow`. When supplied with a golden
structure baseline root, the same cargo-test lane decodes baseline and
candidate `_structure.arrow` sidecars into the shared structure block model and
reports strict parity checked/passed/error counts. The harness also supports a
baseline generation pass that runs the existing ignored cargo probe in `sync`
mode with force refresh, writes fixture-named baseline artifact directories,
and then points the candidate run at that generated root.

A fixture-OCR smoke run against the `2604.17337` PDF confirmed the reporting
path without invoking real Docling OCR: 21 resource rows, 21 structure rows, 21
metrics rows, 21 bbox-covered OCR page blocks, sorted reading order, and zero
error rows. The run also verified that benchmark child-process logs are written
to report-local files instead of undrained pipes, avoiding startup deadlocks
during slow Cargo builds.

A real Docling OCR run on the same PDF, using an isolated shard cache and the
`documents` Python extra, produced 21 resource rows, 21 structure rows, 21 OCR
page blocks, 21 bbox-covered blocks, 21 metrics rows, 103,984 OCR result
characters, sorted reading order, and zero error rows. The cold force path was
21.012 s on the measured host; a forced rebuild from shard cache was 90.402 ms;
the whole-document cache hit was 2.419 ms p50/p95. This is the current
evidence baseline for the source-PDF page-range path. The follow-up source
range worker sweep measured 21.373 s, 20.184 s, and 19.442 s for source-range
overrides 1, 2, and 4 respectively, with identical row counts, sorted
structure, and zero error rows.

## Active Risks

| Risk                                                              | Impact                                                                                  | Mitigation                                                                                                                 |
| ----------------------------------------------------------------- | --------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| Unique OCR-heavy cold miss remains expensive                      | First-time extraction still takes roughly 19-21 s on the best current `2604.17337` runs | Continue source-range batching, safe region discovery, shard cache reuse, and Docling profile measurement                  |
| Native text fast path was retired with the detector dependency    | Some text-only PDFs lose the previous Rust-only proof path                              | Rebuild native text extraction directly over owned PDF primitives only after parity tests exist                            |
| Region OCR without native text merge can produce partial coverage | User-visible document order or coverage could degrade                                   | Keep region mode opt-in and fallback on partial coverage                                                                   |
| PDFium runtime mismatch                                           | Raster proof output could differ across hosts                                           | Keep PDFium confined to opt-in raster/region proof lanes; source-page OCR does not pull PDFium                             |
| Shard cache growth                                                | Project cache can grow under large OCR workloads                                        | Keep oldest-first sweep and report cache size, entry count, and limits in benchmarks                                       |
| Many unique cold documents                                        | One instance cannot absorb 10000 unique OCR-heavy cold misses quickly                   | Use deployment profiles for conversion limits, add horizontal workers, and keep region/text fast paths behind parity gates |

## Recommendation

Do not keep the retired detector crate in the active Wendao PDF extraction
dependency graph. The current hot path should stay on direct `lopdf` page-tree
intake, Arrow shard contracts, Rust-owned ordering and cache gates, and
Python/Docling OCR authority.

The current milestone meets the performance and precision bar for:

1. same-content cold-miss deduplication,
2. source-page OCR cold-path reduction from the 241-256 s class to the 19-21 s
   class on the real OCR-positive fixture,
3. low-millisecond whole-document cache hits,
4. low-hundreds-of-milliseconds shard-cache forced reuse,
5. stable Arrow contracts and sorted structure sidecars, and
6. removing unjustified PDFium usage from the no-OCR and source-page OCR path.

It does not yet solve the many-unique-OCR-heavy-document capacity problem. The
next optimization milestone should focus on precision-safe region discovery
and native text structure extraction over dependencies Wendao already owns or
can justify independently. The acceptance bar remains the same:
`totalErrorRows = 0`, sorted `_structure.arrow`, stable `_resources.arrow`,
complete page coverage, and no precision regression against Docling baselines.
