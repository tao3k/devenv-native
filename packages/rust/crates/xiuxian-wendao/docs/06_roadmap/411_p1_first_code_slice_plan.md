# P1 First Code Slice Plan

:PROPERTIES:
:ID: wendao-p1-first-code-slice-plan
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: roadmap, migration, plugins, runtime, contracts, p1, implementation
:STATUS: ACTIVE
:END:

## Mission

This note turns the `P1` contract staging into a concrete first code slice.

Primary references:

- `[[06_roadmap/410_p1_generic_plugin_contract_staging]]`
- `[[06_roadmap/409_core_runtime_plugin_surface_inventory]]`
- `[[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md]]`

The active task-scoped blueprint for this migration remains recorded in the
paired ExecPlan rather than this persistent code-slice note.

The scope of this slice is intentionally narrow:

1. introduce the first generic runtime-facing type families in feature-folder
   form
2. add compatibility conversions from the existing Julia-specific surfaces
3. keep external behavior unchanged

This slice does not yet split crates.

## Implementation Checkpoint

Current 2026-03-27 checkpoint:

1. `src/link_graph/plugin_runtime/` now exists as a dedicated feature tree with
   `ids/`, `transport/`, `capabilities/`, `artifacts/`, and `compat/` ownership.
2. generic runtime-facing records now exist for:
   - plugin identifiers
   - capability binding
   - transport endpoint and kind
   - launch specification
   - artifact payload
3. the current Julia compatibility path now converts:
   - `LinkGraphJuliaRerankRuntimeConfig` into `PluginCapabilityBinding`
   - `LinkGraphJuliaDeploymentArtifact` into `PluginArtifactPayload`
4. legacy Julia deployment-artifact helpers in `runtime_config.rs` now
   delegate through the generic artifact-resolution path.
5. focused tests now cover the new feature seam under
   `src/link_graph/plugin_runtime/tests/`.
6. gateway, OpenAPI, and Zhenfa deployment-artifact surfaces now route through
   the generic plugin-artifact helpers while preserving the legacy Julia
   outward names.
7. retrieval runtime configuration can now normalize the current rerank
   provider into a generic `PluginCapabilityBinding` through
   `resolve_link_graph_rerank_binding()`, so later call sites no longer need to
   depend directly on `LinkGraphJuliaRerankRuntimeConfig`.
8. the first retrieval hot path in
   `link_graph/index/search/plan/payload/quantum/` now consumes the generic
   rerank binding internally while preserving the current Julia transport and
   telemetry behavior.
9. Arrow transport-client construction for rerank execution now lives under
   `plugin_runtime/transport/`, so retrieval call sites no longer need to own
   transport-config assembly logic directly.
10. `link_graph/index/search/plan/payload/quantum.rs` has now been replaced by
    the `payload/quantum/` feature folder with interface-only `mod.rs` plus
    dedicated `flow.rs`, `rerank.rs`, and `tests.rs` ownership, so the first
    retrieval seam now follows the migration package modularization gates
    instead of continuing as a mixed-responsibility flat file.
11. the semantic-ignition adapters under `link_graph/index/search/quantum_fusion/`
    now expose `build_plugin_rerank_request_batch(...)` as the primary request
    builder name, while `build_julia_rerank_request_batch(...)` remains as a
    compatibility shim; `payload/quantum/rerank.rs` now consumes the generic
    builder name.
12. the semantic-ignition adapter request errors now use
    `*PluginRerankRequestError` as the primary type names, while
    `*JuliaRequestError` remains as a compatibility alias on the legacy shim
    surface.
13. analyzer transport now exposes `PluginArrowRequestRow` and
    `build_plugin_arrow_request_batch(...)` as the primary request-contract
    names; the Julia-named request row and builder remain as compatibility
    aliases and shim entrypoints, and `quantum_fusion/` now consumes the
    generic request vocabulary.
14. analyzer transport now exposes `PluginArrowScoreRow`,
    `decode_plugin_arrow_score_rows(...)`, and
    `fetch_plugin_arrow_score_rows_for_repository(...)` as the primary
    response-contract names; the Julia-named response row and helpers remain
    as compatibility alias/shim surfaces, and `payload/quantum/` now consumes
    the generic response vocabulary.
