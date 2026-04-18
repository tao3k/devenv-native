# Parser Code Observation Surface

:PROPERTIES:
:ID: wendao-parser-code-observation
:PARENT: [[02_parser/index|Wendao Parser Docs]]
:TAGS: parser, observation, markdown
:STATUS: ACTIVE
:END:

## Objective

Wendao now treats Markdown `:OBSERVE:` parsing, extraction, and scope matching
as a parser-owned shared surface in `xiuxian-wendao-parsers`, while
`xiuxian-wendao` keeps semantic-check logic, page-index enrichment, and gateway
consumption as downstream adapters.

## Contract

The canonical parser-owned observation contracts now split into three layers:

1. `CodeObservation`
   - parser-owned parsed `:OBSERVE:` entry
   - includes language, pattern, optional scope, raw value, and validation
     state fields
2. `extract_observations`
   - parser-owned attribute scan over `OBSERVE` / `OBSERVE_n`
   - returns ordered observation entries from one attribute map
3. `path_matches_scope`
   - parser-owned glob matcher for observation scope filters
   - used by downstream adapters when they need to apply observation scope

These contracts do not own semantic-check issue policy, page-index records,
graph edges, or gateway DTOs.

## Consumer Boundary

`xiuxian-wendao` now consumes this parser-owned observation contract in four
ways:

1. `parsers::markdown::code_observation` is an adapter-only re-export surface
   over the parser-owned contract
2. `parsers::markdown::sections` still enriches parser-owned section structure
   with Wendao-owned entities, while observations are now parser-owned inputs
3. `zhenfa_router::native::semantic_check` validates and interprets
   observations, but no longer owns their parsing
4. gateway search/source-index flows may rebuild observation hits from parser-
   owned attributes without depending on `parse_note`

## Regression Coverage

Coverage for this contract lives in:

1. `packages/rust/crates/xiuxian-wendao-parsers/tests/unit/code_observation.rs`
2. `packages/rust/crates/xiuxian-wendao/tests/unit/gateway/studio/search.rs`
3. `packages/rust/crates/xiuxian-wendao/tests/unit/semantic_check_tests.rs`

:RELATIONS:
:LINKS: [[02_parser/index|Wendao Parser Docs]], [[02_parser/architecture|Parser Architecture]], [[02_parser/toc|Parser TOC Surface]], [[02_parser/sections|Parser Sections]], [[06_roadmap/419_parser_substrate_separation|Parser Substrate Separation]]
:END:

---

:FOOTER:
:LAST_SYNC: 2026-04-13
:END:
