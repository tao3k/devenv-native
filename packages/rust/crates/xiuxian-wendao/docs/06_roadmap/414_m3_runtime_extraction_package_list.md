# M3 Runtime Extraction Package List

:PROPERTIES:
:ID: wendao-m3-runtime-extraction-package-list
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: roadmap, migration, plugins, core, runtime, m3
:STATUS: ACTIVE
:END:

## Purpose

This note is the first concrete `M3` deliverable for the Wendao
core/runtime/plugin migration program.

It defines the package list for the first physical extraction of
`xiuxian-wendao-runtime`.

Primary references:

- `[[06_roadmap/412_core_runtime_plugin_program]]`
- `[[06_roadmap/413_m2_core_extraction_package_list]]`
- `[[06_roadmap/409_core_runtime_plugin_surface_inventory]]`
- `[[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md]]`

The active task-scoped blueprint for this migration remains recorded in the
paired ExecPlan rather than this persistent package-list note.

## M3 Goal

Create a physical `xiuxian-wendao-runtime` crate that owns host behavior.

This crate must become the physical home of:

1. launch orchestration
2. transport negotiation
3. config resolution
4. health and readiness
5. fallback routing
6. gateway and tool assembly glue

## Extraction Rule

For `M3`, a module belongs in `runtime` only if it is true that:

1. it resolves, negotiates, launches, routes, renders, or exports host behavior
2. it depends on environment, filesystem, process, network, or deployment
   context
3. it is not purely a stable contract record
4. it is not fundamentally plugin-owned language logic

If a module is primarily a contract record, it should move to `core`, not
`runtime`.

## Candidate Package List

### Package A: Runtime Config Resolution

Current source boundary:

- `src/link_graph/runtime_config/resolve/`
- `src/link_graph/runtime_config.rs`
- runtime-owned parts of `src/link_graph/runtime_config/models/`

Target `runtime` ownership:

- environment-variable resolution
- config-home and filesystem override handling
- provider-default application
- retrieval-policy resolution
- deployment-artifact resolution from live runtime state

Do not move:

- stable DTO/selector/descriptor records that belong in `core`
- plugin-owned Julia launch semantics

### Package B: Transport Negotiation and Runtime Clients

Current source boundary:

- runtime-owned parts of `src/link_graph/plugin_runtime/transport/`
- request/response transport client builders
- Arrow IPC/transport helper wiring

Target `runtime` ownership:

- transport client construction
- runtime endpoint negotiation
- fallback transport choice
- request execution helpers that depend on live runtime config

Do not move:

- transport kind enums and endpoint descriptor records
- stable capability/artifact descriptors

### Package C: Gateway and Studio Assembly

Current source boundary:

- `src/gateway/`
- `src/bin/wendao/execute/gateway/`

Target `runtime` ownership:

- gateway assembly
- Studio handler wiring
- OpenAPI route and path assembly
- gateway health/readiness behavior

Do not move:

- stable outward record shapes that should remain contract-owned
- generic artifact record definitions

### Package D: Zhenfa and Native Tool Assembly

Current source boundary:

- `src/zhenfa_router/`

Target `runtime` ownership:

- native tool routing
- RPC parameter decoding
- deployment artifact export orchestration
- compat-path tool execution

Do not move:

- stable artifact payload record types
- plugin-owned deployment semantics

### Package E: Host Plugin Registry and Bootstrap

Current source boundary:

- `src/analyzers/service/bootstrap.rs`
- `src/analyzers/service/`
- runtime-facing registry wiring in `src/bin/wendao/`

Target `runtime` ownership:

- plugin registry bootstrap
- builtin provider registration
- host integration wiring

Do not move:

- stable analysis/plugin contracts
- plugin-owned language implementations

## Explicit Non-Runtime List

These boundaries must remain out of `M3`:

### Core-Owned

1. plugin ids and selectors
2. capability/artifact/transport descriptors
3. launch-spec and artifact-payload records
4. compatibility export maps that re-export stable records

### Plugin-Owned or Future Plugin-Owned

1. Julia launch details and package-owned defaults
2. Julia deployment artifact semantics that are not generic host contracts
3. remaining sibling-source inclusion hacks that must be eliminated during
   `M4` after the Julia path has been cut over to a normal crate dependency

### Deferred Until After M3

1. final removal of Julia compatibility shims
2. second-plugin onboarding
3. fully generic outward endpoint-only world

## First Physical Crate Cut

The first physical `xiuxian-wendao-runtime` crate should aim to contain only:

1. runtime-config resolution
2. transport client construction and negotiation
3. gateway and Zhenfa host assembly
4. plugin registry/bootstrap wiring

This first cut should not attempt to absorb plugin-owned Julia logic.

Current implementation status:

1. `packages/rust/crates/xiuxian-wendao-runtime/` now exists as a physical
   crate in the workspace
2. the first cut currently contains only the runtime-owned transport-client
   construction slice under `src/transport/`
3. the next runtime-owned helper slices now also exist in the runtime crate:
   generic artifact resolve/render helpers live under `src/artifacts/`
4. `xiuxian-wendao` now depends on `xiuxian-wendao-runtime` and forwards its
   `julia` feature into the runtime crate
5. `src/link_graph/plugin_runtime/transport/client.rs` in the main crate now
   re-exports the transport-client builder from `xiuxian-wendao-runtime`
6. `src/link_graph/plugin_runtime/artifacts/render.rs` in the main crate now
   delegates generic render behavior to `xiuxian-wendao-runtime` while keeping
   local resolver ownership
7. `src/link_graph/plugin_runtime/artifacts/resolve.rs` in the main crate now
   delegates generic selector-to-payload resolution behavior to
   `xiuxian-wendao-runtime` while keeping runtime-state lookup local
8. `packages/rust/crates/xiuxian-wendao-runtime/src/settings/` now owns the
   generic runtime-config settings helpers, including override state, config
   file merge behavior, scalar/sequence access helpers, and directory/parse
   utilities
9. `src/link_graph/runtime_config/settings/mod.rs` in the main crate now keeps
   only the Wendao-embedded TOML/source-path wrapper while re-exporting the
   migrated helper surface from `xiuxian-wendao-runtime`
10. `packages/rust/crates/xiuxian-wendao-runtime/src/runtime_config/` now owns
    the first live-resolution sub-boundary too: cache, related, coactivation,
    and index-scope runtime records/constants/resolvers live there as a
    coherent runtime-owned slice
11. `src/link_graph/runtime_config/models/{cache,related,coactivation,index}.rs`
    and `src/link_graph/runtime_config/resolve/{cache,related,coactivation,index_scope}.rs`
    in the main crate now preserve the old module paths as thin compatibility
    wrappers or re-export seams over the runtime crate
12. `packages/rust/crates/xiuxian-wendao-runtime/src/runtime_config/resolve/agentic/`
    and `src/runtime_config/models/agentic.rs` now own the full agentic
    runtime subtree too: records, env/default constants, apply/finalize logic,
    and the settings-to-runtime resolver all live in the runtime crate
13. `src/link_graph/runtime_config/resolve/agentic/` in the main crate is now
    reduced to a settings-backed compatibility wrapper, and the old
    monolithic-crate `apply/` and `finalize.rs` implementation files have been
    physically removed
14. `packages/rust/crates/xiuxian-wendao-runtime/src/runtime_config/retrieval/semantic_ignition.rs`
    now owns the generic semantic-ignition retrieval record, env/default
    constants, and settings-to-runtime resolver, while
    `src/link_graph/runtime_config/models/retrieval/semantic_ignition.rs` and
    `src/link_graph/runtime_config/resolve/policy/retrieval/semantic_ignition.rs`
    in the main crate are reduced to re-export seams
15. `packages/rust/crates/xiuxian-wendao-runtime/src/runtime_config/retrieval/base.rs`
    now owns the generic retrieval tuning slice too: candidate multiplier,
    max sources, graph sufficiency thresholds, graph rows per source, and the
    semantic-ignition integration path now resolve in the runtime crate,
    while `xiuxian-wendao` keeps only the `mode + julia_rerank` wrapper
    assembly in `resolve/policy/retrieval/base.rs`
16. broader runtime-owned cutover is still pending for config resolution,
    gateway assembly, Zhenfa routing, and bootstrap wiring

## Compatibility Plan

During `M3`:

1. `xiuxian-wendao` may re-export runtime-owned entrypoints temporarily
2. binaries may keep delegating through the monolithic crate while ownership
   moves physically
3. gateway and tool compatibility surfaces may remain, but should become
   runtime-owned wrappers

`M3` is complete only when orchestration ownership is physically moved out of
the monolithic crate boundary.

## Acceptance Criteria

This package list is ready when:

1. each runtime package has a clear include list
2. each runtime package has a clear non-goal list
3. `runtime` extraction can begin without reopening scope arguments
4. the first physical runtime crate cut is intentionally narrow and buildable

## Immediate Follow-Up

After this note lands, the next program artifact should be:

1. Julia externalization package list

At that point, `M2`, `M3`, and `M4` will all have package-level implementation
entrypoints.

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/412_core_runtime_plugin_program|Wendao Core Runtime Plugin Program]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---
