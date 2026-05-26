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
unique common and extension `episteme://` domain ids,
and checks declared RDF, SQL rule, policy, dataset mapping, extension, review
ledger, and API-surface artifacts without mutating them.
The source ontology remains repository-owned, while Rust owns deterministic
admission before downstream read-model and quality-gate slices consume the
ontology contract.

Private Episteme support is modeled as distributed extension packs, not as
hardcoded providers. The common `wendao-episteme` repository owns public common
domains such as Healthcare. A customer, tenant, or vertical deployment may keep
one or many extension repositories that extend those common domains through
semantic `episteme://<authority>/<domain>` ids. Those repositories contain configuration,
RDF, object-model contracts, Org review ledgers, and source manifests; they do
not need executable validators or package-local tools. This crate validates the
extension pack generically through `validate_episteme_extension_contract`.
`episteme.toml` may point at an external private corpus root, while source
manifest paths and source file rows remain safe relative paths inside their
declared contract surfaces. Access, publication, and tenant distribution policy
belongs to the Episteme registry or package metadata, not to the ontology domain
id itself. Object-model `visibility` remains an API exposure field for generated
object surfaces; it is not the authority for repository publication or tenant
distribution.

The source ontology compiler may emit `ontology/registry.json`, but that
snapshot is not treated as a Python-only artifact. This crate also admits the
registry snapshot as a Rust read-model input: it validates source-contract
flags, unique domain ids, artifact paths, RDF term references, dataset mapping
paths, and API object/link/action/query references, then returns deterministic
counts for downstream `xiuxian-wendao` materialization. Python remains the
compiler boundary; Rust owns admission and read-model readiness.

The crate does not invoke Python helpers, write ontology sources, promote RDF
truth, or change Flight/OpenAPI/Arrow contracts.

Large generated ontology payloads should use the shared db-store artifact
substrate when they need restart-reusable byte storage. Registry snapshots,
candidate review packets, candidate read models, RDF drafts, promotion review
packets, structural reasoning projections, and schedule-plan payloads can use
the optional `foyer-artifact-cache` feature, which exposes Episteme-owned
bundle helpers over the db-store `ontology_artifact_key` namespace and
`ArtifactBlobCache`. Bundle writers delegate to db-store fetch-through so
Foyer backends can coalesce same-key misses while Episteme keeps ownership of
ontology identity and directory packing. The artifact cache stores derived
bytes only; ontology truth, promotion status, manifest admission, and
read-model validation remain owned by Episteme and its DuckDB/Arrow consumers.

The private ontology candidate materializer is also Rust-owned in this crate.
It selects the active source contract, reads source rows, mapping-ledger terms,
and cache-local extraction outputs, then writes review-required
`candidate_objects.tsv`, `candidate_relations.tsv`, `candidate_evidence.tsv`,
`review_ledger.org`, and `receipt.json` artifacts under the configured
ontology-generation run root. These rows are candidate evidence only:
`raw_to_rdf_promotion_allowed=false`, `ontology_truth=false`, and raw or
extracted source text is not persisted into the candidate TSV files. These TSV
files are compatibility projections only. Org remains the reviewable reasoning
ledger, and the typed runtime/search surface should be compiled as an
Arrow/Parquet read model rather than extending TSV as a semantic contract.
This crate now writes that first read model as
`ontology_candidate_objects.parquet`,
`ontology_candidate_relations.parquet`, and
`ontology_candidate_evidence.parquet` beside the candidate run. Studio may
orchestrate the command, but promotion into RDF or SQL remains a later
review-gated slice.

The candidate read-model query gate reads those Parquet files back through the
Rust Arrow/Parquet boundary and reports row counts, review-status violations,
promotion-status violations, ontology-truth violations, raw-to-RDF promotion
violations, and missing relation endpoints. This gives downstream
DuckDB/DataFusion, WendaoGraph, and Flight slices a typed readiness check before
they consume candidate facts. The gate does not parse candidate TSV files and
does not make TSV a semantic contract.

The SearchStrategyFlow oracle projection follows the same boundary. It reads
manifest-declared Org review ledgers, derives expected selected and rejected
candidate ids from review and promotion decisions, and writes
`search_strategy_oracle_cases.json`,
`search_strategy_oracle_candidates.json`, and
`search_strategy_oracle_report.json` as read-model evidence for downstream
WendaoGraph and ScienceResearch benchmarks. The projection is derived from
ontology/Org authority, never from hand-maintained benchmark TSV truth, and it
does not parse private source text, mutate source files, or mark any row as
ontology truth. Candidate rows also carry SearchStrategyFlow support fields
such as `revisionId`, `routeRole`, `requiredEvidence`, `finalScore`,
`contextCost`, `action`, and `blocked`, so downstream frontier tests can use
the compiled Episteme projection directly instead of inventing route fixtures.

The structural facts seed compiler is the first deterministic structure pass for
private Episteme repositories. It reads declared source manifests and
`files.tsv` rows, verifies source paths and file metadata against the configured
corpus root, and writes `structural_facts.json`, `structural_facts.org`,
`structural_facts_documents.tsv`, `structural_facts_anchors.tsv`, and
`structural_facts_relations.tsv` under the structure run root. The emitted rows
represent source-contract documents, path anchors, and deterministic containment
relations only. They are not promoted RDF, not raw extracted text, and not
ontology truth. `metadata-only` validation keeps the first pass fast by checking
presence and size; `full-hash` additionally verifies the source SHA-256 values
recorded in `files.tsv`.
The crate owns `episteme.toml` runtime defaults for this pass. Callers may use
the config-driven structural facts request to resolve `runtime.corpus_root` and
`runtime.structure_run_root` inside the Episteme boundary; Studio and Gateway
surfaces should remain thin operators over that crate-owned request.
The same run now also emits `structural_facts_rdf_seed.ttl`,
`structural_facts_read_model_objects.{tsv,json,parquet}`,
`structural_facts_read_model_relations.{tsv,json,parquet}`, and
`structural_facts_read_model_projection_state.json`. These are deterministic
structure facts for downstream graph/search readiness, not model-inferred
ontology truth. The read-model gate rejects duplicate structural ids, missing
relation endpoints, empty projection state, and any attempt to mark structural
rows as ontology truth before later Org/RDF promotion slices.

The structural facts reasoning-packet compiler is the next deterministic
proposal-input pass. It reads a generated `structural_facts.json`, validates that
every document row has a matching document-root anchor, and writes
`reasoning_packet.org`, `reasoning_packet.tsv`, `reasoning_packet.json`, and
`reasoning_packet_report.json` under the ontology-generation run root. The
packet groups bounded source rows by category and extraction route so a later
human, Qianji workflow, or LLM can request targeted evidence before filling Org
review ledgers. It also emits deterministic structure-targeting fields:
`evidenceTargetIntent`, `evidenceAnchorKind`, and `evidenceStructureHint`.
These fields let Docling, parser, table, section, and source-route signals
select a reasoning slot before model execution. The packet compiler does not
read private source text, call an LLM, write RDF, or mark any row as ontology
truth.

The reasoning ledger seed compiler turns `reasoning_packet.json` into generated
Org proposal slots. It writes `reasoning_ledger_seed.org`,
`reasoning_ledger_seed.tsv`, `reasoning_ledger_seed.json`, and
`reasoning_ledger_seed_report.json` under the ontology-generation run root.
Object and relation proposal fields are blank by default; the seed preserves
packet ids, document ids, anchors, hashes, categories, and routes so a human or
LLM can fill only the rows it has inspected. Structure-targeted packet rows may
emit narrower seed kinds such as `service_catalog_review_slot` or
`object_instance_review_slot` instead of generic object-model proposal slots.
This prevents service catalogs, row-like tables, and instance evidence from
being routed as `ObjectType` definitions. The seed still does not read private
source text, call an LLM, write RDF, or mark any row as ontology truth.

