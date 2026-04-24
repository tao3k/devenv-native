# Parser References

:PROPERTIES:
:ID: wendao-parser-references
:PARENT: [[02_parser/index|Wendao Parser Docs]]
:TAGS: parser, references, markdown
:STATUS: ACTIVE
:END:

## Objective

Wendao now treats ordinary Markdown reference parsing as a parser-owned shared
surface in `xiuxian-wendao-parsers` so `SKILL.md` and ordinary Markdown
documents do not diverge into different link grammars, and so ordinary
Markdown links can reuse one parser-owned `ReferenceCore<Kind>` above the
shared addressed-target cores.

## Syntax Contract

The canonical parser preserves ordinary Markdown references in these shapes:

1. `[label](note/path.md)`
2. `[label](note/path.md#Heading)`
3. `[label](#Local Heading)`
4. `[[note]]`
5. `[[note#Heading]]`
6. `[[#Local Heading]]`

The parser separates an optional cross-document target from an optional
structural address for both syntaxes.

That split now lives across these parser-owned contracts:

1. `AddressedTarget` keeps one shared `target + target_address` payload
2. `LiteralAddressedTarget` keeps one shared `AddressedTarget + original
literal` payload
3. `ReferenceCore<Kind>` keeps one shared `kind + LiteralAddressedTarget`
   payload
4. `MarkdownReference` is the Markdown-local alias over
   `ReferenceCore<MarkdownReferenceKind>`

## Extraction Rules

The implementation is comrak-backed and parser-owned:

1. comrak parses ordinary Markdown links and wikilinks in one traversal
2. source spans are converted back into exact byte slices so the parser keeps
   the original literal
3. references are returned in document order
4. images such as `![label](asset.png)` are excluded
5. embedded wikilinks such as `![[note]]` are excluded

## Consumer Boundary

`skill_runtime::manifest::authority` consumes this parser surface:

1. `SKILL.md` manifest intents now use the same parser-owned Markdown
   reference contract as ordinary Markdown documents
2. `extract_references` returns the Markdown-local naming surface over the
   shared `ReferenceCore<MarkdownReferenceKind>`
3. the authority consumer reduces parser output into local manifest-path
   normalization and URI authority checks
4. `SKILL.md` no longer owns a local split between Markdown-link scanning and
   wikilink parsing

The narrower `wikilinks.md` surface documents the Obsidian-only subset used by
consumers that only care about ordinary `[[...]]` topology links.

## Regression Coverage

Coverage for this contract lives in:

1. `packages/rust/crates/xiuxian-wendao-parsers/tests/unit/references.rs`
2. `tests/unit/parsers/markdown/references.rs`
3. `tests/snapshots/parser/markdown/references.json`
4. `src/skill_runtime/manifest/tests.rs`

:RELATIONS:
:LINKS: [[02_parser/index|Wendao Parser Docs]], [[02_parser/architecture|Parser Architecture]], [[02_parser/addressed_target|Parser Addressed Target]], [[02_parser/wikilinks|Parser Wikilinks]], [[01_core/103_package_layering|Wendao Package Layering]]
:END:

---

:FOOTER:
:LAST_SYNC: 2026-04-11
:END:
