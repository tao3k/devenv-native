---
type: knowledge
metadata:
  title: "P1 Python Retirement Matrix"
---

# P1 Python Retirement Matrix

## Status

The retirement wave is largely complete.

## Keep Set

Only these Python package responsibilities remain endorsed:

1. `wendao-core-lib`
   Canonical Python transport client for Arrow Flight and Arrow IPC fallback.
2. `wendao-arrow-interface`
   Downstream-facing Arrow/Polars facade over the transport substrate.
3. `xiuxian-wendao-analyzer`
   Analyzer workflow and Wendao document parsing service package above the
   transport substrate.
4. `foundation`
   Thin config/schema/logging helpers.
5. `core`
   Minimal retained helper surfaces only.

## Already Deleted

These package centers are no longer part of the Python tree:

1. `packages/python/agent/`
2. the removed standalone protocol server package
3. `xiuxian_core.skills`
4. Python-local router, watcher, scanner, workflow, memory, and knowledge-host
   stacks

## Remaining Rule

Any retained Python surface must satisfy all of these:

1. It does not recreate a Python runtime center.
2. It does not depend on deleted local skill/agent/protocol infrastructure.
3. It stays transport-consumer-only or helper-only.
4. It can justify its existence without invoking legacy compatibility as the
   primary reason.

## Next Retirement Pressure

The next deletions should target stale docs, comments, and helper shells that
still describe the old architecture even though the corresponding code is gone.
