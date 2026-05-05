---
id: component.qianji.execution-plane
kind: component
title: Qianji Execution Plane
status: active
confidence:
  score: 1.0
  source: human_signed
owners:
  - scope: packages/rust/crates/xiuxian-qianji
    role: execution_consumer
provenance:
  source: docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md
  recorded_by: codex
  recorded_at: "2026-05-05"
verification:
  required:
    - direnv exec . cargo test -p xiuxian-qianji workdir_semantic -- --nocapture
    - direnv exec . cargo test -p xiuxian-qianji scheduler_preflight -- --nocapture
  evidence:
    - docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-audit.md
relations:
  - kind: depends_on
    target: component.wendao.query-substrate
  - kind: consumed_by
    target: task.semantic-ssot.object-schema-pilot
---

# Qianji Execution Plane

Qianji owns workflow execution and consumes semantic scope as advisory context.
Scheduler preflight may inject a read-only semantic-scope guard trace into
workflow context plus a compact semantic-scope guard route for downstream
mechanisms. Workflow contexts may opt into `semanticScopeGuardPolicy` blocking
for blocked or review-required semantic scope, but Qianji does not own
canonical semantic ontology truth.
