# Wendao Parser Docs

:PROPERTIES:
:ID: wendao-parser-docs
:PARENT: [[index|Wendao Parser Docs]]
:TAGS: parser, architecture, core
:STATUS: ACTIVE
:END:

This directory is the stable documentation home for parser ownership,
canonical parser families, and parser implementation rules in
`xiuxian-wendao`, including the package boundary between Wendao-owned parser
adapters and the independent `xiuxian-wendao-parsers` crate.

Org remains a documented placeholder boundary only. This directory records the
shared parser contracts that a future Org slice may reuse, but it does not
claim that an Org parser implementation is active today.

## Documents

- [architecture.md](architecture.md): Canonical parser namespace, parser-family
  matrix, parser-vs-helper decision rule, and structural parsing principles.
- [addressed_target.md](addressed_target.md): Parser-owned shared
  `target + target_address` contract, the shared source-preserved
  literal wrapper, and the shared reference-core payload reused by Markdown
  references and ordinary wikilinks.
- [blocks.md](blocks.md): Parser-owned shared block core, Markdown block
  naming surface, parser-owned block extraction, and the Wendao page-index
  addressing boundary.
- [code_observation.md](code_observation.md): Parser-owned Markdown
  `:OBSERVE:` parsing, extraction, scope matching, and the Wendao semantic
  consumption boundary.
- [docs_governance.md](docs_governance.md): Parser-owned docs-governance
  line/path parsing, the semantic-check consumer boundary, and the rule that
  `zhenfa_router` does not own parsing.
- [semantic_check.md](semantic_check.md): Parser-owned semantic-check link,
  contract, and suggested-ID grammar helpers plus the rule that
  `zhenfa_router` only consumes this grammar.
- [document.md](document.md): Parser-owned cross-format document core,
  shared document-envelope wrapper, Markdown document naming surface, and
  the Wendao `LinkGraphDocument` adapter boundary.
- [note.md](note.md): Parser-owned Markdown note aggregation and the Wendao
  workspace-aware note adapter boundary, including the shared note-core shape
  and shared top-level note aggregate.
- [toc.md](toc.md): Parser-owned Markdown TOC/document-structure aggregation,
  the shared `TocDocument<Document, Section>` surface, and the Wendao
  projected page-index consumer boundary.
- [targets.md](targets.md): Parser-owned target-occurrence core, Markdown
  target-occurrence naming surface, parser-visible source ranges, and the
  Wendao target-normalization and section-partition adapter boundary.
- [sections.md](sections.md): Parser-owned Markdown section structure,
  shared full section core with nested section scope and section metadata,
  property-drawer extraction, logbook extraction, and the Wendao adapter
  boundary.
- [references.md](references.md): Unified ordinary Markdown reference grammar
  for `[...](...)` and `[[...]]`, plus the shared `ReferenceCore<Kind>`
  boundary and current parser-owned consumers.
- [section_create.md](section_create.md): Parser-owned Markdown section-create
  insertion planning, heading-chain rendering, and the `semantic_edit`
  consumer boundary.
- [wikilinks.md](wikilinks.md): Obsidian-aligned ordinary body wikilink
  grammar, comrak-backed extraction, and the `link_graph_refs` consumer
  boundary.
- [relation_semantics.md](relation_semantics.md): The difference between
  global `[[...]]` topology links, section-scoped property-drawer relations,
  local property scalar metadata, and the snapshot-backed regression contract.
- [../06_roadmap/419_parser_substrate_separation.md](../06_roadmap/419_parser_substrate_separation.md):
  Future-facing package split for shared syntax-only document parsing versus
  Wendao-owned domain adapters.

## Current Canonical Parser Families

