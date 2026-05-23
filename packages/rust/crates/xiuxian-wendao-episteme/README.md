# xiuxian-wendao-episteme

Rust-owned Episteme source-contract runtime boundary for Wendao.

This crate is the implementation home for deterministic Episteme admission,
validation, scheduling, cache identity, materialization, and promotion gates.
Parser syntax remains owned by
[`xiuxian-wendao-parsers`](../xiuxian-wendao-parsers/README.md). Higher-level
Wendao crates consume this crate rather than growing Episteme implementation
inside the search engine crate.

The first landed surface selects the active source contract from
`ontology/manifest.toml`, reads the parser-owned source manifest, and rejects
unsafe manifest paths. The cache materialization surface also consumes
queue-keyed analyzer JSONL for image OCR and Docling document extraction, then
writes review-required, promotion-blocked cache rows with source/path
validation.
The Docling document cache route is intentionally modern-format only:
`document_text_evidence` accepts `pdf`, `docx`, `pptx`, and `xlsx` rows. Legacy
Office binaries such as `doc`, `ppt`, and `xls` must use a separate route, for
example `legacy_office_document_evidence`, until a conversion contract produces
validated modern documents or text evidence. This crate now exposes the first
conversion-admission surface for that route: it reads stable `tasks.tsv`
artifacts and validates that every conversion candidate is a `doc`, `ppt`, or
`xls` task routed through `legacy_office_document_evidence`. It also exposes a
bounded converter runner that invokes an explicit binary as
`converter <source-path> <target-path>`, writes converted artifacts below the
run outputs directory, and records promotion-blocked conversion receipts. The
runner does not make converted artifacts ontology truth. The Studio CLI can
drive this runner through the Episteme source-contract command surface; the
converter path is supplied explicitly by the operator or by the Episteme
runtime config, not by executable tools stored in the private Episteme
repository.

Ontology is the core Episteme source contract. This crate now exposes a
conservative ontology admission surface that reads `ontology/manifest.toml`,
validates ownership boundaries, rejects mutable source contracts, verifies
unique common `episteme://` domain ids or private `episteme://private/`
extension ids,
and checks declared RDF, SQL rule, policy, dataset mapping, extension, review
ledger, and API-surface artifacts without mutating them.
The source ontology remains repository-owned, while Rust owns deterministic
admission before downstream read-model and quality-gate slices consume the
ontology contract.

The source ontology compiler may emit `ontology/registry.json`, but that
snapshot is not treated as a Python-only artifact. This crate also admits the
registry snapshot as a Rust read-model input: it validates source-contract
flags, unique domain ids, artifact paths, RDF term references, dataset mapping
paths, and API object/link/action/query references, then returns deterministic
counts for downstream `xiuxian-wendao` materialization. Python remains the
compiler boundary; Rust owns admission and read-model readiness.

The crate does not invoke Python helpers, write ontology sources, promote RDF
truth, or change Flight/OpenAPI/Arrow contracts.

The private ontology candidate materializer is also Rust-owned in this crate.
It selects the active source contract, reads source rows, mapping-ledger terms,
and cache-local extraction outputs, then writes review-required
`candidate_objects.tsv`, `candidate_relations.tsv`, `candidate_evidence.tsv`,
`review_ledger.org`, and `receipt.json` artifacts under the configured
ontology-generation run root. These rows are candidate evidence only:
`raw_to_rdf_promotion_allowed=false`, `ontology_truth=false`, and raw or
extracted source text is not persisted into the candidate TSV files. Studio may
orchestrate the command, but promotion into RDF or SQL remains a later
review-gated slice.

The structural IDF seed compiler is the first deterministic structure pass for
private Episteme repositories. It reads declared source manifests and
`files.tsv` rows, verifies source paths and file metadata against the configured
corpus root, and writes `structural_idf.json`, `structural_idf.org`,
`structural_idf_documents.tsv`, `structural_idf_anchors.tsv`, and
`structural_idf_relations.tsv` under the structure run root. The emitted rows
represent source-contract documents, path anchors, and deterministic containment
relations only. They are not promoted RDF, not raw extracted text, and not
ontology truth. `metadata-only` validation keeps the first pass fast by checking
presence and size; `full-hash` additionally verifies the source SHA-256 values
recorded in `files.tsv`.

The structural IDF reasoning-packet compiler is the next deterministic
proposal-input pass. It reads a generated `structural_idf.json`, validates that
every document row has a matching document-root anchor, and writes
`reasoning_packet.org`, `reasoning_packet.tsv`, `reasoning_packet.json`, and
`reasoning_packet_report.json` under the ontology-generation run root. The
packet groups bounded source rows by category and extraction route so a later
human, Qianji workflow, or LLM can request targeted evidence before filling Org
review ledgers. It does not read private source text, call an LLM, write RDF,
or mark any row as ontology truth.

The reasoning ledger seed compiler turns `reasoning_packet.json` into generated
Org proposal slots. It writes `reasoning_ledger_seed.org`,
`reasoning_ledger_seed.tsv`, `reasoning_ledger_seed.json`, and
`reasoning_ledger_seed_report.json` under the ontology-generation run root.
Object and relation proposal fields are blank by default; the seed preserves
packet ids, document ids, anchors, hashes, categories, and routes so a human or
LLM can fill only the rows it has inspected. The seed still does not read
private source text, call an LLM, write RDF, or mark any row as ontology truth.

The reasoning fill-plan compiler is the deterministic workflow handoff after a
ledger seed exists. It reads `reasoning_ledger_seed.json` and writes
`reasoning_fill_plan.org`, `reasoning_fill_plan.tsv`,
`reasoning_fill_plan.json`, and `reasoning_fill_plan_report.json` under the
ontology-generation run root. Fill-plan rows carry Qianji/BPMN workflow keys,
activity kinds, seed ids, evidence anchors, and safety flags as data only. The
compiler does not execute Qianji, read source text, call an LLM, mutate source
files, write RDF, or mark any row as ontology truth.

The Qianji reasoning schedule-plan compiler is the next non-mutating handoff.
It reads `reasoning_fill_plan.json` and writes `qianji_schedule_plan.org`,
`qianji_schedule_plan.tsv`, `qianji_schedule_plan.json`, and
`qianji_schedule_plan_report.json` under the ontology-generation run root. Each
row carries a Qianji-shaped activity task payload with stable activity ids,
task queue, input claim-check reference, and idempotency key. The compiler does
not append Qianji control ledger events, enqueue hot-state work, execute
workers, call an LLM, read source text, mutate source files, write RDF, or mark
any row as ontology truth.

When the caller explicitly enables OpenAI-compatible prompt-audit emission, the
same compiler also writes per-task local prompt and context artifacts under the
schedule-plan run directory. The generated Qianji task `input_ref` points to
the prompt artifact, while the original reasoning fill item remains recorded as
source context in task metadata. This only prepares admitted worker input; the
Episteme crate still does not call a provider or promote RDF.

The candidate review gate is the next deterministic step. It reads generated
candidate TSV artifacts and writes `candidate_review.org`,
`candidate_review.tsv`, and `quality_report.json` under the same
ontology-generation run directory. The Org ledger is the authoritative review
table; the TSV file is a generated projection. The review gate reports
duplicate candidate ids, missing relation references, unsafe promotion flags,
ontology-truth flags, evidence strength, quality scores, and precondition
status for later human or LLM-assisted review. It still does not write RDF,
mutate ontology sources, or persist raw extracted text.

The RDF draft export is review-gated and still non-mutating. It requires a
passing `quality_report.json` and consumes review decisions from
`candidate_review.org`, then writes `rdf_draft.ttl`, `promotion_proposal.org`,
and `promotion_proposal.json` beside the reviewed candidate run. These artifacts
make candidate ids, labels, provenance, review decisions, and proposal status
inspectable as RDF-shaped data, but they keep `rawToRdfPromotionAllowed=false`
and `ontologyTruth=false`. Source ontology RDF is not changed by this crate;
promotion into a private ontology source tree is a separate review slice.
Stale or corrupted candidate review TSV projections cannot change RDF draft
review state.

The promotion review packet is the next explicit gate. It requires the clean
draft proposal and reads candidate review rows from `candidate_review.org`, then
writes `promotion_review.tsv`, `promotion_review.org`, and
`promotion_review.json` beside the run. Every generated row starts as
`pending_review` with `sourceMutationAllowed=false` and `ontologyTruth=false`.
The Org packet contains the authoritative review table for human or
LLM-assisted review; the TSV file is a machine projection for reporting and
read-model consumers. The packet gives a reviewer stable candidate ids, quality
scores, evidence strength, and relation endpoint context without approving
promotion or editing ontology source RDF. Stale or corrupted candidate review
TSV projections cannot change promotion review packet rows.

The promotion apply-plan writer consumes only explicit decisions from
`promotion_review.org`. Pending-only reviews produce an empty
`promotion_apply_plan.tsv` plus Org/JSON receipts, while approved rows require
reviewer provenance and satisfied preconditions before they can appear as
`propose_source_patch` plan rows. The apply plan is still non-mutating:
`sourceMutationAllowed=false`, `ontologyTruth=false`, and source ontology RDF is
not changed by this crate. Stale or corrupted TSV projections cannot authorize
ontology promotion.

The source-patch preflight writer is the first gate for approved object and
relation review ledgers. It reads the Episteme manifest, compiles declared
review-ledger Org files, verifies the same source-contract safety rules as
ontology admission, and emits `source_patch_preflight.tsv`, Org, and JSON
receipts. Pending ledgers produce an empty preflight. Approved relation rows
are accepted only when both endpoints are also approved object-instance review
rows. Approved rows must also resolve to a single declared RDF source target
for their domain, so later patch planning cannot silently pick the wrong
ontology file. The preflight still does not generate source patches, mutate
RDF, or mark rows as ontology truth; it only proves that a later source patch
has a consistent review surface.

The source-patch draft exporter consumes that preflight receipt and writes
`source_patch_draft.ttl`, `source_patch_draft.org`, and
`source_patch_draft.json` in the same run directory. It validates the
preflight JSON/TSV row count, rejects mutation or ontology-truth flags again,
and renders approved object-instance and relation rows, including their
domain id and target RDF file, as RDF proposal resources only. It is
intentionally not an apply step: private ontology source files remain unchanged
until a later explicitly approved patch application slice.

The source-patch apply-plan writer is the final non-mutating gate before any
future patch application. It consumes `source_patch_preflight.tsv`,
`source_patch_preflight.json`, and `source_patch_draft.json`, verifies their
row and resource counts agree, and writes `source_patch_apply_plan.tsv`, Org,
and JSON receipts. Apply-plan rows preserve the domain id and target RDF file
and use `propose_targeted_source_patch`; they still keep
`sourceMutationAllowed=false` and `ontologyTruth=false`, so the plan is a
review surface rather than an RDF writer.

The source-patch review packet writer binds that apply plan to the current
private ontology source state. It consumes `source_patch_apply_plan.tsv` and
`source_patch_apply_plan.json`, hashes the apply-plan TSV, hashes every
referenced target RDF source file under the Episteme `ontology/` directory, and
writes `source_patch_review_packet.org` plus JSON. Missing, unsafe, or
out-of-root target RDF paths fail before packet artifacts are written. The
packet is still non-mutating and exists to make the later explicit source
mutation gate hash-checked and reviewable.

The source-patch apply preview is the required inspection step before opening
that mutation gate. It consumes the same review packet and apply-plan
artifacts, requires the operator-provided expected apply-plan TSV hash, and
writes `source_patch_apply_preview.org`, JSON, and per-target bounded RDF/XML
proposal blocks plus complete proposed RDF/XML target files under the run
directory. It recomputes current target hashes and proposed after-write hashes
without editing source ontology files. It also applies a conservative
preview-admission guard to the complete proposed RDF/XML: the file must retain
an RDF root, contain exactly one bounded source-patch block, include the
source-patch namespace, and avoid mutation or ontology-truth escalation. The
report continues to expose `sourceMutationAllowed=false` and
`ontologyTruth=false`.

The source-patch semantic preview is the graph handoff before Julia or
SearchStrategyFlow reasoning. It consumes `source_patch_apply_plan.tsv` and
the admitted `source_patch_apply_preview.json` from the same run directory,
verifies that the preview still matches the current apply-plan TSV hash, and
writes `semantic_objects.tsv`, `semantic_relations.tsv`,
`semantic_evidence.tsv`, JSON projections, `semantic_projection_state.json`,
and a receipt. The rows use the downstream read-model vocabulary (`id`,
`kind`, `source`, `target`, `status`, and projection freshness fields) while
preserving review provenance. This compiler remains non-mutating:
`sourceMutationAllowed=false`, `ontologyTruth=false`, and private ontology
source files remain unchanged.

The source-patch apply gate is the only writer that can mutate ontology source
files in this sequence. It consumes the review packet and apply-plan artifacts,
requires an operator-provided expected apply-plan TSV hash, recomputes target
RDF hashes before writing, and rejects the request unless source mutation is
explicitly enabled. When the gate is opened, it appends a bounded RDF/XML
source-patch proposal block before the closing `rdf:RDF` element and writes an
apply receipt with before/after target hashes. The proposal block is still not
raw extracted text and does not treat the rows themselves as direct ontology
truth.

The existing `xiuxian-wendao::episteme::source_contract` facade now delegates
its manifest/path admission entry points to this crate while keeping its richer
legacy error type for current Studio and Gateway consumers.
