# Parser TOC Surface

:PROPERTIES:
:ID: wendao-parser-toc
:PARENT: [[02_parser/index]]
:TAGS: parser, toc, markdown
:STATUS: ACTIVE
:END:

## Objective

Wendao now treats Markdown table-of-contents parsing as a parser-owned shared
surface in `xiuxian-wendao-parsers`, while `xiuxian-wendao` keeps the
repository-aware adapter that turns parsed sections into repo-scoped page-index
documents and trees.

## Contract

The canonical parser-owned TOC contracts now split into three layers:

1. `TocDocument<Document, Section>`
   - one parser-owned format document wrapper
   - one parser-owned ordered section list
2. `MarkdownTocDocument`
   - Markdown-local alias over
     `TocDocument<MarkdownDocument, MarkdownSection>`
3. `parse_markdown_toc`
   - parser-owned entry point for Markdown document structure
   - returns parser-owned title/body metadata plus parser-owned section rows

None of these contracts include repo identifiers, filesystem-root-relative
`doc_id` derivation, attachment classification, entity enrichment, or
`LinkGraphDocument` assembly.

## Extraction Rules

The shared TOC aggregate follows these rules for Markdown:

1. `parse_markdown_toc` first splits frontmatter and parser-owned body content
2. the hot path then runs one parser-owned structural heading scan over that
   body and reuses it for section extraction plus fallback title/lead assembly
3. the resulting `MarkdownTocDocument` preserves parser-owned section scope,
   section metadata, and source ranges without Wendao repo semantics
4. parser-owned TOC parsing does not resolve workspace-relative identities or
   graph-facing relations
5. heading-like text inside fenced code blocks stays outside the structural
   TOC and does not leak into fallback document titles or leads because both
   metadata fields now follow parser-owned `comrak` structure

## Consumer Boundary

`xiuxian-wendao` now consumes this parser-owned TOC contract in two ways:

1. `parse_markdown_note` reuses `parse_markdown_toc` before adding references,
   targets, workspace-aware link normalization, and `LinkGraphDocument`
   assembly
2. projected page-index document parsing now consumes `parse_markdown_toc`
   directly, while Wendao keeps projected `doc_id` derivation and repo-scoped
   page-index projection local
3. `DocsToolService::get_toc_documents()` remains a repo-scoped capability
   opener over projected page-index documents, not the parsing owner
4. bounded-work markdown SQL row building now consumes `parse_markdown_toc`
   directly because it only needs parser-owned document titles, heading paths,
   attributes, and section text, while Wendao keeps query registration and SQL
   row shaping local
5. the Studio source-index markdown test helper now consumes parser-owned TOC
   structure directly and rebuilds observation hits locally from property
   drawer attributes, while the production `ParsedSection` contract stays
   repo-owned inside Wendao

## Regression Coverage

Coverage for this contract lives in:

1. `packages/rust/crates/xiuxian-wendao-parsers/tests/unit/toc.rs`
2. `packages/rust/crates/xiuxian-wendao-parsers/tests/unit/note.rs`
3. `packages/rust/crates/xiuxian-wendao/tests/integration/repo_projected_page_index_documents.rs`
4. `packages/rust/crates/xiuxian-wendao/tests/unit/search/queries/sql/bounded_work_markdown.rs`
5. `packages/rust/crates/xiuxian-wendao/tests/unit/gateway/studio/search.rs`

:RELATIONS:
:LINKS: [[02_parser/index]], [[02_parser/architecture]], [[02_parser/document]], [[02_parser/note]], [[02_parser/sections]], [[03_features/207_gateway_openapi_contract_surface]], [[06_roadmap/403_document_projection_and_retrieval_enhancement]], [[06_roadmap/419_parser_substrate_separation]]
:END:

---

:FOOTER:
:LAST_SYNC: 2026-04-14
:END:
