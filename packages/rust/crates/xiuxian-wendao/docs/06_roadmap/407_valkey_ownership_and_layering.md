# 407 Valkey Ownership and Layering

## Goal

Freeze the current Valkey ownership boundary before any refactor so semantic
runtime state is not pushed into `xiuxian-vector` by mistake.

## Decision Summary

The main Valkey home should remain `xiuxian-wendao`, not
`packages/rust/crates/xiuxian-vector`.

The correct split is:

1. semantic Valkey state stays with the crate that owns the domain
2. reusable Valkey transport helpers may be extracted later as thin shared infra
3. `xiuxian-vector` only takes Valkey ownership if the state is genuinely
   vector-specific

## Why `xiuxian-vector` Is Not the Primary Valkey Home

`xiuxian-vector` currently owns embedded vector retrieval kernels:

- Lance/LanceDB table work
- Arrow batch operations
- Tantivy-backed keyword search
- vector-side indexing and search execution

Its current package surface does not establish Valkey as a first-class runtime
dependency. The active Valkey semantics live elsewhere.

By contrast, `xiuxian-wendao` already owns several Valkey-backed semantic lanes:

- search-plane request cache and repo publication manifests
- link-graph saliency state and decay
- link-graph stats cache
- link-graph context snapshots
- link-graph suggested-link queues and decision streams
- graph snapshot persistence
- legacy knowledge-entry storage

These are not generic vector concerns. They are knowledge, graph, and search
semantics.

## Current Ownership Map

### Stay in `xiuxian-wendao`

These lanes should remain Wendao-owned because the keys, payloads, and cache
meaning are domain semantics rather than storage mechanics.

#### Search Plane

- `src/search_plane/cache.rs`

This cache stores query response payloads and repo publication manifests. The
keyspace is derived from corpus epochs, repo publication identity, intent, and
query semantics. That belongs to the search plane, not to the vector kernel.

#### Link Graph Runtime State

- `src/link_graph/stats_cache/valkey.rs`
- `src/link_graph/context_snapshot.rs`
- `src/link_graph/agentic/store/suggested.rs`
- `src/link_graph/agentic/store/decision.rs`
- `src/link_graph/index/build/cache/storage.rs`
- `src/link_graph/index/build/graphmem.rs`
- `src/link_graph/saliency/store/`

These keys persist graph-native state: saliency, neighbor structure,
context snapshots, and suggested-link workflow state. They should remain under
Wendao ownership even if common Valkey helpers are later extracted.

#### Graph and Knowledge Persistence

- `src/graph/valkey_persistence.rs`
- `src/storage/crud.rs`
- `src/kg_cache.rs`

This lane persists graph snapshots and legacy knowledge-table state. It is
domain persistence, not vector serving.

#### Future Repo/Analyzer Cache If Activated

- `src/analyzers/cache/`

`ValkeyAnalysisCache` is now a real Wendao-owned Valkey cache. It persists
normalized `RepositoryAnalysisOutput` snapshots under analyzer-specific key
prefixing and revision-scoped identity, and ownership remains in the analyzer
layer because the payload is repository-analysis semantics rather than vector
storage state.

The current repo-index and analyzer-cache audits make the ownership line
concrete:

- repo-backed publication reuse is already handled inside the search-plane
  staged-mutation path
- practical repo-index incrementality now has an additional managed-remote
  revision short-circuit in the coordinator
- normalized analyzer output now also persists in Valkey under the Wendao
  analyzer surface, so repo-analysis reuse across process boundaries is no
  longer a placeholder-only plan
- the remaining follow-up work is operational rather than ownership-related:
  live cache proof, cleanup/eviction policy refinement, and any future helper
  extraction still remain Wendao-owned follow-up work rather than
  vector-kernel concerns

### Shared Infra Candidates

Only the following pieces are reasonable extraction targets:

- Valkey client factory and connection creation
- shared timeout policy helpers
- URL/env resolution helpers
- key-prefix normalization helpers
- common payload serialization wrappers
- low-level retry/logging utilities

These helpers should stay thin and domain-agnostic. They should not absorb
Wendao-specific key names, graph schemas, repo manifest records, or deep-wiki
planner semantics.

### Not for `xiuxian-vector`

The following should not be moved into `xiuxian-vector`:

- repo publication manifests
- search-plane hot-query cache keys
- link-graph saliency snapshots
- suggested-link streams and decision logs
- context snapshots
- graph snapshot persistence
- legacy knowledge-entry storage

If these moved into `xiuxian-vector`, the vector crate would become an
accidental home for knowledge-graph and repo-analysis semantics it does not own.

### Could Move to `xiuxian-vector` Later

Only vector-specific Valkey state is a valid future candidate, for example:

- remote ANN coordination metadata
- vector-side compaction leases
- vector index warmup markers
- vector-specific embedding or rerank caches

That lane does not exist as the primary workload today.

## Cross-Crate Observation

Other crates already use Valkey for their own domains:

- `xiuxian-memory-engine`
- `xiuxian-daochang`

That strengthens the boundary decision: Valkey is a cross-cutting runtime
primitive, not something that should be centralized under `xiuxian-vector`.

## Recommended Migration Sequence

1. Keep semantic Valkey ownership in Wendao.
2. Extract only thin shared Valkey helpers if duplication becomes costly.
3. Move vector-specific Valkey concerns into `xiuxian-vector` only when they
   actually appear.
4. Avoid any migration that makes vector the root owner of graph/search/wiki
   state.

## Immediate Next Slice

If a refactor is needed, the next slice should be a transport-only cleanup:

- inventory duplicated Valkey client/env helpers
- decide whether they belong in a tiny shared infra module or crate
- keep all existing Wendao keyspaces and payload schemas in place

This roadmap note does not authorize moving semantic state into
`xiuxian-vector`.
