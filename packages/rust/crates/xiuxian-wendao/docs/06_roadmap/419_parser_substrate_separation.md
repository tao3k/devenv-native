# Parser Substrate Separation

:PROPERTIES:
:ID: wendao-parser-substrate-separation
:PARENT: [[index]]
:TAGS: roadmap, parser, packages, qianji, qianhuan, markdown, org
:STATUS: ACTIVE
:END:

## Purpose

This note answers one future-facing parser question:

> how should Wendao parser capability be separated so lightweight consumers can
> reuse document parsing without depending on the whole `xiuxian-wendao`
> business-domain crate?

The answer is not "leave parsers under Wendao forever," and it is also not
"move the current directory without changing contracts." The clean target is an
independent parser crate, tentatively `xiuxian-wendao-parsers`, achieved
through contract refactoring rather than a directory lift.

As of 2026-04-14, thirty-three bounded implementation slices are already landed:
frontmatter parsing, cross-format block-core extraction, cross-format
addressed-target extraction, cross-format literal-addressed-target
extraction, cross-format reference-core extraction, cross-format document core
extraction, cross-format document-envelope extraction, Markdown document
wrapping, Markdown note aggregation, cross-format note-core extraction,
cross-format note-aggregate extraction, Markdown target occurrences,
cross-format target-occurrence core extraction, Markdown link syntax,
parser-owned Markdown section structure, cross-format section-scope
extraction, cross-format section-core extraction, cross-format
section-metadata extraction, target-occurrence range cutover, and the parser
crate consumption cutovers that follow those contracts, plus the mixed
`markdown_snapshot` cutover that now reuses one parser-owned note parse for AST
hits and Wendao note adaptation, plus the local
`src/parsers/docs_governance/` cutover that removes docs-governance parsing
ownership from `zhenfa_router`, plus the local
`src/parsers/semantic_check/` cutover that removes semantic-check grammar
ownership from `zhenfa_router`, plus the parser-owned
`xiuxian-wendao-parsers::section_create` cutover that removes Markdown
section-create parsing and rendering ownership from `zhenfa_router`, plus the
parser-owned Markdown note semantic-fingerprint cutover, plus the note-based
`knowledge_section` and `attachment` incremental cutover that reuses that
parser fingerprint instead of rebuilding Wendao-owned row or hit payloads.
The newest landed additions in this phase are the parser-owned Markdown
symbol-fingerprint substrate, the markdown `local_symbol` incremental cutover
that consumes that parser-owned symbol identity before AST-hit materialization,
and the parser-owned Markdown `comrak` section-extraction cutover that removes
line-scanned heading discovery from shared `toc` and `note` parsing, plus the
parser-owned Markdown single-pass structural cutover that stops
`markdown_snapshot` from reparsing the same body only to rebuild the
symbol-fingerprint surface, plus the parser-owned Markdown single-pass
note-scan cutover that stops note aggregation and the shared
`extract_references(...)` / `extract_targets(...)` helpers from owning their
own duplicate `comrak` parse loops, plus the parser-owned structural document
title cutover that keeps note and TOC fallback titles on the same `comrak`
heading surface instead of a separate line-scan path, plus the parser-owned
structural document lead cutover that keeps note and TOC lead snippets on the
same `comrak` structural scan instead of a separate line-based fallback, plus
the standalone Markdown document structural-alignment cutover that routes the
public `parse_markdown_document(...)` surface through that same parser-owned
title and lead scan, plus the standalone document light-metadata cutover that
keeps those structural semantics while avoiding full reference/target/task
collection on the document-only path, plus the Markdown note artifact split
that lets note-only callers stop before symbol-fingerprint work while
`parse_markdown_note_artifacts(...)` remains the richer direct-consumer path.
Parser-only contracts that are already proven
cross-crate now live in `xiuxian-wendao-parsers`, while `xiuxian-wendao`
keeps Wendao-owned adapters, domain-side Markdown note adaptation, and local
parser-owner surfaces that are not yet proven reusable outside Wendao.

## One-Sentence Rule

- parser-owned parsing belongs in an independent crate tentatively named
  `xiuxian-wendao-parsers`
- Wendao-owned adapters that construct graph, retrieval, persistence, or other
  domain records stay in `xiuxian-wendao`

## Why This Split Exists

Three forces now meet in one place:

1. `xiuxian-qianji` already imports
   `xiuxian_wendao_parsers::frontmatter::parse_frontmatter`
