# Ontology JSON Schema Contracts

`xiuxian-wendao-parsers` owns the draft JSON Schema contracts for compiled
Org ontology DTOs. These schemas make parser output deterministic before it is
used by source-contract preview generation, semantic candidate validation, or
future runtime import.

## Boundary

Org-mode remains the authoring grammar for ontology proposals and reasoning
traces. JSON Schema validates the DTOs compiled from native Org structures; it
does not replace Org authoring syntax and does not own ontology truth.

Accepted ontology source contracts remain under
[`wendao-episteme/ontology/`](../../../../../wendao-episteme/ontology/).
The parser package can validate and preview compiled DTOs, but semantic truth
still requires source-contract acceptance and repo-native semantic governance.

## Draft Schemas

The first schema files are internal draft contracts:

- [`org_ontology_authoring_contract.schema.json`](../schemas/ontology/org_ontology_authoring_contract.schema.json)
  validates Org ontology authoring DTOs compiled from headings, property
  drawers, TODO state, tags, tables, source blocks, and source spans.
- [`org_trace_projection_contract.schema.json`](../schemas/ontology/org_trace_projection_contract.schema.json)
  validates Org reasoning trace DTOs compiled from agent or sub-agent evidence
  documents.
- [`ontology_candidate_contract.schema.json`](../schemas/ontology/ontology_candidate_contract.schema.json)
  validates ontology candidate DTOs derived from validated authoring or trace
  rows.
- [`org_reasoning_property_contract.schema.json`](../schemas/ontology/org_reasoning_property_contract.schema.json)
  validates schema-governed Org property drawer records used for Wendao
  reasoning metadata.

The schema id suffix `v0.draft` is intentional. These files are not public
runtime wire schemas yet.

## Validation Shape

The contracts enforce structural facts:

1. source identity and source hash are present
2. source spans are reopenable
3. authoring and candidate kinds are from explicit vocabularies
4. lifecycle states are explicit
5. evidence rows are present before candidates can be promoted
6. table shapes are typed before downstream compilation
7. dataset mapping tables and SQL artifacts are explicit before structured
   data can be projected into ontology observations
8. Org property drawers that opt into `WENDAO_KIND` are limited to explicit
   keys and values before downstream ontology or graph consumers read them

Semantic checks such as relation endpoint authority, lifecycle transitions,
domain SQL rules, and registry freshness remain outside these schema files.
Those checks belong to semantic SSOT validation, source-contract tests, SQL
guards, or future runtime gates.

## Org Compiler Slice

The first compiler function is
`compile_org_ontology_authoring_document`. It consumes native Org content,
extracts ontology authoring sections through `orgize`, and emits the draft
authoring DTO shape validated by
`org_ontology_authoring_contract.schema.json`.

The compiler projects:

1. document id and source hash
2. heading path and title
3. property drawer values
4. TODO/status-derived lifecycle state
5. heading tags
6. source spans
7. section-local Org tables
8. section-local source blocks as embedded artifacts

Dataset mappings use `authoringKind = "dataset_mapping"` with typed table
kinds for dataset columns, object mappings, link mappings, and mapping
evidence. SQL source blocks are carried as `embeddedArtifacts` with
`purpose = "mapping"` so downstream source-contract tooling can verify intent
without treating raw rows or model output as ontology truth.

Candidate and promotion review ledgers also use native Org tables. A table with
`review_decision` and `promotion_precondition_met` columns is projected as
`table.kind = "candidate_review"`, while a table with `promotion_decision` and
`reviewer_id` columns is projected as `table.kind = "promotion_review"`. This
lets runtime crates treat Org as the review authority while keeping TSV files as
generated machine projections.

## Org Property Drawer Schema Gate

Org property drawers remain native Org syntax, but sections that declare
`WENDAO_KIND` are also projected into `OrgReasoningPropertyRecord` rows and
validated by `org_reasoning_property_contract.schema.json`.

The first gated reasoning kinds are:

1. `ontology_mapping`
2. `evidence_summary`
3. `validation_feedback`

The gate requires UUID `ID` values, known `WENDAO_KIND` values, kind-specific
required properties, known `PROMOTION_STATE` values, numeric `CONFIDENCE`
values in the `0..1` range, and valid `SOURCE_SHA256` digests when present.
Unknown property keys produce deterministic parser diagnostics instead of
flowing into ontology truth or graph reasoning as free-form agent metadata.

Compiler compatibility properties such as `ONTOLOGY_KIND`, `STATUS`, and
`LIFECYCLE_STATE` remain allowed because the existing Org compiler uses them to
project authoring kind and lifecycle state. New reasoning metadata should use
the `WENDAO_*` keys rather than inventing ad hoc property names.

## Related RFC

See
[`2026-05-11-wendao-org-mode-agentic-memory-rfc.md`](../../../../../docs/rfcs/2026-05-11-wendao-org-mode-agentic-memory-rfc.md)
for the Org-native ontology authoring boundary.
