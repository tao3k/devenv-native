# Parser Sections

:PROPERTIES:
:ID: wendao-parser-sections
:PARENT: [[02_parser/index|Wendao Parser Docs]]
:TAGS: parser, sections, markdown
:STATUS: ACTIVE
:END:

## Objective

Wendao now treats Markdown and Org section structure as parser-owned shared surfaces
in `xiuxian-wendao-parsers`: one reusable `SectionCore` owns full parser-side
section payload, one nested `SectionMetadata` owns parser-side metadata, and
one nested `SectionScope` owns heading ancestry and source ranges.
`MarkdownSection` and `OrgSection` are now format-local naming surfaces over
that shared section core. `xiuxian-wendao` keeps only the enriched
`ParsedSection` adapter that adds domain-side entities and code observations.
Markdown heading discovery and section-boundary construction come from
parser-owned `comrak` traversal instead of line-scanned `#` detection; Org
heading discovery and property drawers come from `orgize`.

## Contract

The canonical parser-owned section contract now splits into one shared core,
one nested metadata contract, one nested structural contract, and one
compatibility naming surface:

1. `SectionCore` preserves normalized section text plus a lower-cased search
   helper around one nested `SectionScope` and one nested `SectionMetadata`
2. `SectionMetadata` preserves property-drawer attributes extracted from the
   owning heading section
3. `SectionMetadata` preserves `:LOGBOOK:` entries with timestamp, message, and
   1-based line number
4. `SectionScope` preserves heading title, ancestry path, lower-cased
   ancestry, heading depth, and line and byte ranges within the Markdown body
5. `MarkdownSection` is a compatibility alias over `SectionCore`, so current
   Markdown consumers keep one stable naming surface while the shared parser
   contract becomes format-neutral
6. `OrgSection` is the Org-local alias over `SectionCore`

When a document has no headings, or has leading body text before the first
heading, the parser may emit a level-0 root section so structure is preserved
without inventing a fake heading.

## Extraction Rules

The shared extractor follows these rules:

1. parser-owned `comrak` heading nodes establish section boundaries outside
   code fences
2. fenced code blocks do not open or close sections even when they contain `#`
3. section line and byte ranges come from heading source positions plus the
   next parser-owned heading boundary
4. property drawers are parsed from the section body owned by one heading
5. `:LOGBOOK:` blocks are parsed from that same section scope
6. Org-style `:PROPERTIES:` ... `:END:` blocks are preserved as a supported
   metadata shape inside Markdown sections
7. Org sections use native Org headlines and native property drawers, including
   drawers attached directly below a headline

## Consumer Boundary

`xiuxian-wendao` now consumes this parser-owned section contract rather than
owning the structural grammar itself:

1. `ParsedSection` is now an enriched adapter over the shared parser-owned
   section contract, and it consumes `SectionCore.scope` plus
   `SectionMetadata` for the shared heading-range and metadata contracts
2. note-link entity extraction still happens in Wendao because it requires
   workspace-aware link reduction, but it now filters parser-owned
   note-level target occurrences by `SectionScope` byte range instead of
   re-parsing each section body
3. `CodeObservation` enrichment still happens in Wendao because it is a
   domain-specific projection over property attributes
4. property-relation parsing can now operate over parser-owned section scope
   and parser-owned section metadata without depending on the enriched adapter
5. `.org` files reuse the same enriched `ParsedSection` adapter after the
   parser-owned Org section pass

## Regression Coverage

Coverage for this contract lives in:

1. `packages/rust/crates/xiuxian-wendao-parsers/tests/unit/sections.rs`
2. `packages/rust/crates/xiuxian-wendao-parsers/tests/unit/org.rs`
3. `tests/unit/parsers/markdown/sections.rs`
4. `tests/unit/parsers/markdown/relations.rs`
5. `tests/unit/link_graph/index/build/property_drawer_edges.rs`

:RELATIONS:
:LINKS: [[02_parser/index|Wendao Parser Docs]], [[02_parser/architecture|Parser Architecture]], [[02_parser/relation_semantics|Parser Relation Semantics]], [[06_roadmap/419_parser_substrate_separation|Parser Substrate Separation]]
:END:

---

:FOOTER:
:LAST_SYNC: 2026-04-30
:END:
