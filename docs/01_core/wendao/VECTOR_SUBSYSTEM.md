---
type: knowledge
title: "Vector Subsystem"
category: "explanation"
tags:
  - explanation
  - vector
  - wendao
saliency_base: 6.0
decay_rate: 0.04
metadata:
  title: "Vector Subsystem"
---

# Vector Subsystem

> Foundation Layer - LanceDB-backed vector retrieval for skills and knowledge.

## Overview

The vector subsystem is implemented in `xiuxian-vector` and powers semantic retrieval for:

- skill/tool discovery
- router score fusion
- knowledge retrieval

Primary characteristics:

- LanceDB storage
- adaptive index strategy (HNSW/IVF-FLAT decisions by data scale)
- hybrid fusion with keyword signals
- bounded in-memory table cache

## Architecture

```text
Python (foundation/core/agent)
  -> package-local transport or facade layers
  -> Rust core (xiuxian-vector)
```

## Core Modules

- `packages/rust/crates/xiuxian-vector/src/ops/`
- `packages/rust/crates/xiuxian-vector/src/keyword/`
- `packages/rust/crates/xiuxian-vector/src/search.rs`
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

1. Use bounded cache settings in long-lived agent and external-tool runtime processes.
2. Run scalar/vector index creation after bulk ingestion.
3. Keep schema evolution explicit and covered by snapshot/contract tests.

## Checkpoint Note (Historical)

The previous **vector checkpoint system** (LanceDB `CheckpointStore`) has been removed from `xiuxian-vector`.

The phrase `checkpoint schema` is now historical context only. Current workflow checkpoint persistence is file-based and implemented under:

- `packages/python/foundation/src/omni/foundation/workflow_state.py`

Do not add new dependencies on the removed `xiuxian-vector` checkpoint module.

## Related Docs

- `docs/01_core/wendao/roadmap.md`
