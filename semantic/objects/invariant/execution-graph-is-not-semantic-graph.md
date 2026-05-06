---
id: invariant.execution-graph-is-not-semantic-graph
kind: invariant
title: Execution Graph Is Not Semantic Graph
status: active
confidence:
  score: 1.0
  source: human_signed
owners:
  - scope: packages/rust/crates/xiuxian-qianji
    role: workflow_boundary
provenance:
  source: docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md
  recorded_by: codex
  recorded_at: "2026-05-05"
verification:
  required:
    - direnv exec . cargo test -p xiuxian-qianji workdir_semantic -- --nocapture
  evidence:
    - docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md
relations:
  - kind: constrains
    target: component.qianji.execution-plane
  - kind: constrains
    target: task.semantic-ssot.object-schema-pilot
---

# Execution Graph Is Not Semantic Graph

Qianji execution flow describes how work moves. The semantic graph records
objects, meaning, constraints, relations, and governance state.
