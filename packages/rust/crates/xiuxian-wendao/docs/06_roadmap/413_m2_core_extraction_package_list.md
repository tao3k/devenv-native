# M2 Core Extraction Package List

:PROPERTIES:
:ID: wendao-m2-core-extraction-package-list
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: roadmap, migration, plugins, core, runtime, m2
:STATUS: ACTIVE
:END:

## Purpose

This note is the first concrete `M2` deliverable for the Wendao
core/runtime/plugin migration program.

It defines the package list for the first physical extraction of
`xiuxian-wendao-core`.

Primary references:

- `[[06_roadmap/412_core_runtime_plugin_program]]`
- `[[06_roadmap/409_core_runtime_plugin_surface_inventory]]`
- `[[06_roadmap/410_p1_generic_plugin_contract_staging]]`
- `[[06_roadmap/411_p1_first_code_slice_plan]]`
- `[[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md]]`

The active task-scoped blueprint for this migration remains recorded in the
paired ExecPlan rather than this persistent package-list note.

## M2 Goal

Create a physical `xiuxian-wendao-core` crate that owns stable contracts only.

This crate must be usable without:

1. gateway assembly
2. plugin process lifecycle
3. runtime config discovery
4. transport fallback orchestration
5. language-specific deployment ownership

## Extraction Rule

For `M2`, a module belongs in `core` only if it is true that:

1. it defines a stable host contract, record, identifier, schema, or manifest
2. it does not require runtime lifecycle ownership
3. it does not require gateway-, Studio-, or Zhenfa-specific wrapping
4. it does not encode Julia-specific deployment or transport defaults as
   primary ownership

If any of those fail, the module is not part of the first `core` cut.

## Candidate Package List

### Package A: Plugin Runtime Contract Types

Current source boundary:

- `src/link_graph/plugin_runtime/ids/`
- `src/link_graph/plugin_runtime/capabilities/`
- `src/link_graph/plugin_runtime/artifacts/`
- the contract-only parts of `src/link_graph/plugin_runtime/transport/`

Target `core` ownership:

- plugin ids
- capability ids
- artifact selectors
- capability descriptors
- launch-spec records
- artifact payload records
- transport endpoint descriptors
- transport kind enums

Do not move yet:

- transport client builders
- host-side resolution helpers
- Julia compatibility shims

### Package B: Runtime Config Contract Records

Current source boundary:

- `src/link_graph/runtime_config/models/retrieval/julia_rerank/`
- `src/link_graph/runtime_config/models/retrieval/semantic_ignition.rs`
- `src/link_graph/runtime_config/models/retrieval/policy.rs`

Target `core` ownership:

- compat-first DTO aliases only if they represent stable contracts
- generic retrieval-policy records that do not require runtime resolution
- generic provider binding records

Do not move yet:

- environment-variable resolution
- filesystem/config-home resolution
- provider-default application
- Julia deployment helper shims

### Package C: Artifact Contract Surface

Current source boundary:

- contract-only parts of `src/link_graph/plugin_runtime/artifacts/`
- compat-first artifact DTO aliases used as stable records

Target `core` ownership:

- plugin artifact selector records
- plugin artifact payload records
- renderable artifact record shape contracts

Do not move yet:

- artifact resolution from runtime config
- file export helpers with runtime-owned path semantics
- gateway/Studio outward wrappers

### Package D: Compatibility Export Map

Current source boundary:

- `src/compatibility/`

Target `core` ownership:

- compat-first crate-root export map
- legacy Julia compatibility namespace map

Rule:

`core` may own the compatibility export namespaces only as re-export surfaces.
Implementation logic must remain in the owning modules.

### Package E: Repo Intelligence Contract Surface

Current source boundary:

- `src/analyzers/config/`
- `src/analyzers/plugin.rs`
- `src/analyzers/records.rs`
- `src/analyzers/errors.rs`
- `src/analyzers/registry.rs`
- contract-only enum ownership under `src/analyzers/projection/contracts.rs`

Target `core` ownership:

1. repo-intelligence config records
2. repo-intelligence plugin traits and context/output records
3. repo-intelligence record types
4. repo-intelligence registry
5. stable projection contract enums needed by public error types
6. repo-intelligence error types
7. Julia Arrow analyzer column-name contracts
8. Julia Arrow request/response schema builders that express stable transport
   shape only

Do not move yet:

1. analyzer bootstrap and builtin registration
2. analyzer projection implementation ownership beyond stable contract enums
3. host-side transport client builders
4. any runtime lifecycle or filesystem-discovery behavior

## Explicit Non-Core List

These boundaries must remain out of `M2`:

### Runtime-Owned

1. `src/link_graph/runtime_config/resolve/`
2. `src/gateway/`
3. `src/zhenfa_router/`
4. `src/bin/wendao/`
5. transport client construction under `plugin_runtime/transport/`
6. launch, readiness, fallback, and routing codepaths

### Plugin-Owned or Future Plugin-Owned

1. Julia launcher defaults and package-owned deployment assembly
2. Julia artifact semantics that are not generic contract records
3. any sibling-source inclusion path

### Deferred Until After M2

1. analyzer registry bootstrap
2. gateway OpenAPI composition
3. Studio compatibility DTOs
4. Zhenfa compatibility tool wrappers

## First Physical Crate Cut

The first physical `xiuxian-wendao-core` crate should aim to contain only:

1. plugin ids and selectors
2. capability/artifact/transport descriptors
3. launch/artifact payload records
4. compat-first export maps that re-export those records

This first cut should be intentionally narrow.

It should not try to absorb all retrieval or query records in one move.

Current implementation status:

1. `packages/rust/crates/xiuxian-wendao-core/` now exists as a physical crate
2. the first cut currently contains:
   - ids
   - capability binding and selector records
   - launch/artifact payload records
   - transport endpoint and kind records
3. the crate is currently added to the workspace and builds independently
4. main-crate consumer cutover has now started for the first contract slice:
   `xiuxian-wendao` plugin-runtime record modules now re-export their stable
   contract types from `xiuxian-wendao-core`
5. the `plugin_runtime/mod.rs` main export map now also exposes those contract
   records directly from `xiuxian-wendao-core`, so the first slice reaches
   both leaf record modules and the primary host-side plugin-runtime barrel
6. `runtime_config` pure contract imports have now started moving onto
   `xiuxian-wendao-core`, specifically the Julia rerank selector, binding,
   launch, artifact, and transport record dependencies under
   `models/retrieval/julia_rerank/` and `models/retrieval/policy.rs`
7. the next outward pure-contract consumer slice has now also started:
   `runtime_config.rs` and the Studio UI type surfaces now source their stable
   launch, artifact, and binding records from `xiuxian-wendao-core`
8. the next helper-consumer cutover has also started: helper modules that
   still own runtime behavior but only consume stable contract records now
   source those records from `xiuxian-wendao-core`, including
   `plugin_runtime/artifacts/resolve.rs`,
   `plugin_runtime/transport/client.rs`, and the quantum rerank flow modules
9. `plugin_runtime` subtree cleanup has also started for the remaining
   selector/enum consumers, including `artifacts/render.rs` and the focused
   compatibility regression tests that only need stable transport-kind records
10. a new `repo_intelligence/` feature folder now exists in
    `xiuxian-wendao-core` and owns the first analyzer contract slice:
    config records, plugin traits/context/output, public repo-intelligence
    records, registry, `RepoIntelligenceError`, the stable
    `ProjectionPageKind` enum, and the Julia Arrow analyzer transport
    column/schema contracts
11. the main-crate analyzer contract modules now behave as re-export seams for
    that slice, including `src/analyzers/config/types.rs`,
    `src/analyzers/plugin.rs`, `src/analyzers/records.rs`,
    `src/analyzers/errors.rs`, `src/analyzers/registry.rs`, and the enum
    ownership under `src/analyzers/projection/contracts.rs`
12. `xiuxian-julia-core` now consumes repo-intelligence contracts from
    `xiuxian-wendao-core`, and that now includes the Julia Arrow analyzer
    transport contract; the Julia package no longer depends on the monolithic
    host crate directly
13. broader consumer cutover is still pending outside these first
    `plugin_runtime`, `runtime_config`, Studio type, helper-consumer, and
    analyzer-contract slices

## Compatibility Plan

During `M2`:

1. `xiuxian-wendao` may re-export `xiuxian-wendao-core` types
2. `src/compatibility/` may remain as the temporary crate-root compatibility
   map
3. Julia-named top-level exports may remain deprecated compatibility shims

`M2` is complete only when the ownership move is physical, not only logical.

## Acceptance Criteria

This package list is ready when:

1. each candidate package has a clear include list
2. each candidate package has a clear non-goal list
3. `core` extraction can begin without reopening scope arguments
4. the first physical crate cut is intentionally narrow and buildable

## Immediate Follow-Up

After this note lands, the next program move should be:

1. continue `M2` consumer cutover beyond the first plugin-runtime records
2. use analyzer-contract extraction as the next `M2` slice that directly
   reduces `M4` Julia-package coupling

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/412_core_runtime_plugin_program|Wendao Core Runtime Plugin Program]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---
