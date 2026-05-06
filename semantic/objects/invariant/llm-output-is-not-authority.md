---
id: invariant.llm-output-is-not-authority
kind: invariant
title: LLM Output Is Not Authority
status: active
confidence:
  score: 1.0
  source: human_signed
owners:
  - scope: docs/rfcs
    role: semantic_governance
provenance:
  source: docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md
  recorded_by: codex
  recorded_at: "2026-05-05"
verification:
  required:
    - direnv exec . wendao-client lint semantic
  evidence:
    - docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md
relations:
  - kind: constrains
    target: decision.semantic-ssot.projections-are-read-models
  - kind: constrains
    target: task.semantic-ssot.object-schema-pilot
---

# LLM Output Is Not Authority

LLM-generated summaries, objects, relations, and projections remain proposals
or read models until accepted through repository governance.
