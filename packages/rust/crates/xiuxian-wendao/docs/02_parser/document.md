# Parser Document Metadata

:PROPERTIES:
:ID: wendao-parser-document
:PARENT: [[02_parser/index]]
:TAGS: parser, document, markdown
:STATUS: ACTIVE
:END:

## Objective

Wendao now treats one cross-format document core plus one Markdown-specific
document wrapper as parser-owned shared surfaces in
`xiuxian-wendao-parsers`, while `xiuxian-wendao` keeps only the domain adapter
that assembles `LinkGraphDocument`.

## Contract

The canonical parser-owned document contracts now split into three layers:

1. `DocumentCore`
   - document format family
   - format-normalized body with top-level metadata stripped
   - best-effort title
   - best-effort tags
   - optional semantic document type
   - best-effort leading content snippet
   - best-effort body word count
2. `DocumentEnvelope<RawMetadata>`
   - one optional raw metadata payload owned by the source format
   - one embedded `DocumentCore`
3. `MarkdownDocument`
   - Markdown-local alias over `DocumentEnvelope<serde_yaml::Value>`
   - preserves raw YAML frontmatter when the document starts with a valid
     frontmatter block

`DocumentCore` is the reusable cross-format metadata and body contract.
`DocumentEnvelope<RawMetadata>` is the reusable cross-format top-level wrapper
shape for `raw metadata + document core`. `MarkdownDocument` is the
Markdown-local naming surface that keeps raw YAML metadata available for
current Wendao adapters. None of these contracts include path identity,
filesystem timestamps, saliency defaults, or graph records.

## Extraction Rules

The shared extractor follows these rules for Markdown:

1. frontmatter is split before metadata extraction
2. title prefers frontmatter `title`, then the first parser-owned structural
   heading, then the caller-provided fallback
3. standalone document parsing, note parsing, and TOC parsing all assemble
   `MarkdownDocument` with the same parser-owned structural heading candidate,
   so heading-like text inside fenced code blocks does not leak into parser-owned
   fallback titles
4. tags follow the historical note-parser contract and only read top-level
   frontmatter `tags`
5. `type` and `kind` are normalized into one optional `doc_type`
6. lead is derived from the first parser-owned structural paragraph snippet
   rather than a line-based fallback, so fenced code content does not leak into
   parser-owned lead snippets
7. standalone document parsing uses a light parser-owned structural metadata
   scan for title and lead, while note and TOC parsing continue to consume the
   richer full-structure scan that also carries references, targets, and
   section-driving heading/task items
8. `DocumentCore.format` is set to `markdown`

## Consumer Boundary

`xiuxian-wendao` now consumes these parser-owned document contracts:

1. `parse_markdown_document` now assembles standalone Markdown document
   metadata from the same parser-owned structural heading and paragraph
   semantics already shared by note and TOC parsing, but through a lighter
   document-only metadata scan that avoids full reference/target/task
   collection
2. `parse_note` consumes `MarkdownDocument.core` through the parser-owned
   `MarkdownNote` aggregate for title, tags, doc type, lead, body, and word
   count
3. `parse_markdown_toc` and `parse_markdown_note` now assemble
   `MarkdownDocument` with fallback titles sourced from the same parser-owned
   structural heading scan that drives their section discovery
4. standalone document parsing plus those same hot paths now also assemble
   `MarkdownDocument.lead` from the parser-owned structural scan instead of a
   separate line-based fallback
5. Wendao still consumes `MarkdownDocument.raw_metadata` for saliency and
   timestamp adapters that are still Markdown-specific today
6. Wendao still owns `doc_id`, `path`, timestamps, saliency defaults, and
   `LinkGraphDocument` assembly
7. link extraction and section enrichment still happen in Wendao because they
   require workspace-aware and domain-aware adapters

## Regression Coverage

Coverage for this contract lives in:

1. `packages/rust/crates/xiuxian-wendao-parsers/tests/unit/document.rs`
2. `tests/unit/parsers/markdown/document.rs`
3. `tests/unit/parsers/markdown/namespace.rs`

:RELATIONS:
:LINKS: [[02_parser/index]], [[02_parser/architecture]], [[02_parser/note]], [[02_parser/sections]], [[06_roadmap/419_parser_substrate_separation]]
:END:

---

:FOOTER:
:LAST_SYNC: 2026-04-14
:END:
