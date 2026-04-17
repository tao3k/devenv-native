# Parser Note Aggregate

:PROPERTIES:
:ID: wendao-parser-note
:PARENT: [[02_parser/index]]
:TAGS: parser, note, markdown
:STATUS: ACTIVE
:END:

## Objective

Wendao now treats one cross-format note core plus one cross-format top-level
note aggregate as parser-owned shared surfaces in
`xiuxian-wendao-parsers`, while `xiuxian-wendao` keeps the workspace-aware
adapter that resolves links and assembles `LinkGraphDocument`.

## Contract

The canonical parser-owned note contracts now split into three layers:

1. `NoteCore<Reference, Target, Section>`
   - format-owned references in document order
   - format-owned note-level target occurrences in document order
   - format-owned section structure in body order
2. `NoteAggregate<Document, Reference, Target, Section>`
   - one parser-owned format document wrapper
   - one embedded `NoteCore<Reference, Target, Section>`
3. `MarkdownNote`
   - Markdown-local alias over
     `NoteAggregate<MarkdownDocument, MarkdownReference, MarkdownTargetOccurrence, MarkdownSection>`
   - `MarkdownTargetOccurrence` remains the Markdown-local naming surface over
     `TargetOccurrenceCore<MarkdownTargetOccurrenceKind>`

`NoteCore` is the reusable cross-format note-body aggregation shape.
`NoteAggregate<Document, Reference, Target, Section>` is the reusable
cross-format top-level note aggregate shape.
`MarkdownNote` is the Markdown-specific naming surface that keeps
`DocumentEnvelope<serde_yaml::Value>` plus Markdown-owned item contracts
available. None of these contracts include path identity,
attachment classification,
workspace-aware link normalization,
timestamps, or graph records.

## Extraction Rules

The shared aggregate follows these rules for Markdown:

1. `parse_markdown_note` first parses `MarkdownDocument`
2. `parse_markdown_note(...)` now stops at the parser-owned `MarkdownNote`
   aggregate when callers only need the note surface
3. `parse_markdown_note_artifacts(...)` keeps the richer direct-consumer path
   and runs one parser-owned markdown scan against `MarkdownDocument.core.body`
4. fallback document title on the note hot path now comes from the first
   parser-owned structural heading in that same scan rather than a separate
   line-scan title pass
5. fallback document lead on the note hot path now comes from the first
   parser-owned structural paragraph snippet in that same scan rather than a
   separate line-scan lead pass
6. section extraction consumes that same parser-owned scan for heading and
   task discovery, while property drawer and `:LOGBOOK:` parsing stay on the
   parser-owned line helpers layered beneath `MarkdownSection`
7. ordinary reference extraction consumes the same parser-owned scan and
   preserves parser-owned document order
8. target-occurrence extraction consumes that same parser-owned scan and
   preserves parser-visible occurrence ranges without filesystem context
9. the resulting `NoteAggregate<...>` preserves parser-owned raw targets
   plus parser-visible occurrence ranges without filesystem context
10. sections preserve parser-owned heading scope plus shared `SectionMetadata`
    without Wendao enrichments

## Consumer Boundary

`xiuxian-wendao` now consumes this parser-owned note contract:

1. `parse_note` uses `MarkdownNote` as the parser-owned aggregate entry point
2. Wendao consumes `MarkdownDocument.core` for reusable document metadata and
   `MarkdownDocument.raw_metadata` for the current Markdown-specific saliency
   and timestamp adapters
3. Wendao consumes `MarkdownNote.core` for reusable note-body aggregation
   shape while keeping Markdown-specific item types intact
4. Wendao still owns `doc_id`, `path`, timestamps, saliency defaults, and
   `LinkGraphDocument` assembly
5. workspace-aware note-link and attachment resolution still happen in Wendao
6. Wendao still enriches parser-owned sections into `ParsedSection` by adding
   note-link entities and `CodeObservation` rows
7. section entity enrichment now partitions parser-owned note-level target
   occurrences by section byte range before normalization

## Fingerprint Boundary

Parser-owned Markdown note fingerprinting now follows one explicit split:

1. `xiuxian_wendao_parsers::note::fingerprint_markdown_note` owns the
   parser-level semantic fingerprint over `MarkdownNote`
2. the fingerprint intentionally ignores layout-only churn such as byte ranges,
   line ranges, and raw body formatting noise
3. the fingerprint invalidates when parser-owned note semantics change, such as
   document metadata, references, targets, or section payloads
4. `xiuxian_wendao::search::markdown_snapshot` caches that parser-owned note
   fingerprint beside the shared parser-owned parse result
5. note-based Wendao corpora such as `knowledge_section` and `attachment`
   consume the cached parser fingerprint for incremental equivalence instead of
   rebuilding Wendao-owned rows or attachment-hit payloads only to re-hash them
6. `xiuxian_wendao_parsers::note::fingerprint_markdown_symbol_surface` owns
   one parser-level Markdown symbol fingerprint for the local-symbol surface
   over headings, task items, property drawers, and `:OBSERVE:` entries
7. that symbol fingerprint is derived from parser-owned `comrak` traversal for
   headings and task items, then combined with parser-owned section metadata
   for property-drawer and observation semantics
8. `xiuxian_wendao::search::markdown_snapshot` caches the symbol fingerprint
   beside the parser-owned note parse and lazily materializes markdown AST hits
   only when a downstream consumer actually needs them
9. the markdown branch of `local_symbol` now compares that parser-owned symbol
   fingerprint before AST-hit materialization, so metadata-only Markdown edits
   can short-circuit without rebuilding Wendao-owned hit payloads
10. `xiuxian_wendao_parsers::note::parse_markdown_note_artifacts` now owns one
    parser-level single-pass path that returns `MarkdownNote` plus the
    symbol-surface fingerprint derived from the same structural traversal
11. `xiuxian_wendao::search::markdown_snapshot` now consumes that single-pass
    parser artifact instead of reparsing the same markdown body with `comrak`
    only to compute the symbol fingerprint after note aggregation
12. the same single-pass parser-owned scan now also feeds
    `extract_references(...)` and `extract_targets(...)`, so note aggregation
    no longer reparses one markdown body a second and third time only to
    recover ordered references and raw target occurrences
13. note-only Wendao consumers such as `parse_note(...)` and Studio markdown
    metadata now stay on `parse_markdown_note(...)`, while direct richer
    consumers such as `markdown_snapshot` remain on
    `parse_markdown_note_artifacts(...)`

## Regression Coverage

Coverage for this contract lives in:

1. `packages/rust/crates/xiuxian-wendao-parsers/tests/unit/note.rs`
2. `tests/unit/parsers/markdown/document.rs`
3. `tests/unit/parsers/markdown/namespace.rs`
4. `tests/unit/search/knowledge_section/build/mod.rs`
5. `tests/unit/search/attachment/build/mod.rs`
6. `tests/unit/search/local_symbol/build/mod.rs`
7. `tests/unit/workflow_demo.rs`

:RELATIONS:
:LINKS: [[02_parser/index]], [[02_parser/architecture]], [[02_parser/document]], [[02_parser/targets]], [[02_parser/sections]], [[06_roadmap/419_parser_substrate_separation]]
:END:

---

:FOOTER:
:LAST_SYNC: 2026-04-14
:END:
