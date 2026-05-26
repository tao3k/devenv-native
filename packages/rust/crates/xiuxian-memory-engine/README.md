# xiuxian-memory-engine

Episodic operational memory for the Wendao stack.

## Responsibility

`xiuxian-memory-engine` owns the bounded memory layer only:

- memory lifecycle contracts for cache, temporary, scheduled, episodic, and
  knowledge records
- episode storage
- semantic recall plus utility reranking
- Q-value or utility estimation
- recall feedback bias
- episodic lifecycle and gate decisions

This crate does not own durable docs, projected pages, or a generic knowledge
registry.

## Boundary

The formal cross-layer boundary is defined in
[`docs/rfcs/2026-04-05-wendao-memory-layer-boundaries-rfc.md`](../../../../../docs/rfcs/2026-04-05-wendao-memory-layer-boundaries-rfc.md).

Within that model, `xiuxian-memory-engine` owns the lifecycle contract and
recall priors for memory records. It does not own the durable documentation or
knowledge registry that stores promoted knowledge.

## Current Model

The current crate model is intentionally episodic:

- `Episode` is an interaction or experience unit
- `MemoryLayer`, `MemoryStatus`, and `MemoryRecallDefault` define the
  lifecycle envelope for cache, temporary, scheduled, episodic, and knowledge
  records
- `MemoryLifecycleFacts::evaluate()` computes a deterministic recall prior:
  `layer_prior * status_multiplier * recall_default_multiplier`
- two-phase retrieval is semantic recall followed by Q-value reranking
- `QTable` currently implements online utility smoothing, not full
  temporal-difference future-return learning
- persisted state stores episodes, Q-values, and scope-level recall feedback
  bias

The current hygiene contract also makes these distinctions explicit:

- `retrieval_count` is separate from `success_count` and `failure_count`
- `created_at` is separate from `updated_at`
- memory-gate promotion uses an explicit target layer instead of implying
  direct durable publication

The current host-read-model seam also stays inside episodic ownership:

- `EpisodeStore::memory_projection_rows(...)` exports read-only episode features
- `MemoryProjectionRow` carries scope, embeddings, utility counters, and
  timestamps for Julia compute lanes
- the projection surface does not expose lifecycle mutation or registry writes

## Non-Goals

Do not place the following in this crate:

- durable docs or projected-page ownership
- generic cache-registry behavior
- validated working-knowledge registry behavior
- durable publication or archival policy

The crate may define the lifecycle state and recall prior for a promoted
knowledge record, but publication and long-term registry writes stay with the
owning knowledge surface.

## References

- [`docs/01_core/memory/architecture.md`](../../../../../docs/01_core/memory/architecture.md)
- [`packages/rust/crates/xiuxian-memory-engine/src/`](./src/)
