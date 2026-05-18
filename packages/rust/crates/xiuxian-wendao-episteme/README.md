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

Ontology is the core Episteme source contract. This crate now exposes a
conservative ontology admission surface that reads `ontology/manifest.toml`,
validates ownership boundaries, rejects mutable source contracts, verifies
unique `episteme://` domain ids, and checks declared RDF, SQL rule, policy,
dataset mapping, extension, and API-surface artifacts without mutating them.
The source ontology remains repository-owned, while Rust owns deterministic
admission before later registry, read-model, and quality-gate slices consume
the ontology contract.

The crate does not invoke Python helpers, write ontology sources, promote RDF
truth, or change Flight/OpenAPI/Arrow contracts.

The existing `xiuxian-wendao::episteme::source_contract` facade now delegates
its manifest/path admission entry points to this crate while keeping its richer
legacy error type for current Studio and Gateway consumers.
