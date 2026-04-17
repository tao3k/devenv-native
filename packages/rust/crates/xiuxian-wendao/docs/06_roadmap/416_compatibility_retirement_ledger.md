# Compatibility Retirement Ledger

:PROPERTIES:
:ID: wendao-compatibility-retirement-ledger
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: roadmap, migration, plugins, compatibility, ledger
:STATUS: ACTIVE
:END:

## Purpose

This note is the final missing program artifact for the Wendao
core/runtime/plugin migration program.

It defines what compatibility surfaces remain, why they remain, what phase
unlocks their retirement, and what the target retirement state is.

Primary references:

- `[[06_roadmap/412_core_runtime_plugin_program]]`
- `[[06_roadmap/409_core_runtime_plugin_surface_inventory]]`
- `[[06_roadmap/413_m2_core_extraction_package_list]]`
- `[[06_roadmap/414_m3_runtime_extraction_package_list]]`
- `[[06_roadmap/415_m4_julia_externalization_package_list]]`

## Retirement Rule

A compatibility surface may be retired only when:

1. its replacement ownership is physically landed
2. its replacement import or endpoint path is documented
3. the compatibility shim no longer carries primary implementation meaning
4. the relevant macro phase exit criteria are satisfied

## Retirement Ledger

| Compatibility surface                       | Current location                                                          | Retirement unlock                                       | Target retirement state                                                                                             |
| :------------------------------------------ | :------------------------------------------------------------------------ | :------------------------------------------------------ | :------------------------------------------------------------------------------------------------------------------ |
| Package-owned Julia compatibility namespace | `packages/rust/crates/xiuxian-wendao-julia/src/compatibility/link_graph/` | post-`M5` downstream import cleanup in plugin consumers | Retire package-owned legacy Julia DTO imports once downstream users no longer need the compatibility naming surface |

## Retirement Order

The expected retirement order is now:

1. top-level crate re-export retirement
2. Studio/OpenAPI/Zhenfa outward compatibility retirement
3. host crate-root compatibility namespace retirement
4. package-owned Julia compatibility import retirement

This order keeps the widest public surfaces shrinking first and the narrow
regression seams shrinking last.

## Protected Compatibility Surfaces

The following compatibility seams should remain protected until their unlock
phase is complete:

1. `packages/rust/crates/xiuxian-wendao-julia/src/compatibility/link_graph/`

No new host-owned primary implementation logic may be reintroduced behind a
crate-root compatibility namespace in `xiuxian-wendao`.

## Current M5 Status

The active `M5` status is now:

1. Studio routing and OpenAPI inventory expose only the canonical generic
   plugin-artifact endpoint at
   `/api/ui/plugins/{plugin_id}/artifacts/{artifact_id}`
2. the former Studio compat deployment-artifact route, its OpenAPI constants,
   and its query/handler glue are retired from the live tree
3. the canonical Studio schema-export seam matches that same rule:
   `studio_type_collection()` exports only the generic artifact types
4. the Julia-named Studio route/query shims, DTO symbols, and compatibility
   type leaves are retired from code entirely
5. Zhenfa now exposes only `wendao.plugin_artifact` as the live generic
   tool/RPC artifact surface
6. the Julia Zhenfa outward tool name, test-only RPC shim, native
   compatibility helper folder, and former Julia helper/type aliases are all
   retired from code
7. the top-level crate, `src/link_graph/mod.rs`, and `runtime_config` flat
   compat re-export blocks are retired
8. the former host crate-root compatibility namespace is retired too:
   `src/compatibility/link_graph.rs`, `src/compatibility/mod.rs`, and the
   `pub mod compatibility;` mount are all gone from the live tree
9. the remaining retirement work is therefore centered on package-owned Julia
   compatibility import cleanup rather than on any remaining host UI, Zhenfa,
   or crate-root artifact endpoint

## Completion Condition

The compatibility retirement program is complete when:

1. Julia-named host surfaces are compatibility-only or removed
2. generic plugin-artifact and plugin-capability surfaces are canonical
3. `xiuxian-wendao-julia` owns Julia-specific meaning physically
4. no migration blocker still depends on the monolithic crate boundary

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/412_core_runtime_plugin_program|Wendao Core Runtime Plugin Program]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]]
:END:

---
