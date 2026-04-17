# Parser Blocks

:PROPERTIES:
:ID: wendao-parser-blocks
:PARENT: [[02_parser/index|Wendao Parser Docs]]
:TAGS: parser, blocks, markdown
:STATUS: ACTIVE
:END:

## Objective

Wendao now treats Markdown block extraction as a parser-owned shared surface in
`xiuxian-wendao-parsers`: one reusable `BlockCore<Kind>` owns the parser-side
block payload, `MarkdownBlockKind` owns the Markdown-local block variants, and
`MarkdownBlock` is now the Markdown-local naming surface over that shared
contract. `xiuxian-wendao` keeps page-index addressing and block-path matching
as domain-owned consumers.

## Contract

The canonical parser-owned block contract now splits into one shared core plus
one Markdown-local kind and naming surface:

1. `BlockCore<Kind>` preserves block identity, byte and line ranges, content
   hash, raw content, optional explicit ID, and parent structural path
2. `MarkdownBlockKind` preserves the current Markdown block variants:
   paragraph, code fence, list, blockquote, thematic break, table, and HTML
   block
3. `MarkdownBlock` is the Markdown naming surface over
   `BlockCore<MarkdownBlockKind>`
4. `extract_blocks` is the parser-owned entry point for top-level block
   extraction from one Markdown section body

This contract is parser-owned and syntax-facing. It does not include block-path
grammar, section-address lookup, or page-index semantic routing.

## Extraction Rules

The shared extractor follows these rules:

1. `extract_blocks` runs over one section body after section boundaries are
   already decided
2. only top-level block nodes become block records; inline nodes remain inside
   parent block content
3. heading nodes stay owned by section extraction, not block extraction
4. empty or whitespace-only block content is ignored
5. byte and line ranges are offset back into document-relative coordinates

## Consumer Boundary

`xiuxian-wendao` now consumes this parser-owned block contract:

1. `parsers::markdown::blocks` is now only a bounded compatibility re-export
   over `xiuxian-wendao-parsers`
2. page-index building consumes parser-owned Markdown blocks directly
3. `BlockAddress` and `BlockKindSpecifier` remain Wendao-owned because they are
   semantic addressing grammar, not Markdown parsing grammar
4. block-to-address matching remains a Wendao-owned helper layered on top of
   parser-owned block payloads

## Regression Coverage

Coverage for this contract lives in:

1. `packages/rust/crates/xiuxian-wendao-parsers/tests/unit/blocks.rs`
2. `tests/unit/parsers/markdown/blocks.rs`
3. `tests/unit/link_graph/models/records/markdown_block.rs`
4. `tests/integration/repo_projected_page_index_tree.rs`

:RELATIONS:
:LINKS: [[02_parser/index|Wendao Parser Docs]], [[02_parser/architecture|Parser Architecture]], [[02_parser/sections|Parser Sections]], [[06_roadmap/419_parser_substrate_separation|Parser Substrate Separation]]
:END:

---

:FOOTER:
:LAST_SYNC: 2026-04-11
:END:
