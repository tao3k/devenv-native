# Parser Semantic Check Surface

:PROPERTIES:
:ID: wendao-parser-semantic-check
:PARENT: [[02_parser/index|Wendao Parser Docs]]
:TAGS: parser, semantic-check, zhenfa
:STATUS: ACTIVE
:END:

## Objective

Wendao now treats semantic-check grammar helpers as a parser-owned local
surface under `src/parsers/semantic_check/`, while `zhenfa_router` keeps
semantic-check policy, issue shaping, rendering, and tool wiring.

## Contract

The canonical semantic-check parser contracts now split into four layers:

1. link reference extraction helpers
   - `extract_id_references`
   - `extract_hash_references`
2. contract mini-language parsing
   - `extract_function_args`
   - `validate_contract`
3. identity suggestion helper
   - `generate_suggested_id`
4. parser-owned helper type
   - `HashReference`

These contracts do not own semantic-check issue policy, health scoring,
registry traversal, or native tool registration.

## Consumer Boundary

`xiuxian-wendao` now consumes this parser-owned semantic-check surface in
three ways:

1. `zhenfa_router::native::semantic_check::checks::*` consumes it for dead
   link, hash alignment, contract, and missing-identity checks, but no longer
   owns the grammar
2. `zhenfa_router::native::semantic_check::test_api` forwards to the parser
   owner path so existing semantic-check proofs keep their public shape while
   the grammar owner moves out of `zhenfa_router`
3. parser-owned unit coverage now lives under `tests/unit/parsers/`, while
   semantic-check behavior coverage stays under the `semantic_check` test tree

## Why It Stays In Wendao For Now

This surface is parser-owned, but it remains Wendao-local rather than a
cross-crate parser export because the grammar still describes Wendao-specific
semantic-check conventions rather than a cross-package document-format
contract.

## Regression Coverage

Coverage for this contract lives in:

1. `packages/rust/crates/xiuxian-wendao/tests/unit/parsers/semantic_check.rs`
2. `packages/rust/crates/xiuxian-wendao/tests/unit/semantic_check.rs`
3. `packages/rust/crates/xiuxian-wendao/tests/unit/semantic_check_tests.rs`

:RELATIONS:
:LINKS: [[02_parser/index|Wendao Parser Docs]], [[02_parser/architecture|Parser Architecture]], [[02_parser/docs_governance|Parser Docs Governance Surface]], [[06_roadmap/419_parser_substrate_separation|Parser Substrate Separation]]
:END:

---

:FOOTER:
:LAST_SYNC: 2026-04-14
:END:
