# Parser Section Create Surface

:PROPERTIES:
:ID: wendao-parser-section-create
:PARENT: [[02_parser/index|Wendao Parser Docs]]
:TAGS: parser, markdown, semantic-edit
:STATUS: ACTIVE
:END:

## Objective

Wendao now treats Markdown section-create insertion planning and heading-chain
rendering as a parser-owned shared surface in `xiuxian-wendao-parsers`, while
`semantic_edit` remains the consumer that applies file mutations and formats
tool responses.

## Contract

The canonical parser-owned section-create surface now splits into four layers:

1. `InsertionInfo`
   - parser-owned insertion-byte and start-level contract
   - keeps only remaining heading path plus bounded sibling context
2. `BuildSectionOptions`
   - parser-owned rendering options for optional `:ID:` drawers
   - caller-owned `id_prefix` remains plain data
3. `find_insertion_point` and `parse_heading_line`
   - parser-owned Markdown heading traversal and heading-line parsing
4. `build_new_sections_content_with_options` and `compute_content_hash`
   - parser-owned heading-chain rendering and content hashing

None of these contracts read or write files, resolve Wendao addresses, or
format native-tool XML payloads.

## Consumer Boundary

`xiuxian-wendao` now consumes this parser-owned surface in two layers:

1. `parsers::markdown::section_create` is an adapter-only re-export so local
   Wendao code can keep parser access under the crate parser namespace
2. `zhenfa_router::native::semantic_edit` consumes the parser-owned helper
   surface for `create_if_missing`, while keeping address resolution, byte-range
   replacement, optimistic hash verification, XML rendering, and file writes
   local

That means `zhenfa_router` no longer owns Markdown heading parsing or section
creation planning.

## Regression Coverage

Coverage for this contract lives in:

1. `packages/rust/crates/xiuxian-wendao-parsers/tests/unit/section_create.rs`
2. `packages/rust/crates/xiuxian-wendao/tests/unit/zhenfa_router/native/semantic_edit.rs`

:RELATIONS:
:LINKS: [[02_parser/index|Wendao Parser Docs]], [[02_parser/architecture|Parser Architecture]], [[02_parser/sections|Parser Sections]], [[02_parser/toc|Parser TOC Surface]], [[06_roadmap/419_parser_substrate_separation|Parser Substrate Separation]]
:END:

---

:FOOTER:
:LAST_SYNC: 2026-04-14
:END:
