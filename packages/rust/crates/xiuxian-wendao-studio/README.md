---
type: knowledge
kind: readme
title: "xiuxian-wendao-studio"
category: "package-docs"
status: "active"
author: Xiuxian Artisan Workshop
date: 2026-05-05T00:00-07:00
description: "Package README for the Wendao Studio gateway adapter and Docling scheduling adoption boundary."
tags:
  - studio
  - wendao
  - docling
  - polyglot
metadata:
  title: "xiuxian-wendao-studio"
  retrieval:
    saliency_base: 7.0
    decay_rate: 0.03
---

# xiuxian-wendao-studio

`xiuxian-wendao-studio` owns the Studio-facing HTTP gateway adapter for Wendao.

This crate may depend on Wendao domain crates and on `xiuxian-wendao-server` for
Flight/gRPC transport contracts. The dependency direction remains one-way:
Studio adapters can call Wendao domain services and server transport contracts,
but `xiuxian-wendao` and `xiuxian-wendao-server` must not depend on this crate.

## Ownership

This crate owns:

- Studio HTTP route composition and handler state.
- Studio OpenAPI and route-contract exports.
- Studio Flight route providers backed by Wendao services.
- Frontend-facing API response shaping and gateway startup health checks.

`xiuxian-wendao-server` owns only the high-throughput Flight/gRPC transport
boundary. `xiuxian-wendao` continues to own graph, search, repository indexing,
parser, analyzer, and domain-runtime behavior.

## Feature Boundaries

The lightweight `contracts` feature owns route contracts and OpenAPI route
inventory. It must remain free of runtime gateway dependencies such as Axum,
Tonic, Arrow Flight, DuckDB, DataFusion, notify, `xiuxian-db-store`, and
`xiuxian-wendao-core`. Studio owns the frontend Specta type collection,
capability/search-manifest DTOs, Studio UI project configuration DTOs, and
plugin artifact inspection DTOs. It also owns the Studio-facing graph, code
AST, Markdown analysis, symbol/autocomplete, retrieval atom, navigation, and
search response DTOs. Runtime conversion from Wendao domain search records is
compiled only with `local-runtime`; the lightweight schema surface does not
re-export those domain records.

Plugin artifact DTO conversion is compiled only with `local-runtime` because
it depends on runtime plugin payload records.
Wendao domain search accepts project configuration through its own
`SearchProjectConfig`/`ProjectConfigView` boundary, so the Studio UI schema does
not have to live in the domain crate.

Runtime concerns are layered behind explicit features:

- `http-router`: Axum/Tower router and handler composition.
- `flight-transport`: Arrow Flight and gRPC provider adapters.
- `local-runtime`: repository indexing, search-plane, DuckDB/DataFusion,
  watcher, parser, and local project integration.
- `studio`: full Studio composition.
- `cli-bin-support`: binary-only support for commands that require the full
  Studio runtime.

`local-runtime` uses `xiuxian-zhenfa` only through its native registry surface.
The HTTP gateway/client features remain opt-in through `zhenfa-router` or
`cli-bin-support`, so Studio's local runtime does not inherit Zhenfa HTTP
composition by accident.

## Polyglot Docling Scheduling

The Polyglot Compute Orchestrator boundary is tracked in
[RFC: Polyglot Compute Orchestrator](../../../../docs/rfcs/2026-05-04-polyglot-compute-orchestrator-rfc.md).

