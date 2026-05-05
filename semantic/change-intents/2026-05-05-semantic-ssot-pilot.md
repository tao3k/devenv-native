---
type: semantic_change_intent
id: change.semantic-ssot.runtime-pilot
title: Semantic SSOT Runtime Pilot
status: active
touched_objects:
  - component.qianji.execution-plane
  - component.wendao.query-substrate
  - component.docs.projection-system
  - component.valkey.runtime-state-spine
  - decision.semantic-ssot.repo-native-first
  - decision.semantic-ssot.projections-are-read-models
  - invariant.llm-output-is-not-authority
  - invariant.execution-graph-is-not-semantic-graph
  - invariant.valkey-is-not-semantic-authority
  - task.semantic-ssot.object-schema-pilot
changed_relations:
  - source: task.semantic-ssot.object-schema-pilot
    kind: validates
    target: invariant.llm-output-is-not-authority
    action: add
  - source: task.semantic-ssot.object-schema-pilot
    kind: validates
    target: invariant.execution-graph-is-not-semantic-graph
    action: add
  - source: task.semantic-ssot.object-schema-pilot
    kind: validates
    target: invariant.valkey-is-not-semantic-authority
    action: add
affected_invariants:
  - invariant.llm-output-is-not-authority
  - invariant.execution-graph-is-not-semantic-graph
  - invariant.valkey-is-not-semantic-authority
required_validations:
  - direnv exec . cargo test -p xiuxian-wendao-parsers semantic -- --nocapture
  - direnv exec . cargo test -p xiuxian-wendao-client semantic -- --nocapture
  - CARGO_TARGET_DIR=.cache/cargo-target/semantic-ssot direnv exec . cargo run -p xiuxian-wendao-client --bin wendao-client -- lint semantic
projections_to_refresh:
  - llm_compression
candidate_suggestions: []
---

# Semantic SSOT Runtime Pilot

This change intent records the first repo-native semantic SSOT physical pilot.
It declares the canonical objects touched by the pilot, the invariant evidence
relations introduced by the validation path, and the projection that must stay
fresh after semantic object updates.
