---
type: semantic_projection
projection: llm_compression
source_objects:
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
source_revision: "blake3:51bdbdcbb775f90a679b945f56c1f5dbd944601331975f26434ab9b0d9f5d3f9"
projection_revision: "2026-05-05.semantic-ssot-runtime-pilot.v2"
staleness: fresh
status: active
---

# LLM Compression Projection

For agent context, compress this semantic graph as:

1. repository artifacts own semantic truth
2. Wendao validates and serves semantic object scopes
3. Qianji consumes semantic scope before execution
4. docs and LLM summaries are projections
5. Valkey is runtime state, not authority

This projection is a read model. Its authority comes only from the source
object IDs listed in frontmatter.
