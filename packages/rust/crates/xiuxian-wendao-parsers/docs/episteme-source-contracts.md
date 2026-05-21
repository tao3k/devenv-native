# Episteme Source Contracts

`xiuxian-wendao-parsers` owns parser-level source-contract DTOs for Wendao
episteme surfaces. These DTOs turn authoring files such as TOML manifests, TSV
source indexes, RDF/Org authoring ledgers, and mapping tables into stable
records that runtime crates can validate and schedule.

The parser crate does not walk source corpus roots, compute source-file
hashes, execute OCR or ASR, call LLMs, run DuckDB, mutate RDF, or open Gateway
routes. Runtime crates consume parser DTOs and own those environment-dependent
steps.

## Source Contract Shape

The first episteme source-contract parser covers:

- `source_manifest.toml`
- `files.tsv`
- `extraction_queue.tsv`

The parser API emits:

- `EpistemeSourceManifest`
- `EpistemeFileRow`
- `EpistemeExtractionQueueRow`

Repository-level `ontology/manifest.toml` selects source manifests and mapping
ledgers. A repository with one source contract can be selected automatically;
a repository with multiple configured domains or source contracts must declare
`[active_source_contract]` so runtime validation is deterministic.

The Wendao product crate consumes these DTOs for source validation,
source hash drift checks, queue selection, extraction planning, scheduling, and
cache identity. It can also compile validated DTO-derived facts into downstream
read-model seed batches after backend validation succeeds. The parser API
remains pure text-to-DTO parsing so episteme repositories can keep
domain-specific authoring while runtime behavior stays backend-owned.
The structure/TOC ledger command consumes those same DTOs after Rust
validation and writes an evidence-only Org ledger. Its fast structure-preview
mode validates metadata without hashing file contents; full source-content
hash proof remains a Rust validation mode outside the parser boundary. The
command does not make parser DTOs responsible for extraction, SQL, RDF, or
promotion.
The targeted evidence reader uses parser DTOs only to resolve a selected
`file_id` to source metadata. Runtime code owns source path resolution,
bounded text preview, binary no-preview policy, and validation mode handling.
Evidence selection plans use the same DTO boundary: parser rows provide stable
`file_id`, source metadata, and extraction-route facts, while runtime code
owns duplicate/unknown-id rejection, Org/TSV/JSON artifact writing, and the
guarantee that selection ledgers do not embed raw source content or promote
RDF truth.
Selection-driven extraction planning also remains outside parser ownership:
runtime code reads generated selection artifacts, validates selected ids
against queue rows, and writes extraction tasks without changing parser DTOs.
Episteme runtime defaults from `episteme.toml` are also outside parser
ownership. They configure runtime roots for corpus and generated artifacts;
parser DTOs continue to cover source manifests, TSV rows, and Org ledgers only.

Deployment registry loading is outside the parser boundary. Studio and the
Wendao Rust backend may load episteme repositories from `wendao.toml` entries
with local `path` or Git `url` fields, then pass the materialized repository
root into this parser contract. Parser DTOs never own Git checkout, cache, or
resolved revision behavior. Runtime registry validation may also read
repository-level manifest domain ids and extension targets to prove that loaded
episteme repositories form a complete reference graph before parser DTOs are
used for source-contract scheduling. That validated graph can become
read-model seed rows without invoking parser DTOs, because common ontology
repositories may provide domain topology without source-contract TSV files.
