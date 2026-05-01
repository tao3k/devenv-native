# Document Extraction PR Closing Report

:PROPERTIES:
:ID: research-wendao-document-extract-pr-closing-report
:PARENT: [[301_research_papers|Research Index: Map of Content]]
:TAGS: research, document-extraction, attachments, pdf, ocr, docling, arrow, benchmark
:END:

## Purpose

This report closes the current document extraction optimization PR with real
precision and speed evidence. It combines the structured attachment benchmark,
the OCR-positive PDF benchmark, and direct Docling timing probes.

The conclusion is intentionally narrow:

1. Rust is now effective as the deterministic control plane: content hashing,
   async job state, artifact registry reuse, OCR shard cache, routing,
   scheduling, structure ordering, provenance, and benchmark gates.
2. Docling remains the OCR, layout, and semantic parsing authority.
3. The stable Arrow contracts are unchanged: `_resources.arrow`,
   `_structure.arrow`, and OCR shard input/result v1.
4. The remaining cold-path bottlenecks are Docling conversion cost for unique
   heavy documents, not Arrow Flight, Rust merge, or cache reconstruction.

## Closing Commands

The closing evidence used the benchmark harness under `tests/scripts/` and the
existing cargo-test performance lane. The relevant profiles were:

1. real Docling structured attachments, excluding live audio;
2. real Docling hybrid PDF OCR on arXiv `2604.17337`;
3. source-range OCR override probes for 1 and 4 range workers;
4. a local Python OCR endpoint-pool probe with four endpoints;
5. a direct Docling `convert(page_range=(1, 21))` timing probe.

Report artifacts remain local benchmark outputs. Canonical documentation records
the measured values and does not link to hidden runtime report directories.

## Structured Attachment Result

The structured attachment suite covered DOCX, XLSX, PPTX, Markdown, AsciiDoc,
HTML, CSV, USPTO XML, JATS XML, XBRL XML, METS GBS, Docling JSON, WebVTT, and
LaTeX.

| Metric                         |       Value |
| ------------------------------ | ----------: |
| Fixtures                       |          14 |
| Error rows                     |           0 |
| Precision gate                 |        true |
| Structure rows                 |         116 |
| BBox blocks                    |          23 |
| Structure order sorted         |        true |
| Structure order stable         |        true |
| Max force latency              | 16213.839 ms |
| Max Docling convert latency    | 15988.353 ms |
| Max cache p95 latency          |    10.482 ms |
| Max artifact registry reuse    |   120.525 ms |
| Archive audit count            |           1 |
| Archive member count           |          10 |
| Archive XML members            |           1 |
| Archive image members          |           3 |
| Archive acceleration candidate | `mets_gbs_member_manifest_candidate` |

Class-level result:

| Class            | Fixtures | Error rows | Structure rows | BBox blocks | Max force ms | Max Docling ms | Max cache p95 ms | Archive members |
| ---------------- | -------: | ---------: | -------------: | ----------: | -----------: | -------------: | ---------------: | --------------: |
| archive_document |        1 |          0 |              4 |           3 |    16213.839 |      15988.353 |            4.271 |              10 |
| docling_json     |        1 |          0 |             12 |          11 |      484.789 |        224.458 |            5.758 |               0 |
| office           |        3 |          0 |             14 |           9 |      280.692 |         90.816 |            6.104 |               0 |
| structured_text  |        3 |          0 |              4 |           0 |      205.606 |          4.716 |           10.482 |               0 |
| subtitle         |        1 |          0 |              1 |           0 |      193.506 |         15.557 |            6.293 |               0 |
| table_data       |        1 |          0 |              2 |           0 |      206.526 |          4.271 |            6.094 |               0 |
| web              |        1 |          0 |             30 |           0 |      757.807 |        285.303 |            9.023 |               0 |
| xml              |        3 |          0 |             49 |           0 |     1796.847 |       1147.026 |            8.043 |               0 |

Interpretation:

1. Office, text, table, subtitle, HTML, and Docling JSON are not the current
   cold-path risk.
2. XML is still Docling-bound but below two seconds in the real suite.
3. METS GBS is the structured attachment hotspot. Rust archive audit now proves
   member shape cheaply, but live extraction still correctly falls back to
   Docling until member-level parity and routing are proven.
4. Cache and artifact registry reuse are already in the low-millisecond to
   low-hundreds-millisecond class.

## OCR-Positive PDF Result

The real OCR-positive PDF fixture was arXiv `2604.17337`, with 21
OCR-positive pages.

| Profile                         | Force ms | Cache p95 ms | Shard-cache rebuild ms | Resource rows | Structure rows | OCR page blocks | BBox blocks | Result chars | Order stable | Error rows |
| ------------------------------- | -------: | -----------: | ---------------------: | ------------: | -------------: | --------------: | ----------: | -----------: | ------------ | ---------: |
| Current default adaptive profile | 45941.076 |       11.921 |                144.232 |            21 |             21 |              21 |          21 |      103984 | true         |          0 |
| Source-range override 4          | 43917.250 |       23.209 |                213.161 |            21 |             21 |              21 |          21 |      103984 | true         |          0 |
| Source-range override 1          | 53258.791 |        5.954 |                171.860 |            21 |             21 |              21 |          21 |      103984 | true         |          0 |
| Four local OCR endpoints         | 47726.578 |        5.021 |                168.824 |            21 |             21 |              21 |          21 |      103984 | true         |          0 |

The current local closing run is slower than the earlier 19-21 second
source-range evidence recorded during the PDF milestone, but it keeps the same
precision shape: 21 resource rows, 21 structure rows, 21 OCR page blocks, 21
bbox-bearing blocks, the same order signature, stable reading order, and zero
error rows.

A direct Docling probe on the same PDF showed the source of the slowdown:

| Probe                                   |       Value |
| --------------------------------------- | ----------: |
| Docling version in lockfile             |      2.91.0 |
| `convert(page_range=(1, 21))`           | 47855.014 ms |
| Page-break markdown export             |    94.900 ms |
| Exported page parts                     |          21 |
| Exported markdown chars                 |      104903 |
| `num_threads=8` convert                 | 50130.220 ms |
| `num_threads=12` convert                | 52044.386 ms |

Interpretation:

1. The closing-run OCR cost is inside Docling conversion. Markdown export,
   Arrow Flight, Rust merge, structure ordering, and cache rebuild are not the
   dominant cost.
2. Increasing Docling thread count did not help on this host; it regressed the
   direct convert probe.
3. Four local Python OCR endpoints did not improve the single-PDF case, so
   endpoint-pool expansion should be treated as capacity scaling for many
   documents, not a guaranteed single-document latency reduction.
4. The shard cache is doing the intended work: fresh-output rebuild from cached
   OCR shards stayed in the 144-213 ms range, and whole-document cache p95
   stayed below 24 ms.

## Precision Assessment

The current PR preserves the precision-critical contracts:

1. `totalErrorRows=0` across the structured attachment closing suite and all
   OCR-positive PDF closing probes.
2. `precisionGatePassed=true` for the structured attachment suite.
3. PDF OCR output keeps 21 resource rows, 21 structure rows, 21 OCR page
   blocks, and 21 bbox-bearing blocks.
4. Structure ordering is sorted and stable; OCR shard completion order does not
   define document order.
5. Docling JSON remains an optional exported resource, not the Python-to-Rust
   transport contract.
6. Rust archive and image audits remain preflight evidence surfaces, not live
   parser replacements.

This is the right boundary for precision: Rust accelerates and verifies the
pipeline, while Docling remains the authority until a class-specific fast path
passes baseline parity.

## Performance Assessment

The PR closes three avoidable performance risks:

1. Repeated same-content cold misses no longer fan out into repeated Docling
   work because content-hash deduplication and artifact registry reuse are in
   the Rust control plane.
2. OCR shard reuse avoids repeated OCR for the same page shards; fresh-output
   rebuild is now low hundreds of milliseconds instead of repeating the full
   Docling OCR conversion.
3. Structured attachment cache hits and artifact-registry reuse are already
   below interactive latency thresholds.

The PR does not eliminate the cost of first-time Docling conversion for unique
heavy inputs:

1. unique OCR-heavy PDFs are still bounded by Docling OCR convert time;
2. archive-backed METS GBS remains a Docling-bound cold path;
3. image OCR remains a Docling-bound cold path when the input is a unique,
   uncached image.

For 10000 users, the production risk is therefore split:

1. same document or same shard repeated by many users: low risk after dedup,
   artifact reuse, and shard cache;
2. many different heavy documents arriving cold at once: still a capacity
   planning problem, handled by Rust-owned scheduling, endpoint pools, queue
   status, and deployment upper bounds, but not solved by a single-node parser
   optimization.

## Next Optimization Candidates

The next optimization should not replace Docling OCR or layout authority. The
most defensible slices are:

1. add a Docling-version performance guard so `precisionSpeedSummary` can flag
   current-run regressions against the stored PDF milestone envelope;
2. add member-level artifact keys for archive attachments, starting with METS
   GBS manifest/image members;
3. add precision-safe region/page selection only where coverage and order gates
   can prove no loss;
4. add deployment-profile benchmark lanes for multiple distinct OCR-heavy
   documents to size the Rust scheduler and Python endpoint pool;
5. keep collecting direct Docling convert/export timings whenever Docling is
   upgraded, because the current closing evidence shows upstream conversion
   cost can dominate the whole path.

## PR Closing Status

The milestone is ready to close from a precision and control-plane standpoint:

1. Arrow contracts remained stable.
2. Rust did not replace Docling semantic authority.
3. Real structured attachments passed precision gates with zero error rows.
4. Real OCR-positive PDF kept structure order, bbox coverage, row counts, and
   zero error rows.
5. The remaining latency is now clearly attributed to Docling conversion for
   unique heavy cold misses, with cache and rebuild paths already reduced to
   low-millisecond or low-hundreds-millisecond latency.
