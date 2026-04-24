---
type: knowledge
title: "Lance Vector-Store Boundary"
category: "explanation"
tags:
  - explanation
  - vector
  - wendao
saliency_base: 6.0
decay_rate: 0.04
metadata:
  title: "Lance Vector-Store Boundary"
---

# Lance Vector-Store Boundary

> Storage Boundary - Lance-backed vector-table storage for multimodal evidence,
> not the search-plane owner.

## Overview

The Lance vector-store surface is now a storage-format boundary behind
`xiuxian-db-store`. Search semantics, linting, routing behavior, and result
contracts belong to Wendao/DuckDB-owned query layers or the routing crate that
serves the specific command.

The retiring `xiuxian-vector` crate remains only as a Lance-backed
vector-table storage shell while live consumers are migrated behind the
facade.

Primary characteristics:

- Lance-backed vector-table storage
- Arrow/RecordBatch compatibility helpers
- adaptive Lance index operations
- bounded table-cache and maintenance utilities
- no ownership of skill routing, keyword fusion, tool search, or DuckDB query
  semantics

## Architecture

```text
Caller crate or command
  -> xiuxian-db-store facade
  -> Lance vector-store compatibility surface
  -> retiring xiuxian-vector storage shell
```

## Core Modules

- `packages/rust/crates/xiuxian-db-store/src/lib.rs`
- `packages/rust/crates/xiuxian-vector/src/ops/`
- `packages/rust/crates/xiuxian-vector/src/search/`
- `packages/rust/crates/xiuxian-vector/src/search_engine/`
- `packages/rust/crates/xiuxian-vector/src/query_support.rs`
- `packages/rust/crates/xiuxian-vector/src/search_cache.rs`

## Runtime Configuration

Configuration source:

- system: `packages/conf/settings.yaml`
- user override: `$PRJ_CONFIG_HOME/xiuxian-artisan-workshop/settings.yaml`

Active vector keys:

```yaml
vector:
  index_cache_size_bytes: 134217728
  max_cached_tables: 4
  default_partition_column: "skill_name"
```

## Operational Guidance

1. Depend on `xiuxian-db-store` for storage-facing compatibility instead of
   adding new direct callers to the retiring storage shell.
2. Use bounded cache settings in long-lived runtime processes.
3. Run scalar/vector index creation after bulk ingestion when Lance storage is
   still required.
4. Keep schema evolution explicit and covered by snapshot/contract tests.
5. Keep search, routing, and lint semantics in Wendao/DuckDB-owned layers.

## Checkpoint Note (Historical)

The previous **vector checkpoint system** (LanceDB `CheckpointStore`) has been removed from `xiuxian-vector`.

The phrase `checkpoint schema` is now historical context only. Current workflow checkpoint persistence is file-based and implemented under:

- `packages/python/foundation/src/omni/foundation/workflow_state.py`

Do not add new dependencies on the removed `xiuxian-vector` checkpoint module.

## Related Docs

- `docs/01_core/wendao/roadmap.md`