15. `analyzers/service/julia_transport.rs` has now been replaced by the
    `analyzers/service/julia_transport/` feature folder with interface-only
    `mod.rs` plus dedicated `schema.rs`, `request.rs`, `response.rs`,
    `fetch.rs`, `errors.rs`, and `tests.rs` ownership, so analyzer transport
    now follows the same modularization gate as the retrieval seams.
16. `link_graph/runtime_config/resolve/policy.rs` has now been replaced by the
    `runtime_config/resolve/policy/` feature folder with interface-only
    `mod.rs`, and retrieval resolution now lives under the dedicated
    `runtime_config/resolve/policy/retrieval/` feature folder with
    interface-only `mod.rs` plus leaf ownership for `base.rs`,
    `semantic_ignition.rs`, and `provider.rs`, so the retrieval-policy
    resolver no longer shares one flat file boundary with unrelated runtime
    concerns and the retrieval namespace itself now satisfies the same
    interface-only `mod.rs` rule.
17. `LinkGraphJuliaRerankRuntimeConfig` now exposes provider/capability-first
    primary helpers such as `provider_launch_descriptor()`,
    `plugin_launch_spec()`, `plugin_artifact_payload()`, and
    `rerank_provider_binding()`, while the Julia-named helper methods remain
    as compatibility wrappers over the same runtime records.
18. `runtime_config/models/retrieval.rs` has now been replaced by the
    `runtime_config/models/retrieval/` feature folder with interface-only
    `mod.rs` plus dedicated `policy.rs`, `semantic_ignition.rs`, and
    `julia_rerank/` ownership, so the new retrieval/runtime record surface
    does not regress into another mixed-responsibility warehouse file.
19. `plugin_runtime/compat/julia.rs` now acts as a thin compatibility shim:
    Julia plugin ids and conversion/build ownership have moved into
    `runtime_config/models/retrieval/julia_rerank/`, while the compat layer
    only re-exports that ownership for existing generic-call sites.
20. outward deployment-artifact call sites now route through a typed
    `PluginArtifactSelector` plus `julia_deployment_artifact_selector()`
    helper, so gateway, Zhenfa, runtime-config wrappers, and plugin-runtime
    tests no longer directly stitch Julia plugin/artifact constants together.
21. the Julia rerank capability path now exposes
    `julia_rerank_provider_selector()` and uses that typed selector in the
    generic binding builder and focused tests, so rerank capability routing is
    now symmetric with the deployment-artifact selector seam.
22. Julia runtime-to-artifact payload assembly now also routes through
    `julia_deployment_artifact_selector()`, and the focused runtime tests now
    assert against that typed selector instead of duplicating raw Julia
    plugin/artifact strings in host-side checks.
23. the Julia rerank binding builder now exposes
    `build_rerank_provider_binding(...)` as the primary helper name, while
    `build_plugin_capability_binding_for_julia_rerank(...)` remains as a thin
    compatibility shim; direct host call sites and focused tests now consume
    the provider-oriented primary helper.
24. raw `JULIA_*` constants are no longer re-exported from the higher-level
    `plugin_runtime` or `runtime_config::models` host surfaces; selector/helper
    seams are now the primary host-facing integration entrypoints, while raw
    ids remain scoped to the Julia compatibility ownership layer.
25. the legacy
    `build_plugin_capability_binding_for_julia_rerank(...)` shim is no longer
    re-exported from higher-level host surfaces; it now remains only inside
    the Julia ownership seam with a focused compatibility test, while
    `build_rerank_provider_binding(...)` is the only high-level host-facing
    binding-builder entrypoint.
26. `LinkGraphJuliaAnalyzerServiceDescriptor` is no longer re-exported from
    the higher-level `runtime_config`, `link_graph`, or crate-root host
    surfaces; it remains only in the Julia ownership seam, narrowing the
    first Julia-named record that is no longer needed as a general host-facing
    contract.
27. `gateway/studio/types/config.rs` now builds Julia-compatible UI payloads
    directly from `PluginLaunchSpec` and `PluginArtifactPayload`; the Studio
    type layer no longer depends directly on Julia runtime DTOs in order to
    expose the legacy Julia UI compatibility surface.
28. the Julia deployment-artifact HTTP handler now converts resolved generic
    `PluginArtifactPayload` values directly into
    `UiJuliaDeploymentArtifact`, removing the extra `UiPluginArtifact`
    hop from that outward JSON path.
29. `gateway::studio::types` no longer re-exports
    `UiJuliaAnalyzerLaunchManifest` from its higher-level module surface; the
    Julia launch-manifest UI DTO now stays behind the config ownership seam
    while the outward Studio API surface continues to expose the Julia
    deployment artifact and generic plugin UI types.
30. `UiJuliaDeploymentArtifact` is now also consumed through the
    `gateway::studio::types::config` ownership seam for crate-internal call
    sites; the higher-level `gateway::studio::types` surface no longer needs
    to re-export that Julia-specific UI DTO.
31. `LinkGraphJuliaAnalyzerLaunchManifest`,
    `LinkGraphJuliaDeploymentArtifact`, and
    `LinkGraphJuliaRerankRuntimeConfig` no longer pass through the
    `link_graph` middle-layer re-export surface; crate-root exports are now
    wired directly from the `runtime_config` ownership seam, keeping the
    crate-root API stable while shrinking the intermediate host surface.
32. `runtime_config::models` no longer re-exports
    `LinkGraphJuliaAnalyzerLaunchManifest` or
    `LinkGraphJuliaDeploymentArtifact`; `runtime_config.rs` now re-exports
    those Julia compatibility DTOs directly from the narrower
    `models::retrieval` ownership seam instead of the broader models surface.
33. `LinkGraphJuliaRerankRuntimeConfig` no longer re-exports from the
    higher-level `runtime_config::models` surface either; crate-internal call
    sites now use `models::retrieval`, while `runtime_config.rs` continues to
    expose the public compatibility runtime record from that narrower seam.
34. gateway OpenAPI and route inventory now use
    `API_UI_COMPAT_DEPLOYMENT_ARTIFACT_*` as the primary internal constants
    for the legacy Julia deployment-artifact endpoint, while the older
    `API_UI_JULIA_DEPLOYMENT_ARTIFACT_*` names remain as thin compatibility
    aliases.
35. integration fixtures and support utilities that still need Julia-backed
    retrieval behavior can now consume the compat DTO aliases
    (`LinkGraphCompat*`) as their primary vocabulary, while the remaining
    legacy Julia-named shims and route aliases are gated behind focused
    compatibility seams and `#[cfg(test)]` where they are only needed for
    regression coverage, keeping the touched host-side scope warning-free
    under the Julia feature build.
36. the Studio capability handler layer no longer re-exports the
    Julia-named deployment query/handler shim from the higher-level
    `handlers::capabilities` module; the legacy alias coverage now lives in
    the `deployment.rs` ownership test module itself, so the compat handler
    path remains the only middle-layer entrypoint while the Julia shim stays
    available for focused regression checks.
37. the Julia deployment-artifact helper functions no longer pass through the
    `link_graph` middle-layer re-export surface either; crate-root keeps the
    legacy compatibility exports by wiring them directly from
    `runtime_config`, while `link_graph` itself now exposes only the compat
    deployment helper path at its higher-level host surface.
38. the remaining focused host-side test fixtures for generic rerank binding
    and transport override validation now construct their runtime inputs with
    `LinkGraphCompatRerankRuntimeConfig` rather than the Julia-named alias, so
    even the narrow internal regression seams now default to compatibility-first
    vocabulary unless they are explicitly validating alias behavior.
39. crate-root and `runtime_config` now organize their public exports in a
    compatibility-first block followed by an explicit legacy Julia
    compatibility block, so the outward API remains stable while the primary
    reading path for downstream callers now centers the generic compat
    vocabulary before the Julia-named aliases.
40. `zhenfa_router` and `zhenfa_router::native` now expose
    `WendaoCompatDeploymentArtifactTool` as the primary higher-level tool type
    name, while the macro-generated Julia-named tool type remains behind the
    deployment ownership seam as the concrete implementation artifact.
41. the user-visible Zhenfa deployment-artifact descriptions, success
    messages, and compat-path error messages now use compatibility-oriented
    wording, while the legacy Julia shim remains available for focused
    regression coverage and the stable tool name stays unchanged.
42. the Julia-named OpenAPI route-contract alias no longer exists at the
    `shared::inventory` route-contract level; the legacy Julia route path is
    now preserved only as an API-path constant alias, while the inventory
    surface itself retains only the compat deployment-artifact contract.
43. the remaining Julia-named UI route-path constants in `gateway/openapi`
    are now grouped under an explicit legacy compatibility section, so the
    compat deployment-artifact path remains the primary reading path even
    where the stable Julia API-path aliases must continue to exist.
44. the compatibility deployment-artifact JSON-RPC failure wording in
    `zhenfa_router/rpc.rs` now also uses compatibility-oriented language, so
    the compat deployment path is consistent across Zhenfa native rendering,
    JSON-RPC transport, and Studio/OpenAPI gateway surfaces.
45. the remaining Julia-facing documentation comments in the Studio UI config
    types and Zhenfa deployment helpers now describe those surfaces as legacy
    Julia-compatible compatibility shims rather than primary Julia-specific
    contracts, keeping the implementation names stable while the default
    architectural narrative stays compat-first.
46. `link_graph/runtime_config/compatibility/` now exists as an explicit
    feature folder for the remaining Julia-named runtime DTOs and helper
    functions, so the main `runtime_config.rs` seam no longer owns those
    legacy symbols directly and instead re-exports them from a dedicated
    compatibility namespace.
47. `gateway/studio/types/compatibility/` was introduced as a temporary
    feature folder for `UiJulia*` compatibility DTOs, but the current `M5`
    state has already retired that type leaf and moved the remaining compat
    JSON adaptation back into the Studio deployment handler.
48. `zhenfa_router/native/compatibility/` now exists as the explicit feature
    folder for the remaining test-only `WendaoJulia*` deployment alias/shim
    surface, so `native/deployment.rs` keeps only the compat-first
    implementation path plus the macro-generated tool type while the legacy
    Julia test seam is owned separately.
49. `src/compatibility/` now exists as an explicit crate-root feature folder,
    and its surviving live surface is `src/compatibility/link_graph.rs`.
50. the former `src/compatibility/julia.rs` bridge was useful as a temporary
    migration seam, but the current `M5` state has already retired it from the
    tree.
51. the crate-root `LinkGraphCompat*` runtime-config exports now route through
    `src/compatibility/link_graph.rs`, so the compat-first runtime-config
    export path remains physically separated from the primary host entry
    modules.

This means the first code slice is active in the tree, even though the gateway,
OpenAPI, and Zhenfa compatibility shims have not yet been moved to generic
artifact helpers.

## Intended Code Outcome

After this first code slice lands, the tree should support both of these
statements:

1. Wendao can still serve the current Julia deployment artifact and rerank flow
   without user-visible regression.
2. New host logic can be written against generic plugin capability and artifact
   types instead of `LinkGraphJulia*` types.

## First New Module Tree

The first new module tree should be introduced in-place under the current
`xiuxian-wendao` crate.

Suggested runtime-facing layout:

```text
packages/rust/crates/xiuxian-wendao/src/link_graph/plugin_runtime/
  mod.rs
  ids.rs
  transport/
    mod.rs
    kind.rs
    endpoint.rs
  capabilities/
    mod.rs
    binding.rs
    selector.rs
    version.rs
  artifacts/
    mod.rs
    payload.rs
    launch.rs
    resolve.rs
    render.rs
  compat/
    mod.rs
    julia.rs
```

Module responsibilities:

1. `ids.rs`
   - `PluginId`
   - `CapabilityId`
   - `ArtifactId`
2. `transport/`
   - transport kind and endpoint payloads
3. `capabilities/`
   - generic provider selection and capability binding
4. `artifacts/`
   - generic launch spec and artifact payload
   - generic resolver and renderer entrypoints
5. `compat/julia.rs`
   - one-way conversions from Julia-named legacy types into the generic model

`mod.rs` files must remain interface-only.

## Existing Files to Touch First

The first code slice should touch these files in this order:

1. `packages/rust/crates/xiuxian-wendao/src/link_graph/mod.rs`
   - add a new `plugin_runtime` feature folder export seam
2. `packages/rust/crates/xiuxian-wendao/src/link_graph/plugin_runtime/...`
   - add the new generic runtime-facing types
3. `packages/rust/crates/xiuxian-wendao/src/link_graph/runtime_config.rs`
   - rewrite top-level Julia artifact resolver helpers as compatibility shims
     over the generic artifact resolver
4. `packages/rust/crates/xiuxian-wendao/src/link_graph/runtime_config/models/`
   - keep the existing Julia structs, but add conversions into the generic
     binding and artifact payload types behind a feature-folder namespace
5. `packages/rust/crates/xiuxian-wendao/src/gateway/studio/types/config.rs`
   - add generic UI artifact payload types while keeping Julia DTO shims
6. `packages/rust/crates/xiuxian-wendao/src/gateway/openapi/paths/ui.rs`
   - add generic plugin artifact path constants without removing legacy Julia
     constants yet
7. `packages/rust/crates/xiuxian-wendao/src/gateway/studio/router/handlers/capabilities/deployment.rs`
   - switch implementation to generic artifact resolution, then map back to
     Julia response shape for compatibility
8. `packages/rust/crates/xiuxian-wendao/src/zhenfa_router/native/deployment.rs`
   - switch rendering/export implementation to generic artifact helpers, while
     preserving the Julia command name

This slice should not yet touch:

1. `src/analyzers/languages/mod.rs`
2. `src/analyzers/service/bootstrap.rs`
3. Julia package ownership or source inclusion

Those belong to later `P3` and `P4` work.

## Compatibility Strategy

The compatibility stack for the first slice should look like this:

```text
legacy Julia API surface
  -> Julia compatibility conversion
  -> generic plugin artifact / capability runtime type
  -> generic resolver or renderer
```

Concrete rules:

1. `resolve_link_graph_julia_deployment_artifact()` becomes a thin wrapper over
   `resolve_plugin_artifact("xiuxian-julia-core", "deployment")`
2. `export_link_graph_julia_deployment_artifact_toml()` becomes a thin wrapper
   over generic artifact rendering
3. `UiJuliaDeploymentArtifact` becomes a compatibility DTO converted from a
   generic `UiPluginArtifact`
4. Julia-named OpenAPI and route constants remain, but the generic path
   constants are introduced in the same slice

## First Generic Function Targets

The first generic functions should be deliberately small:

```rust
pub fn resolve_plugin_artifact(
    plugin_id: &str,
    artifact_id: &str,
) -> PluginArtifactPayload

pub fn render_plugin_artifact_toml(
    plugin_id: &str,
    artifact_id: &str,
) -> Result<String, toml::ser::Error>

pub fn build_plugin_capability_binding_for_rerank(
    plugin_id: &str,
    runtime: &LinkGraphJuliaRerankRuntimeConfig,
) -> PluginCapabilityBinding
```

The first two functions are generic runtime seams.

The third function is still Julia-fed, but it is acceptable in the first slice
because it converts legacy runtime config into generic binding vocabulary.

## First Shim Rules

The following files may keep Julia-named types after the first slice:

1. `src/link_graph/runtime_config/models/`
2. `src/gateway/studio/types/config.rs`
3. `src/zhenfa_router/native/deployment.rs`

But they must follow these rules:

1. new business logic goes into `plugin_runtime/`
2. Julia-named types only perform conversion or delegation
3. no new fields may be added to Julia-named types unless required for strict
   compatibility

## Tests to Add or Move First

The first verification slice should add focused tests in the new feature seam:

```text
packages/rust/crates/xiuxian-wendao/src/link_graph/plugin_runtime/tests/
  mod.rs
  compat_julia.rs
  artifact_resolution.rs
  render.rs
```

Minimum test coverage:

1. Julia runtime config converts into `PluginCapabilityBinding` correctly
2. Julia deployment artifact converts into `PluginArtifactPayload` correctly
3. generic artifact resolver returns a deployment artifact for the Julia plugin
4. generic TOML rendering matches the legacy Julia TOML payload

Existing tests that should remain green:

1. `src/link_graph/runtime_config/tests.rs`
2. `src/gateway/studio/router/tests/config.rs`
3. `src/tests/unit/zhenfa_router/native/deployment.rs`

## File-by-File Edit Boundaries

The first code slice should preserve the following ownership boundaries:

1. `src/link_graph/runtime_config/models/`
   - may define legacy Julia structs and their conversions
   - must not become the permanent home of generic plugin runtime types
2. `src/gateway/studio/types/config.rs`
   - may define compatibility UI shims
   - generic plugin artifact DTOs should move into a dedicated feature folder
     if this file starts mixing unrelated UI concerns
3. `src/zhenfa_router/native/deployment.rs`
   - may keep the Julia tool entrypoint name
   - must delegate rendering and export behavior to generic helpers

## Cut Sequence

Recommended implementation order:

1. add `link_graph/plugin_runtime/` and its tests
2. add conversion implementations from Julia runtime structs into generic types
3. add generic artifact resolver and TOML renderer
4. redirect legacy Julia resolver helpers through the generic path
5. redirect gateway and Zhenfa deployment code through the generic path
6. add generic OpenAPI path constants and UI DTOs without removing legacy
   constants yet

Each step should pass its touched tests before proceeding.

## Latest Checkpoint

The current `zhenfa` deployment tooling now uses compatibility-oriented primary
names while preserving the Julia-facing tool and shim surface:

1. `zhenfa_router/native/deployment.rs` now treats
   `WendaoCompatDeploymentArtifactOutputFormat`,
   `WendaoCompatDeploymentArtifactArgs`,
   `render_compat_deployment_artifact_*`, and
   `export_compat_deployment_artifact` as the primary host-side API
2. legacy Julia names remain as thin compatibility aliases and forwarding
   helpers inside the deployment ownership seam
3. `zhenfa_router/native/mod.rs`, `zhenfa_router/mod.rs`,
   `zhenfa_router/http.rs`, `zhenfa_router/rpc.rs`, and the Studio capability
   handler now consume the compatibility-oriented surface
4. focused compatibility tests pin the continued availability of the legacy
   Julia aliases and JSON-RPC shim so host refactoring does not silently break
   them
5. the Studio capability handler surface now treats
   `CompatDeploymentArtifactQuery` and `get_compat_deployment_artifact` as the
   primary host-side API, while Julia query/handler names remain available as
   `pub(crate)` compatibility shims inside the capability ownership seam
6. compatibility wording now also owns the Studio handler error codes/messages
   and the route-inventory primary constant name, while Julia-named aliases
   remain available as compatibility surface only
7. `link_graph/runtime_config.rs`, `link_graph/mod.rs`, and crate-root exports
   now treat compat deployment-artifact helpers as the primary host-side API,
   while Julia-named helpers remain as public compatibility shims
8. compat-first DTO aliases now exist for the Julia deployment-artifact,
   launch-manifest, and rerank-runtime records, and the high-level
   `runtime_config` / `link_graph` / crate-root exports expose those compat
   names without removing the Julia aliases yet

Validation for this checkpoint:

1. `direnv exec . cargo test -p xiuxian-wendao zhenfa_router::native::deployment::tests:: --lib`
2. `direnv exec . cargo test -p xiuxian-wendao zhenfa_router::rpc::tests:: --lib`
3. `direnv exec . cargo test -p xiuxian-wendao gateway::studio::types::config::tests:: --lib`
4. `direnv exec . cargo test -p xiuxian-wendao gateway::studio::router::tests::config:: --lib`
5. `direnv exec . cargo test -p xiuxian-wendao route_inventory_keeps_core_endpoints --lib`
6. `direnv exec . cargo test -p xiuxian-wendao route_inventory_paths_are_unique --lib`
7. `direnv exec . cargo test -p xiuxian-wendao link_graph::runtime_config::tests:: --lib`
8. `direnv exec . cargo test -p xiuxian-wendao link_graph::plugin_runtime:: --lib`
9. `direnv exec . git diff --check`

## Stop Conditions

Stop this first slice if any of the following happen:

1. the new generic types start accumulating unrelated responsibilities in one
   file
2. `runtime_config/models/` becomes a second generic contract warehouse
3. the code needs plugin discovery or crate extraction to proceed
4. the generic path cannot reproduce the existing Julia deployment artifact
   exactly

If any of those occur, split the slice and finish only the smaller stable seam.

## Exit Criteria

This first code slice is complete when:

1. the new `plugin_runtime/` feature tree exists
2. generic artifact and capability types are importable from that tree
3. legacy Julia deployment artifact helpers delegate to the generic path
4. existing Julia-facing tests still pass
5. no crate splitting has been attempted yet

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/405_large_rust_modularization|Large Rust File Modularization]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---

:FOOTER:
:STANDARDS: v2.0
:LAST_SYNC: 2026-03-27
:END:
