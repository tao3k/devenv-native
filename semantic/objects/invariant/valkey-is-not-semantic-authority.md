---
id: invariant.valkey-is-not-semantic-authority
kind: invariant
title: Valkey Is Not Semantic Authority
status: active
confidence:
  score: 1.0
  source: human_signed
owners:
  - scope: runtime/valkey
    role: runtime_state_boundary
provenance:
  source: docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md
  recorded_by: codex
  recorded_at: "2026-05-05"
verification:
  required:
    - direnv exec . wendao-client lint semantic semantic
  evidence:
    - docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-audit.md
relations:
  - kind: constrains
    target: component.valkey.runtime-state-spine
  - kind: constrains
    target: decision.semantic-ssot.repo-native-first
---

# Valkey Is Not Semantic Authority

Runtime cache loss may slow retrieval or projection, but it must not destroy
semantic truth.