2. future consumers such as `xiuxian-qianhuan` persona and template flows will
   benefit from shared document parsing
3. the parser lane is expected to grow beyond Markdown into Org document
   structure

Those forces justify a reusable parser substrate, but they do not justify
keeping parser ownership inside the main crate forever.

## Why Not A Raw Directory Move

The current `src/parsers/` tree mixes three different layers:

1. syntax-only parsing
2. parser-plus-intermediate-model assembly
3. Wendao domain projection

Examples:

1. Markdown note parsing currently builds `LinkGraphDocument`
2. link-graph query parsing currently merges into `LinkGraphSearchOptions`
3. graph persistence parsing currently constructs `Entity` and `Relation`

That means a clean direct extraction requires contract refactoring first. The
target is still direct parser independence, but the path is:

1. define parser-owned intermediate contracts
2. move parser execution to the independent crate
3. keep Wendao as a consumer and adapter over those contracts

## Current Audit Findings

### Direct parser-layer candidates

These families are good extraction candidates because their outputs can become
parser-owned and reusable:

1. Markdown frontmatter parsing
2. Markdown document-content metadata
3. Markdown heading and section structure
4. Markdown references and wiki-link structure
5. shared source spans and format-agnostic document coordinates
6. future Org structural parsing

### Wendao-owned adapters

These families stay in `xiuxian-wendao` because they currently encode Wendao
semantics, not only syntax:

1. note parsing that builds `LinkGraphDocument`
2. link-graph query parsing that merges into `LinkGraphSearchOptions`
3. graph persistence parsing that constructs `Entity` and `Relation`
4. resource and graph adapters that enrich syntax into Wendao-specific records

### Local helpers that should not be promoted

These remain local until they prove reusable:

1. gateway request DTO parsing
2. adapter-local config parsing
3. subsystem-local payload decoders
4. one-off rendering or validation helpers

## Target Package Shape

| Package                       | Owns                                                                  | Must not own                                                                   |
| :---------------------------- | :-------------------------------------------------------------------- | :----------------------------------------------------------------------------- |
| `xiuxian-wendao-parsers`      | independent parser execution, parser-owned Markdown and Org contracts | Wendao graph/search/storage records, query DSLs, persistence semantics         |
| `xiuxian-wendao`              | document-to-domain adapters, graph semantics, retrieval DSLs, storage | parser ownership that non-Wendao consumers should import directly              |
| `xiuxian-qianji` / `qianhuan` | app-specific interpretation of syntax records                         | private copies of shared Markdown or Org grammar that should live in substrate |
| `xiuxian-ast`                 | code AST and language analysis                                        | shared Markdown or Org document grammar                                        |

## Extraction Rules

A parser surface should move to the independent parser layer only when all of
the following are true:

1. the input is a durable document-format grammar rather than a Wendao route or
   query language
2. the output can be represented as parser-owned contracts without
   `LinkGraphDocument`,
   `LinkGraphSearchOptions`, `Entity`, `Relation`, or other Wendao-owned types
3. at least one non-Wendao consumer can use the result directly
4. parser-owned tests can express the contract without booting Wendao domain
   services

If any of those conditions are false, keep the parser in `xiuxian-wendao`
temporarily and extract a smaller parser-owned contract first.

## Current Slice Sequence

1. Planning and audit
   - classify current parser families by owner
   - record the direct parser-layer direction and non-goals
   - status: landed
2. Frontmatter extraction
   - create `xiuxian-wendao-parsers`
   - move shared Markdown frontmatter parsing and its parser-owned record there
   - retarget `xiuxian-qianji` so it no longer depends on `xiuxian-wendao`
     only for frontmatter
   - status: landed
3. Markdown link syntax extraction
   - move shared Markdown references, wikilinks, and source-position helpers
     there
   - keep Wendao link-graph and manifest authority logic as parser consumers
   - status: landed
4. Markdown section-contract extraction
   - move shared Markdown section structure, property-drawer parsing, and
     logbook parsing there
   - keep `ParsedSection` in Wendao as an enriched adapter that adds
     `entities` and `observations`
   - retarget property-relation parsing to consume the parser-owned section
     contract
   - status: landed
5. Markdown document-metadata extraction
   - move shared title, tags, doc type, lead, and body word-count extraction
     into the parser crate
   - keep note-to-`LinkGraphDocument` assembly in Wendao
   - status: landed
6. Markdown note-aggregate extraction
   - move parser-owned note orchestration into the parser crate
   - keep workspace-aware link normalization and note-to-`LinkGraphDocument`
     assembly in Wendao
   - status: landed
7. Markdown target-occurrence extraction
   - move note-level inline-link and image target capture into the parser
     crate
   - keep workspace-aware normalization, attachment classification, and final
     note-link reduction in Wendao
   - status: landed
8. Cross-format document-model evaluation
   - introduce one parser-owned `DocumentCore` so future Org support can reuse
     normalized document metadata and body without inheriting Markdown-only raw
     frontmatter representation
   - keep `MarkdownDocument` as the Markdown-specific wrapper that retains raw
     YAML frontmatter
   - status: landed
9. Cross-format note-model evaluation
   - introduce one parser-owned `NoteCore<Reference, Target, Section>` so
     future Org support can reuse the note-body aggregation shape without
     inheriting Markdown-only wrapper structure
   - keep `MarkdownNote` as the Markdown-specific wrapper that retains
     `MarkdownDocument` plus Markdown-owned item contracts
   - status: landed
10. Cross-format addressed-target evaluation

- introduce one parser-owned `AddressedTarget` so future Org support can
  reuse one neutral `target + target_address` item contract
- keep `MarkdownReference` and `MarkdownWikiLink` as Markdown-specific
  wrappers that retain syntax-kind and original-literal fields
- status: landed

11. Cross-format section-scope evaluation

- introduce one parser-owned `SectionScope` so future Org support can reuse
  one neutral heading-ancestry and source-range contract
- keep `MarkdownSection` as the Markdown-specific wrapper that retains
  normalized section text, property drawers, and logbook payload
- status: landed

12. Cross-format section-core evaluation

- introduce one parser-owned `SectionCore` so future Org support can reuse
  one neutral section payload contract above `SectionScope`
- keep `MarkdownSection` as the Markdown-local naming surface over that
  shared section core
- status: landed

13. Cross-format target-occurrence-core evaluation

- introduce one parser-owned `TargetOccurrenceCore<Kind>` so future Org
  support can reuse one neutral ordered `kind + target` occurrence contract
- keep `MarkdownTargetOccurrence` as the Markdown-local naming surface over
  that shared core
- status: landed

14. Cross-format literal-addressed-target evaluation

- introduce one parser-owned `LiteralAddressedTarget` so future Org
  support can reuse one neutral `AddressedTarget + original literal`
  contract
- keep `MarkdownWikiLink` as the Markdown-local naming surface over that
  shared source-preserved core
- status: landed

15. Cross-format reference-core evaluation

- introduce one parser-owned `ReferenceCore<Kind>` so future Org support
  can reuse one neutral `kind + LiteralAddressedTarget` contract
- keep `MarkdownReference` as the Markdown-local naming surface over that
  shared core
- status: landed

16. Cross-format note-aggregate evaluation

- introduce one parser-owned
  `NoteAggregate<Document, Reference, Target, Section>` so future Org
  support can reuse one neutral top-level `document + note-core` contract
- keep `MarkdownNote` as the Markdown-local naming surface over that
  shared aggregate
- status: landed

17. Cross-format document-envelope evaluation

- introduce one parser-owned `DocumentEnvelope<RawMetadata>` so future Org
  support can reuse one neutral top-level `raw metadata + document core`
  contract
- keep `MarkdownDocument` as the Markdown-local naming surface over
  `DocumentEnvelope<serde_yaml::Value>`
- status: landed

18. Cross-format section-metadata evaluation

- introduce one parser-owned `SectionMetadata` so future Org support can
  reuse one neutral `attributes + logbook` payload contract above
  `SectionScope`
- keep `MarkdownSection` as the Markdown-local naming surface over the
  updated shared `SectionCore`
- status: landed

19. Cross-format block-core evaluation

- introduce one parser-owned `BlockCore<Kind>` so future Org support can
  reuse one neutral block payload contract for ranges, content, and
  structural path without inheriting Markdown-only ownership
- keep `MarkdownBlockKind` and `MarkdownBlock` as the Markdown-local kind
  and naming surface over that shared core
- keep `BlockAddress` and `BlockKindSpecifier` in Wendao because they are
  semantic addressing grammar, not parser grammar
- status: landed

20. Target-occurrence range cutover

- extend `TargetOccurrenceCore<Kind>` with parser-visible source ranges
- cut Wendao section enrichment over to note-level parser occurrences filtered
  by section byte range instead of re-running section-local comrak scans
- retire the old Wendao section re-scan path used only for section entity
  extraction
- status: landed

21. Compatibility re-export retirement and Org placeholder boundary

- retarget remaining Wendao internal frontmatter consumers to
  `xiuxian-wendao-parsers`
- retire Wendao compatibility exports for parser-owned frontmatter, blocks,
  references, and wikilinks now that parser-crate ownership is proven
- record Org as a deferred placeholder boundary rather than the active next
  implementation slice
- status: landed

22. Cross-format item-contract continuation

- defer richer shared item-contract work until a concrete non-Wendao consumer
  or explicit Org implementation slice needs it
- status: deferred

23. Markdown note semantic-fingerprint extraction

- introduce one parser-owned `fingerprint_markdown_note(...)` surface over
  `MarkdownNote`
- keep the fingerprint semantic by ignoring layout-only byte and line churn
  while invalidating on parser-owned note semantic changes
- status: landed

24. Note-corpora incremental fingerprint cutover

- store the parser-owned note fingerprint in `search::markdown_snapshot`
- cut `knowledge_section` and `attachment` incremental equivalence over to that
  parser-owned fingerprint before row or hit materialization
- status: landed

## Current Implementation Snapshot

After the frontmatter, cross-format block-core, cross-format addressed-target,
cross-format literal-addressed-target, cross-format reference-core,
document-core, cross-format document-envelope, cross-format note-core,
cross-format note-aggregate, Markdown target-occurrence, cross-format
target-occurrence core, Markdown link syntax, section-contract,
section-scope, section-core, section-metadata, and target-occurrence range
cutover slices:

1. `xiuxian-wendao-parsers` owns `parse_frontmatter`,
   `split_frontmatter`, `NoteFrontmatter`, `BlockCore`, `MarkdownBlockKind`,
   `MarkdownBlock`, `extract_blocks`, `extract_references`,
   `parse_reference_literal`, `AddressedTarget`, `LiteralAddressedTarget`,
   `ReferenceCore`, `MarkdownReference`, `extract_wikilinks`,
   `parse_wikilink_literal`, `MarkdownWikiLink`, the shared `sourcepos`
   helper, `DocumentCore`, `DocumentEnvelope`, `DocumentFormat`,
   `MarkdownDocument`, `NoteCore`, `NoteAggregate`, `MarkdownNoteCore`,
   `parse_markdown_document`, `MarkdownNote`, `parse_markdown_note`,
   `TargetOccurrenceCore`, `MarkdownTargetOccurrence`, `extract_targets`,
   `SectionCore`, `SectionMetadata`, `SectionScope`, `MarkdownSection`,
   `extract_sections`, property-drawer parsing, and logbook parsing
2. `xiuxian-wendao` now imports parser-owned frontmatter and Markdown link
   syntax directly where it needs them and no longer re-exports
   `parse_frontmatter`, blocks, references, or wikilinks from Wendao-owned
   parser namespaces
3. `parse_note` now consumes parser-owned Markdown note aggregation for
   content interpretation, consumes `DocumentCore` for reusable document
   fields, consumes `DocumentEnvelope::raw_metadata` for current
   Markdown-specific metadata adapters, consumes `MarkdownNoteCore` for
   reusable note-body aggregation, consumes parser-owned target occurrences
   for note-level target capture, and still owns workspace-aware link
   normalization plus final
   `LinkGraphDocument` assembly
4. `SectionCore` now keeps one parser-owned `SectionScope`, one parser-owned
   `SectionMetadata`, plus normalized section text, while
   `MarkdownSection` remains the Markdown-local naming surface and
   `ParsedSection` remains Wendao-owned as the enriched adapter that adds
   note-link `entities` and `CodeObservation` rows
5. property-relation parsing now consumes parser-owned section scope and
   section metadata attributes rather than depending on the enriched Wendao
   adapter
6. `xiuxian-qianji` imports `xiuxian_wendao_parsers::frontmatter::parse_frontmatter`
   directly for persona annotation parsing
7. Wendao adapters such as `link_graph_refs`, docs-governance parsing, and
   internal-skill authority now consume parser-owned Markdown link syntax
   directly
8. docs-governance line/path parsing now lives under
   `xiuxian-wendao::parsers::docs_governance`, so `zhenfa_router` no longer
   serves as the ownership home for parser grammar that gateway and semantic
   checks both consume
