# Attachment Format Precision And Speed Baseline

:PROPERTIES:
:ID: research-wendao-attachment-format-baseline
:PARENT: [[301_research_papers|Research Index: Map of Content]]
:TAGS: research, document-extraction, attachments, docling, arrow, benchmark
:END:

## Purpose

This note records the first non-PDF attachment baseline after the PDF hybrid
OCR milestone. The goal is evidence before optimization: every future
attachment fast path must preserve Docling authority, stable Arrow contracts,
structure order, and cache behavior before it can be considered for live
routing.

The benchmark harness now emits `summary.attachmentClassSummary`, grouping the
same precision and speed fields by attachment class. The class summary also
aggregates resource type counts, structure block type counts, bbox block
counts, and slowest force/cache fixtures. This makes mixed Docling fixture
suites auditable without changing the document extraction Flight contract.

## Boundary

The current non-PDF baseline does not add a new parser fast path. Python and
Docling remain the parsing authority. Rust and the benchmark harness only
observe resource rows, structure sidecars, order signatures, cache timing, and
class-level summaries.

Stable contracts remain unchanged:

1. `_resources.arrow` keeps the existing nine-column resource schema.
2. `_structure.arrow` keeps schema version
   `xiuxian_wendao.document_structure.v1`.
3. Benchmark JSON and Markdown summaries are evidence artifacts, not
   Python-to-Rust extraction contracts.

## Real Fixture Baseline

The first real non-PDF run used Docling fixtures for DOCX, XLSX, PPTX, PNG,
HTML, CSV, Markdown, and USPTO XML. The run used sync extraction to measure
the direct Python/Docling path with Rust's existing performance probe and the
new class grouping.

Overall result:

| Metric                     |       Value |
| -------------------------- | ----------: |
| Fixtures                   |           8 |
| Error rows                 |           0 |
| Precision gate             |        true |
| Structure rows             |          67 |
| BBox blocks                |          10 |
| Structure order sorted     |        true |
| Structure order stable     |        true |
| Structure order mismatches |           0 |
| Max force latency          | 5512.288 ms |
| Max cache p95 latency      |    4.865 ms |
| Minimum cache speedup      |      4.122x |

Class-level result:

| Class           | Fixtures | Error rows | Structure rows | Order stable | Max force ms | Max cache p95 ms | Minimum speedup |
| --------------- | -------: | ---------: | -------------: | ------------ | -----------: | ---------------: | --------------: |
| image           |        1 |          0 |              2 | true         |     5512.288 |            2.104 |       2619.753x |
| office          |        3 |          0 |             14 | true         |      168.242 |            4.865 |         13.024x |
| structured_text |        1 |          0 |              1 | true         |       11.007 |            2.487 |          4.427x |
| table_data      |        1 |          0 |              2 | true         |        8.031 |            1.948 |          4.122x |
| web             |        1 |          0 |             30 | true         |      317.101 |            1.910 |        166.054x |
| xml             |        1 |          0 |             18 | true         |     1620.380 |            2.776 |        583.763x |

## Interpretation

Office attachments are not the immediate latency hotspot in this fixture set:
DOCX, XLSX, and PPTX all stayed below 170 ms cold force latency, produced
tables/images where Docling exposed them, and kept stable structure order.

The main non-PDF cold path candidates are image and XML:

1. image conversion took roughly 5.5 seconds cold, which is expected to include
   image OCR and layout work;
2. USPTO XML took roughly 1.6 seconds cold while producing many table rows;
3. both became low-millisecond cache hits after `_resources.arrow` and
   `_structure.arrow` were materialized.

This means the next optimization should not start by replacing Office parsing.
The better next target is image attachment OCR observation and cache granularity,
followed by XML/table-heavy extraction profiling.

## Image Observability Follow-Up

The follow-up image-only real fixture run confirms the added class composition
fields work on real artifacts:

| Field                  |                                 Value |
| ---------------------- | ------------------------------------: |
| Force latency          |                           6835.697 ms |
| Cache p95 latency      |                              6.108 ms |
| Resource rows          |                                     3 |
| Structure rows         |                                     2 |
| BBox blocks            |                                     1 |
| Error rows             |                                     0 |
| Resource type counts   | `docling_json=1, document=1, table=1` |
| Structure block counts |                 `document=1, table=1` |

The image fixture is therefore slow while producing a small result shape. The
next image slice should split Docling image conversion timing before attempting
any cache or scheduler change.

## Rust Image Audit Follow-Up

The Rust side now has a bounded image preflight audit surface in
`xiuxian-wendao-attachments`. It reads file metadata plus bounded headers for
known Docling image suffixes, records MIME/format hints, and extracts
PNG/JPEG/BMP/GIF/WebP/TIFF dimensions when those dimensions are available
directly from the header. The `xiuxian-wendao` benchmark probe can include the
audit through the `document-extract-attachment-audit` feature and then
aggregate `imageAccelerationCandidates` in `summary.attachmentClassSummary`.

This is an optimization planning signal, not a parser change. Rust can now
identify future whole-image OCR cache and oversized-image preflight candidates,
but live image extraction still uses Python/Docling and the same resource and
structure Arrow contracts.

## Full-Document Timing Follow-Up

The Python analyzer now writes an additive `_document_metrics.arrow` sidecar on
full-document conversion cache misses. The sidecar records phase timings for
Docling conversion, Markdown export, resource row construction, structure row
construction, and Arrow cache writes. The Rust benchmark artifact inspector
reads this sidecar and the benchmark summary aggregates total elapsed
milliseconds plus phase-level elapsed milliseconds.

This does not change the extraction contract. `_resources.arrow`,
`_structure.arrow`, OCR `_metrics.arrow`, Flight routes, REST routes, and
Docling parser authority remain unchanged. The purpose is to identify whether a
slow image, XML, or Office cold path is parser-bound, export-bound,
row-construction-bound, or Arrow-write-bound before adding any fast path.

The first timing run showed a one-time PyArrow IPC/table writer initialization
cost in `writeStructureArrow`. The Python document Flight service now warms the
resource, structure, and timing Arrow writers during startup so the first
request does not carry that initialization cost. Benchmark reports now also
compute document timing overhead as `forceRefreshMs -
documentTimingTotalElapsedMs`; this keeps Python extraction time separate from
Flight/Rust request-boundary time.

`precisionSpeedSummary` now carries Docling convert time and two ratios:
Docling convert share of full-document extraction time, and request-boundary
overhead share of force latency. These are benchmark fields only. They make
the image, XML, Office, audio, and structured-text lanes comparable with the
same precision gate, and they let each future optimization prove whether it
reduced parser-bound work or only shifted overhead around the Rust/Flight
boundary.

## Sync Artifact Reuse Follow-Up

The real image timing breakdown after writer warmup shows the image cold path
is dominated by Docling conversion, not Rust/Flight overhead. Sync
`force=false` extraction now participates in the same Rust content-hash
artifact registry used by async extraction: it checks the requested output
cache first, then mirrors an already completed content-hash artifact before
calling Python. When Python does perform a successful sync conversion, Rust
mirrors that output back into the artifact registry for future reuse.

This is a control-plane cache optimization. It does not replace Docling image
OCR or layout, and it does not change `_resources.arrow`, `_structure.arrow`,
or Flight/REST schemas.

The current real image benchmark confirms the boundary after the
`precisionSpeedSummary` Docling-share fields were added:

| Metric                          |       Value |
| ------------------------------- | ----------: |
| Error rows                      |           0 |
| Precision gate                  |        true |
| Structure order stable          |        true |
| Resource rows                   |           3 |
| Structure rows                  |           2 |
| BBox blocks                     |           1 |
| Force latency                   | 6961.179 ms |
| Cache p95 latency               |    2.344 ms |
| Docling convert                 | 6923.778 ms |
| Docling convert share           |      99.53% |
| Request-boundary overhead       |    4.437 ms |
| Request-boundary overhead share |       0.06% |

This shifts the next real optimization away from Flight/Arrow overhead and
toward image-specific Docling invocation shape, image OCR cache granularity,
and parity-gated crop or tile planning. Rust should continue to act as the
control plane: content hash, preflight, routing, cache, scheduler, provenance,
and validation. Docling remains the image OCR and layout authority until a
class-specific fast path proves parity.

The benchmark summary now also exposes image audit format counts,
dimension-source counts, known-dimension counts, maximum width, maximum height,
and maximum pixel count at both whole-run and attachment-class levels. These
fields make the next crop/tile/cache decision evidence-based: if Rust cannot
prove dimensions from bounded headers, the image remains a Docling passthrough
candidate; if it can prove dimensions, the report has enough control-plane
signal to select whole-image cache, oversized preflight, or a future
parity-gated crop/tile proof without changing Docling OCR authority.

## Adaptive Conversion Capacity Follow-Up

The general Rust document extraction scheduler no longer uses a frozen
four-conversion default for full-document Docling cache misses. The default
permit count now follows host `available_parallelism()`, while
`WENDAO_DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS` acts as a deployment upper
bound. This gives image, XML, Office, audio, and other non-PDF cold misses the
same Rust-owned scheduling posture as the PDF OCR lane: Rust controls capacity,
Python/Docling executes conversion, and queued jobs remain observable through
the existing job status snapshot.

The full-document path now also supports `WENDAO_DOCUMENT_EXTRACT_ENDPOINTS`,
a comma- or whitespace-separated Python Flight worker pool. Rust keeps the
queue, content-hash deduplication, and artifact registry centralized, then
round-robins cache misses across the configured Python workers. This mirrors
the PDF OCR endpoint-pool model for image, XML, Office, audio, and other
non-PDF conversion lanes without changing Docling authority or Arrow schemas.

## Next Slices

1. Image attachment OCR lane:
   - use class-level resource/block composition and slowest-fixture fields to
     identify whether the image cost is OCR, table reconstruction, or export
     overhead;
   - use Rust image audit fields to decide whether whole-image OCR cache,
     oversized image preflight, or crop/tile planning is the right next proof;
   - use `_document_metrics.arrow` to split image cold-miss timing before
     changing routing or cache behavior;
   - preserve Docling OCR authority;
   - reuse the Rust scheduler/cache model only after the image lane has stable
     parity evidence.
2. XML/table-heavy lane:
   - record table count, table character volume, and Arrow IPC size per XML
     fixture;
   - identify whether time is parser-bound or Arrow/export-bound.
3. Office lane:
   - keep as a regression gate first;
   - optimize only if larger real Office fixtures show cold-path pressure.

## Acceptance Rule

A non-PDF attachment optimization is acceptable only when it preserves:

1. zero error rows;
2. stable structure order across force and cache runs;
3. stable resource and structure Arrow schemas;
4. class-level precision-speed reporting;
5. Docling fallback or Docling authority for any semantic parsing not proven by
   parity evidence.
