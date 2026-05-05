---
id: component.wendao.query-substrate
kind: component
title: Wendao Query Substrate
status: active
confidence:
  score: 1.0
  source: human_signed
owners:
  - scope: packages/rust/crates/xiuxian-wendao-server
    role: transport_contract_owner
  - scope: packages/rust/crates/xiuxian-wendao-studio
    role: semantic_scope_provider
  - scope: packages/rust/crates/xiuxian-wendao-sql
    role: semantic_read_model_owner
provenance:
  source: docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md
  recorded_by: codex
  recorded_at: "2026-05-05"
verification:
  required:
    - direnv exec . cargo test -p xiuxian-wendao-server semantic_scope -- --nocapture
    - direnv exec . cargo test -p xiuxian-wendao-studio --features zhenfa-router --test semantic_scope_provider semantic_scope -- --nocapture
    - direnv exec . cargo test -p xiuxian-wendao-sql semantic_read_model -- --nocapture
    - direnv exec . cargo test -p xiuxian-wendao-sql semantic_read_model_query_validation -- --nocapture
    - direnv exec . cargo test -p xiuxian-wendao-client read_model_summary -- --nocapture
    - direnv exec . cargo test -p xiuxian-wendao-client query_read_model -- --nocapture
  evidence:
    - docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md
relations:
  - kind: implements
    target: decision.semantic-ssot.repo-native-first
  - kind: projects_to
    target: component.qianji.execution-plane
---

# Wendao Query Substrate

Wendao exposes parser-backed semantic object scopes through query contracts
and advisory read-model summaries and queries while leaving canonical truth in
repository artifacts. The read-model query surface admits only one read-only
SQL statement per request.
