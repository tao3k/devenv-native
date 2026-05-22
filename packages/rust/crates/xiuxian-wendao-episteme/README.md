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

The existing `xiuxian-wendao::episteme::source_contract` facade now delegates
its manifest/path admission entry points to this crate while keeping its richer
legacy error type for current Studio and Gateway consumers.
