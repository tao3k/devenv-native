# Parser Architecture

:PROPERTIES:
:ID: wendao-parser-architecture
:PARENT: [[02_parser/index|Wendao Parser Docs]]
:TAGS: parser, architecture, implementation
:STATUS: ACTIVE
:END:

## Objective

`xiuxian-wendao` keeps Wendao-owned parser adapters under the single crate-root
namespace `src/parsers/`, while reusable parser-owned syntax, block, target,
and note-aggregate contracts may move to `xiuxian-wendao-parsers` once they
are cleanly separated from Wendao domain records.

## Canonical Parser Families

| Namespace                                             | Input shape                         | Canonical output                           | Notes                                                                                                                                                                                                                                                                                                                                                                                                                             |
| ----------------------------------------------------- | ----------------------------------- | ------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `parsers::docs_governance`                            | docs-governance markdown/path lines | line slices, drawer/footer/link helpers    | Shared by semantic-check docs governance and Studio markdown metadata; keeps path/line parsing out of `zhenfa_router`, while issue policy, rendering, and tool registration stay in `zhenfa_router`; currently Wendao-local because the grammar is still Wendao-specific docs governance policy                                                                                                                                   |
| `parsers::semantic_check`                             | semantic-check wiki links/contracts | reference helpers, contract parse helpers  | Shared by semantic-check checks and test bridges; keeps semantic-check grammar and helper types out of `zhenfa_router`, while issue policy, rendering, and tool registration stay in `zhenfa_router`; currently Wendao-local because the grammar is still Wendao-specific semantic-check policy                                                                                                                                   |
| `parsers::markdown`                                   | Markdown and Org notes              | sections, note adapters                    | Shared by indexing, search, enhancement, semantic checks, and semantic edit; frontmatter, Org document metadata, code observation parsing, block extraction, document metadata, target occurrences, note aggregation, references, wikilinks, sourcepos, parser-owned section structure, and parser-owned section-create planning/rendering now live in `xiuxian-wendao-parsers`, while Wendao keeps enrichments and note adapters |
| `parsers::link_graph::query`                          | link-graph search query strings     | `ParsedLinkGraphQuery`                     | Shared query-language parsing                                                                                                                                                                                                                                                                                                                                                                                                     |
| `parsers::zhixing::tasks`                             | zhixing task lines                  | task projections and normalized identities | Shared by ingest and stats                                                                                                                                                                                                                                                                                                                                                                                                        |
| `parsers::languages::rust::cargo::dependencies`       | `Cargo.toml` dependency tables      | dependency projections                     | Shared by dependency indexing                                                                                                                                                                                                                                                                                                                                                                                                     |
| `parsers::languages::python::pyproject::dependencies` | `pyproject.toml` dependency tables  | dependency projections                     | Shared by dependency indexing                                                                                                                                                                                                                                                                                                                                                                                                     |
| `parsers::search::repo_code_query`                    | repo-code search query strings      | typed repo-code query                      | Shared by repo-search flows                                                                                                                                                                                                                                                                                                                                                                                                       |
| `parsers::graph::persistence`                         | graph JSON dicts                    | `Entity` and `Relation`                    | Shared by graph save/load persistence                                                                                                                                                                                                                                                                                                                                                                                             |

## Parser vs Local Helper Rule

Code belongs under `src/parsers/` when all of the following are true:

1. it parses a durable external or cross-subsystem input surface
2. it returns a canonical typed output reused by multiple consumers
3. the parsing semantics are domain-core, not tied to one adapter DTO
4. parser-owned unit coverage can live under `tests/unit/parsers/`

Code stays outside `src/parsers/` when it is one of these:

1. adapter-local request parsing, such as `search/queries/graphql/document.rs`
2. gateway-local validation and DTO decoding, such as
   `gateway/studio/router/handlers/repo/parse.rs`
3. subsystem-local config or payload decode helpers, such as
   `analyzers/config/parse.rs`,
   `search_plane/repo_entity/query/hydrate/parse.rs`, and
   `pybindings/link_graph_py/engine/refresh/parse.rs`
4. query models or execution modules, such as `entity/query.rs` and
   `storage/query.rs`

## Implementation Rules

1. `mod.rs` is interface-only and should re-export leaf modules.
2. Medium or complex parser work should use feature folders.
3. Direct migration is preferred over compatibility shims.
4. Parser-owned unit coverage should live under `tests/unit/parsers/<family>/`.
5. Consumer subsystems may import parser services, but they do not own
   duplicate parser namespaces.
6. `zhenfa_router` may consume parser helpers, but it must not become the
   ownership home for parser grammar or parser-owned intermediate types.

## Cross-Crate Reuse Rule

`xiuxian-wendao` is still the ownership home for Wendao domain parser
adapters, but not every parser family under `src/parsers/` should stay
Wendao-owned forever.

When a parser family becomes reusable across packages such as `xiuxian-qianji`
or future document-flow crates, the long-term extraction target is
an independent parser crate, tentatively `xiuxian-wendao-parsers`, rather than
another consumer-local helper tree.

That extraction is no longer theoretical: the parser-owned frontmatter
contract, raw frontmatter splitter, cross-format addressed-target core,
cross-format literal-addressed-target core, cross-format reference core,
cross-format document core, cross-format document-envelope core, cross-format
note core, cross-format note-aggregate core, shared target-occurrence core
with Markdown naming surface, shared block core with Markdown block naming
surface, Markdown reference grammar, Markdown wikilink grammar, shared
source-position helper, shared full section-core contract with its nested
section-scope core, parser-owned Markdown naming surfaces, and parser-owned Org
document/note/section surfaces already live in `xiuxian-wendao-parsers`, while
`xiuxian-wendao` keeps only Wendao-owned adapters and domain-side consumption.

A parser surface is a direct parser-crate candidate only when all of the
following are true:

1. the input is a durable document-format grammar such as Markdown or Org
2. the output can be expressed as parser-owned intermediate contracts without
   Wendao-owned domain records such as
   `LinkGraphDocument`, `LinkGraphSearchOptions`, `Entity`, `Relation`, or
   `WendaoResourceUri`
3. at least one non-Wendao package can consume the result directly

If the parser surface builds Wendao graph, retrieval, persistence, or other
business semantics, it stays in `xiuxian-wendao` and should be treated as a
domain adapter over any future independent parser crate.

Org is now an active parser-owned document-format slice. Native `.org` files
route through `orgize` for headline and property-drawer extraction, then reuse
the shared document, note, and section contracts before Wendao adapts them into
graph/search records. Markdown files may still carry Org-style
`:PROPERTIES:` drawers as a compatibility syntax because that drawer shape is
an Org-native metadata convention.

See [../06_roadmap/419_parser_substrate_separation.md](../06_roadmap/419_parser_substrate_separation.md)
for the package-split plan.

## Block Contract Boundary

Markdown block extraction is now split across five explicit contracts:

1. `xiuxian_wendao_parsers::blocks::BlockCore<Kind>` owns one reusable block
   payload shape for block identity, ranges, content hash, raw content,
   optional explicit ID, and structural path
2. `xiuxian_wendao_parsers::blocks::MarkdownBlockKind` owns the Markdown-local
   block variants
3. `xiuxian_wendao_parsers::blocks::MarkdownBlock` is the Markdown-local
   naming surface over `BlockCore<MarkdownBlockKind>`
4. `xiuxian_wendao_parsers::blocks::extract_blocks` is the shared parser-owned
   entry point for block extraction from one section body
5. `xiuxian_wendao::link_graph::BlockAddress` and
   `xiuxian_wendao::link_graph::BlockKindSpecifier` remain Wendao-owned
   because they encode semantic addressing grammar, not Markdown parsing
6. Wendao page-index building consumes parser-owned Markdown blocks directly
7. block-to-address matching stays Wendao-owned as a domain helper layered on
   top of parser-owned block payloads

## Section Contract Boundary

Markdown section extraction is now split across five explicit contracts:

1. `xiuxian_wendao_parsers::sections::SectionCore` owns shared normalized
   section text plus one nested `SectionScope` and one nested
   `SectionMetadata`
2. `xiuxian_wendao_parsers::sections::SectionMetadata` owns shared
   property-drawer attributes and logbook entries reusable across formats
3. `xiuxian_wendao_parsers::sections::SectionScope` stays the nested shared
   heading-ancestry and source-range contract inside `SectionCore`
4. `xiuxian_wendao_parsers::sections::MarkdownSection` is the Markdown-local
   naming surface over `SectionCore`
5. `xiuxian_wendao::parsers::markdown::ParsedSection` is an enriched adapter
   that adds Wendao-owned `entities` and parser-owned `CodeObservation` rows
6. property-relation parsing can consume the parser-owned section contract
   because it only needs heading scope and parser-owned metadata attributes
7. note parsing that assembles `LinkGraphDocument` remains Wendao-owned
8. `xiuxian_wendao_parsers::sections::extract_sections` now owns parser-level
   Markdown heading discovery and section-boundary construction through
   `comrak` traversal plus source-position ranges
9. property drawer and `:LOGBOOK:` parsing remain parser-owned line helpers
   layered beneath the same shared section contract

## Code Observation Boundary

Markdown `:OBSERVE:` handling is now split across four explicit contracts:

1. `xiuxian_wendao_parsers::code_observation::CodeObservation` owns the
   parser-owned parsed observation entry
2. `xiuxian_wendao_parsers::code_observation::extract_observations` owns
   parser-owned extraction from one attribute map
3. `xiuxian_wendao_parsers::code_observation::path_matches_scope` owns
   parser-owned scope matching for observation filters
4. `xiuxian_wendao::parsers::markdown::code_observation` is now adapter-only
   and re-exports the parser-owned contract
5. semantic-check issue policy and downstream interpretation remain
   Wendao-owned

## Document Contract Boundary

Markdown document-content parsing is now split across four explicit contracts:

1. `xiuxian_wendao_parsers::document::DocumentCore` owns cross-format
   document format, normalized body, title, tags, doc type, lead, and word
   count
2. `xiuxian_wendao_parsers::document::DocumentEnvelope<RawMetadata>` owns one
   shared top-level `raw metadata + document core` contract reusable across
   formats
3. `xiuxian_wendao_parsers::document::MarkdownDocument` is the Markdown-local
   alias over `DocumentEnvelope<serde_yaml::Value>`
4. `xiuxian_wendao::parsers::markdown::parse_note` is the Wendao adapter that
   adds `doc_id`, path identity, timestamps, saliency defaults, links,
   sections, and `LinkGraphDocument` assembly
5. this keeps content-owned parsing reusable without moving graph or retrieval
   semantics into the parser crate

## Note Aggregate Boundary

Markdown note parsing is now split across four explicit contracts:

1. `xiuxian_wendao_parsers::note::NoteCore<Reference, Target, Section>` owns
   one reusable note-body aggregation shape for ordered references, targets,
   and sections
2. `xiuxian_wendao_parsers::note::NoteAggregate<Document, Reference, Target, Section>`
   owns one reusable top-level `document + note-core` aggregate shape
3. `xiuxian_wendao_parsers::note::MarkdownNote` is the Markdown-local alias
   over `NoteAggregate<MarkdownDocument, MarkdownReference, MarkdownTargetOccurrence, MarkdownSection>`
4. `xiuxian_wendao_parsers::note::parse_markdown_note` is the shared
   parser-owned entry point for Markdown note aggregation
5. `xiuxian_wendao::parsers::markdown::parse_note` is the Wendao adapter that
   consumes `MarkdownDocument.core` for reusable document metadata, consumes
   `MarkdownDocument.raw_metadata` for current Markdown-specific adapters,
   consumes `MarkdownNote.core` for reusable note-body aggregation, and adds
   workspace-aware link normalization, attachment classification, enriched
   sections, and final `LinkGraphDocument` assembly
6. this keeps parser orchestration reusable without moving filesystem or graph
   semantics into the parser crate
7. `xiuxian_wendao::search::markdown_snapshot` is now an explicit mixed
   consumer: it builds markdown AST/property/observation hits from parser-owned
   `MarkdownSection` rows, then adapts the same parser-owned note into
   Wendao `ParsedNote` for knowledge-section and attachment consumers without
   reparsing the markdown body
8. `xiuxian_wendao_parsers::note::fingerprint_markdown_note` now owns the
   parser-level semantic fingerprint for `MarkdownNote`, and
   `xiuxian_wendao::search::markdown_snapshot` plus the note-based
   `knowledge_section` and `attachment` planners consume that parser-owned
   fingerprint for incremental reuse instead of rebuilding Wendao-owned rows or
   hit payloads just to compare unchanged markdown semantics
9. `xiuxian_wendao_parsers::note::fingerprint_markdown_symbol_surface` now
   owns the parser-level Markdown symbol fingerprint for headings, task items,
   property drawers, and `:OBSERVE:` entries at parser-owned `comrak`
   structural granularity
10. `xiuxian_wendao::search::markdown_snapshot` now keeps markdown AST hits as
    a lazy Wendao-owned derivative over the cached parser-owned note parse and
    symbol fingerprint, so the markdown branch of `local_symbol` can compare
    parser-owned symbol identity before materializing Wendao-owned hit payloads
11. `parse_markdown_toc(...)` and `parse_markdown_note(...)` now share the
    same parser-owned `comrak` section extraction path, so heading-like text
    inside fenced code blocks no longer participates in parser-owned section
    discovery
12. `xiuxian_wendao_parsers::note::parse_markdown_note_artifacts(...)` now
    owns a parser-level single-pass path that returns `MarkdownNote` plus the
    symbol fingerprint derived from the same structural traversal
13. `xiuxian_wendao::search::markdown_snapshot` now consumes that parser-owned
    single-pass note artifact instead of reparsing one markdown body only to
    rebuild the symbol-fingerprint surface
14. the same parser-owned single-pass markdown scan now also feeds
    `extract_references(...)` and `extract_targets(...)`, so note aggregation
    and the standalone shared extractors no longer own separate `comrak`
    parse loops for ordinary references and raw target occurrences
15. `parse_markdown_note(...)` and `parse_markdown_toc(...)` now assemble
    fallback document titles from the same parser-owned structural heading scan
    they already use for section discovery, so fenced code-block text no longer
    participates in title fallback on those hot paths
16. those same hot paths now also assemble fallback document leads from the
    parser-owned structural scan rather than a separate line-based lead path,
    so fenced code content no longer participates in note/TOC lead snippets
17. `parse_markdown_document(...)` now also consumes that parser-owned
    structural title and lead scan directly, so standalone Markdown document
    parsing no longer keeps a separate historical line-scan fallback for those
    metadata fields
18. the standalone document path now uses a lighter parser-owned metadata scan
    for title and lead, while note/TOC/full-structure consumers keep the
    richer `MarkdownStructure` path for references, targets, and
    heading/task-driven section discovery
19. `parse_markdown_note(...)` now uses a true note-only parser path, while
    `parse_markdown_note_artifacts(...)` remains the richer direct-consumer
    surface for callers that still need the symbol fingerprint

## TOC Boundary

Markdown TOC/document-structure parsing is now split across four explicit
contracts:

1. `xiuxian_wendao_parsers::toc::TocDocument<Document, Section>` owns one
   reusable `document + ordered sections` aggregate shape
2. `xiuxian_wendao_parsers::toc::MarkdownTocDocument` is the Markdown-local
   alias over `TocDocument<MarkdownDocument, MarkdownSection>`
3. `xiuxian_wendao_parsers::toc::parse_markdown_toc` is the parser-owned
   entry point for Markdown document structure without repo or filesystem-root
   semantics
4. `xiuxian_wendao::analyzers::projection::markdown` consumes that parser-owned
   TOC surface for projected page-index document parsing, while Wendao keeps
   projected `doc_id` derivation and repo-scoped page-index outputs local
5. `DocsToolService::get_toc_documents()` remains a Wendao capability opener
   over repo-scoped projected page-index documents rather than the parser
   owner for Markdown TOC extraction

## Section Create Boundary

Markdown section-create planning is now split across five explicit contracts:

1. `xiuxian_wendao_parsers::section_create::InsertionInfo` owns the
   parser-owned insertion-byte, start-level, remaining-path, and sibling-context
   contract
2. `xiuxian_wendao_parsers::section_create::find_insertion_point` owns
   parser-owned heading traversal and insertion planning for one Markdown
   heading path
3. `xiuxian_wendao_parsers::section_create::build_new_sections_content_with_options`
   owns parser-owned heading-chain rendering with optional `:ID:` drawers
4. `xiuxian_wendao::parsers::markdown::section_create` is now adapter-only and
   re-exports the parser-owned helper surface
5. `xiuxian_wendao::zhenfa_router::native::semantic_edit` remains the consumer
   that owns mutation policy, XML response rendering, and file writes

## Docs Governance Boundary

Docs-governance parsing is now split across four explicit contracts:

1. `xiuxian_wendao::parsers::docs_governance` owns line/path parsing helpers
   for opaque IDs, canonical-doc classification, line slicing, relations
   blocks, footer blocks, and hidden-path link extraction
2. `zhenfa_router::native::semantic_check::docs_governance` owns issue policy,
   workspace traversal, rendering, and fix planning
3. `gateway::studio::analysis::markdown::metadata` consumes the same parser
   helper surface directly rather than importing parsing from `zhenfa_router`
4. this surface stays local to Wendao for now because the grammar is still a
   Wendao docs-governance policy surface rather than a proven cross-crate
   parser contract

## Addressed Target and Reference Boundary

Markdown ordinary body links now split across five explicit contracts:

1. `xiuxian_wendao_parsers::AddressedTarget` owns one reusable parser-owned
   `target + target_address` contract for cross-format structural link
   coordinates
2. `xiuxian_wendao_parsers::LiteralAddressedTarget` owns one reusable
   parser-owned `AddressedTarget + original literal` contract for
   source-preserved link items
3. `xiuxian_wendao_parsers::ReferenceCore<Kind>` owns one reusable
   parser-owned `kind + LiteralAddressedTarget` contract for
   source-preserved reference items that still carry one format-local kind tag
4. `xiuxian_wendao_parsers::references::MarkdownReference` is the
   Markdown-local alias over `ReferenceCore<MarkdownReferenceKind>`
5. `xiuxian_wendao_parsers::wikilinks::MarkdownWikiLink` is the
   Markdown-local naming surface over `LiteralAddressedTarget`
6. Wendao consumers such as `link_graph_refs` and `skill_runtime` reduce this
   parser-owned core into their own domain-specific adapters
7. Wendao-owned relation targets still use `Address` and are not part of this
   parser-owned addressed-target and reference contract

## Target Occurrence Boundary

Markdown note-level target capture is now split across two explicit contracts:

1. `xiuxian_wendao_parsers::targets::TargetOccurrenceCore<Kind>` owns the
   shared parser-visible `kind + target + source ranges` occurrence payload
   reusable across formats
2. `xiuxian_wendao_parsers::targets::MarkdownTargetOccurrence` is the
   Markdown-local naming surface over that shared core
3. `xiuxian_wendao_parsers::targets::extract_targets` is the shared
   parser-owned entry point for note-level target capture
4. `xiuxian_wendao::parsers::markdown::extract_link_targets_from_occurrences`
   is the Wendao adapter that applies workspace-aware normalization and
   attachment classification
5. Wendao section enrichment now filters note-level parser occurrences by
   section byte range before normalization, instead of re-running a second
   Markdown syntax pass per section
6. embedded wikilinks remain ignored on the current comrak-backed target path,
   matching the existing Wendao note-level behavior

## Parsing Strategy

Parser implementations should prefer structural signals over loose text
matching:

1. explicit fields, structured delimiters, and graph-visible links come first
2. ordinary wiki links create graph topology first; semantic upgrades come
   later and only from explicit metadata owners
3. Obsidian-style wiki-link fragments such as `#Heading` or `#^block-id`
   should be treated as real target addresses, not semantic type suffixes
4. file suffix or owned path conventions may classify resources such as
   attachments without introducing link-token string matches
5. heuristic or path-based fallbacks should stay bounded and local
6. keyword-only matching should not become the primary contract when a
   structural signal already exists

## Structural Relation Rule

When Wendao parses `[[...]]` links across the workspace, the first parser job
is to establish graph connectivity:

1. outbound wiki links define structural edges
2. reverse edges or backlinks are graph facts derived from the same link set
3. plain link text does not automatically become a semantic relation label

This means a link such as `[[notes/design]]` or `[[assets/logo.png]]` is first
handled as graph structure. If Wendao later needs to know that a target is an
attachment, that classification should come from an explicit structural signal
such as the file suffix, not from a special relation index note or a
hardcoded link label.

For ordinary body links, Wendao follows one parser-owned Markdown reference
contract:

1. `[label](note/path.md)` means a Markdown reference target
2. `[label](note/path.md#Heading)` means a Markdown reference plus structural
   address
3. `[label](#Local Heading)` means a local same-note structural address
4. `[[note]]` means a wiki-link note target
5. `[[note#Heading]]` means a wiki-link note plus heading target
6. `[[note#^block-id]]` means a wiki-link note plus block target
7. `[[#Local Heading]]` means a local same-note structural address

These address fragments are structural coordinates, not semantic type tags.

The canonical implementation for ordinary Markdown references now lives in
`xiuxian-wendao-parsers` and uses comrak AST parsing plus source-span
reconstruction, so ordinary Markdown reference parsing is not owned by
consumer-local scanners or by the Wendao domain crate itself.

The narrower wikilink-only subset is also exposed from
`xiuxian-wendao-parsers` for consumers that only care about ordinary
Obsidian-style topology links, while `xiuxian-wendao` keeps compatibility
re-exports for existing internal consumers.

Typed relation semantics belong to explicit metadata surfaces, such as
property drawers or subsystem-owned metadata, not to hardcoded string matches
inside parser helpers.

## Property Drawer Scope Rule

Property drawers are the explicit metadata surface for section-scoped relation
semantics.

This means Wendao distinguishes three different parser contracts:

1. ordinary global `[[...]]` links in note content:
   topology, backlinks, and structural adjacency
2. property-drawer relation values:
   explicit typed relations scoped to the owning heading or section
3. property-drawer scalar values:
   local metadata such as limits, weights, policy tags, or scope markers that
   do not create graph edges by default

Inside a property drawer, Wendao uses an explicit target grammar so a value
such as `[[file-b#section-2]]` means a scoped relation target rather than the
ordinary body-link interpretation of `#...`.

Stable cross-document section relations should prefer explicit `:ID:` anchors.
Path- and hash-scoped targets are still preserved by the parser, but the
current graph adapter only resolves the safe subset that can be mapped without
guessing.

The shared property-drawer and logbook extraction now live in
`xiuxian-wendao-parsers`, so Wendao relation and indexing flows consume one
parser-owned section metadata contract before adding domain semantics.

See [relation_semantics.md](relation_semantics.md) for the detailed contract.

## Persistence Rule

Graph persistence parsers may decode exact internal enum tokens written by
Wendao itself, but they must not reinterpret arbitrary wiki-link-shaped
strings as known semantic relation types. Unknown labels are preserved rather
than promoted.

:RELATIONS:
:LINKS: [[02_parser/index|Wendao Parser Docs]], [[02_parser/addressed_target|Parser Addressed Target]], [[02_parser/document|Parser Document Metadata]], [[02_parser/note|Parser Note Aggregate]], [[02_parser/targets|Parser Target Occurrences]], [[02_parser/sections|Parser Sections]], [[02_parser/references|Parser References]], [[02_parser/wikilinks|Parser Wikilinks]], [[02_parser/relation_semantics|Parser Relation Semantics]], [[01_core/103_package_layering|Wendao Package Layering]], [[03_features/210_search_queries_architecture|Search Queries Architecture]], [[06_roadmap/405_large_rust_modularization|Large Rust File Modularization]], [[06_roadmap/419_parser_substrate_separation|Parser Substrate Separation]]
:END:

---

:FOOTER:
:LAST_SYNC: 2026-04-12
:END:
