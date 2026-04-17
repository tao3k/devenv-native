---
type: knowledge
title: "RFC: Python Arrow Flight Boundary and Removal Program"
category: "rfc"
status: "implemented"
authors:
  - codex
created: 2026-03-29
tags:
  - rfc
  - python
  - arrow
  - flight
  - boundary
  - migration
metadata:
  title: "RFC: Python Arrow Flight Boundary and Removal Program"
---

# RFC: Python Arrow Flight Boundary and Removal Program

## 1. Summary

This RFC defines the remaining Python scope for the repository and turns that
scope into a staged removal program.

The decision is:

1. Python is not a host runtime center.
2. Python is not a knowledge-skill execution layer.
3. Python is not a local orchestration or discovery layer.
4. Python remains only as:
   - Arrow Flight transport client surface
   - Arrow IPC fallback surface
   - narrow config/schema/serialization helpers
   - narrow test adapters that validate retained transport contracts

Everything outside that boundary is delete-first unless a concrete Arrow/Flight
transport use case requires it to remain.

## 2. Alignment

This RFC is governed by:

1. [2026-03-27-wendao-arrow-plugin-flight-rfc.md](./2026-03-27-wendao-arrow-plugin-flight-rfc.md)
2. [2026-03-27-wendao-core-runtime-plugin-migration-rfc.md](./2026-03-27-wendao-core-runtime-plugin-migration-rfc.md)

The paired execution tracking also follows the active core/runtime/plugin
migration blueprint, but canonical RFCs do not link hidden workspace paths
directly.

The blueprint already fixes the architectural center:

1. Arrow is the canonical data plane.
2. DataFusion is the execution kernel.
3. Runtime ownership belongs outside language helper packages.
4. Flight is preferred and Arrow IPC is the sanctioned fallback.

This RFC applies those mandates specifically to the retained Python tree.

## 3. Problem Statement

The repository has already deleted large parts of the historical Python runtime
surface, but the remaining work can still drift if it is justified as isolated
cleanup instead of boundary enforcement.

Without an explicit Python boundary program:

1. local helper packages can quietly regrow into a parallel runtime shell
2. test-only helper directories can quietly regrow into a second utility
   platform instead of staying close to owning slices
3. knowledge and skill helper surfaces can survive under test labels even after
   the runtime architecture moved to Arrow/Flight
4. deletion work becomes fragmented and hard to audit

## 4. Goals

This RFC has the following goals:

1. define exactly what Python is still allowed to own
2. define exactly what Python must stop owning
3. group remaining deletions into phase gates
4. provide stop conditions so cleanup does not drift into unrelated churn
5. make GTD, ExecPlan, and future code deletions auditable against one boundary

## 5. Non-Goals

This RFC does not:

1. redesign Rust-owned runtime or routing behavior
2. replace the existing Wendao Arrow/Flight RFCs
3. require that every Python helper be deleted in one batch
4. force immediate removal of narrow transport-adjacent helpers with live uses

## 6. Retained Python Boundary

Python may retain only these categories:

1. Flight transport clients
2. Arrow IPC fallback serializers or adapters
3. concrete config/schema helpers required by retained transport clients
4. narrow package-local test helpers that directly verify retained transport or
   schema contracts
5. package metadata and packaging glue needed to ship those clients

Examples of retained scope:

1. `wendao_core_lib.transport`
2. narrow `xiuxian_foundation.config.*` helpers
3. narrow schema locator and serialization helpers used by retained clients
4. narrow test-local helpers under owning slices such as
   `packages/python/foundation/tests/unit/services` or `scripts/tests`

## 7. Delete-First Boundary

Python must not retain these categories:

1. local runtime orchestration
2. local routing and discovery layers
3. local command/decorator/registration systems
4. local skill execution shims
5. local knowledge-graph or knowledge-skill fixture layers
6. broad package-root convenience facades
7. local bridge abstractions that are not directly serving Arrow/Flight
8. broad standalone Python test-helper platforms that are not transport
   verification

Examples already inside the delete-first boundary:

1. runtime packages and bridge packages
2. local decorator and handler layers
3. the retired standalone Python test-helper package and its helper facades
4. the retired local RAG/knowledge fixture slices
5. package-root forwarding in `xiuxian_foundation` and `wendao_core_lib`

## 8. Package Classification

### 8.1 Retained Packages

1. `packages/python/wendao-core-lib`
   Purpose: Flight and Arrow IPC transport client
2. `packages/python/foundation`
   Purpose: narrow config/schema/serialization helpers and limited adapters
3. `packages/python/core`
   Purpose: minimal retained helper surface only if it still serves the kept
   Python client path

### 8.2 Retained Packages with Active Shrink Pressure

1. `foundation`
   Rule: no new runtime, bridge, local orchestration, or package-root facade
   surface
2. `core`
   Rule: no re-expansion into local skill or host-runtime glue

## 9. Macro Phases

### PY-P1: Public Surface Collapse

Goal:

1. remove package-root facades
2. remove forwarding `__init__` exports
3. force all callers onto concrete submodules

Exit gate:

1. no retained package root acts as a convenience facade
2. removal guards lock deleted roots and forwarding surfaces

### PY-P2: Local Runtime and Bridge Retirement

Goal:

1. delete runtime, bridge, decorator, handler, and local orchestration shells
2. move any genuinely retained helpers to narrower non-runtime homes

Exit gate:

1. no standalone Python runtime package remains
2. no standalone Python bridge package remains
3. no decorator or local handler design remains in active Python

### PY-P3: Knowledge and Test Surface Retirement

Goal:

1. delete knowledge-skill helper surfaces in Python
2. delete standalone Python test-helper platforms
3. move any still-retained test-only logic into owning slices and private
   test-local helpers

Exit gate:

1. no standalone Python test-helper package remains
2. no public knowledge or benchmark helper surface survives outside owning
   test slices
3. retained transport/contract helpers live only under owning test trees

### PY-P4: Boundary Closure

Goal:

1. confirm retained Python is only Flight/IPC plus narrow helpers
2. write the remaining inventory and stop conditions
3. reject new Python-local runtime surfaces by policy

Exit gate:

1. retained package list is stable
2. each retained module has a direct Arrow/Flight or schema/config rationale
3. remaining local helper modules have explicit keep reasons

## 10. Phase Deliverables

Each phase must produce:

1. concrete removals or narrowed surfaces
2. removal guards or focused tests
3. GTD synchronization
4. ExecPlan progress updates
5. a boundary check against this RFC

## 11. Acceptance Criteria

This RFC is considered operational when:

1. future Python deletions cite one of `PY-P1` to `PY-P4`
2. GTD entries and ExecPlan updates reference the same macro phase language
3. no remaining Python work is justified only as local cleanup if it advances
   the boundary

## 11.1 Operational Status

This RFC is now operationally complete.

The boundary program defined by `PY-P1` through `PY-P4` has been executed to
the point where:

1. broad Python runtime, bridge, knowledge, and test-helper surfaces have been
   retired
2. the standalone Python test-helper platform has been removed
3. retained Python scope is centered on `wendao-core-lib` plus narrow
   transport-adjacent helpers
4. active work has moved from boundary closure into Flight query-contract
   evolution

Follow-on work on repo-search and rerank request/response semantics is no
longer tracked as part of this boundary RFC. That work now belongs to the
successor query-contract RFC.

## 12. Open Questions

1. should any part of `packages/python/core` survive once Flight/IPC transport
   closure is complete?
2. which remaining `foundation` helpers are truly transport-adjacent versus
   historical convenience layers?

## 13. Decision

Adopt the Python boundary program immediately.

From this point:

1. Python work must be grouped by phase
2. deletions should be justified against the retained Arrow/Flight boundary
3. new Python runtime-center behavior is architecturally out of bounds

## 14. Completion Notes

This RFC should now be treated as the completed boundary baseline for Python.

Subsequent work should reference this RFC only when:

1. defending the retained Python boundary
2. rejecting new Python-local runtime regrowth
3. auditing whether a helper belongs inside or outside the retained boundary

Subsequent work should not use this RFC as the primary planning surface for:

1. repo-search request-contract growth
2. rerank request/response contract growth
3. real-host query semantics
4. Flight response-shape evolution
