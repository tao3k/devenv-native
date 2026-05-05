---
id: component.valkey.runtime-state-spine
kind: component
title: Valkey Runtime State Spine
status: active
confidence:
  score: 1.0
  source: human_signed
owners:
  - scope: runtime/valkey
    role: runtime_state_surface
provenance:
  source: docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md
  recorded_by: codex
  recorded_at: "2026-05-05"
verification:
  required:
    - direnv exec . wendao-client lint semantic
  evidence:
    - docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-audit.md
relations:
  - kind: depends_on
    target: invariant.valkey-is-not-semantic-authority
---

# Valkey Runtime State Spine

Valkey stores runtime state, checkpoints, events, and caches. It is not the
source of canonical semantic truth.