The reasoning fill-plan compiler is the deterministic workflow handoff after a
ledger seed exists. It reads `reasoning_ledger_seed.json` and writes
`reasoning_fill_plan.org`, `reasoning_fill_plan.tsv`,
`reasoning_fill_plan.json`, and `reasoning_fill_plan_report.json` under the
ontology-generation run root. Fill-plan rows carry Qianji/BPMN workflow keys,
activity kinds, seed ids, evidence anchors, and safety flags as data only. The
compiler preserves the same structure-targeting fields and maps seed kinds to
typed target ledger groups, including `service_catalog_review` and
`object_instance_review` for non-object-model evidence. It does not execute
Qianji, read source text, call an LLM, mutate source files, write RDF, or mark
any row as ontology truth.

The ontology bootstrap pipeline is the crate-owned convenience boundary for the
same deterministic sequence. It resolves `episteme.toml`, compiles structural
facts, writes the reasoning packet, seeds fillable Org ledger rows, and emits
the reasoning fill plan without reading private source text, calling an LLM,
executing Qianji, mutating RDF, or declaring ontology truth. Studio, Gateway,
and later Flight surfaces should call this Episteme API instead of rebuilding
the sequence in their own command handlers.
When the optional `foyer-artifact-cache` feature is enabled, callers can run
the bootstrap pipeline through
`run_episteme_ontology_bootstrap_pipeline_with_artifact_cache` to write the
four generated stage run directories into a runtime-supplied
`ArtifactBlobCache`. The wrapper does not construct a cache backend and does
not change the default bootstrap command behavior. The companion
`read_through_episteme_ontology_bootstrap_artifacts` API first restores the
four deterministic stage directories from the same substrate and only
regenerates when one or more stage bundles are missing. Generated bundle
storage uses db-store fetch-through, so repeated same-key agent/review flows
can reuse existing bundle bytes without repacking the run directory.
Callers should build cache options through
`admit_episteme_ontology_bootstrap_artifact_cache_options`, which validates
source/profile digest components with the same ontology artifact-key rules used
by the bundle writer. The artifact-cache wrappers also validate supplied
options before running the deterministic pipeline, so unsafe cache identities
cannot trigger unnecessary generation work.
The crate also exposes a thin `wendao-episteme` operator binary over the same
API. The first command is
`wendao-episteme ontology bootstrap-pipeline`, which accepts an Episteme root
and run id, resolves runtime defaults from `episteme.toml`, and prints the
existing bootstrap report as JSON. This keeps local/private Episteme operation
inside the Episteme package while still allowing Studio or Gateway to call the
same Rust API when they orchestrate a larger workflow.
With the `foyer-artifact-cache` feature enabled, the same command accepts explicit
artifact-cache modes: `write-through`, `read-through`, and `restore-only`.
These modes require caller-provided source/profile digest components and use
the shared db-store Foyer artifact backend resolver. The operator does not
derive cache identity from paths, validates digest components before resolving
the backend, and does not construct a route-local backend.

The Qianji reasoning schedule-plan compiler is the next non-mutating handoff.
It reads `reasoning_fill_plan.json` and writes `qianji_schedule_plan.org`,
`qianji_schedule_plan.tsv`, `qianji_schedule_plan.json`, and
`qianji_schedule_plan_report.json` under the ontology-generation run root. Each
row carries a Qianji-shaped activity task payload with stable activity ids,
task queue, input claim-check reference, and idempotency key. The compiler does
not append Qianji control ledger events, enqueue hot-state work, execute
workers, call an LLM, read source text, mutate source files, write RDF, or mark
any row as ontology truth.
The same compiler is now reachable from the crate-owned operator binary through
`wendao-episteme ontology qianji-schedule-plan`. The command consumes the
reasoning fill-plan JSON, infers the ontology-generation run root from that path
when `--run-root` is not supplied, and prints the existing schedule-plan report
as JSON. The command remains an admission-artifact compiler only: it does not
run Qianji or a provider. OpenAI-compatible prompt-audit metadata is emitted
only when explicitly requested.
Callers may restrict schedule generation by target ledger field group or
evidence target intent before the limit is applied. This keeps live proofs and
operator batches aligned with the deterministic structure target instead of
depending on prompt wording or fill-plan row order.
Callers may also enable deterministic reasoning context sharding for prompt-audit
tasks. The first reusable shard planner supports `service-catalog-table-rows`,
which preserves the Markdown table header and emits bounded row-window context
artifacts for `service_catalog_review` rows. This prevents a long structured
table from becoming one oversized model request while keeping the original
fill-plan and evidence contracts intact. Sharding is disabled by default, and
the default enabled row window is two table data rows.