- `packages/rust/crates/xiuxian-wendao-parsers/src/addressed_target.rs`
- `packages/rust/crates/xiuxian-wendao-parsers/src/blocks/`
- `packages/rust/crates/xiuxian-wendao-parsers/src/code_observation/`
- `packages/rust/crates/xiuxian-wendao-parsers/src/document/`
- `packages/rust/crates/xiuxian-wendao-parsers/src/frontmatter/`
- `packages/rust/crates/xiuxian-wendao-parsers/src/literal_addressed_target.rs`
- `packages/rust/crates/xiuxian-wendao-parsers/src/note/`
- `packages/rust/crates/xiuxian-wendao-parsers/src/reference_core.rs`
- `packages/rust/crates/xiuxian-wendao-parsers/src/references/`
- `packages/rust/crates/xiuxian-wendao-parsers/src/section_create/`
- `packages/rust/crates/xiuxian-wendao-parsers/src/sections/`
- `packages/rust/crates/xiuxian-wendao-parsers/src/toc/`
- `packages/rust/crates/xiuxian-wendao-parsers/src/targets/`
- `packages/rust/crates/xiuxian-wendao-parsers/src/wikilinks/`
- `packages/rust/crates/xiuxian-wendao-parsers/src/sourcepos.rs`
- `src/parsers/docs_governance/`
- `src/parsers/semantic_check/`
- `src/parsers/markdown/`
- `src/parsers/link_graph/query/`
- `src/parsers/zhixing/tasks/`
- `src/parsers/languages/rust/cargo/`
- `src/parsers/languages/python/pyproject/`
- `src/parsers/search/repo_code_query/`
- `src/parsers/graph/persistence/`

## What This Directory Governs

1. Which parser behavior belongs under `src/parsers/`
2. Which parse-like helpers stay adapter-local or subsystem-local
3. How parser modules should be split and tested
4. Why `[[...]]` links should establish graph topology before any semantic
   typing
5. Why explicit metadata and real structural signals should win over hardcoded
   link strings
6. How `PROPERTIES` scoped relations differ from global wiki links and scalar
   metadata
7. How ordinary Markdown reference parsing stays parser-owned and comrak-backed
8. How the narrower ordinary wikilink subset relates to the shared reference
   parser surface
9. How parser-owned shared addressed-target coordinates, the
   source-preserved literal-addressed-target wrapper, and the shared
   reference-core payload are separated from Markdown-specific wrappers and
   Wendao-owned relation targets
10. How parser-owned cross-format document metadata and the shared
    document-envelope wrapper are separated from the Markdown-specific naming
    surface and from `LinkGraphDocument` assembly
11. How parser-owned Markdown note aggregation is separated from the
    shared note-core shape, the shared top-level note aggregate, and the
    workspace-aware Wendao note adapter
12. How parser-owned shared full section core, nested section scope, and
    shared section metadata are separated from Wendao-side enrichments
13. How parser-owned shared target-occurrence core and Markdown
    target-occurrence naming surface are separated from Wendao-side target
    normalization and section-byte-range partitioning
14. How parser-owned shared block core and Markdown block naming surface are
    separated from Wendao-side block-path addressing
15. How shared syntax-only document parsing should be separated from
    Wendao-owned domain adapters for cross-crate consumers
16. How docs-governance parsing stays under `src/parsers/` while
    `zhenfa_router` only owns semantic-check policy and tool wiring
17. How semantic-check grammar helpers stay under `src/parsers/` while
    `zhenfa_router` only consumes the grammar for checks and testing
18. How Markdown section-create planning stays parser-owned while
    `semantic_edit` consumes it without `zhenfa_router` owning the helper
    surface

:RELATIONS:
:LINKS: [[01_core/103_package_layering|Wendao Package Layering]], [[06_roadmap/405_large_rust_modularization|Large Rust File Modularization]], [[06_roadmap/419_parser_substrate_separation|Parser Substrate Separation]], [[03_features/210_search_queries_architecture|Search Queries Architecture]], [[02_parser/addressed_target|Parser Addressed Target]], [[02_parser/blocks|Parser Blocks]], [[02_parser/code_observation|Parser Code Observation Surface]], [[02_parser/document|Parser Document Metadata]], [[02_parser/note|Parser Note Aggregate]], [[02_parser/toc|Parser TOC Surface]], [[02_parser/targets|Parser Target Occurrences]], [[02_parser/sections|Parser Sections]], [[02_parser/references|Parser References]], [[02_parser/section_create|Parser Section Create Surface]], [[02_parser/wikilinks|Parser Wikilinks]], [[02_parser/relation_semantics|Parser Relation Semantics]], [[02_parser/semantic_check|Parser Semantic Check Surface]]
:END:

---

:FOOTER:
:LAST_SYNC: 2026-04-14
:END:
