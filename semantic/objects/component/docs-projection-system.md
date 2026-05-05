---
id: component.docs.projection-system
kind: component
title: Documentation Projection System
status: active
confidence:
  score: 1.0
  source: human_signed
owners:
  - scope: docs
    role: projection_surface
provenance:
  source: docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md
  recorded_by: codex
  recorded_at: "2026-05-05"
verification:
  required:
    - direnv exec . wendao-client lint semantic semantic
  evidence:
    - docs/index.md
relations:
  - kind: implements
    target: decision.semantic-ssot.projections-are-read-models
  - kind: consumed_by
    target: task.semantic-ssot.object-schema-pilot
---

# Documentation Projection System

Documentation remains a stable human-facing projection. It must point back to
semantic object IDs when it represents semantic authority.
