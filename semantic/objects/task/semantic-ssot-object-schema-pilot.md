---
id: task.semantic-ssot.object-schema-pilot
kind: task
title: Semantic SSOT Object Schema Pilot
status: active
confidence:
  score: 1.0
  source: human_signed
owners:
  - scope: packages/rust/crates/xiuxian-wendao-parsers
    role: schema_validator
  - scope: packages/rust/crates/xiuxian-qianji
    role: semantic_scope_consumer
provenance:
  source: docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md
  recorded_by: codex
  recorded_at: "2026-05-05"
verification:
  required:
    - direnv exec . cargo test -p xiuxian-wendao-parsers semantic -- --nocapture
    - direnv exec . cargo test -p xiuxian-wendao-client semantic -- --nocapture
    - direnv exec . cargo test -p xiuxian-wendao-server semantic_scope -- --nocapture
    - direnv exec . cargo test -p xiuxian-wendao-studio --features zhenfa-router --test semantic_scope_provider semantic_scope -- --nocapture
    - direnv exec . cargo test -p xiuxian-wendao-sql bounded_work_markdown -- --nocapture
    - direnv exec . cargo test -p xiuxian-qianji workdir_semantic -- --nocapture
    - direnv exec . cargo test -p xiuxian-qianji scheduler_preflight -- --nocapture
    - direnv exec . cargo test -p xiuxian-qianji router -- --nocapture
    - direnv exec . uv run pytest tests/test_wendao_semantic_refresh_process_nix.py -q
    - direnv exec . wendao-client lint semantic
  evidence:
    - docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-audit.md
relations:
  - kind: implements
    target: decision.semantic-ssot.repo-native-first
  - kind: implements
    target: decision.semantic-ssot.projections-are-read-models
  - kind: affects
    target: component.wendao.query-substrate
  - kind: affects
    target: component.qianji.execution-plane
  - kind: validates
    target: invariant.llm-output-is-not-authority
  - kind: validates
    target: invariant.execution-graph-is-not-semantic-graph
---

# Semantic SSOT Object Schema Pilot

This task lands the first parser-validated semantic object schema, seed object
set, Wendao semantic-scope route, Qianji semantic-surface consumer, and
workflow preflight semantic-scope trace plus downstream route with optional
policy enforcement. It also exposes a read-only semantic projection refresh
plan and an explicit refresh worker entrypoint that can run one pass or
supervised recurring passes while keeping mutation explicit, including an
optional clean-worktree startup guard for supervised projection writeback.
The runtime packaging slice also exposes the same guarded runner as the
`wendao-semantic-refresh` process-compose entry. Qianji router nodes can now
opt into semantic guard route branch selection, making the advisory route
usable by ordinary workflow templates.