When the caller enables OpenAI-compatible prompt-audit emission, the same
compiler also writes per-task local prompt and context artifacts under the
schedule-plan run directory. Studio defaults this route to
`deepseek/deepseek-v4-pro`; alternate model ids are comparator runs, not the
canonical ontology-reasoning default. The generated Qianji task `input_ref`
points to the prompt artifact, while the original reasoning fill item remains
recorded as source context in task metadata. Prompt-audit emission must also
name at least one extraction run id. The compiler reads succeeded cache
outputs, validates the source hash and relative path against the fill item, and
embeds non-empty `contextEvidence` rows in the Qianji context artifact. The
context also carries an object-model `targetContract` for the fill item's target
ledger field group.
When a reasoning context shard is present, the context and task metadata carry the
shard id, row window, and carry-forward metadata so the worker reviews only the
bounded reasoning-context shard.
Object proposals target review-only `ObjectType` candidates and relation
proposals target review-only `LinkType` candidates. RDF remains the semantic
source authority, runtime mutation stays disabled, and provider output is not
ontology truth. This only prepares admitted worker input; the Episteme crate
still does not call a provider or promote RDF.
When the target ledger group is `service_catalog_review` or
`object_instance_review`, the generated contract uses concrete
`object_candidate` review patches instead of object-model `ObjectType` patches.
The model may still return blockers when evidence is insufficient; correctness
and reviewability take priority over candidate volume.

Qianji review artifacts can be imported back into the same candidate-review
surface after a worker writes a canonical `episteme_review` object. The importer
accepts the Qianji OpenAI-compatible response envelope only when the review is
schema-valid, `review_only`, bound to the expected target field group, and
declares `rdfMutation=false`. It supports review-only object-model `ObjectType`
and `LinkType` candidates. Link candidates are expanded into review-required
endpoint object candidates plus a relation row so the existing review gate can
validate relation endpoints before any promotion slice. The importer writes
`candidate_objects.tsv`, `candidate_relations.tsv`, `candidate_evidence.tsv`,
`qianji_review_candidate_import_report.json`, and the normal candidate review
gate artifacts. It records source ids, paths, evidence hashes, and text length,
but it does not persist raw private quotes in the candidate TSVs, mutate source
RDF, or mark any row as ontology truth.
If a canonical review returns zero candidate patches, the importer accepts it
only when the model supplied blockers explaining why evidence was insufficient.
That path writes header-only candidate TSVs, records
`zeroCandidateReviewCount` and `reviewBlockerCount` in the import report, and
keeps the candidate review gate passing with zero promotion rows.

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
inspectable as RDF-shaped data. Relation candidates also expose source and
target candidate resource references, so graph consumers can traverse proposal
topology without parsing string ids. The exporter rejects relation rows whose
source or target candidate ids are absent from the reviewed object candidate
set. Draft artifacts keep `rawToRdfPromotionAllowed=false` and
`ontologyTruth=false`. Source ontology RDF is not changed by this crate;
promotion into a private ontology source tree is a separate review slice. Stale
or corrupted candidate review TSV projections cannot change RDF draft review
state.

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