9. `MarkdownTargetOccurrence` is now the Markdown-local naming surface over
   `TargetOccurrenceCore<MarkdownTargetOccurrenceKind>`, so future Org support
   can add its own occurrence-kind enum without inheriting Markdown-only type
   ownership
10. `TargetOccurrenceCore<Kind>` now preserves parser-visible occurrence byte
    and line ranges, so Wendao can partition note-level occurrences without a
    second syntax pass
11. `MarkdownWikiLink` is now the Markdown-local naming surface over
    `LiteralAddressedTarget`, so future Org support can reuse one
    source-preserved addressed-target contract without inheriting Markdown-only
    wrapper ownership
12. `MarkdownReference` is now the Markdown-local naming surface over
    `ReferenceCore<MarkdownReferenceKind>`, so future Org support can add its
    own reference-kind enum without inheriting Markdown-only wrapper ownership
13. `MarkdownNote` is now the Markdown-local naming surface over
    `NoteAggregate<MarkdownDocument, MarkdownReference, MarkdownTargetOccurrence, MarkdownSection>`,
    so future Org support can reuse one neutral top-level note aggregate
    without inheriting Markdown-only wrapper ownership
14. `MarkdownDocument` is now the Markdown-local naming surface over
    `DocumentEnvelope<serde_yaml::Value>`, so future Org support can reuse one
    neutral top-level document wrapper without inheriting Markdown-only
    wrapper ownership
15. `ParsedSection` entity enrichment now filters parser-owned note-level
    target occurrences by section byte range before workspace-aware
    normalization, so Wendao no longer re-parses section bodies to find note
    links
16. embedded wikilinks remain ignored on the current parser-owned target
    path, matching the old Wendao note-level behavior
17. note-to-`LinkGraphDocument` assembly, workspace-aware link reduction,
    link-graph query, and persistence decoding remain Wendao-owned adapters
    and have not moved
18. `MarkdownBlock` is now the Markdown-local naming surface over
    `BlockCore<MarkdownBlockKind>`, so future Org support can add its own
    block-kind enum without inheriting Markdown-only payload ownership, while
    Wendao keeps block-address matching and page-index addressing grammar
19. parser-owned Markdown section-create insertion planning and heading-chain
    rendering now live under `xiuxian_wendao_parsers::section_create`, so
    `semantic_edit` consumes the parser helper surface without
    `zhenfa_router` owning it
20. Org remains a documented placeholder boundary only; no Org parser
    implementation slice is active until a direct consumer or explicit parser
    requirement appears
21. `xiuxian_wendao_parsers::note::fingerprint_markdown_note` now owns one
    parser-level semantic fingerprint for `MarkdownNote`, so parser consumers
    can compare note semantics without hashing Wendao-owned row or hit payloads
22. `search::markdown_snapshot` now caches that parser-owned note fingerprint
    beside the parser-owned parse result and the adapted Wendao `ParsedNote`,
    so note-based consumers do not need to recompute row or hit digests just to
    detect unchanged markdown semantics
23. `knowledge_section` and `attachment` incremental planning now reuse the
    parser-owned Markdown note fingerprint before row or hit materialization, so
    metadata-only note edits stop before row or attachment-hit rebuild while
    semantic note changes still invalidate as expected
24. `xiuxian_wendao_parsers::note::fingerprint_markdown_symbol_surface` now
    owns one parser-level Markdown symbol fingerprint for headings, task
    items, property drawers, and `:OBSERVE:` entries instead of leaving the
    markdown symbol surface coupled to Wendao-owned `AstSearchHit` hashing
25. `search::markdown_snapshot` now caches that parser-owned symbol
    fingerprint and lazily materializes markdown AST hits, so the markdown
    branch of `local_symbol` can compare parser-owned symbol identity first and
    only build Wendao-owned hit payloads when the symbol surface actually
    changed

## Migration Bias

Do not optimize for file-count balance.

Optimize for these outcomes:

1. lightweight consumers stop pulling `xiuxian-wendao` for syntax-only parsing
2. Wendao keeps business semantics and document-to-domain projection ownership
3. Markdown and future Org support converge on one independent parser lane
4. no long-lived compatibility shim leaves the same parser API owned by two
   crates

:RELATIONS:
:LINKS: [[02_parser/architecture]], [[06_roadmap/417_wendao_package_boundary_matrix]], [[06_roadmap/405_large_rust_modularization]]
:END:

---

:FOOTER:
:LAST_SYNC: 2026-04-14
:END:
