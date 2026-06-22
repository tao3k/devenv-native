# P1 Generic Plugin Contract Staging

:PROPERTIES:
:ID: wendao-p1-generic-plugin-contract-staging
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: roadmap, migration, plugins, core, runtime, contracts, p1
:STATUS: SUPERSEDED
:END:

> Historical staging note. The active transport/runtime truth now lives in
> `409_core_runtime_plugin_surface_inventory.md` and
> `412_core_runtime_plugin_program.md`. This document remains only as the
> Phase-1 migration record and has been updated so its examples do not conflict
> with the current Flight-only plugin transport contract.

## Mission

This note defines the first implementation-grade contract staging for
`Gate P1: In-Place Generalization`.

Primary references:

- `[[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md]]`
- `[[docs/rfcs/2026-03-27-wendao-arrow-plugin-flight-rfc.md]]`
- `[[06_roadmap/409_core_runtime_plugin_surface_inventory]]`
- `[[06_roadmap/411_p1_first_code_slice_plan]]`

The active task-scoped blueprint for this migration remains recorded in the
paired ExecPlan rather than this persistent staging note.

The goal of this phase is not to finish extraction. The goal is to freeze the
generic vocabulary and first target types so implementation can stop adding
new Julia-specific host contracts.

## `P1` Outcome

`P1` is successful when:

1. new host-facing abstractions are named by capability, artifact, provider,
   or transport rather than by language
2. the first generic type family exists in feature-folder form
3. Julia-named host surfaces become compatibility aliases, not the primary
   destination for new logic
4. request-batch builders on retrieval adapters adopt capability-oriented
   names first, with Julia-named entrypoints retained only as compatibility
   shims while the Arrow contract remains unchanged
5. adapter-local request error types adopt capability-oriented names first,
   with Julia-named compatibility aliases retained only where older call sites
   still need the legacy vocabulary
6. analyzer-side Arrow request-row and request-batch builders adopt
   capability-oriented names first, with Julia-named aliases retained only as
   compatibility shims over the same Arrow contract
7. analyzer-side Arrow response rows and decode/fetch helpers adopt
   capability-oriented names first, with Julia-named aliases retained only as
   compatibility shims over the same Arrow contract
8. runtime-config helper methods adopt provider/capability-oriented primary
   names first, with Julia-named helper methods retained only as compatibility
   shims over the same runtime records
9. runtime-config binding builders adopt provider/capability-oriented primary
   names first, with Julia-named builder helpers retained only as thin
   compatibility shims

## Naming Freeze

The following words become the stable host vocabulary:

1. `plugin`
   - a discoverable, independently published provider package
2. `capability`
   - a unit of behavior such as `rerank` or `analyze_repository`
3. `artifact`
   - a resolved payload that describes or exports plugin-owned state such as
     deployment metadata
4. `provider`
   - the plugin implementation selected for a capability at runtime
5. `transport`
   - the data-plane mechanism used to invoke the provider
6. `binding`
   - the runtime-owned attachment between a capability and a provider

New host code should avoid inventing replacement synonyms while this migration
is active.

## First Target Namespaces

The first `P1` feature-folder targets are:

```text
xiuxian-wendao-core
  capabilities/
    ids.rs
    descriptor.rs
    schema.rs
  artifacts/
    ids.rs
    descriptor.rs
  transport/
    kind.rs
    descriptor.rs

xiuxian-wendao-runtime
  runtime_config/
    capabilities/
    providers/
  capabilities/
    rerank/
      binding.rs
      request.rs
  artifacts/
    resolve/
    render/
  transport/
  negotiation/
```

`mod.rs` remains interface-only across all of these folders.

## First Generic Core Types

The first generic host contracts should be small, naming-stable, and transport
neutral.

Suggested `core` types:

```rust
pub struct PluginId(pub String);

pub struct CapabilityId(pub String);

pub struct ArtifactId(pub String);

pub struct ContractVersion(pub String);

pub enum PluginTransportKind {
    ArrowFlight,
}

pub struct PluginTransportDescriptor {
    pub kind: PluginTransportKind,
    pub default_route: Option<String>,
    pub health_route: Option<String>,
    pub schema_version: Option<ContractVersion>,
}

pub struct PluginCapabilityDescriptor {
    pub plugin_id: PluginId,
    pub capability_id: CapabilityId,
    pub contract_version: ContractVersion,
    pub transport: PluginTransportDescriptor,
}

pub struct PluginArtifactDescriptor {
    pub plugin_id: PluginId,
    pub artifact_id: ArtifactId,
    pub contract_version: ContractVersion,
    pub media_type: String,
}
```

These are not final richness targets. They are the minimum vocabulary needed
to stop growing `Julia*` host types.

## First Generic Runtime Types

The first runtime abstractions should own provider selection, launch metadata,
and artifact rendering without importing language names into the host boundary.

Suggested `runtime` types:

```rust
pub struct PluginProviderSelector {
    pub capability_id: CapabilityId,
    pub provider: PluginId,
}

pub struct PluginTransportEndpoint {
    pub base_url: Option<String>,
    pub route: Option<String>,
    pub health_route: Option<String>,
    pub timeout_secs: Option<u64>,
}

pub struct PluginLaunchSpec {
    pub launcher_path: String,
    pub args: Vec<String>,
}

pub struct PluginCapabilityBinding {
    pub selector: PluginProviderSelector,
    pub endpoint: PluginTransportEndpoint,
    pub launch: Option<PluginLaunchSpec>,
    pub transport: PluginTransportKind,
    pub contract_version: ContractVersion,
}

pub struct PluginArtifactPayload {
    pub plugin_id: PluginId,
    pub artifact_id: ArtifactId,
    pub artifact_schema_version: ContractVersion,
    pub generated_at: String,
    pub endpoint: Option<PluginTransportEndpoint>,
    pub launch: Option<PluginLaunchSpec>,
}
```

These types should live behind feature folders such as:

1. `runtime/runtime_config/capabilities/`
2. `runtime/runtime_config/providers/`
3. `runtime/artifacts/resolve/`
4. `runtime/artifacts/render/`

## Julia Mapping Table

The current Julia-specific host surfaces should map as follows:

| Current Julia surface                            | `P1` generic target                                              | Notes                                                                                |
| :----------------------------------------------- | :--------------------------------------------------------------- | :----------------------------------------------------------------------------------- |
| `LinkGraphJuliaRerankRuntimeConfig`              | `PluginCapabilityBinding` plus provider-scoped config leaf       | Split endpoint, launch, and provider binding instead of keeping one warehouse struct |
| `LinkGraphJuliaAnalyzerServiceDescriptor`        | provider-scoped launch options under `runtime_config/providers/` | Runtime keeps the binding, Julia package keeps provider option semantics             |
| `LinkGraphJuliaAnalyzerLaunchManifest`           | `PluginLaunchSpec`                                               | Keep Julia compatibility conversion until `P4`                                       |
| `LinkGraphJuliaDeploymentArtifact`               | `PluginArtifactPayload`                                          | Deployment becomes one artifact kind, not one language-only DTO                      |
| `UiJuliaAnalyzerLaunchManifest`                  | `UiPluginLaunchSpec`                                             | UI contract should mirror runtime generic payload names                              |
| `UiJuliaDeploymentArtifact`                      | `UiPluginArtifact`                                               | Route by `plugin_id` and `artifact_id`                                               |
| `resolve_link_graph_julia_deployment_artifact()` | `resolve_plugin_artifact(plugin_id, artifact_id)`                | Keep Julia-named shim only as compatibility                                          |
| `/api/ui/julia-deployment-artifact`              | `/api/ui/plugins/{plugin_id}/artifacts/{artifact_id}`            | Introduce generic endpoint before removing legacy path                               |

## Retrieval Config Staging

The current retrieval runtime shape:

```toml
[link_graph.retrieval.julia_rerank]
base_url = "http://127.0.0.1:8815"
route = "/rerank"
```

Should stage toward:

```toml
[link_graph.retrieval.rerank]
provider = "xiuxian-julia-core"
contract_version = "v1"
transport = "arrow_flight"

[link_graph.retrieval.rerank.endpoint]
base_url = "http://127.0.0.1:8815"
route = "/rerank"
health_route = "/healthz"
timeout_secs = 15
```

Provider-owned launch options should move below provider-scoped config rather
than stay in the capability selector:

```toml
[plugins.xiuxian-julia-core.launch]
launcher_path = ".data/WendaoAnalyzer/scripts/run_analyzer_service.sh"
service_mode = "stream"
analyzer_strategy = "linear_blend"
vector_weight = 0.7
similarity_weight = 0.3
```

The compatibility rule is:

1. `julia_rerank` may continue to parse during migration
2. new runtime logic must read from the generic model first
3. legacy Julia config should normalize into the generic binding during
   resolution

## Artifact Staging

The first generic artifact contract should treat deployment as a plugin-owned
artifact kind rather than a Julia-only surface.

Suggested runtime path:

```text
runtime/artifacts/
  ids.rs
  payload.rs
  resolve/
    mod.rs
    deployment.rs
  render/
    mod.rs
    json.rs
    toml.rs
```

Suggested UI path:

```text
runtime/gateway/studio/types/artifacts/
  mod.rs
  payload.rs
  launch.rs

runtime/gateway/studio/router/handlers/plugin_artifacts/
  mod.rs
  get.rs
  query.rs
```

## Compatibility Rules for `P1`

During `P1`, the following are acceptable:

1. public Julia-named aliases that convert to generic types
2. legacy config normalization from `julia_rerank` into generic bindings
3. legacy route and RPC shims delegating to generic artifact resolvers

The following are not acceptable:

1. adding a new host type named after Julia
2. adding a new host route named after Julia
3. adding new implementation logic to Julia-only DTOs instead of the generic
   feature folders
4. placing the new generic contracts into flat warehouse files

## Implementation Order Inside `P1`

The first three implementation slices should be:

1. create generic ids, descriptors, and transport kinds under
   responsibility-oriented feature folders
2. create generic runtime binding and artifact payload types
3. add compatibility conversions from existing Julia types into the new
   generic model without changing external behavior

Only after those three slices are stable should code start replacing the live
call sites.

## Exit Criteria

`P1` can close when:

1. the generic contract names are live in the tree
2. no new Julia-specific host contracts have been introduced
3. the runtime can express rerank selection and deployment artifact rendering
   without depending on Julia in its primary type names
4. the new contracts live in feature folders with interface-only `mod.rs`

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/405_large_rust_modularization|Large Rust File Modularization]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---

:FOOTER:
:STANDARDS: v2.0
:LAST_SYNC: 2026-03-27
:END:
