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

## Document Extraction Artifact Cache

Studio consumes the shared L2 artifact cache interface from
[`xiuxian-db-store`](../xiuxian-db-store/README.md) for PDF OCR shard result
bytes. The OCR scheduler stores successful shard result Arrow IPC batches under
the `ArtifactBlobCache` contract, using the source digest, profile digest, and
shard digest as cache identity. This keeps the cache owner in `xiuxian-db-store`
while Studio remains responsible for scheduling, row-order restoration, and
precision validation.

Legacy Office extraction uses the same shared artifact cache contract for its
Rust parser projection. Studio writes a local projection report beside the
resource output with source digest, parser profile, cache backend/status,
character counts, line counts, tabular row counts, maximum visible columns, and
Markdown fenced-block counts. These fields are diagnostics and precision-gate
evidence only; the stable document resource Arrow schema is unchanged.

Standalone image extraction stays on the primary document-extract resource
schema. In Gateway `auto` mode, Studio keeps image files on the synchronous
document route but rewrites the default `full` document profile to
`hosted-vlm-image-extract-v1`, letting the analyzer call the configured
OpenAI-compatible vision endpoint and return ordinary Markdown resource rows.
Attachment-owned image audit and tile facts remain materialization inputs, not
a separate public Flight schema.

## Model Routing Metadata

Studio consumes the Wendao route contract from
[`xiuxian-llm`](../xiuxian-llm/docs/01_core/101_xiuxian_llm_core_boundary.md)
when forwarding model-backed work to analyzer Flight routes. Studio may attach
Gateway-selected provider/model/backend profile metadata to Arrow Flight
requests, but model/provider policy belongs to the routing plane and the shared
`xiuxian-llm` contract. Analyzer workers execute the selected backend profile;
they do not decide routing policy locally.

Audio document extraction is the first concrete admission path. When
`WENDAO_MODEL_ROUTING_MODE=vllm-sr` is active, Studio builds a route intent from
source hash, duration, shard count, plan strategy, and backend profile, calls
the vLLM-SR OpenAI-compatible route probe, and forwards selected route metadata
on the unchanged `/analysis/audio-shards` Flight contract. The audio cache
manifest includes the selected provider/model/backend profile so warm transcript
artifacts cannot cross incompatible route decisions. `deterministic` mode must
be configured explicitly.

Standalone image extraction uses the same model-routing contract when the
profile resolves to `hosted-vlm-image-extract-v1`. Studio admits the image route
through vLLM-SR before sync cache reuse, forwards selected provider/model/backend
metadata on the unchanged `/analysis/document-extract` Flight route, and writes
a route manifest beside the resource cache so image artifacts cannot cross
incompatible route decisions.

Full document extraction artifact directories are still mirrored by the hybrid
route because resource-path rewriting depends on directory context. Moving that
directory mirror behind the L2 byte-cache contract requires a separate manifest
or archive encoding slice; it must not bypass resource-order or precision gates.

## SearchStrategyFlow Flight Materialization

Studio owns the native Arrow Flight materialization layer for
SearchStrategyFlow retrieval routes. `xiuxian-julia-core` may emit the
graph-owned strategy trace and Rust bridge route receipts, but decoded payload
proof remains here because Studio owns the service-backed `/search/repos/main`,
`/analysis/repo-projected-page-index-tree`,
`/analysis/repo-projected-retrieval-context`, and `/graph/neighbors`
providers. The current execution proof decodes all four routes, records
route-level row counts, records decoded payload anchors, and keeps direct file
reads outside the SearchStrategyFlow path.
This proof is an in-process Studio test helper only. The production
materialization path must stay Arrow Flight-native: external agent surfaces
talk to the Rust bridge, and the Rust bridge talks to the Studio/Wendao Flight
service endpoint.
SearchStrategyFlow intent and query understanding stay outside the public
Gateway OpenAPI surface; `pi-wendao` owns that agent-facing layer. Studio does
not expose a `/api/search/strategy-flow` REST route. Gateway REST or Flight is
used after the query-understanding subagent and WendaoGraph.jl have selected
concrete retrieval/materialization routes.

## Dataset Ontology Flight Handoff

Studio owns the Gateway host provider for the dataset-to-ontology Flight
handoff route `/ontology/dataset/materialize`. The route contract and metadata
validation live in `xiuxian-wendao-server`; DuckDB/Arrow-SQL materialization
remains in Wendao runtime code. Studio attaches the provider to the same
Gateway Flight service bundle as search, graph, VFS, analysis, and SQL routes.

The current provider resolves admitted manifest table payload ids to
cache-local Arrow IPC streams, loads the accepted Healthcare mapping SQL from
the ontology source-contract tree, and delegates execution to the Wendao
DuckDB materializer. Missing payloads, unsafe payload ids, source-content
fingerprint mismatches, row-count mismatches, unsupported mappings, and
materialization validation failures fail deterministically at the Studio
provider boundary instead of falling back to Python, direct RDF mutation, or
server-crate DuckDB execution. The response uses a stable Arrow envelope:
the first batch is the compact materialization report, followed by
`semantic_objects`, `semantic_relations`, and `semantic_projection_state`
read-model rows encoded with their source table name and row JSON payload. This
keeps the Flight stream schema stable while preserving the compiled ontology
read-model facts for downstream graph/proof consumers.

## Ontology Candidate Inspection Flight

Studio owns the Gateway host provider for `/ontology/candidates/inspect`. The
route contract and metadata admission live in `xiuxian-wendao-server`; Studio
resolves the `epistemeRegistryId` from `wendao.toml`, loads the selected
Episteme repository config, locates the requested ontology-generation `runId`,
and delegates candidate Parquet inspection to `xiuxian-wendao-sql`.

The provider returns a compact Arrow summary batch with DuckDB inspection
counts and pass/fail status, while `FlightInfo.app_metadata` carries the
bounded JSON inspection report for traceability. The route does not add a REST
surface, read candidate TSV files, call Python DuckDB, mutate RDF, or promote
raw candidates into ontology truth. It exists so pi-wendao, Julia, and other
runtime callers can stay on the Arrow/Flight data plane when checking whether a
candidate read model is ready for downstream proof.

## Episteme Source Contract Admission

The first episteme onboarding command writes a structure/TOC Org ledger:

```bash
wendao episteme structure write-toc \
  --episteme-registry-id medical \
  --corpus-root <corpus-root> \
  --validation-mode metadata-only \
  --run-id toc_seed
```

This command reads source file rows and writes ignored `toc.org` and
`receipt.json` artifacts under `<episteme-root>/runs/structure/<run-id>/` by
default. The Org ledger records route/category summary tables first, then file
ids, source-relative paths, byte sizes, hashes, categories, languages, and
extraction routes. The default
`metadata-only` validation mode checks manifest shape, file presence, byte
sizes, duplicate ids and paths, extension-route alignment, categories,
language, and unlisted corpus files without reading file contents for sha256.
Use `--validation-mode full-hash` when the run must prove source-content
fingerprints. The TOC ledger does not embed raw corpus text, execute OCR,
execute ASR, call LLMs, export SQL/RDF, or promote ontology truth.

The source-contract command surface can also compile the first structural facts
seed for ontology work:

```bash
wendao episteme source-contract write-structural-facts \
  --episteme-registry-id medical \
  --validation-mode metadata-only \
  --run-id structural_seed
```

This command resolves `episteme.toml` runtime defaults in the same way as the
TOC command, then delegates implementation to
[`xiuxian-wendao-episteme`](../xiuxian-wendao-episteme/README.md). It writes
ignored `structural_facts.json`, `structural_facts.org`, document rows, path-anchor
rows, and containment relation rows under
`<episteme-root>/runs/structure/<run-id>/` by default. It does not read raw
file text, execute OCR/ASR/LLM extraction, write RDF, or promote ontology truth.
Use `--validation-mode full-hash` when the seed must prove all source
fingerprints before LLM-assisted reasoning or later RDF proposal work.

The same source-contract surface can compile the deterministic reasoning packet
from a structural facts run:

```bash
wendao episteme source-contract write-structural-facts-reasoning-packet \
  --episteme-registry-id medical \
  --structural-facts-run-id structural_seed \
  --run-id reasoning_seed
```

By default this reads
`<episteme-root>/runs/structure/<structural-facts-run-id>/structural_facts.json`
and writes `reasoning_packet.org`, `reasoning_packet.tsv`,
`reasoning_packet.json`, and `reasoning_packet_report.json` under
`<episteme-root>/runs/ontology-generation/<run-id>/`. The packet is still an
evidence proposal input surface only: it does not read private source text,
call an LLM, run OCR/ASR, write RDF, or promote ontology truth. It carries
structure-targeting fields such as `evidenceTargetIntent`,
`evidenceAnchorKind`, and `evidenceStructureHint` so downstream seed and
fill-plan rows do not rely on prompt prose to decide the reasoning slot.

After a reasoning packet exists, Studio can ask the Episteme crate to seed a
fillable Org proposal ledger:

```bash
wendao episteme source-contract write-structural-facts-reasoning-ledger-seed \
  --episteme-registry-id medical \
  --reasoning-packet-run-id reasoning_seed \
  --run-id reasoning_ledger_seed
```

The command reads
`<episteme-root>/runs/ontology-generation/<reasoning-packet-run-id>/reasoning_packet.json`
and writes `reasoning_ledger_seed.org`, `reasoning_ledger_seed.tsv`,
`reasoning_ledger_seed.json`, and `reasoning_ledger_seed_report.json` under
`<episteme-root>/runs/ontology-generation/<run-id>/`. It creates blank,
review-gated proposal slots from the packet target intent. Coarse document
rows still produce object and relation review slots, while service catalogs and
row-like evidence are routed to service-catalog or object-instance review slots
instead of object-model `ObjectType` slots. The command does not read private
source text, call an LLM, write RDF, or promote ontology truth.

After a ledger seed exists, Studio can compile a Qianji/BPMN-oriented fill plan
without executing the workflow:

```bash
wendao episteme source-contract write-structural-facts-reasoning-fill-plan \
  --episteme-registry-id medical \
  --ledger-seed-run-id reasoning_ledger_seed \
  --run-id reasoning_fill_plan
```

The command reads
`<episteme-root>/runs/ontology-generation/<ledger-seed-run-id>/reasoning_ledger_seed.json`
and writes `reasoning_fill_plan.org`, `reasoning_fill_plan.tsv`,
`reasoning_fill_plan.json`, and `reasoning_fill_plan_report.json` under
`<episteme-root>/runs/ontology-generation/<run-id>/`. The plan records workflow
keys, activity kinds, seed ids, evidence anchors, target intents, and target
ledger groups as typed data. It does not execute Qianji, read private source
text, call an LLM, mutate source files, write RDF, or promote ontology truth.

After a fill plan exists, Studio can compile Qianji schedule-admission inputs:

```bash
wendao episteme source-contract write-structural-facts-reasoning-qianji-schedule-plan \
  --episteme-registry-id medical \
  --fill-plan-run-id reasoning_fill_plan \
  --run-id qianji_schedule_plan \
  --target-ledger-field-group service_catalog_review \
  --evidence-target-intent service_catalog_extraction \
  --reasoning-context-shard-mode service-catalog-table-rows \
  --reasoning-context-shard-row-limit 2 \
  --evidence-extraction-run-id docling_document_cache \
  --openai-compatible-max-tokens 1024
```

The command reads
`<episteme-root>/runs/ontology-generation/<fill-plan-run-id>/reasoning_fill_plan.json`
and writes `qianji_schedule_plan.org`, `qianji_schedule_plan.tsv`,
`qianji_schedule_plan.json`, and `qianji_schedule_plan_report.json` under
`<episteme-root>/runs/ontology-generation/<run-id>/`. It emits Qianji-shaped
activity task payloads with stable activity ids, task queue, input claim-check
reference, and idempotency key. It does not append Qianji control ledger events,
enqueue hot-state work, execute workers, call an LLM, read private source text,
mutate source files, write RDF, or promote ontology truth.
The target filter flags are optional. When provided, Studio forwards them to
the Episteme compiler so schedule selection happens before `--limit`; this is
the preferred way to run a structure-specific live proof such as service
catalog or object-instance review.
The reasoning-context-shard flags are also optional and default to disabled. For
service-catalog prompt-audit runs, `service-catalog-table-rows` asks the
Episteme compiler to split a Docling Markdown table into deterministic row
windows before Qianji admission. Each emitted schedule item carries a shard id
and row range in JSON, TSV, Org, task metadata, and the generated context
artifact. This is a latency and reliability guard for hosted model calls; it
does not lower extraction DPI, change source evidence, or promote ontology
truth.
The OpenAI-compatible prompt-audit model defaults to
`deepseek/deepseek-v4-pro`; pass `--openai-compatible-model` only for an
intentional comparator run. The prompt-audit flags ask the Episteme crate to
emit local prompt and context artifacts plus Qianji request-audit metadata. They
require at least one `--evidence-extraction-run-id`; Studio resolves the
extraction run root from `episteme.toml` or the episteme run layout unless
`--evidence-extraction-run-root` is supplied. Provider execution still belongs
to the Qianji worker. The generated context includes the same target intent and
structure hint. Object-model review contracts are only emitted for object or
relation target groups; service-catalog and object-instance target groups use
review-only concrete `object_candidate` contracts.

After TOC generation, callers can read one targeted evidence row by file id:

```bash
wendao episteme evidence read \
  --episteme-registry-id medical \
  --corpus-root <corpus-root> \
  --file-id <source-contract-file-id>
```

This command resolves the selected source-contract row, checks the configured
source boundary, and returns deterministic source metadata. Plain-text safe
extensions can include a bounded UTF-8 preview controlled by
`--max-preview-bytes`; binary documents, PDFs, images, and audio are returned
as route references only. The command does not accept arbitrary source paths,
copy raw private files, execute OCR/ASR/LLM extraction, export SQL/RDF, or
promote ontology truth. Use `--validation-mode full-hash` when the read must
prove source-content fingerprints before downstream extraction or promotion.

After a human, agent, or LLM selects relevant source-contract ids from the TOC,
Studio can write a deterministic evidence selection ledger:

```bash
wendao episteme evidence write-selection-plan \
  --episteme-registry-id medical \
  --run-id selection_seed \
  --file-id <source-contract-file-id> \
  --selection-reason "agent selected source files"
```

This command writes ignored `selection.org`, `selection.tsv`, and
`receipt.json` artifacts under
`<episteme-root>/runs/evidence-selection/<run-id>/` by default. It rejects
duplicate or unknown `file_id` values, records only source-contract metadata
and next-route hints, and never embeds raw corpus text or binary content. Use
`--validation-mode full-hash` when the selection must prove source-content
fingerprints before extractor planning. If the episteme repository provides
`episteme.toml` with `[runtime].corpus_root`, `[runtime].evidence_selection_run_root`,
`[runtime].structure_run_root`, or `[runtime].extraction_run_root`, Studio uses
those values as defaults. Explicit CLI flags still override repository
defaults.

The `wendao` CLI exposes the episteme source-contract run-plan writer as:

```bash
wendao episteme source-contract plan-extraction-run \
  --episteme-root <episteme-root> \
  --run-id source_contract_seed \
  --selection-run-id selection_seed \
  --route document_text_evidence \
  --limit 12
```

This command delegates to Rust-owned Episteme source-contract services. New
implementation surfaces belong in
[`xiuxian-wendao-episteme`](../xiuxian-wendao-episteme/README.md), while
existing migration surfaces may still be reached through
[`xiuxian-wendao`](../xiuxian-wendao/README.md). The implementation consumes
[`xiuxian-wendao-parsers`](../xiuxian-wendao-parsers/docs/episteme-source-contracts.md)
DTOs. It writes ignored `tasks.tsv`, `receipt.json`, and `outputs/` run-plan
artifacts only. It does not execute OCR, ASR, LLM extraction, or RDF promotion,
and it does not promote raw content into ontology truth. Planning uses
`contract_shape_only` validation, so it checks manifest and mapping-ledger
shape, queue/file consistency, filters, and selected `file_id` coverage without
walking or hashing the source corpus. The emitted report includes total queue
rows plus selected route/category counts for direct route-census auditing. Full
sha256 proof remains on explicit validation, read-model, or promotion paths. When
`--selection-run-id` is supplied, the planner reads
`<episteme-root>/runs/evidence-selection/<selection-run-id>/selection.tsv` by
default and treats its `file_id` values as a hard constraint. Every selected id
must map to a pending queue row after any route/category filters; the planner
fails instead of silently dropping selected evidence. `--selection-root` can
point at a non-default selection artifact root. Image OCR admission uses the
same command with `--route image_ocr_evidence`; this only writes planned
cache-only OCR tasks and still performs no OCR, ASR, LLM extraction, or RDF
promotion.

When a private episteme wants the current image OCR cache bridge to run through
one Rust-owned entry point, use:

```bash
wendao episteme source-contract run-image-ocr-cache \
  --episteme-root <episteme-root> \
  --run-id image_ocr_seed \
  --limit 12
```

This command first writes an `image_ocr_evidence` run plan, then invokes the
configured analyzer command, defaulting to `wendao-image-ocr-jsonl`. Cache
materialization is delegated to
[`xiuxian-wendao-episteme`](../xiuxian-wendao-episteme/README.md), which
validates the queue-keyed JSONL and writes ignored evidence cache outputs.
`--dry-run` prints the resolved analyzer command without calling external OCR
or writing cache outputs. The wrapper is an execution convenience only: Studio
owns CLI orchestration; the Episteme crate owns source-contract cache identity,
cache writes, and report emission; Python remains the OCR adapter boundary.
OCR text is written only to the ignored evidence cache as review-required,
promotion-blocked material. The command does not add a public Gateway/OpenAPI
route and does not promote raw OCR output into RDF truth.

For real private corpus probes, operators may split analyzer execution from
Rust cache materialization. First run the package-owned analyzer adapter
directly, writing queue-keyed JSONL under the run directory. Then ask Studio to
reuse that result file:

```bash
wendao episteme source-contract run-image-ocr-cache \
  --episteme-root <episteme-root> \
  --run-id image_ocr_seed \
  --ocr-results-jsonl <episteme-root>/runs/extraction/image_ocr_seed/ocr_results.jsonl \
  --use-existing-results
```

The same split bridge is available for Docling document evidence:

```bash
wendao episteme source-contract run-docling-document-cache \
  --episteme-root <episteme-root> \
  --run-id docling_document_seed \
  --document-results-jsonl <episteme-root>/runs/extraction/docling_document_seed/document_results.jsonl \
  --use-existing-results
```

`--use-existing-results` skips analyzer process startup only. Studio still
rewrites and validates the task plan, the Episteme crate still validates the
queue-keyed JSONL against planned tasks and source hashes, and promotion into
RDF remains blocked.

After cache-local evidence exists, Studio can ask the Episteme crate to generate
review-gated ontology candidates:

```bash
wendao episteme source-contract generate-ontology-candidates \
  --episteme-root <episteme-root> \
  --run-id ontology_seed \
  --extraction-run-id docling_document_seed \
  --extraction-run-id image_ocr_seed
```

The command writes `candidate_objects.tsv`, `candidate_relations.tsv`,
`candidate_evidence.tsv`, `ontology_candidate_objects.parquet`,
`ontology_candidate_relations.parquet`, `ontology_candidate_evidence.parquet`,
`review_ledger.org`, and `receipt.json` under
`<episteme-root>/runs/ontology-generation/<run-id>/` by default. It consumes
source-contract metadata, mapping-ledger terms, and ignored cache outputs, then
emits candidate rows with promotion blocked. The TSV files are compatibility
projections; the Parquet files are the typed read model for downstream SQL,
search, and proof slices. It does not write RDF, mutate the source ontology,
persist raw extracted text in the candidate artifacts, or create a Gateway
route.

Studio can also import canonical Qianji review artifacts into the same
candidate-review surface:

```bash
wendao episteme source-contract import-qianji-review-candidates \
  --episteme-root <episteme-root> \
  --run-id ontology_seed_from_qianji \
  --qianji-review-artifact <episteme-root>/runs/ontology-generation/<qianji-run>/review.json
```

The command delegates parsing and validation to `xiuxian-wendao-episteme`. It
requires a Qianji OpenAI-compatible response envelope with a canonical
`episteme_review` object, imports only review-only object or relation candidate
patches, writes the normal candidate TSVs plus
`qianji_review_candidate_import_report.json`, and runs the deterministic
candidate review gate. Relation patches are expanded into endpoint object rows
plus a relation row so endpoint references are checked before promotion. It
stores source ids, paths, evidence hashes, and evidence character counts; it
does not persist raw private quote text in the candidate TSVs, mutate RDF, or
mark rows as ontology truth.
Canonical zero-candidate reviews are accepted only when they include model
blockers. In that case the command writes header-only candidate TSVs, records
the blocker count in the import report, and leaves promotion surfaces empty.

The generated run can then be reviewed through the deterministic quality gate:

```bash
wendao episteme source-contract review-ontology-candidates \
  --episteme-root <episteme-root> \
  --run-id ontology_seed
```

This writes `candidate_review.tsv` and `quality_report.json` under the same
ontology-generation run directory. The report flags duplicate candidate ids,
missing relation references, unsafe promotion flags, ontology-truth flags, and
evidence strength. A passing review report is only a precondition for a later
promotion-review slice; it is not RDF export and does not promote private
content into ontology truth.

Studio can also inspect the typed candidate Parquet read model through the
bounded DuckDB surface in `xiuxian-wendao-sql`:

```bash
wendao episteme source-contract inspect-ontology-candidates \
  --episteme-root <episteme-root> \
  --run-id ontology_seed
```

This resolves
`<episteme-root>/runs/ontology-generation/<run-id>/`, reads the standard
candidate Parquet files, and reports row counts, kind counts, blocked-review
checks, ontology-truth checks, raw-to-RDF promotion checks, and relation
endpoint integrity. The command does not read candidate TSV projections and
does not mutate RDF or start a Gateway route.

After the review gate passes, Studio can ask the Episteme crate to write a
reviewable RDF draft and promotion proposal:

```bash
wendao episteme source-contract write-ontology-rdf-draft \
  --episteme-root <episteme-root> \
  --run-id ontology_seed
```

This command writes `rdf_draft.ttl`, `promotion_proposal.org`, and
`promotion_proposal.json` beside the reviewed candidate run. It fails if the
review report is missing, failed, or inconsistent with the candidate rows. The
artifacts are proposal surfaces only: Studio does not mutate source ontology
RDF, does not embed raw extracted text, and does not mark candidates as
ontology truth.

The next command writes the pending promotion review packet:

```bash
wendao episteme source-contract write-ontology-promotion-review \
  --episteme-root <episteme-root> \
  --run-id ontology_seed
```

This command consumes the clean RDF draft proposal and writes
`promotion_review.tsv`, `promotion_review.org`, and `promotion_review.json`.
All rows are initialized with `promotion_decision=pending_review`,
`source_mutation_allowed=false`, and `ontology_truth=false`. The packet is the
input for human or LLM review; it still does not mutate ontology source RDF.

After review decisions are written, Studio can ask the Episteme crate to write
a non-mutating promotion apply plan:

```bash
wendao episteme source-contract write-ontology-promotion-apply-plan \
  --episteme-root <episteme-root> \
  --run-id ontology_seed
```

This command consumes `promotion_review.tsv` and writes
`promotion_apply_plan.tsv`, `promotion_apply_plan.org`, and
`promotion_apply_plan.json`. Pending-only reviews produce an empty apply plan.
Approved rows require reviewer provenance and satisfied preconditions, and the
generated plan still carries `sourceMutationAllowed=false` and
`ontologyTruth=false`; it is a source-patch proposal plan, not source RDF
mutation.

The selected source manifest and mapping ledger come from
`<episteme-root>/ontology/manifest.toml`. Runtime defaults may come from
`<episteme-root>/episteme.toml`; if no `[runtime].corpus_root` is configured and
`--corpus-root` is omitted, Studio reads the selected source manifest's
`corpus_root_env` field and resolves the corpus root from that environment
variable. Studio does not hardcode customer repository names, domain names, or
corpus environment variable names. A single-contract episteme repository can be
selected automatically; multi-domain repositories must declare
`[active_source_contract]`.

`wendao.toml` may also declare episteme registries for user-owned or
Wendao-owned topology repositories:

```toml
[episteme.registries.local_knowledge]
path = ".data/local-episteme"

[episteme.registries.remote_knowledge]
url = "https://github.com/example/example-episteme.git"
```

The user-facing syntax is intentionally thin. `path` means local episteme
repository; `url` means Git episteme repository. Rust infers the source kind,
materializes managed Git checkouts through the repository substrate, and writes
resolved revision/cache facts into receipts instead of requiring backend
materialization fields in `wendao.toml`.
When `epistemeRegistryId` is used, Studio loads the enabled registry entries
from the same deployment config, asks the Wendao backend to validate the
manifest reference graph, and only then selects the requested episteme root.
That graph validation checks unique domain ids and manifest extension targets;
it does not require additional user-facing registry syntax.

For approved source-patch preview runs, Studio can ask the Episteme crate to
compile a non-mutating semantic read-model preview:

```bash
wendao episteme source-contract write-ontology-source-patch-semantic-preview \
  --episteme-root <episteme-root> \
  --run-id source_patch_seed
```

This command writes `semantic_objects.tsv`, `semantic_relations.tsv`,
`semantic_evidence.tsv`, `semantic_projection_state.json`, and JSON
projections under the source-patch run directory. Studio only resolves the
episteme root and run id; the Episteme crate owns source-patch validation and
read-model compilation. The command does not mutate RDF source files and does
not make approved preview rows ontology truth.

Studio can also discover the optional WendaoGraph ontology quality proof
service from the effective deployment `wendao.toml`:

```toml
[wendaograph.ontology_read_model_quality]
base_url = "http://127.0.0.1:19091/"
timeout_seconds = 30
max_in_flight_requests = 2
```

When this table is present, the dataset ontology Flight provider sends the
compiled semantic read-model rows to the runtime Arrow Flight binding and
returns the advisory proof summary in `FlightInfo.app_metadata`. When it is
absent, the provider keeps the materialization metadata stable and leaves the
proof summary empty. Julia still receives only compiled read-model tables; it
does not read `wendao.toml`, RDF files, raw corpus files, or private source
manifests.

The CLI can target a configured registry id:

```bash
wendao episteme source-contract plan-extraction-run \
  --episteme-registry-id local_knowledge \
  --corpus-root <corpus-root> \
  --run-id source_contract_seed
```

Studio also exposes the same writer through a bounded operational Gateway
endpoint:

```http
POST /api/episteme/source-contract/extraction-run-plan
```

The JSON body accepts either `epistemeRoot` or `epistemeRegistryId`, plus
`corpusRoot`, `runRoot`, `selectionRunId`, `selectionRoot`, `runId`, `route`,
`category`, and `limit`. Relative request paths resolve from the Gateway
project root. Repository-owned `episteme.toml` values are used as defaults for
the corpus root, extraction run root, and evidence-selection root before
falling back to the selected source manifest's corpus-root environment
variable. When `selectionRunId` is supplied, the Gateway reads that
selection's `selection.tsv` and treats its `file_id` values as hard constraints,
matching the CLI semantics without requiring a CLI process. This endpoint is a
service-admission path only. It
intentionally does not enter the stable OpenAPI route inventory in this slice,
and it does not execute OCR, ASR, LLM extraction, or RDF promotion.

Studio also exposes targeted source-contract evidence reads through the
resident Gateway path:

```http
POST /api/episteme/evidence/read
```

The JSON body accepts either `epistemeRoot` or `epistemeRegistryId`, optional
`corpusRoot`, required `fileId`, optional `maxPreviewBytes`, and optional
`validationMode` (`metadata-only` or `full-hash`). Repository-owned
`episteme.toml` supplies the corpus-root default when the request omits
`corpusRoot`. The response is the Rust-owned targeted evidence read report: it
can include bounded plain-text previews for text-like sources, returns binary
sources as evidence references, and keeps `extractionExecuted=false` and
`rawToRdfPromotionAllowed=false`. This operational Gateway path exists so
agent/LLM workflows can move from TOC to selected `file_id` evidence without a
CLI process; it does not run OCR, ASR, LLM extraction, or RDF promotion.

The resident Gateway path can also write evidence-only selection plans:

```http
POST /api/episteme/evidence/selection-plan
```

The JSON body accepts either `epistemeRoot` or `epistemeRegistryId`, optional
`corpusRoot`, optional `runRoot`, required `runId`, required `fileIds`,
optional `selectionReason`, and optional `validationMode` (`metadata-only` or
`full-hash`). Repository-owned `episteme.toml` supplies the corpus-root and
evidence-selection run-root defaults when the request omits them. The response
is the Rust-owned evidence selection write report, and the written
`selection.tsv` can be consumed by the extraction run-plan endpoint through
`selectionRunId`. This route writes a reviewable selection ledger only; it does
not execute extraction, OCR, ASR, LLM inference, or RDF promotion.

Studio also exposes ontology registry snapshot read-model admission through a
bounded operational Gateway path:

```http
POST /api/episteme/ontology-registry/read-model
```

The JSON body accepts either `epistemeRoot` or `epistemeRegistryId`, plus
optional `qualityProofMode` (`disabled` or `if-configured`). The Gateway
resolves the selected Episteme root, delegates `ontology/registry.json`
admission to `xiuxian-wendao-episteme`, asks `xiuxian-wendao` to materialize
the admitted snapshot into `semantic_objects`, `semantic_relations`, and
`semantic_projection_state`, and returns only schema, status, source revision,
and row-count summaries. By default the route stays local and does not run a
WendaoGraph proof. When `qualityProofMode="if-configured"` and the Julia
bridge plus WendaoGraph quality endpoint are available, the response includes
a bounded `qualityProof` summary with request row counts, Arrow IPC payload
byte sizes, selected transport, and response batch/row counts. The route does
not return Arrow batches through JSON, parse registry files in Julia or
Python, execute extractors, or promote RDF truth.

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
For audio shard execution, Studio owns live Flight dispatch over the analyzer
`/analysis/audio-shards` route. Attachments supplies the model-neutral shard
plans, Arrow rows, transcript admission, and merge/precision gates; analyzer
workers supply Docling or hosted transcript rows. Studio keeps backend selection
as data/configuration, uses the attachment polyglot bridge to select
`x-wendao-audio-workers` when it needs to bound analyzer parallelism, and
invokes the attachment-owned merge helpers while surfacing failed, skipped,
missing, or duplicate shard coverage to the route precision gate.
The audio route also owns the runtime feedback controller for that worker
budget. Workflow transport failures, incomplete shard coverage, and precision
gate failures reduce the next Rust-selected budget; consecutive healthy
workflows can increase it up to the host cap. This keeps production audio
dispatch pressure in Studio/polyglot instead of analyzer-side Python
heuristics.
Studio can start from attachment-owned speech-segment timing facts, ask
attachments to build the Rust speech-window plan, materialize normalized shards,
and submit the stable audio shard input rows over Flight. Python/analyzer
remains the model invocation and result normalization boundary, not the owner of
shard timing or transcript admission identity.
The same optional speech timing facts can now constrain recovery planning:
`execute_recovery_split` accepts model-neutral speech windows, clips them to
selected failed parent shards, and skips the second Flight pass when no speech
fact intersects the failed span. The production route can load those facts from
`WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_SEGMENTS_JSONL`, with optional
`WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_MERGE_GAP_MS`,
`WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_MIN_WINDOW_MS`, and
`WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_LIMIT_CHUNKS` controls. When the sidecar
is unset, default audio behavior is unchanged.
For short-window audio recovery, Studio consumes attachment-owned recovery
candidate mapping and patch gates. A base analyzer response can be merged with a
second recovery analyzer response only after Rust maps every recovery window to
exactly one parent logical shard and the attachment gate accepts the text
quality delta. Failed base rows from analyzer transcript quality gates are also
eligible recovery parents, so unsafe hosted responses become local short-window
retry work instead of merged transcript text. This keeps recovery scheduling and
final precision promotion in Rust while Python remains a model adapter. Studio
can also derive the recovery split plan from the base response, request latency
facts, and attachment-owned risk selection thresholds before sending the second
Flight request. The typed `execute_recovery_split` client path performs the full
base Flight request,
Rust recovery selection/split, recovery Flight request, and Rust patch merge
without depending on analyzer diagnostic CLI artifacts. It is bound to a Qianji
workflow topology and records same-process Arrow memory checkpoints for both
base and recovery input/result batches.
The audio shard client also exposes a Qianji Rust-native workflow-kernel proof
that runs the same plan, materialization, Arrow row construction, Flight
exchange, and merge gate as one typed execution report. The proof returns the
plan, materialized shards, input rows, analyzer response, merge report, and
workflow trace without changing `/analysis/audio-shards` or the audio shard
Arrow schemas. Qianji owns the typed workflow kernel; Studio keeps the live
dispatch and precision boundary. The proof binds an explicit Qianji
`WorkflowTopology` before execution, so missing required stages or out-of-order
stage edges are rejected before the audio merge report can be treated as a
promotion candidate.
The single-pass and recovery workflow proofs register in-process memory
checkpoints for audio shard input and result Arrow batches. Those checkpoints
keep same-process `RecordBatch` buffers available for retry or precision
rechecks, while the wire contract, analyzer boundary, and durable checkpoint
ownership stay unchanged.
The Gateway document-extract route now has an explicit opt-in `audio-shards`
mode. In this mode Studio probes the source duration, asks attachments to build
a full-timeline Rust audio shard plan, invokes attachment-owned planned
transcript admission before byte materialization, materializes normalized shard
files for misses, calls analyzer `/analysis/audio-shards` only for non-admitted
rows, runs the recovery workflow, and returns the existing `audio-transcript`
`text/plain` document resource row plus a parallel `audio-transcript-ledger`
`text/org` evidence resource row only when shard coverage is complete. The Org
ledger uses standard `attachment:` links when materialized shard files exist, so
downstream Org tooling can export Markdown or HTML without changing the audio
Arrow shard schemas. The mode is model-neutral: backend identity comes from
configuration, while concrete local or hosted model invocation remains inside
the analyzer worker registry.
Audio shard materialization enables the db-store Foyer-backed
`ArtifactBlobCache` substrate in the `document-extract-audio-shards` feature.
Studio passes the resolved artifact root into attachments, and attachments
builds the configured db-store backend. `WENDAO_DOCUMENT_EXTRACT_AUDIO_ARTIFACT_CACHE_DIR`
remains a route-specific override; otherwise Studio uses
`WENDAO_ARTIFACT_CACHE_ROOT`, then `$PRJ_CACHE_HOME/wendao/artifacts` when the
project cache root is available. Explicit `WENDAO_ARTIFACT_CACHE_BACKEND` values
can still select `filesystem` or `foyer`, but the Foyer feature path defaults
to Foyer through the shared db-store config. The cache is byte reuse only and
does not affect analyzer request schemas, worker selection, transcript merge
rules, or precision gates.
Attachments also owns accepted transcript admission and a planned admission
index for accepted rows. Studio supplies route/runtime identity and calls that
API before materialization. The planned admission index can satisfy warm rows
before byte materialization on force refresh. Mixed warm/cold runs materialize
only planned miss manifests and send only those rows to analyzer Flight, while
stale or failed planned admissions continue through the existing materialization
and analyzer route.
The document-extract benchmark harness can now exercise this same route with
`--flight-mode audio-shards`. Local provider and Gateway benchmark starts add
the `document-extract-audio-shards` feature automatically for this mode and
forward `--rust-audio-*` controls into model-neutral Studio environment
variables for chunk duration, context windows, materialization format,
optional materialization bitrate, base-worker budget, recovery-worker budget,
and optional speech-timestamp recovery planning. `--rust-audio-speech-segments-jsonl` and its merge,
minimum-window, optional max-window, boundary-snap tolerance, and chunk-limit
companions are forwarded to the same Studio environment variables used by
production startup, so benchmark evidence covers the Rust-owned speech-window
recovery path without changing the audio shard Arrow schema. Analyzer backend
selection still
comes from data-driven request metadata or Python worker flags such as
`--audio-worker hosted`; the managed Wendao analyzer startup selects hosted
OpenRouter audio by default, while local Qwen3-compatible testing uses the same
hosted worker with an OpenAI-compatible local base URL. Docling audio remains an
explicit comparator, not the managed default.
The current source-range auto policy targets seven source PDF pages per worker
before clamping to the adaptive budget, machine cap, remaining permits, and
shard count; diagnostic worker overrides remain benchmark-only.
For legacy Microsoft Office inputs, Studio routes `.doc`, `.xls`, and `.ppt`
through the attachment-owned Rust parser feature before Python analyzer
dispatch. The default managed gateway feature set includes
`document-extract-legacy-office` together with PDF render and audio shards, so
legacy Office support is a Rust gateway capability rather than a Python
source-preparation header.
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
The canonical precision-first planner for the next slice is
`docling-structure-recovery`. In that mode Studio treats Docling as the
structure authority rather than a final full-document fallback. Attachment
page facts classify structure-heavy pages, text-shortcut pages, and OCR/VLM
patch regions. Studio keeps structure-heavy pages on Docling-compatible rows,
routes text-only pages through backend text only when the page is not
structure-heavy, and sends OCR/VLM work only as a patch over Docling structure
blocks or rendered recovery regions. Failed or empty backend-text page rows
may be replaced by page-range Docling output through
`x-wendao-document-extract-page-range`; missing structure, empty rows, or
resource/structure mismatches still escalate to the existing full Docling
fallback. This planner is opt-in and does not change the OCR shard Arrow
input/result schema.
`WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_SIZE` is a benchmark
override for splitting contiguous page-range fallback into smaller Docling
conversion ranges. In `docling-structure-recovery`, Studio defaults the
direct page-range path to three-page chunks because the current DocLayNet
fixture evidence shows full-range page conversion regresses while preserving
the same Docling structure parity. `WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_CONCURRENCY`
can cap how many of those ranges run at once, separating chunk-size evidence
from Python/Docling executor contention. The analyzer benchmark pairs this
with `WENDAO_DOCUMENT_EXTRACT_FULL_THREADS=1` in auto mode so Rust owns outer
parallelism and each Python Docling worker avoids nested thread contention.
The May 8, 2026 DocLayNet fixture evidence for that shape is
`10127.429667 ms` cold force refresh with zero error rows, stable order, and
Docling structure parity, below the locked `12856.546292 ms` baseline.
When the analyzer benchmark explicitly prewarms Docling document extraction
before worker readiness, the same shape crossed the `<10s` stretch target:
prewarming page range `1:1` preserved zero errors, stable order, Docling
structure parity, `13` resource rows, and `12` structure blocks while reducing
force refresh to a best sample of `8715.070334 ms`. A repeat of the same shape
preserved correctness but measured `11627.203583 ms`, so this is a benchmark
readiness control with visible variance, not a Studio default and not an output
cache bypass.
Studio normalizes duplicate chunk wrapper rows, keeps `docling_json` as
transport metadata rather than a structure block, and reports pure page-range
conversion cost as
`doclingPageRangeFallback` instead of OCR scheduler time. The timing sidecar
also records page-range chunk counts, per-chunk elapsed time, row counts, and
the slowest chunk so benchmark reports can isolate Docling conversion tail
latency from Rust scheduler or Flight overhead. When the planner has
no hosted/local region controls, Studio can use a direct page-range recovery
path that skips render/profile setup while leaving the same Docling structure
parity and full-fallback guards active.
For benchmark-only tail splitting, Studio accepts
`WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_PLAN` as an exact
1-based inclusive range list such as `1:3,4:4,5:6,7:9`. The plan must cover the
Docling fallback page set exactly; missing, duplicate, or out-of-set pages fail
the run instead of falling through to a lossy merge. This lets benchmarks split
known slow page ranges without changing the default three-page evidence shape.
Studio now applies the same tail-preserving shape automatically for large
`docling-structure-recovery` fallback runs when source-page profiles are
available: it keeps the final three-page context together and spends the extra
chunk on the highest structure-cost non-tail page group. Structure cost is owned
by `xiuxian-wendao-attachments`, and Studio records that cost in the page-range
plan and timing report so benchmark review can distinguish Docling structural
pressure from worker-count tuning. The current DocLayNet fixture evidence
preserves zero error rows, stable order, and Docling structure parity with the
default `1:3,4:4,5:6,7:9` plan. For opt-in high-cost tail diagnostics,
`WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_STRUCTURE_COST_BUDGET` splits
automatic `docling-structure-recovery` ranges whose estimated structure cost
exceeds the configured budget. This control is evidence-gated, preserves
contiguous page coverage, and may spend spare document-extract endpoint
capacity only when the resulting ranges still run in one Docling execution
wave. It does not change the Arrow OCR shard schema and is not a default
promotion path. The May 9, 2026 spare-capacity canary with five endpoints,
converter-cache profile mode, and the `1:3,4:4,5:6,7:7,8:9` plan preserved
zero error rows and Docling structure parity while measuring `5549.271542 ms`
force refresh on the DocLayNet structure fixture.
Force refresh remains variance bound:
plan-aligned readiness-control samples using the benchmark prewarm token
`rust-page-range-chunk-plan` measured `10222.013209 ms` and `10525.865292 ms`,
the comparable default sample is `13054.962625 ms`, and a default repeat
regressed to `18782.907959 ms` due to Docling worker tail latency. Therefore
this planner repair is accepted for structure-preserving routing, but speed
promotion still requires a separate worker warm-path or tail-hedging fix. The
targeted explicit canary `1:2,3:3,4:4,5:6,7:9` also preserved structure parity
but measured `10459.849834 ms`; it reduced the front chunk cost while leaving
the `7:9` Docling convert tail near `9670.715584012214 ms`, so additional
front splitting is not an accepted default. The tail-split canary
`1:3,4:4,5:6,7:8,9:9` also preserved structure parity but measured
`10570.345 ms`; its single-page `9:9` convert still reached
`9849.450749985408 ms`, so further page-count splitting is not an accepted
default. The
`docling-structure-text` page-range profile is rejected for this fixture because
it preserved structure parity but regressed to `17290.500583 ms`. A naive
8-endpoint, `7000 ms` hedge canary also preserved structure parity but
regressed to `20073.933416 ms`, so duplicating all slow page-range requests is
not an accepted default.
The current OpenRouter benchmark canary also enables analyzer-side
`region-whitespace-trim` request-image optimization. It preserves render DPI,
Arrow OCR shard rows, and Rust row/order validation while cutting hosted region
payload bytes. The current promoted OpenRouter evidence uses
`mistralai/ministral-3b-2512`, render-dispatch with render-ahead `3`, region
trim, and an explicit `2s` hosted hedge. Two May 9, 2026 milestone runs
preserved zero error rows, stable `27` rows, `21/6` page/region OCR blocks, and
the frozen character floor: best force refresh `7338.796584 ms` with
`metricsResultChars=115735`, and repeat force refresh `8322.027792 ms` with
`metricsResultChars=115925`. The hosted request wall span was `5166 ms` and
`6140 ms`; both runs used `12` HTTP attempts for `6` logical hosted region
requests, so this remains an explicit benchmark profile decision rather than a
global default. The older endpoint-local `4s` hedge sample at `8201.568417 ms`
and the 2026-05-07 r59/r60 evidence remain historical regression controls. The
older Qianfan OCR trim run remains valid but non-promoted at `13992.340875 ms`.
A same-shape `1s` hedge canary stayed precision-valid at `8562.0245 ms` but
did not beat the `2s` envelope.
Follow-up current-rev canaries keep that envelope unchanged. Disabling
fast-text top-up is rejected because force refresh measured `9169.448167 ms`
but `metricsResultChars=100981` fell below the frozen `103984` floor. A page
`5` prewarm plus `single-page-first` affinity run preserved precision and
measured `8516.511291 ms`, but it did not beat the current repeat
`8322.027792 ms`. A current-rev composite size `3` run reduced hosted requests
to `4` and preserved precision, but the composite request tail regressed force
refresh to `10797.20775 ms`. Single-region render chunks are useful only as a
diagnostic: they moved first hosted region readiness from about `1.9s` to about
`0.70-0.72s` across two current-rev runs, but force refresh measured
`8202.969708 ms` and `8927.807167 ms`, so the default remains page-grouped
region chunks. The next opt-in chunk-shape canary is `region-seed-page`: it
renders the smallest recovery region first, then keeps the remaining regions
page-grouped. That mode is designed to keep the early-dispatch benefit without
paying the full single-region tail cost, and it still relies on the unchanged
row/order and precision gates before any promotion. Two explicit PDFium runs
preserved zero error rows, stable `27` rows, `21/6` page/region OCR blocks, and
the frozen character floor: `8250.492790999999 ms` with
`metricsResultChars=116286`, then `8445.105417 ms` with
`metricsResultChars=116270`. That beats the locked `12856.546292 ms` baseline
but does not beat the active `7338.796584/8322.027792 ms` envelope, so it
remains a canary.
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
hedge to `4s` produced the previous `8201.568417 ms` promoted evidence. The
current-rev `2s` hedge repeats now supersede it as the active OpenRouter
region-recovery envelope. The r71
endpoint `0-3` prewarm diagnostic reduced the page `11-13` fast-text chunk to
`5972.05625 ms` but regressed force refresh to `10336.721667 ms` because the
hosted region tail dominated.
The scheduler now applies `single-page-first` to source-PDF fast-text chunks
directly. r79b verified that deterministic route under the real OpenRouter gate:
precision stayed intact, force refresh was `12067.125959 ms`, hosted p95 was
`6341.191 ms`, and the source tail moved back to page `5` at
`8030.604042 ms`. This remains below the locked baseline but is not promoted
over the current `7338.796584/8322.027792 ms` OpenRouter envelope.
Hedge `2s` is promoted only in the current render-dispatch, render-ahead `3`,
region-trim, Ministral OpenRouter canary.
Direct-crop rendering, scaffold/composite, local fast-text replacement,
fast-text single-page source-range splitting, and hosted VLM full-page top-up
replacement remain rejected canaries for this fixture.
The later Ministral same-page region composite size `3` diagnostics also stay
rejected. The older run preserved the precision envelope, but the page `12`
three-region composite request tailed at `14430.981 ms` and force refresh
regressed to `17806.492208 ms`. The current-rev repeat reduced request count
from `6` to `4`, but hosted p95 still reached `8528.296 ms` and force refresh
measured `10797.20775 ms`. The r84 fallback-guard rerun completed with valid
OCR metrics and passed the locked baseline at `12658.151 ms`, but it is still
diagnostic-only because it did not beat the current
`7338.796584/8322.027792 ms` OpenRouter envelope.
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
current `7338.796584/8322.027792 ms` OpenRouter envelope and is not promoted.
The current-rev scheduler fairness canary adds
`WENDAO_DOCUMENT_EXTRACT_PDF_OCR_SCHEDULER_LANE_FAIRNESS=source-first`, also
forwarded by the analyzer benchmark as
`--rust-pdf-ocr-scheduler-lane-fairness source-first`. It lets source-PDF
page-range OCR groups enter the Python Flight endpoints before rendered-region
fanout when both lanes coexist. The valid r32d run preserved precision with
zero errors, stable `28` rows, `21/7` page/region OCR blocks,
`metricsResultChars=115704`, and `forceRefreshMs=10874.933208`, but it is not
promoted over the current direct-crop r26 candidate at `8463.964667 ms` because
the source-range fast-text chunks became the critical path.
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
`WENDAO_DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_EMPTY_PAGE=verified-empty`, or pass
`--pdf-ocr-backend-text-empty-page verified-empty`, only when the matching
Python worker canary is enabled. This lets the Rust precision gate accept an
empty successful OCR result for a `docling-backend-text-ocr-v1` page shard only
when the input row is a source-page-range placeholder. It does not accept empty
hosted VLM, raster, region, or compatible-page OCR output, and it does not
change the Arrow OCR shard input/result schemas. The default remains
`disabled`. The 2026-05-08 r108c real Docling canary is rejected as a
promotion candidate because it improved force latency while failing structure
parity on all three skipped-page/redp fixtures; keep full fallback active until
a narrower merge path preserves baseline coverage. Set
`WENDAO_DOCUMENT_EXTRACT_PDF_FAST_TEXT_SOURCE_RANGE_SPLIT=single-page`, or pass
`--rust-pdf-fast-text-source-range-split single-page`, only as a source-range
chunk-shape diagnostic. It keeps Rust scheduler permits as the final admission
owner and preserves precision on the milestone fixture, but it is rejected
because page `5` fast-text conversion alone regressed force refresh to
`23629.474667 ms`.
The current safe source-range tail lever is analyzer-side prewarm cache reuse,
not local fast-text replacement. When benchmark workers receive
`--pdf-ocr-prewarm-profile docling-fast-text-ocr`,
`--pdf-ocr-prewarm-source-path`, and matching
`--pdf-ocr-prewarm-page-indices`, the Python worker stores fingerprinted
Docling Markdown for those exact source pages and the Rust scheduler can route
single-page fast-text chunks back to the prewarmed endpoint through
`--rust-pdf-fast-text-endpoint-affinity single-page-first`. The May 21, 2026
r40 canary preserved zero errors, stable `28` rows, `21/7` page/region OCR
blocks, and `metricsResultChars=108850`; source-range max chunk latency fell
to `280.219583 ms` and force refresh measured `9037.577666 ms`. r26 remains
the fastest global baseline at `8463.964667 ms`, so this remains an explicit
readiness lever.
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
Advanced benchmark runs can tune the adaptive slice planner with
`WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_TARGET_PIXELS` and
`WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_MAX_SLICES`. These controls are
opt-in, preserve the default planner when unset, and enter the artifact cache
signature so region image reuse cannot cross incompatible patch-sizing
settings. Smaller patches remain diagnostic unless the benchmark also proves
lower hosted wall span under the existing precision and row-order gates.
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
A May 20, 2026 private LTC image-route probe found the historical free
OpenRouter aliases for Qianfan OCR returned no live endpoints, while
`baidu/qianfan-ocr-fast` produced one queue-keyed OCR JSONL row that the Rust
source-contract cache bridge consumed with attempted 1, succeeded 1, and
failed 0.
The live Gateway route now uses the same hosted VLM configuration family through
`hosted-vlm-image-extract-v1` on `/analysis/document-extract`, so chat
attachments are converted into resource evidence before a non-vision chat model
sees the conversation context.
Within that opt-in pipeline, the benchmark can set
`WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_AHEAD` above `1`, or pass
`--rust-pdf-hosted-vlm-region-render-ahead`, to pre-render multiple page-region
chunks while hosted requests are in flight. Final OCR inputs are normalized
back to deterministic reading order before the Rust row/order gate runs.
When the override is omitted, Studio now treats the render-ahead window as a
Rust scheduling decision: it reserves one live endpoint for the base OCR batch,
uses the remaining endpoint capacity for region render-ahead, and clamps the
window by the planned render chunk count. Timing reports expose the planned
chunk count, endpoint count, effective render-ahead limit, and render spawn
count so benchmark evidence can distinguish default scheduler behavior from an
explicit operator override.
The analyzer can also opt into OpenRouter provider routing for hosted-region
tail studies by setting `WENDAO_HOSTED_VLM_OCR_OPENROUTER_PROVIDER_JSON`, or by
passing `--hosted-vlm-ocr-openrouter-provider-json` in the benchmark harness.
The JSON object is sent as OpenRouter's request-body `provider` field and is
reported back in benchmark JSON and Markdown. This route-control knob is
diagnostic by default; it does not change Studio's OCR shard schemas, rendered
DPI, merge gates, or Docling structure authority.
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
but it still missed the current `7338.796584/8322.027792 ms` OpenRouter
envelope, so composite remains a benchmark-only canary.
Hosted VLM/OCR modes stay opt-in and promote per profile only when the real
benchmark gate proves the current precision envelope and beats the 12,856.546
ms `fast-risk-window` force-refresh evidence; promoted replacements should
also beat the current `7338.796584/8322.027792 ms` OpenRouter region-recovery
envelope.
Benchmark reports expose that decision through `hostedVlmPromotionGate`, which
keeps hosted profile promotion tied to the frozen precision, row/order,
character-floor, hosted-request, force-refresh, shard-cache reuse, and zero
scaffold-validation-failure gates.
Hybrid OCR timing reports keep route-local region render cache counters
separate from shared artifact-substrate counters. `ocr2RegionRenderCache*`
describes reuse of an already materialized region render directory, while
`ocr2RegionRenderArtifactCacheHitCount`,
`ocr2RegionRenderArtifactCacheMissCount`,
`ocr2RegionRenderArtifactCacheThrottledCount`, and
`ocr2RegionRenderArtifactCacheByteCount` describe the underlying
`ArtifactBlobCache` read-through work performed by the attachment-owned raster
and crop renderer.

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