For the `document-extract-pdf-source-range` lane, Studio owns the live OCR
semaphore permits, endpoint selection, cache hits, in-flight shard coalescing,
adaptive pressure observations, queue wait observation, and the
`x-wendao-pdf-ocr-workers` Flight metadata header. The shared
`xiuxian-polyglot-orchestrator` contract owns the pure Docling scheduling plan,
including source-range auto worker sizing from owner-supplied system facts.
Studio consumes that plan through the attachment polyglot bridge while keeping
live dispatch local to this crate.
The current source-range auto policy targets seven source PDF pages per worker
before clamping to the adaptive budget, machine cap, remaining permits, and
shard count; diagnostic worker overrides remain benchmark-only.
Studio also owns the opt-in source-range OCR profile planner exposed through
`WENDAO_DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER` and the benchmark flag
`--rust-pdf-ocr-profile-planner`. The proven `fast-risk-window` mode uses
attachment-owned source-page structure facts to keep table-risk pages on the
default Docling-compatible profile while assigning `docling-fast-text-ocr`
only to non-risk source-page ranges. The same planner now exposes `ocr2-all`
for full OCR2 probes and `ocr2-risk-window` for surgical recovery: ordinary
pages stay on `docling-fast-text-ocr`, while the source-profile risk window
uses `deepseek-ocr2-direct-vlm`. The `ocr2-risk-window` route keeps the primary
manifest on source-range rows and materializes rendered page images only for
OCR2 recovery pages, so ordinary fast pages do not pay page-raster cost. Those
same planner semantics apply to explicit region-shard recovery: the parent page
stays on the fast source-range profile and the rendered region rows are
appended as supplemental OCR2 inputs, preserving the stable OCR shard schema
and the Rust-side row/order validation gate. Region recovery now normalizes the
rendered region's parent shard id to the retained fast parent page and records
`sentinel-sidecar-v1` in structure provenance; this is a safe sidecar patch
protocol, not default in-place Markdown replacement.
When no explicit region JSON is configured, the benchmark can opt into
`WENDAO_DOCUMENT_EXTRACT_PDF_OCR2_REGION_PLANNER=profile-risk-window` through
`--rust-pdf-ocr2-region-planner profile-risk-window`. That first automatic
planner only acts on pages already selected by `ocr2-risk-window` and builds a
conservative content-band region from the page crop box; it is a recovery
surface probe, not a claimed table-detector or default routing policy.
`profile-risk-window-slices` is the next benchmark-only variant: it preserves
the same source page selection but splits each content band into
top/middle/bottom same-page regions in reading order, giving the analyzer's
region composite canary a real hosted benchmark surface without changing the
OCR shard schema.
`profile-risk-window-adaptive` is the current algorithmic follow-up: it keeps
the same source page selection, reuses attachment-owned source-page structure
profiles, estimates content-band pixel area, and chooses one, two, or three
same-page slices. Exact structure-risk pages may receive more slices, while
low-complexity neighbor pages can stay as one region. The goal is to avoid
both a broad single-region provider tail and blanket three-slice request
overhead while keeping 300 DPI, semantic padding, parent binding, and the
stable shard schema.
The analyzer-side direct OCR2 worker can additionally opt into a same-page,
same-parent region composite canary through
`WENDAO_DEEPSEEK_OCR2_REGION_COMPOSITE_SIZE` or the benchmark flag
`--deepseek-ocr2-region-composite-size`. Composite responses are accepted only
when sentinel markers split back into one non-empty Markdown result per region;
otherwise the worker falls back to individual region requests and the Rust
row/order gate still sees the unchanged OCR shard result schema. The benchmark
can additionally set `WENDAO_DEEPSEEK_OCR2_REGION_ATLAS_MODE=same-page-json`
or `--deepseek-ocr2-region-atlas-mode same-page-json` to pack each same-page
region composite group into one labeled PNG atlas and require strict JSON keyed
by exact shard markers. Atlas mode remains a request-surface canary: validation
failure falls back to individual region requests, and promotion still depends
on the unchanged Rust row/order, character-floor, and force-refresh gates.
For table and complex-layout region recovery, the benchmark can also opt into
`--deepseek-ocr2-scaffold-mode region-table-json`. Studio forwards that as
`WENDAO_DOCUMENT_EXTRACT_PDF_OCR2_SCAFFOLD_MODE=region-table-json` and writes
`_ocr2_region_scaffolds.json` beside the rendered OCR2 region images. The
sidecar records shard ids, parent shard ids, source and raster fingerprints,
render DPI, crop boxes, source pixel boxes, source-page profile signals, and a
conservative scaffold kind such as `table_candidate`,
`complex_layout_candidate`, or `manual_region_candidate`. Studio does not
invent hard table row or column counts in this slice; the sidecar is a
fingerprinted routing and prompt contract consumed by the analyzer worker,
which must return failed rows on scaffold validation errors so the Rust
precision fallback remains authoritative.
All OCR2 modes stay opt-in until the real benchmark gate proves the current
precision envelope and beats the 12,856.546 ms `fast-risk-window`
force-refresh evidence. Benchmark reports expose that decision as
`ocr2PromotionGate`, which keeps OCR2 profile promotion tied to the frozen
precision, row/order, character-floor, hosted-request, force-refresh,
shard-cache reuse, and zero scaffold-validation-failure gates.

The active Studio `rust-lang-project-harness` lib-policy profile marks the OCR
capacity-control file as the polyglot Docling scheduler adoption point. That
profile keeps Studio accountable for live permits and dispatch while verifying
the orchestrator plan consumption boundary.

For full-document Docling extraction, Studio also consumes the runtime-owned
inert schedule plan before selecting from the existing
`WENDAO_DOCUMENT_EXTRACT_ENDPOINTS` pool. The existing conversion semaphore is
the owner budget: when capacity is available the request dispatches through the
current endpoint round-robin path, and when capacity is exhausted the request
waits on the existing permit instead of creating a second queue. This does not
change Python Flight routes, schemas, endpoint environment variables, cache/job
registry behavior, or Python worker lifecycle.

The active Studio harness profile also marks the full-document provider
transport as the runtime-adoption point for this boundary.
