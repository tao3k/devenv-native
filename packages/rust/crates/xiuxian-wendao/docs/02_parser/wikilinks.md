# Parser Wikilinks

:PROPERTIES:
:ID: wendao-parser-wikilinks
:PARENT: [[02_parser/index|Wendao Parser Docs]]
:TAGS: parser, wikilinks, markdown
:STATUS: ACTIVE
:END:

## Objective

Wendao now treats ordinary body wikilink parsing as the narrower
Obsidian-style subset of the shared parser-owned reference surface in
`xiuxian-wendao-parsers`, backed by the same parser-owned addressed-target
core that ordinary Markdown references use.

## Syntax Contract

The canonical parser preserves ordinary body wikilinks in these shapes:

1. `[[note]]` and `[[note.md]]`
2. `[[note#Heading]]` and nested addressed forms such as `[[note#Heading#Subheading]]`
3. `[[note#^block-id]]`
4. `[[#Local Heading]]` and `[[#^local-block-id]]`
5. `[[note|Alias]]`, including note targets that contain spaces such as
   `[[Three laws of motion|Overview]]`
6. `[[note#Heading|Display Name]]`
7. `![[note]]`, `![[note#Heading]]`, and `![[note#^block-id]]` remain valid Obsidian embed syntax, but they are outside the ordinary body-link extraction surface documented here

The parser treats `#...` as a structural address, never as a semantic type
suffix.

## Repository Authoring Rule

Repository Markdown is intentionally stricter than the parser compatibility
surface. For authored docs, prefer:

1. `[[target|label]]`
2. `[label](target)`

Bare `[[target]]` forms remain parser-compatible for interoperability, but the
repository Markdown linter treats them as authoring violations so human and
LLM-facing docs always carry explicit display text.

The same split now applies to diagnostics:

1. official Obsidian-incompatible shapes such as `[[target]](label)` are
   reported as syntax failures
2. official Obsidian-compatible but repository-discouraged shapes such as
   bare `[[target]]`, redundant `[[target|target]]`, or target-like reversed
   aliases with an explicit path or address on the right side are reported as
   repo authoring policy findings
3. repository-authoring wikilink findings are now derived from the same
   parser-owned ordinary body-link traversal used by `references` and
   `wikilinks`, while the invalid mixed `[[...]](...)` shape stays on one
   lightweight surface scan because it never becomes a valid ordinary node

## Extraction Rules

The implementation is comrak-backed and derived from the shared reference
parser, not regex-driven:

1. the shared reference parser walks Markdown links and wikilinks in one
   parser-owned traversal
2. the wikilink surface filters that shared output down to ordinary
   `[[...]]` references only
3. source spans are converted back into exact byte slices so the parser keeps
   the original literal, including aliases
4. ordinary body wikilinks are returned in document order
5. embedded forms such as `![[note]]` are excluded from this ordinary
   body-link surface

This gives Wendao one parser-owned structural interpretation for body links
before any consumer-specific reduction happens.

`MarkdownWikiLink` is now the Markdown-local naming surface for
`LiteralAddressedTarget`. That means the note target plus optional scoped
address come from the shared `AddressedTarget` contract, while the original
literal comes from the shared source-preserved literal wrapper.

## Consumer Boundary

`link_graph_refs` is now a consumer over this parser surface:

1. it filters out local-only body addresses because `LinkGraphEntityRef`
   requires a cross-note target name
2. it keeps its historical deduplication behavior for `LinkGraph` consumers
3. it no longer owns its own regex-based wikilink grammar

`docs_governance` also consumes this parser surface for ordinary `:LINKS:`
and index-body wikilink collection:

1. relation-line and index-body checks now reduce canonical parser output
   instead of re-owning a local wikilink scanner
2. hidden-path governance still keeps its own line/offset helper because that
   adapter-local contract needs byte ranges rather than just wikilink targets

`skill_runtime::manifest::authority` no longer consumes this narrower
surface directly. It now consumes the shared `references` parser so `SKILL.md`
ordinary Markdown links and ordinary wikilinks follow one parser-owned
contract.

## Semantic Boundary

Ordinary body wikilinks only establish structure:

1. note-to-note topology
2. note-to-heading or note-to-block addressing
3. local address visibility

Typed semantics still belong to explicit metadata owners such as property
drawers, section tags, or other subsystem-owned metadata.

## Regression Coverage

Coverage for this contract lives in:

1. `packages/rust/crates/xiuxian-wendao-parsers/tests/unit/wikilinks.rs`
2. `tests/unit/parsers/markdown/wikilinks.rs`
3. `tests/snapshots/parser/markdown/wikilinks.json`
4. `tests/unit/link_graph_refs.rs`
5. `src/zhenfa_router/native/semantic_check/docs_governance/tests/index_links/relations.rs`

:RELATIONS:
:LINKS: [[02_parser/index|Wendao Parser Docs]], [[02_parser/addressed_target|Parser Addressed Target]], [[02_parser/references|Parser References]], [[02_parser/architecture|Parser Architecture]], [[02_parser/relation_semantics|Parser Relation Semantics]], [[01_core/103_package_layering|Wendao Package Layering]]
:END:

---

:FOOTER:
:LAST_SYNC: 2026-04-17
:END:
