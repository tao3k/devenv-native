# Parser Addressed Target

:PROPERTIES:
:ID: wendao-parser-addressed-target
:PARENT: [[02_parser/index|Wendao Parser Docs]]
:TAGS: parser, links, target, markdown
:STATUS: ACTIVE
:END:

## Objective

Wendao now treats one cross-format addressed-target core as a parser-owned
shared surface in `xiuxian-wendao-parsers` so Markdown reference and wikilink
wrappers can share one neutral `target + target_address` contract before any
Wendao-specific reduction happens.

## Contract

The canonical parser-owned addressed-target contract is:

1. `AddressedTarget`
   - optional note or resource target without any address fragment
   - optional structural address inside the target note or the current note
2. `LiteralAddressedTarget`
   - one flattened `AddressedTarget`
   - original literal slice preserved from the source document
3. `ReferenceCore<Kind>`
   - one shared `kind + LiteralAddressedTarget` payload
   - reusable across link syntaxes that need one format-local kind tag

Markdown-specific wrappers now embed that shared contract:

1. `MarkdownReference`
   - Markdown-local alias over `ReferenceCore<MarkdownReferenceKind>`
2. `MarkdownWikiLink`
   - Markdown-local alias over `LiteralAddressedTarget`

`AddressedTarget` is the reusable cross-format structural target contract.
`LiteralAddressedTarget` is the reusable cross-format source-preserved
contract. `ReferenceCore<Kind>` is the reusable cross-format
source-preserved-plus-kind contract. The Markdown wrappers keep only the
syntax-specific naming or syntax-kind layer above those shared cores.

## Extraction Rules

The shared addressed-target extraction follows these rules for Markdown:

1. ordinary Markdown links and ordinary body wikilinks both normalize into the
   same parser-owned `AddressedTarget`
2. local-only addresses such as `#Heading` keep `target = None`
3. cross-note targets such as `note#Heading` keep `target = Some("note")`
   plus `target_address = Some("#Heading")`
4. the shared contract does not classify graph semantics or filesystem
   ownership

## Consumer Boundary

Current consumers over this contract are:

1. `references.md`, which keeps Markdown syntax kind and original literal on
   top of the shared reference core
2. `wikilinks.md`, which exposes the narrower ordinary-`[[...]]` subset as a
   Markdown-local naming surface over `LiteralAddressedTarget`
3. `link_graph_refs`, which reduces parser-owned addressed targets into
   `LinkGraphEntityRef` rows and keeps its own deduplication and local-only
   skip rules
4. `skill_runtime::manifest::authority`, which only consumes the
   cross-document `target` part when checking manifest intents

Wendao-owned relation-target parsing still uses a different domain contract
based on `Address`, so it is not part of this parser-owned addressed-target
surface.

## Regression Coverage

Coverage for this contract lives in:

1. `packages/rust/crates/xiuxian-wendao-parsers/tests/unit/references.rs`
2. `packages/rust/crates/xiuxian-wendao-parsers/tests/unit/wikilinks.rs`
3. `tests/unit/parsers/markdown/references.rs`
4. `tests/unit/parsers/markdown/wikilinks.rs`
5. `tests/unit/link_graph_refs.rs`
6. `packages/rust/crates/xiuxian-wendao-parsers/src/literal_addressed_target.rs`
7. `packages/rust/crates/xiuxian-wendao-parsers/src/reference_core.rs`

:RELATIONS:
:LINKS: [[02_parser/index|Wendao Parser Docs]], [[02_parser/architecture|Parser Architecture]], [[02_parser/references|Parser References]], [[02_parser/wikilinks|Parser Wikilinks]], [[06_roadmap/419_parser_substrate_separation|Parser Substrate Separation]]
:END:

---

:FOOTER:
:LAST_SYNC: 2026-04-11
:END:
