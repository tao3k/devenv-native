# Parser Target Occurrences

:PROPERTIES:
:ID: wendao-parser-targets
:PARENT: [[02_parser/index|Wendao Parser Docs]]
:TAGS: parser, targets, markdown
:STATUS: ACTIVE
:END:

## Objective

Wendao now treats note-level Markdown target capture as a parser-owned shared
surface in `xiuxian-wendao-parsers`: one reusable `TargetOccurrenceCore<Kind>`
owns the ordered parser-visible target payload, and `MarkdownTargetOccurrence`
is now only the Markdown-local naming surface over that shared contract.
`xiuxian-wendao` keeps the workspace-aware adapter that normalizes those
targets into note links and attachment paths.

## Contract

The canonical parser-owned target-occurrence contract now splits into one
shared core plus one Markdown-local naming surface:

1. `TargetOccurrenceCore<Kind>` preserves one syntax discriminant plus one
   parser-visible target string
2. `TargetOccurrenceCore<Kind>` preserves document order for target
   occurrences through the surrounding note aggregate
3. `TargetOccurrenceCore<Kind>` preserves parser-visible source surface syntax
   for each occurrence, such as `[Guide](docs/guide.md)`, `![Logo](logo.png)`,
   `[[note|Alias]]`, or `![[note#Section]]`
4. `TargetOccurrenceCore<Kind>` preserves parser-visible occurrence byte and
   line ranges within the frontmatter-stripped document body
5. `MarkdownTargetOccurrence` is a compatibility alias over
   `TargetOccurrenceCore<MarkdownTargetOccurrenceKind>`
6. `MarkdownTargetOccurrenceKind` preserves the current Markdown surfaces:
   inline links, images, ordinary wikilinks, and wiki embeds

This contract is parser-owned and syntax-facing. It does not include path
resolution, note-vs-attachment normalization, or graph-edge semantics.

## Extraction Rules

The shared extractor follows these rules:

1. `extract_targets` runs over the frontmatter-stripped Markdown body
2. ordinary Markdown links and images are preserved as separate occurrence
   kinds
3. ordinary body wikilinks are preserved as parser-owned target occurrences
4. wiki embeds such as `![[note#Section]]` are preserved as target occurrences
   even though the ordinary body-wikilink surface still skips them
5. local address-only targets such as `#section` are preserved for the adapter
   to ignore or consume
6. occurrence ranges point at the parser-visible syntax occurrence, not only
   the bare target substring

## Consumer Boundary

`xiuxian-wendao` now consumes this parser-owned target contract:

1. `parse_note` normalizes parser-owned target occurrences instead of
   re-running note-level comrak scanning locally
2. workspace-aware normalization, attachment classification, and deduplication
   still happen in Wendao
3. section-level entity extraction now filters note-level parser occurrences by
   `SectionScope` byte range before applying Wendao-side normalization
4. Wendao no longer keeps a second section-local Markdown re-scan path for
   section entity extraction

## Regression Coverage

Coverage for this contract lives in:

1. `packages/rust/crates/xiuxian-wendao-parsers/tests/unit/targets.rs`
2. `packages/rust/crates/xiuxian-wendao-parsers/tests/unit/note.rs`
3. `tests/unit/parsers/markdown/document.rs`
4. `tests/unit/markdown_syntax_algorithm_fixtures.rs`

:RELATIONS:
:LINKS: [[02_parser/index|Wendao Parser Docs]], [[02_parser/architecture|Parser Architecture]], [[02_parser/note|Parser Note Aggregate]], [[06_roadmap/419_parser_substrate_separation|Parser Substrate Separation]]
:END:

---

:FOOTER:
:LAST_SYNC: 2026-04-18
:END:
