---
type: knowledge
title: "Audit: Qianji RFC Implementation Coverage"
category: "audit"
status: "active"
authors:
  - codex
created: 2026-04-08
tags:
  - audit
  - qianji
  - rfc
  - flowhub
  - retrieval
---

# Audit: Qianji RFC Implementation Coverage

This audit records the current implementation coverage of the active Flowhub
and bounded-work RFC cluster in `xiuxian-qianji`.

It is not a new RFC. It is the code-backed progress matrix for the existing
RFC set.

## 1. Scope

This audit covers the current Flowhub and bounded-work RFC cluster:

- [RFC: Qianji Flowhub Graph Contract Model](2026-04-07-qianji-flowhub-graph-contract-model-rfc.md)
- [RFC 0003: Wendao SQL Minimal Retrieval Surface for Bounded Plan Work](2026-04-07-wendao-sql-minimal-retrieval-surface-rfc.md)
- [RFC 0004: Compact Validation and Flowchart Alignment](2026-04-08-compact-validation-flowchart-alignment-rfc.md)
- [RFC 0005: Markdown Skeleton Minimal Rules for Bounded Plan Work](2026-04-08-markdown-skeleton-minimal-rules-rfc.md)
- [RFC 0006: Markdown Row Segmentation Minimal Rules for Bounded Plan Work](2026-04-08-markdown-row-segmentation-minimal-rules-rfc.md)
- [RFC 0007: Flowchart Backbone Minimal Rules for Bounded Plan Work](2026-04-08-flowchart-backbone-minimal-rules-rfc.md)
- [RFC 0008: Heading Path Minimal Conventions for Bounded Plan Work](2026-04-08-heading-path-minimal-conventions-rfc.md)

The older data-centric workflow RFC is outside the scope of this audit.

## 2. Coverage Summary

| RFC | Coverage | Primary code anchors | Primary tests | Main remaining gap |
| --- | --- | --- | --- | --- |
| [Flowhub Graph Contract Model](2026-04-07-qianji-flowhub-graph-contract-model-rfc.md) | Implemented core path; partial target-model tail | `packages/rust/crates/xiuxian-qianji/src/flowhub/check.rs`; `packages/rust/crates/xiuxian-qianji/src/flowhub/show.rs`; `packages/rust/crates/xiuxian-qianji/src/flowhub/scenario/show.rs`; `packages/rust/crates/xiuxian-qianji/src/flowhub/scenario/check.rs`; `packages/rust/crates/xiuxian-qianji/src/flowhub/materialize/scenario.rs`; `packages/rust/crates/xiuxian-qianji/src/workdir/check.rs`; `packages/rust/crates/xiuxian-qianji/src/workdir/query.rs`; `qianji-flowhub/qianji.toml` | `packages/rust/crates/xiuxian-qianji/tests/integration/flowhub_contracts.rs`; `packages/rust/crates/xiuxian-qianji/tests/integration/flowhub_scenario_show_check.rs`; `packages/rust/crates/xiuxian-qianji/tests/integration/flowhub_materialize.rs`; `packages/rust/crates/xiuxian-qianji/tests/integration/workdir_contracts.rs` | the live Flowhub root intentionally exposes only registered top-level node anchors; richer semantic subnodes remain target-model vocabulary unless explicitly declared through `[contract]` |
| [Wendao SQL Minimal Retrieval Surface](2026-04-07-wendao-sql-minimal-retrieval-surface-rfc.md) | Partial | `packages/rust/crates/xiuxian-wendao-sql/src/bounded_work_markdown/mod.rs`; `packages/rust/crates/xiuxian-wendao-sql/src/bounded_work_markdown/register.rs`; `packages/rust/crates/xiuxian-wendao-sql/src/bounded_work_markdown/query.rs`; `packages/rust/crates/xiuxian-wendao-sql/src/bounded_work_markdown/skeleton.rs`; `packages/rust/crates/xiuxian-qianji/src/workdir/query.rs` | `packages/rust/crates/xiuxian-wendao/tests/unit/search/queries/sql/bounded_work_markdown.rs`; `packages/rust/crates/xiuxian-qianji/tests/integration/workdir_contracts.rs` | the bounded `markdown` SQL surface is real, but the full RFC acceptance matrix for retrieval semantics is not yet closed |
| [Compact Validation and Flowchart Alignment](2026-04-08-compact-validation-flowchart-alignment-rfc.md) | Implemented core path | `packages/rust/crates/xiuxian-qianji/src/contracts/workdir/manifest.rs`; `packages/rust/crates/xiuxian-qianji/src/workdir/check.rs`; `packages/rust/crates/xiuxian-qianji/src/flowhub/materialize/scenario.rs` | `packages/rust/crates/xiuxian-qianji/tests/integration/workdir_contracts.rs`; `packages/rust/crates/xiuxian-qianji/tests/integration/flowhub_materialize.rs` | current code proves the compact manifest and check path, but future optional extensions described in the RFC remain outside the current implementation |
| [Markdown Skeleton Minimal Rules](2026-04-08-markdown-skeleton-minimal-rules-rfc.md) | Partial | `packages/rust/crates/xiuxian-wendao-sql/src/bounded_work_markdown/skeleton.rs`; `packages/rust/crates/xiuxian-wendao-sql/src/bounded_work_markdown/rows.rs`; `packages/rust/crates/xiuxian-qianji/src/workdir/query.rs` | `packages/rust/crates/xiuxian-wendao/tests/unit/search/queries/sql/bounded_work_markdown.rs`; `packages/rust/crates/xiuxian-qianji/tests/integration/workdir_contracts.rs` | the current code emits and queries `skeleton`, but the RFC still needs tighter clause-by-clause acceptance coverage for preservation and omission rules |
| [Markdown Row Segmentation Minimal Rules](2026-04-08-markdown-row-segmentation-minimal-rules-rfc.md) | Partial | `packages/rust/crates/xiuxian-wendao-sql/src/bounded_work_markdown/discovery.rs`; `packages/rust/crates/xiuxian-wendao-sql/src/bounded_work_markdown/rows.rs` | `packages/rust/crates/xiuxian-wendao/tests/unit/search/queries/sql/bounded_work_markdown.rs` | row generation exists for bounded work surfaces, but the RFC does not yet have a dedicated acceptance set proving every anti-fragmentation rule |
| [Flowchart Backbone Minimal Rules](2026-04-08-flowchart-backbone-minimal-rules-rfc.md) | Implemented core path | `packages/rust/crates/xiuxian-qianji/src/workdir/check.rs`; `packages/rust/crates/xiuxian-qianji/src/flowhub/mermaid/validate.rs` | `packages/rust/crates/xiuxian-qianji/tests/integration/workdir_contracts.rs`; `packages/rust/crates/xiuxian-qianji/tests/integration/flowhub_contracts.rs` | current validation covers the live bounded-surface backbone and scenario-case graph legality, not every possible future graph flavor |
| [Heading Path Minimal Conventions](2026-04-08-heading-path-minimal-conventions-rfc.md) | Partial | `packages/rust/crates/xiuxian-wendao-sql/src/bounded_work_markdown/rows.rs`; `packages/rust/crates/xiuxian-wendao-sql/src/bounded_work_markdown/schema.rs` | `packages/rust/crates/xiuxian-wendao/tests/unit/search/queries/sql/bounded_work_markdown.rs` | `heading_path` is emitted into the bounded `markdown` table, but the RFC still lacks a tighter dedicated acceptance proof for every external convention it specifies |

No audited RFC in this cluster is purely spec-only anymore. The remaining
spec-only scope is clause-level tail content inside otherwise implemented
lanes.

## 3. Live Proof Points

The current live proof for the implemented Flowhub lane is:

```bash
direnv exec "$PRJ_ROOT" cargo run -p xiuxian-qianji --features llm --bin qianji -- \
  show --dir "$PRJ_ROOT/qianji-flowhub"

direnv exec "$PRJ_ROOT" cargo run -p xiuxian-qianji --features llm --bin qianji -- \
  check --dir "$PRJ_ROOT/qianji-flowhub"

direnv exec "$PRJ_ROOT" cargo run -p xiuxian-qianji --features llm --bin qianji -- \
  show --dir "$PRJ_ROOT/qianji-flowhub/plan"
```

The retrieval lane is currently code-proven through library and unit-test
surfaces rather than through a new user-facing CLI verb.

## 4. Draft Exit Criteria

The current audited cluster should leave pure draft-state only after the
following closures:

1. the Flowhub graph-contract RFC either implements or explicitly splits out
   its remaining target-model-only semantic subnode language
2. the retrieval RFC cluster gains clause-by-clause acceptance coverage for
   `skeleton`, row segmentation, and `heading_path`
3. the audited RFC frontmatter statuses are updated deliberately after that
   coverage is reviewed, rather than being left as stale `draft`

## 5. Current Verdict

The Flowhub graph-contract lane is substantially code-backed today.

The retrieval lane is no longer speculative, but it is still only partially
closed from an RFC-governance point of view.
