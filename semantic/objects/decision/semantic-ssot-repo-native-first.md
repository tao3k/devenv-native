---
id: decision.semantic-ssot.repo-native-first
kind: decision
title: Repo-Native Semantic Authority First
status: active
confidence:
  score: 1.0
  source: human_signed
owners:
  - scope: docs/rfcs
    role: architecture_decision
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
  - kind: governs
    target: component.wendao.query-substrate
  - kind: governs
    target: component.qianji.execution-plane
  - kind: governs
    target: task.semantic-ssot.object-schema-pilot
---

# Repo-Native Semantic Authority First

The first authoritative semantic layer lives in versioned repository artifacts.
Databases and generated views may materialize read models later.
