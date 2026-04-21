---
type: knowledge
metadata:
  title: "Python Packages"
---

# Python Packages

## Position

`packages/python` is a thin consumer layer around Rust-owned contracts.

Rust owns:

- the execution kernel
- routing
- memory
- indexing
- knowledge storage
- workflow/runtime orchestration
- transport/server ownership

Python retains only:

1. Arrow Flight client access
2. Arrow IPC fallback helpers
3. thin config/schema/logging helpers
4. package-local adapter and contract tests
5. pure cross-language workflow payload contracts

Retained runtime contract prefixes:

- `xiuxian.runtime.*`
- `xiuxian.router.*`
- `xiuxian.discover.*`
- `xiuxian_wendao.link_graph.*`

## Retained Package Set

```text
packages/python/
  wendao-core-lib/           Arrow Flight transport client
  wendao-arrow-interface/    downstream-facing Arrow facade with optional dataframe examples
  qianji-workflow-contracts/ pure workflow payload contracts for BPMN/DMN handoff
  xiuxian-wendao-analyzer/   analyzer workflows on top of the same substrate
  foundation/                thin config/schema/logging helpers
  core/                      minimal retained helper surface
```

The root workspace now includes the retained substrate packages
`wendao-core-lib`, `foundation`, and `core`, plus the public consumer facade
`wendao-arrow-interface` and the pure contract package
`qianji-workflow-contracts`. The beta analyzer package
`xiuxian-wendao-analyzer` remains an active adjacent consumer package rather
than part of the root default workspace surface.

Retained Python code now ships under direct top-level packages:

- `xiuxian_core`
- `xiuxian_foundation`
- `qianji_workflow_contracts`
- `wendao_core_lib`
- `wendao_arrow_interface`
- `xiuxian_wendao_analyzer`

The recommended downstream Arrow-consumer facade now lives under
`packages/python/wendao-arrow-interface/` as `wendao_arrow_interface`.
It is intentionally a composition layer over `wendao-core-lib`, not a new
transport owner.

The workflow contract package now lives under
`packages/python/qianji-workflow-contracts/` as
`qianji_workflow_contracts`. It owns stable JSON-safe exchange shapes for BPMN
host work, DMN decision payloads, dataset references, shared workflow
envelopes, and minimal execution references or status snapshots. It does not
own Arrow tables, Flight sessions, or execution logic.

The analyzer-layer package at
`packages/python/xiuxian-wendao-analyzer/` is now an active consumer package.
It stays outside the transport-substrate set, but it is no longer a mere
scaffold; it is the analyzer workflow layer above `wendao-core-lib`, focused
on analyzing rows and tables that Rust-owned query surfaces already returned.
Rerank transport remains owned by the transport and facade packages, not by
the analyzer package.

## Removed Surface

The historical Python runtime-center architecture is gone. This includes the
former `agent` package, `xiuxian_core.skills`, and the old Python-local router,
memory, workflow, knowledge-host, bindings, watcher, scanner, hot-reload, and
skill-runner stacks.

The old `src/omni/...` namespace layout is gone as well.

## Rules

1. Python is not a peer runtime center.
2. Arrow Flight is the default Python integration path.
3. Arrow IPC is the sanctioned fallback.
4. New Python code must stay transport-consumer-only or helper-only.
5. Pure contract packages may define stable exchange payloads, but they must
   not recreate execution, transport, or dataframe ownership behind the new
   boundary.
6. If Rust already owns a responsibility, Python must not recreate it behind a
   compatibility label.
7. Delete stale local-runtime surfaces rather than preserving them as legacy
   architecture.
8. Downstream ergonomics facades must compose retained transport helpers
   instead of taking transport ownership themselves.

## Documentation Notes

`P0_surface_inventory.md`, `P1_retirement_matrix.md`, and the developer guides
now describe only the retained Python scope. Historical references to deleted
Python runtime surfaces should be treated as archive material only.
