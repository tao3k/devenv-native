# Parser Relation Semantics

:PROPERTIES:
:ID: wendao-parser-relation-semantics
:PARENT: [[02_parser/index|Wendao Parser Docs]]
:TAGS: parser, relations, properties
:STATUS: ACTIVE
:END:

## Objective

Wendao distinguishes structural workspace links from explicit scoped relation
metadata.

The parser contract is:

1. ordinary global `[[...]]` links define graph topology and backlink facts
2. property-drawer relation fields define explicit semantic relations scoped to
   the owning section
3. numeric or scalar property values remain local metadata unless another
   subsystem explicitly interprets them

## Global Wiki Links

Ordinary `[[...]]` links are workspace-visible structural edges.

Examples:

- `[[file-b]]`
- `[[notes/design]]`
- `[[notes/design#Implementation]]`
- `[[notes/design#^parser-block]]`
- `[[#Implementation]]`
- `[[assets/logo.png]]`

These links answer questions such as:

1. which notes link to which other notes
2. which documents or resources are attached or referenced structurally
3. which backlinks or neighborhood edges exist in the graph

They do not, by themselves, declare a typed semantic relation.

When a body wiki link includes `#...`, Wendao follows the Obsidian address
rule:

1. `#Heading` means a real heading target
2. `#^block-id` means a real block target
3. `[[#Heading]]` is a local same-note address, not a typed relation
4. `#...` is not a semantic type suffix

## Property Drawer Relations

Property drawers are the explicit metadata surface for scoped semantic
relations.

Examples:

```markdown
## Heading 1

:PROPERTIES:
:ID: heading-1
:RELATED: [[file-b#section-2]]
:DEPENDS_ON: [[#local-anchor]]
:SEE_ALSO: [[/Appendix]]
:WEIGHT: 5
:END:
```

In this shape:

1. `Heading 1` is the owning source scope
2. `:RELATED:`, `:DEPENDS_ON:`, `:EXTENDS:`, and `:SEE_ALSO:` are explicit
   relation owners
3. `:WEIGHT: 5` is metadata, not a graph edge

This lets Wendao distinguish:

1. broad note-to-note topology from ordinary `[[...]]`
2. section-scoped explicit relations declared by `PROPERTIES`
3. local property values that affect scope, ranking, or policy without adding
   a graph edge

## Target Grammar Inside Property Drawers

The property-drawer relation parser accepts an explicit target grammar:

1. local or same-note address targets:
   - `[[#anchor-id]]`
   - `[[@content-hash]]`
   - `[[/Heading/Path]]`
2. cross-document wiki-link targets:
   - `[[file-b]]`
   - `[[file-b#section-2]]`
   - `[[file-b#/Heading/Path]]`
   - `[[file-b@content-hash]]`

This grammar is intentionally distinct from ordinary wiki-link parsing in the
document body. Inside a property drawer, `[[file-b#section-2]]` means an
explicit relation target with note `file-b` plus scoped target address
`#section-2`, not an entity-type hint.

## Scope Semantics

The source scope of a property-drawer relation is the owning section.

Resolution priority is:

1. explicit `:ID:` on the owning heading
2. structural heading path if no explicit `:ID:` exists

This is what allows Wendao to model shapes such as:

1. `fileA.heading1 related to fileB.heading2`
2. `fileA.heading1 depends on fileA.heading3`
3. `fileA.heading1 see-also fileB appendix section`

## Current Resolution Boundary

The canonical parser already preserves path, hash, and explicit-ID targets.
The current link-graph adapter safely resolves:

1. document targets such as `[[file-b]]`
2. explicit-ID targets such as `[[file-b#section-2]]` or `[[#local-anchor]]`

Path- or hash-scoped targets still remain visible at parser and enhancer level
even when the current graph builder cannot yet resolve them into a stable graph
node without a later page-index-aware pass.

## Design Rule

Wendao intentionally differs from systems that let all `[[...]]` links and
scoped relations collapse into one implicit behavior surface.

The rule is:

1. global wiki links are topology
2. property-drawer relation fields are explicit scoped semantics
3. property scalars are metadata, not edges

## Regression Coverage

The regression contract for this distinction is snapshot-backed.

Current coverage lives in:

1. [`wikilinks.md`](wikilinks.md), which defines the parser-owned ordinary
   body-wikilink extraction contract
2. [`../../tests/unit/parsers/markdown/wikilinks.rs`](../../tests/unit/parsers/markdown/wikilinks.rs),
   which binds parser-owned unit and snapshot coverage to the ordinary
   body-wikilink grammar
3. [`../../tests/snapshots/parser/markdown/wikilinks.json`](../../tests/snapshots/parser/markdown/wikilinks.json),
   which records alias-preserving note targets, addressed note targets, and
   local same-note addresses while excluding embedded `![[...]]`
4. [`../../src/enhancer/tests.rs`](../../src/enhancer/tests.rs), which binds
   a unit test to the parser-enhancement relation payload
5. [`../../tests/snapshots/parser/markdown/reference_relations.json`](../../tests/snapshots/parser/markdown/reference_relations.json),
   which records:
   structural body-link relations with `null` semantic metadata, plus
   property-drawer-scoped relations with `source_address`,
   `target_address`, `relation_type`, and `metadata_owner`

:RELATIONS:
:LINKS: [[02_parser/index|Wendao Parser Docs]], [[02_parser/wikilinks|Parser Wikilinks]], [[02_parser/architecture|Parser Architecture]], [[03_features/201_property_drawers|Property Drawers]], [[01_core/103_package_layering|Wendao Package Layering]]
:END:

---

:FOOTER:
:LAST_SYNC: 2026-04-05
:END:
