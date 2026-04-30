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
same precision and speed fields by attachment class. This makes mixed Docling
fixture suites auditable without changing the document extraction Flight
contract.

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

## Next Slices

1. Image attachment OCR lane:
   - add image-specific timing and structure metrics where Docling exposes
     OCR/layout phases;
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
