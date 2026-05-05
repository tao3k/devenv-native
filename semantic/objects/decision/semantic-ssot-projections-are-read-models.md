---
id: decision.semantic-ssot.projections-are-read-models
kind: decision
title: Projections Are Read Models
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
    - direnv exec . wendao-client lint semantic semantic
  evidence:
    - semantic/projections/llm-compression.md
relations:
  - kind: governs
    target: component.docs.projection-system
  - kind: governs
    target: invariant.llm-output-is-not-authority
---

# Projections Are Read Models

Human documents, LLM compression views, review summaries, and operational
views are derived from semantic objects and relations.
