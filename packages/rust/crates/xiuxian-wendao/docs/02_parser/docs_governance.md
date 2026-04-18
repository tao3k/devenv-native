# Parser Docs Governance Surface

:PROPERTIES:
:ID: wendao-parser-docs-governance
:PARENT: [[02_parser/index|Wendao Parser Docs]]
:TAGS: parser, docs-governance, semantic-check
:STATUS: ACTIVE
:END:

## Objective

Wendao now treats docs-governance line/path parsing as a parser-owned local
surface under `src/parsers/docs_governance/`, while `zhenfa_router` keeps
semantic-check policy, issue shaping, rendering, and tool wiring.

## Contract

The canonical docs-governance parser contracts now split into three layers:

1. path identity helpers
   - `derive_opaque_doc_id`
   - `is_opaque_doc_id`
   - `is_package_local_crate_doc`
   - `is_canonical_repo_doc`
2. line/block parsing helpers
   - `collect_lines`
   - `parse_top_properties_drawer`
   - `parse_relations_links_line`
   - `parse_footer_block`
   - `collect_index_body_links`
3. link extraction helpers
   - `extract_wikilinks`
   - `extract_hidden_path_links`

These contracts do not own semantic-check issue types, fix planning,
workspace-scope traversal policy, or native tool registration.

## Consumer Boundary

`xiuxian-wendao` now consumes this parser-owned docs-governance surface in
three ways:

1. `zhenfa_router::native::semantic_check::docs_governance` consumes it for
   issue collection and rendering, but no longer owns parsing
2. `gateway::studio::analysis::markdown::metadata` consumes the same parser
   helper surface directly for relations-line metadata extraction instead of
   importing from `zhenfa_router`
3. parser-owned unit coverage lives under `tests/unit/parsers/`, while
   semantic-check behavior coverage stays under the `zhenfa_router` test tree

## Why It Stays In Wendao For Now

This surface is parser-owned, but it is still Wendao-local rather than a
cross-crate parser-crate export because it currently parses Wendao docs
governance grammar and hidden-path policy conventions that are not yet proven
reusable outside Wendao.

## Regression Coverage

Coverage for this contract lives in:

1. `packages/rust/crates/xiuxian-wendao/tests/unit/parsers/docs_governance.rs`
2. `packages/rust/crates/xiuxian-wendao/tests/unit/gateway/studio/analysis.rs`
3. `packages/rust/crates/xiuxian-wendao/tests/unit/zhenfa_router/native/semantic_check/docs_governance/`

:RELATIONS:
:LINKS: [[02_parser/index|Wendao Parser Docs]], [[02_parser/architecture|Parser Architecture]], [[02_parser/toc|Parser TOC Surface]], [[03_features/203_agentic_navigation|Agentic Navigation (wendao.agentic_nav)]], [[06_roadmap/419_parser_substrate_separation|Parser Substrate Separation]]
:END:

---

:FOOTER:
:LAST_SYNC: 2026-04-13
:END:
