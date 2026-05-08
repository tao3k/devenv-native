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
only to non-risk source-page ranges. The same planner now exposes
`hosted-vlm-all` for full hosted VLM/OCR probes and
`hosted-vlm-risk-window` for surgical recovery: ordinary pages stay on
`docling-fast-text-ocr`, while the source-profile risk window uses the
model-agnostic `hosted-vlm-direct-ocr-v1` profile. The hosted VLM
risk-window route keeps the primary manifest on source-range rows and
materializes rendered page images only for recovery pages, so ordinary fast
pages do not pay page-raster cost. Narrow exact-risk-only routing is not the
promotion path because the real milestone run lost the frozen character floor.
`hosted-vlm-risk-window-backend-text` is a benchmark-only follow-up canary: it
routes ordinary low-risk pages to `docling-backend-text-ocr-v1`, keeps dense
text top-up pages on `docling-fast-text-ocr`, and still routes the hosted VLM
risk window through the rendered recovery region path. Source-range dispatch
prioritizes those top-up chunks before backend-text chunks so mixed-profile
base OCR does not leave the slowest local work to the final wave. When the
backend-text canary is present, the source-range scheduler also expands the
requested dispatch budget to the number of contiguous source profile runs,
then still clamps through the live Rust worker permits. In the opt-in hosted
region `render-dispatch` pipeline, the same run-count floor also applies after
local backend-text rows are satisfied. This keeps precision top-up ranges, such
as a single dense page and a later multi-page risk window, from serializing
behind hosted region requests while still respecting the live Rust worker
permit cap. Promotion still requires the same character floor, row/order
stability, zero error rows, and force-refresh gate.
The current OpenRouter benchmark canary also enables analyzer-side
`region-whitespace-trim` request-image optimization. It preserves render DPI,
Arrow OCR shard rows, and Rust row/order validation while cutting hosted region
payload bytes. The current promoted OpenRouter evidence uses
`mistralai/ministral-3b-2512`, local `rust-lopdf` backend text, run-parallel
source top-up dispatch, endpoint-`0` Docling fast-text prewarm,
`single-page-first` fast-text affinity, region trim, and a `4s` hosted hedge:
zero error rows, stable `27` rows, `21/6` page/region OCR blocks,
`metricsResultChars=107562`, force refresh `8201.568417 ms`, shard-cache reuse
`123.758583 ms`, artifact reuse `157.076542 ms`, hosted request wall span
`5598 ms`, and hosted request p95 `5225.923 ms`. It preserves the frozen
character floor, beats the locked `12856.546292 ms` promotion gate, and is the
current best accepted sample. The older 2026-05-07 OpenRouter r59/r60 envelope
remains historical evidence at best `9363.09725 ms` and promoted repeat
`12130.139833 ms`. The older Qianfan OCR trim run remains valid but
non-promoted at `13992.340875 ms`.
Analyzer-side source-page prewarm is an accepted stability diagnostic for this
canary: `--pdf-ocr-prewarm-source-path` plus
`--pdf-ocr-prewarm-page-index 0` triggered Docling table-structure warmup
before worker readiness, reduced page `5` top-up to about `7.2-7.4s`, and
passed repeat promotion at `11990.357708 ms` and `11537.015125 ms`; r59
remains the best sample because the prewarm runs were still bounded by hosted
region tail.
The benchmark harness can narrow that prewarm with
`--pdf-ocr-prewarm-endpoint-count N`, and Studio can opt into
`WENDAO_DOCUMENT_EXTRACT_PDF_FAST_TEXT_ENDPOINT_AFFINITY=single-page-first`
or `--rust-pdf-fast-text-endpoint-affinity single-page-first` to send
single-page fast-text source-PDF chunks to the first OCR endpoint. The r70
canary prewarmed only endpoint `0`, preserved the precision envelope, reduced
page `5` fast-text top-up to `5274.754916 ms`, and completed force refresh at
`9636.47725 ms`. It remains accepted as the endpoint-locality proof. A
2026-05-08 control without endpoint-local prewarm and affinity regressed to
`22329.780375 ms`, with page `5` fast-text source-range work tailing at
`20193.906625 ms`. Restoring the r70 shape brought the same canary back to
`10164.795292 ms` with a `5s` hosted hedge, and tightening only the hosted
hedge to `4s` produced the current `8201.568417 ms` promoted evidence. The r71
endpoint `0-3` prewarm diagnostic reduced the page `11-13` fast-text chunk to
`5972.05625 ms` but regressed force refresh to `10336.721667 ms` because the
hosted region tail dominated.
The scheduler now applies `single-page-first` to source-PDF fast-text chunks
directly. r79b verified that deterministic route under the real OpenRouter gate:
precision stayed intact, force refresh was `12067.125959 ms`, hosted p95 was
`6341.191 ms`, and the source tail moved back to page `5` at
`8030.604042 ms`. This remains below the locked baseline but is not promoted
over the current `8201.568417 ms` OpenRouter sample.
Hedge `4s` is promoted only in the current endpoint-`0` prewarm,
`single-page-first` affinity, region-trim, Ministral OpenRouter canary.
Direct-crop rendering, scaffold/composite, local fast-text replacement,
fast-text single-page source-range splitting, and hosted VLM full-page top-up
replacement remain rejected canaries for this fixture.
The later Ministral same-page region composite size `3` diagnostic also stays
rejected: it preserved the precision envelope, but the page `12` three-region
composite request tailed at `14430.981 ms` and force refresh regressed to
`17806.492208 ms`. The r84 fallback-guard rerun completed with valid OCR
metrics and passed the locked baseline at `12658.151 ms`, but it is still
diagnostic-only because it did not beat the current `8201.568417 ms`
OpenRouter envelope.
Hosted region chunk-order diagnostics also stay rejected. `page-area-desc`
preserved precision but regressed force refresh to `12773.714667 ms`; the
request p95 was `8899.291 ms` and request wall span was `9598 ms`.
`page-max-area-desc` preserved precision but still measured
`11270.017292 ms`; traces showed page `13` was gated by large-region render
completion rather than sort order. The accepted default remains page-grouped
region chunks with render-dispatch and render-ahead `3`.
The analyzer benchmark can also prewarm multiple Docling source pages through
`--pdf-ocr-prewarm-page-indices`, but the r75 page `5,11` endpoint `0-3`
diagnostic is rejected: precision stayed intact, force refresh regressed to
`18784.875625 ms`, page `5` fast-text tailed at `11507.441291 ms`, and hosted
request p95 reached `8979.77 ms`.
No-hedge and region-token-cap diagnostics are also bounded by the same
promotion gate. The r76 no-hedge canary preserved precision but regressed force
refresh to `14075.232875 ms` with hosted p95 `10515.504 ms`, so speculative
retry stays enabled for the current OpenRouter provider. After a narrow
attachments PDFium rotation hardening, r78 reran `regionMaxTokens=1536` and
passed precision at `12726.140916 ms`; this is valid near-baseline diagnostic
evidence below the locked `12856.546292 ms` floor, but it is slower than the
current `8201.568417 ms` OpenRouter sample and is not promoted.
Studio also exposes two opt-in source-range diagnostics for this canary. Set
`WENDAO_DOCUMENT_EXTRACT_PDF_LOCAL_BACKEND_TEXT=rust-lopdf`, or pass
`--rust-pdf-local-backend-text rust-lopdf`, to let Rust satisfy
`docling-backend-text-ocr-v1` source-PDF rows through the attachment-owned
`lopdf` text helper. This is promoted only as part of the hosted OpenRouter
risk-window canary above. Set
`WENDAO_DOCUMENT_EXTRACT_PDF_LOCAL_BACKEND_TEXT_EMPTY=fail-fast`, or pass
`--rust-pdf-local-backend-text-empty fail-fast`, only as a source-page-range
placeholder diagnostic. It turns empty or locally unextractable backend-text
source pages into failed OCR rows immediately, so the existing precision
fallback can run without retrying a non-image placeholder through Python raster
OCR. The default remains `dispatch-python`. Local fast-text replacement is
diagnostic-only and is
rejected for the milestone fixture because it drops `metricsResultChars` below
the frozen floor. Set
`WENDAO_DOCUMENT_EXTRACT_PDF_FAST_TEXT_SOURCE_RANGE_SPLIT=single-page`, or pass
`--rust-pdf-fast-text-source-range-split single-page`, only as a source-range
chunk-shape diagnostic. It keeps Rust scheduler permits as the final admission
owner and preserves precision on the milestone fixture, but it is rejected
because page `5` fast-text conversion alone regressed force refresh to
`23629.474667 ms`.
Set
`WENDAO_DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_TOPUP=disabled`, or pass
`--rust-pdf-backend-text-topup disabled`, only for character-floor canaries.
The default remains `profile`; the disabled canary is rejected for the
milestone fixture because it drops `metricsResultChars` below the frozen floor.
Set `WENDAO_DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_TOPUP=hosted-vlm`, or pass
`--rust-pdf-backend-text-topup hosted-vlm`, only to test full-page hosted
VLM/OCR as a dense top-up replacement. That canary is rejected for the
milestone fixture because it regressed force refresh to `35374.309 ms` and
dropped `metricsResultChars` to `91265`; the current hosted model did not
preserve page `5` dense text coverage.
The same planner semantics apply to explicit region-shard recovery: the parent page
stays on the fast source-range profile and the rendered region rows are
appended as supplemental hosted VLM inputs, preserving the stable OCR shard schema
and the Rust-side row/order validation gate. Region recovery now normalizes the
rendered region's parent shard id to the retained fast parent page and records
`sentinel-sidecar-v1` in structure provenance; this is a safe sidecar patch
protocol, not default in-place Markdown replacement.
When no explicit region JSON is configured, the benchmark can opt into
`WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER=profile-risk-window`
through `--rust-pdf-hosted-vlm-region-planner profile-risk-window`. That first automatic
planner only acts on pages already selected by the hosted VLM risk-window planner and builds a
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
The benchmark can also opt into
`WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PIPELINE=render-dispatch`, or
pass `--rust-pdf-hosted-vlm-region-pipeline render-dispatch`, to start the
ordinary source-range OCR scheduler work before hosted recovery regions finish
rendering. This is a local-overhead probe for the measured gap between hosted
request wall span and force-refresh latency; it remains disabled by default
and must preserve the same row/order and precision gates as the non-pipelined
path.
Studio scheduler trace rows now include optional queue-wait and chunk
dispatch start/end timing for live OCR shard requests. The benchmark harness
uses those additive fields to distinguish source-range request latency from
scheduler admission and wave-order gaps; they are internal diagnostics and do
not alter OCR shard schemas or profile routing.
A May 8, 2026 Docling real-fixture diagnostic forced one DocLayNet region into
the hosted lane. It preserved zero errors and stable order with `9/1`
page/region OCR blocks, but the `baidu/qianfan-ocr-fast:free` request tailed
at about `16.7s`; the scheduler trace showed rendered-region queue wait near
zero. That result keeps Qianfan as a diagnostic provider and directs the next
optimization toward provider/model choice, region payload shape, or
scaffolded decoding rather than scheduler admission.
After normalizing quoted OpenRouter API-key values in the analyzer, the same
real DocLayNet region completed through `mistralai/ministral-3b-2512` with
zero errors, stable order, `9/1` page/region blocks, force refresh around
`9.3s`, hosted request p95 around `7.4s`, and rendered-region queue wait near
zero. `qwen/qwen3-vl-8b-instruct` preserved correctness on the same probe but
tailed around `27.3s`, so it is not a promotion candidate for this region.
Within that opt-in pipeline, the benchmark can set
`WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_AHEAD` above `1`, or pass
`--rust-pdf-hosted-vlm-region-render-ahead`, to pre-render multiple page-region
chunks while hosted requests are in flight. Final OCR inputs are normalized
back to deterministic reading order before the Rust row/order gate runs.
The benchmark can additionally set
`WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_CHUNK=region`, or pass
`--rust-pdf-hosted-vlm-region-render-chunk region`, to split each recovery
region into its own render chunk instead of grouping by page. This is
diagnostic-only: the milestone canary brought first region readiness down to
`1589.972792 ms` and preserved all precision gates, but force refresh regressed
to `14942.6205 ms`, so page-grouped chunks remain the default. The all-region
render chunk diagnostic is also rejected: r80e preserved precision but delayed
the first hosted dispatch to `12528.588583 ms` and regressed force refresh to
`21670.3075 ms`.
The analyzer-side direct hosted VLM/OCR worker can additionally opt into a same-page,
same-parent region composite canary through
`WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE` or the benchmark flag
`--hosted-vlm-ocr-region-composite-size`. Composite responses are accepted only
when sentinel markers split back into one non-empty Markdown result per region;
otherwise the worker falls back to individual region requests and the Rust
row/order gate still sees the unchanged OCR shard result schema. The benchmark
can additionally set `WENDAO_HOSTED_VLM_OCR_REGION_ATLAS_MODE=same-page-json`
or `--hosted-vlm-ocr-region-atlas-mode same-page-json` to pack each same-page
region composite group into one labeled PNG atlas and require strict JSON keyed
by exact shard markers. Atlas mode remains a request-surface canary: validation
failure falls back to individual region requests, and promotion still depends
on the unchanged Rust row/order, character-floor, and force-refresh gates.
For table and complex-layout region recovery, the benchmark can also opt into
`--hosted-vlm-ocr-scaffold-mode region-table-json`. Studio forwards that as
`WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_SCAFFOLD_MODE=region-table-json` and writes
`_hosted_vlm_region_scaffolds.json` beside the rendered hosted recovery region images. The
sidecar records shard ids, parent shard ids, source and raster fingerprints,
render DPI, crop boxes, source pixel boxes, source-page profile signals, and a
conservative scaffold kind such as `table_candidate`,
`complex_layout_candidate`, or `manual_region_candidate`. Studio does not
invent hard table row or column counts in this slice; the sidecar is a
fingerprinted routing and prompt contract consumed by the analyzer worker,
which must return failed rows on scaffold validation errors so the Rust
precision fallback remains authoritative.
The current OpenRouter/Qianfan fast real run rejects scaffold composite as a
promotion path: same-page composite scaffold responses failed row-count
validation and a single-region scaffold response returned empty canonical
text, causing the hybrid provider to fall back to full Docling. Scaffold mode
therefore remains a strict provider-capability canary rather than the default
hosted recovery path.
The non-scaffold region-composite canary also remains rejected for the current
OpenRouter path. r72 preserved the precision envelope but tailed at
`17806.492208 ms`; r83 attempted composite size `3` but failed with a Flight
`BrokenPipe` before a valid OCR metrics report. Composite request-surface
reduction now has analyzer-side exception fallback for failed composite
attempts. r84 verified the guard with valid OCR metrics and zero error rows,
but it still missed the current `8201.568417 ms` OpenRouter envelope, so
composite remains a benchmark-only canary.
Hosted VLM/OCR modes stay opt-in and promote per profile only when the real
benchmark gate proves the current precision envelope and beats the 12,856.546
ms `fast-risk-window` force-refresh evidence; promoted replacements should
also beat the current `8201.568417 ms` OpenRouter region-recovery envelope.
Benchmark reports expose that decision through `hostedVlmPromotionGate`, which
keeps hosted profile promotion tied to the frozen precision, row/order,
character-floor, hosted-request, force-refresh, shard-cache reuse, and zero
scaffold-validation-failure gates.

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
