# Julia Plugin-First Rollout

:PROPERTIES:
:ID: wendao-julia-plugin-first-rollout
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: roadmap, migration, plugins, julia, runtime, host-thinning
:STATUS: ACTIVE
:END:

## Purpose

Make `xiuxian-wendao-julia` the first plugin-first execution lane of the
Wendao package split.

The goal is not only to externalize Julia ownership. The goal is to prove the
desired host architecture:

1. `core` defines contracts
2. `runtime` loads and orchestrates plugins
3. the plugin crate owns its thick implementation
4. the host crate avoids plugin-specific growth

## Why Julia Goes First

`xiuxian-wendao-julia` is already a real external crate with:

1. plugin entry and registration
2. Julia-specific transport option interpretation
3. Julia-specific launch and compatibility meaning
4. Julia-specific graph-structural exchange helpers

That makes it the best first lane for turning the package split into a real
plugin architecture rather than a host-local convention.

## Architectural Target

The target dependency and ownership shape is:

```text
xiuxian-wendao-core
  -> stable contracts

xiuxian-wendao-runtime
  -> host config, Flight/DataFusion wiring, plugin loading, registry

xiuxian-wendao-julia
  -> Julia parser/analyzer/projection/launch/transport specifics

xiuxian-wendao
  -> Wendao domain core plus only thin plugin consumption seams
```

## Plugin-First Rule

For the Julia lane, prefer this implementation pattern:

1. add or update the plugin crate dependency
2. register the plugin through runtime-owned registry seams
3. compile and load the plugin

Avoid this implementation pattern:

1. add Julia-specific domain logic to `xiuxian-wendao`
2. add Julia-specific gateway handlers as thick host code
3. add Julia-specific parser, projection, or launch semantics to the host crate

## Julia Ownership Boundary

`xiuxian-wendao-julia` should own:

1. Julia-specific parser and analyzer behavior
2. Julia-specific projection and result shaping
3. Julia-specific launch and artifact semantics
4. Julia-specific transport option interpretation beyond generic Flight wiring
5. Julia-specific graph-structural request or response helpers
6. Julia-specific compatibility meaning during the migration window

`xiuxian-wendao-runtime` should own:

1. plugin registry and bootstrap
2. generic Arrow Flight negotiation
3. generic DataFusion or Flight host execution glue
4. loading and dispatching plugin capabilities

`xiuxian-wendao` should retain only:

1. Wendao domain semantics
2. plugin-agnostic parser or graph behavior
3. thin consumption seams over plugin capabilities
4. transitional compatibility wrappers that are explicitly scheduled for
   retirement

## Memory-Family Julia Compute Boundary

The memory-family Julia lane follows the same plugin-first rule, but it is
stricter about ownership:

1. `WendaoMemory.jl` owns compute kernels only
2. `xiuxian-wendao-julia` owns Julia-specific memory ABI logic: typed rows,
   request and response batches, manifest projection, schema validation,
   decoding, route defaults, and plugin-owned host-adapter helpers over Rust
   memory-engine read models or evidence
3. `xiuxian-wendao` consumes that plugin-owned seam directly instead of
   keeping a second local host-adapter namespace
4. `xiuxian-memory-engine` and the Rust host remain authoritative for memory
   state, lifecycle, fallback, audit, and final mutation decisions

For this lane, do not add new memory-family profile semantics, schema
fragments, manifest rules, decoder logic, or Julia-specific validation logic
to `xiuxian-wendao`. If the work is Julia ABI meaning rather than host-domain
adaptation, it belongs in `xiuxian-wendao-julia`.

## Landed Slice: Thin Memory Host Bridge

The next bounded ownership move makes the host-facing memory entry explicit
without pulling Julia profile logic back into the product crate.

`xiuxian-wendao` now keeps:

1. the feature-gated namespace `xiuxian_wendao::memory::julia`
2. a thin set of re-exported host-facing memory Julia types and downcall entry
   points
3. no local profile shaping, validation, transport, or decode logic

`xiuxian-wendao-julia` still keeps:

1. typed memory-family Arrow contracts
2. host staging over Rust read models and evidence
3. runtime-facing Flight transport
4. composed profile downcalls

This slice matters because it turns the approved RFC layering into a readable
crate boundary: host consumers can enter through `xiuxian_wendao::memory::julia`,
but the Julia-specific implementation remains plugin-owned.

## Landed Slice: Memory Runtime Resolution

The next bounded move makes the memory-family bridge useful to the host
without pushing Julia ABI logic back into the product crate.

`xiuxian-wendao` now keeps:

1. one crate-private merged-settings seam shared by `link_graph` and `memory`
2. `xiuxian_wendao::memory::julia::resolve_memory_julia_compute_runtime()`
3. `xiuxian_wendao::memory::julia::resolve_memory_julia_compute_bindings()`

`xiuxian-wendao` still does not keep:

1. Julia memory profile contracts
2. Julia memory transport logic
3. Julia memory host staging logic
4. Julia memory response decoding

This slice matters because it creates the first real host runtime assembly path
for the memory-family Julia lane: merged Wendao settings now flow through a
Wendao-owned bridge into plugin-owned binding materialization, instead of
leaving `memory::julia` as a pure re-export shell.

## Landed Slice: Runtime-Resolved Memory Downcalls

The next bounded move makes the host-facing memory bridge directly callable
without making `xiuxian-wendao` own a second downcall layer.

`xiuxian-wendao` now keeps:

1. runtime-resolved async wrappers under `xiuxian_wendao::memory::julia::*`
2. the host-facing re-export surface for input and output types required by
   those wrappers
3. no local request shaping, Flight transport, or response decode logic

`xiuxian-wendao-julia` still keeps:

1. memory-family host staging over read models and evidence
2. the composed downcall logic that still accepts explicit runtime config
3. the actual Arrow Flight transport and typed response decoding

This slice matters because host callers can now enter through
`xiuxian_wendao::memory::julia::*` with domain inputs only, while the product
crate still delegates every Julia-specific step below runtime resolution into
`xiuxian-wendao-julia`.

## Landed Slice: Memory Host Client

The next bounded move cleans up the host-facing seam into a proper feature
folder and one configured client surface.

`xiuxian-wendao` now keeps:

1. `packages/rust/crates/xiuxian-wendao/src/memory/julia/` as a feature folder
2. interface-only `mod.rs` plus separated runtime and client submodules
3. `xiuxian_wendao::memory::julia::ComputeClient` as the primary configured
   host entry for memory-family Julia compute

`xiuxian-wendao` still does not keep:

1. Julia-specific host staging
2. Julia-specific Arrow transport
3. Julia-specific response decoding
4. a product-local second compute implementation layer

This slice matters because the host-facing seam is now both thinner and more
usable: callers can construct one configured client inside the product crate,
while the actual compute lane still belongs below the seam in
`xiuxian-wendao-julia`.

## Landed Slice: Memory Public Surface Thinning

The next bounded move removes helper leakage from the product-crate bridge.

`xiuxian-wendao` now keeps:

1. `xiuxian_wendao::memory::julia::ComputeClient` as the primary host-facing
   entry
2. runtime and binding resolution helpers
3. only the minimum typed DTOs needed to call the configured client

`xiuxian-wendao` no longer re-exports:

1. raw runtime-resolved downcall free functions
2. plugin-owned runtime-to-binding builder helpers
3. gate-evidence builder helpers that belong under
   `xiuxian-wendao-julia::memory::host::*`

This slice matters because it makes the boundary harder to drift: host callers
can still enter through the product crate, but helper-heavy Julia ABI details
are no longer presented as part of the Wendao product surface.

## Landed Slice: Memory Host-Adapter Extraction

The next bounded ownership move removes the last local memory Julia helper
namespace from `xiuxian-wendao`.

`xiuxian-wendao-julia` now keeps:

1. the memory-family typed Arrow contracts
2. the plugin-owned host-adapter helpers under `memory::host::*`
3. the focused tests that prove Rust memory-engine projections and evidence can
   be turned into staged Julia request batches inside the plugin crate

`xiuxian-wendao` no longer keeps:

1. `src/memory/julia/`
2. a crate-local memory Julia adapter namespace
3. a direct `xiuxian-memory-engine` dependency just to stage Julia memory
   request batches

## Parser Interpretation

The parser rule is intentionally split:

1. general Wendao parser logic stays in `xiuxian-wendao`
2. Julia-specific parser or analyzer logic moves into `xiuxian-wendao-julia`

This keeps the host crate responsible for the platform-level understanding of
knowledge while allowing language-specific intelligence to remain plugin-owned.

## Gateway Thinning Rule

The Julia rollout should also thin host-side gateway ownership.

The gateway boundary should remain:

1. protocol adapter
2. contract validation
3. request dispatch
4. response encoding

The gateway boundary should not remain:

1. Julia-specific business logic
2. Julia-specific parser behavior
3. Julia-specific query materialization
4. a second thick graph-structural adapter layer

## First Execution Slices

The first Julia plugin-first slices should prefer extracting the following
classes of behavior from the host path:

1. Julia-specific parser or analyzer semantics
2. Julia-specific query and projection shaping
3. Julia-specific launch and artifact meaning
4. Julia-specific gateway-side helper logic that can become plugin-owned

## Landed Slice: Julia Arrow Rerank Exchange

The first bounded ownership move is now landed for the Julia rerank exchange
lane.

`xiuxian-wendao-julia` now owns:

1. typed Julia Arrow rerank request rows
2. typed Julia Arrow rerank score rows
3. request-batch assembly for the Julia rerank contract
4. response decoding for the Julia rerank contract
5. repository fetch helpers for the Julia rerank contract
6. plugin-local tests for that exchange seam

`xiuxian-wendao` temporarily retained only:

1. a temporary thin host re-export seam for the Julia transport exchange
2. plugin registration bootstrap that depends on the external Julia crate
3. plugin-consumption call sites that import the Julia-owned helpers

This slice was important because it removed one more thick Julia transport
implementation folder from the host crate before the later host-facade
retirement cut.

The temporary host re-export seam is now gone:

1. `analyzers::service::julia_transport` no longer exports Julia-owned DTO or
   fetch helpers
2. `analyzers::languages` no longer re-exports Julia or Modelica plugin
   registration or graph-structural helper APIs
3. the remaining in-repo Julia facade consumer now imports directly from
   `xiuxian-wendao-julia`

## Landed Slice: Compatibility Wrapper Thinning

The next bounded ownership move thins the host compatibility shell for the
link-graph Julia rerank lane.

`xiuxian-wendao` now keeps only:

1. direct re-exports of Julia compatibility types from `xiuxian-wendao-julia`
2. local compatibility type aliases where existing host tests still depend on
   the legacy names
3. the host-side runtime policy and plugin-runtime call sites that consume
   those types

`xiuxian-wendao` no longer keeps:

1. one wrapper file per Julia compatibility type under
   `link_graph/runtime_config/models/retrieval/julia_rerank/`
2. a second forwarding file under `link_graph/plugin_runtime/compat/`

This slice matters because it removes another layer of host-owned Julia file
structure while preserving the same host API and compatibility test surface.

## Landed Slice: Gateway Flight Boundary Thinning

The next bounded host-thinning move relocates the Studio/search-plane Flight
adapter from the `link_graph` plugin-runtime tree into the gateway Flight
handler area.

`xiuxian-wendao` now keeps:

1. the `gateway/studio/search/handlers/flight/search_plane.rs` implementation
   as the canonical home for search-plane repo-search Flight batching and
   Studio Flight-service assembly
2. gateway-local repo-content search code that imports the search-plane Flight
   provider through the gateway-owned Studio surface instead of through
   `link_graph::plugin_runtime`
3. gateway-bin command paths and standalone sample-server binaries that now
   import the Studio Flight-service builder and sample-data bootstrap helpers
   directly from the gateway-owned Studio surface
4. `link_graph::runtime_config` as the canonical home for the Wendao-specific
   plugin artifact resolve/render helpers that read retrieval policy state and
   attach transport diagnostics
5. `link_graph::plugin_runtime` only as a shrinking legacy shell for the
   compatibility helper that has not yet moved outward

`xiuxian-wendao` no longer keeps:

1. the Studio/search-plane Flight adapter implementation under
   `link_graph/plugin_runtime/transport/`
2. a gateway-internal dependency on the `link_graph::plugin_runtime` namespace
   just to reach the repo-search Flight provider
3. a host-internal dependency on `link_graph::plugin_runtime` for building the
   gateway Studio Flight service
4. a host-internal dependency on `link_graph::plugin_runtime` for sample-data
   Flight bootstrapping in the standalone binaries
5. the legacy `link_graph/plugin_runtime/transport/server.rs` re-export layer
   for gateway-owned Flight helpers
6. the remaining `link_graph/plugin_runtime/transport/` wrapper directory for
   runtime transport negotiation and core transport types
7. the `link_graph/plugin_runtime/artifacts/` wrapper directory for
   runtime-config-backed plugin artifact resolution and TOML rendering

This slice matters because it moves one more thick gateway adapter out of the
plugin-runtime namespace and back to the protocol boundary where it belongs.

## Landed Slice: Plugin Runtime Module Retirement

The next bounded host-thinning move removes the remaining
`link_graph::plugin_runtime` shell entirely.

`xiuxian-wendao` now keeps:

1. `link_graph::runtime_config` as the host-owned surface for Wendao-specific
   rerank binding resolution and plugin artifact resolve/render helpers
2. direct imports from `xiuxian-wendao-julia` for the Julia compatibility
   helper that materializes rerank provider bindings
3. runtime-config-local tests for compatibility binding generation and direct
   artifact resolve/render behavior

`xiuxian-wendao` no longer keeps:

1. the `link_graph/plugin_runtime/` module tree as a forwarding namespace
2. host-local re-exports of core capability or identifier types through
   `link_graph::plugin_runtime`
3. a dedicated plugin-runtime test namespace just to exercise
   runtime-config-backed Julia compatibility behavior

This slice matters because it completes the transition from a host-local
plugin-runtime wrapper model to direct ownership boundaries:
`xiuxian-wendao-julia` owns Julia-specific compatibility helpers,
`xiuxian-wendao-runtime` owns transport/runtime behavior, and
`xiuxian-wendao` retains only its domain-owned runtime configuration seams.

## Landed Slice: Retrieval Julia Shell Retirement

The next bounded host-thinning move removes the remaining retrieval-level Julia
forwarding shell and the host-local compatibility test aliases.

`xiuxian-wendao` now keeps:

1. the Wendao-owned retrieval policy record and runtime-config resolution logic
2. direct use of Julia-owned runtime/config types from
   `xiuxian-wendao-julia::compatibility::link_graph`
3. runtime-config tests that exercise Julia deployment artifact rendering and
   rerank binding generation through the Julia-owned type names

`xiuxian-wendao` no longer keeps:

1. the `link_graph/runtime_config/models/retrieval/julia_rerank/` forwarding
   folder
2. `LinkGraphCompatDeploymentArtifact`,
   `LinkGraphCompatAnalyzerLaunchManifest`, and
   `LinkGraphCompatRerankRuntimeConfig` as host-local alias names
3. test-only runtime-config helpers with `compat` naming for Julia deployment
   artifacts

This slice matters because it removes the last retrieval-side naming indirection
between Wendao runtime-config code and the Julia plugin crate, while preserving
the Wendao-owned retrieval-policy semantics.

## Landed Slice: Julia Selector Surface Retirement

The next bounded host-thinning move removes the remaining public Julia selector
re-export from `xiuxian-wendao`.

`xiuxian-wendao` now keeps:

1. Wendao-owned runtime-config resolution and plugin-artifact rendering logic
2. direct internal imports from `xiuxian-wendao-julia` for Julia deployment
   selectors and runtime types where the host still needs them
3. package docs that point downstream callers at the Julia crate for
   Julia-owned selectors and compatibility records

`xiuxian-wendao` no longer keeps:

1. a public `link_graph::julia_deployment_artifact_selector` re-export
2. host-local re-export chains in `runtime_config::{models,mod}.rs` just to
   surface the Julia deployment selector
3. documentation that describes a surviving host re-export seam for Julia
   runtime records

This slice matters because it removes the last public Julia selector shim from
the host crate while leaving Wendao-owned runtime behavior intact.

## Landed Slice: Julia Integration Support Ownership

The next bounded plugin-first move retires the host-local Julia official
example wrapper under `tests/integration/support/` and makes the plugin crate
own that integration-support surface directly.

`xiuxian-wendao-julia` now keeps:

1. a bounded public `integration_support` surface for Julia-owned official
   example services
2. the official-example spawn helpers needed by Julia rerank and analyzer
   integration tests
3. the Julia-specific process guard and readiness polling logic for those
   official examples

`xiuxian-wendao` no longer keeps:

1. `tests/integration/support/wendaoarrow_official_examples.rs` as a host-local
   Julia wrapper module
2. host-side imports for the four official-example planned-search integration
   tests
3. a support-mod declaration that exists only to surface Julia-specific
   official examples from the host crate

This slice matters because plugin-first ownership now covers not only runtime
and compatibility behavior, but also the Julia-specific integration-support
surface that used to widen the host test tree.

## Acceptance Signal

The Julia lane is moving in the right direction when a new Julia capability can
be landed primarily by:

1. editing `xiuxian-wendao-julia`
2. updating runtime registration or config
3. avoiding new production logic in `xiuxian-wendao`

If a new Julia capability still requires widening the host crate with fresh
Julia-specific implementation modules, the plugin-first goal has not been met.

## Landed Slice: Julia Custom Service Ownership

The next bounded plugin-first move retires the remaining host-local
WendaoArrow custom scoring support under `tests/integration/support/` and
moves that custom service seam into `xiuxian-wendao-julia::integration_support`.

`xiuxian-wendao-julia` now keeps:

1. a folderized `integration_support::{common,official_examples,custom_service}`
   surface for Julia-specific integration services
2. the custom scoring service helpers used by the Julia rerank planned-search
   tests
3. the shared Julia integration-service guard, package-path resolution, and
   readiness polling logic used by both official-example and custom-scoring
   service launch helpers

`xiuxian-wendao` no longer keeps:

1. `tests/integration/support/wendaoarrow_common.rs`
2. `tests/integration/support/wendaoarrow_custom_service.rs`
3. host-local rerank test imports for the custom scoring service helpers

This slice matters because it removes the last Julia-specific test-service
wrapper layer from the host test tree and keeps the Julia integration-support
surface coherent inside the plugin crate.

## Landed Slice: Julia Runtime Config Ownership

The next bounded plugin-first move retires the remaining host-local Julia
rerank runtime-config application helper under
`link_graph/runtime_config/resolve/policy/retrieval/` and makes
`xiuxian-wendao-julia` own the Julia-specific settings and environment override
translation for `LinkGraphJuliaRerankRuntimeConfig`.

`xiuxian-wendao-julia` now keeps:

1. a dedicated `compatibility/link_graph/settings.rs` module for Julia rerank
   settings keys, env constants, and normalization/application logic
2. focused compatibility tests that prove config-file values, env fallback, and
   settings-over-env precedence without mutating process-global environment
3. the `LinkGraphJuliaRerankRuntimeConfig::resolve_with_settings` entrypoint
   consumed by the host retrieval runtime resolver

`xiuxian-wendao` no longer keeps:

1. `link_graph/runtime_config/resolve/policy/retrieval/provider.rs`
2. Julia rerank env constants in `runtime_config/constants.rs`
3. the dead `runtime_config/settings/parse.rs` helper layer that only existed
   for the removed host-local Julia provider helper

This slice matters because the host runtime-config tree no longer owns new
Julia-specific settings-application code once the plugin crate already owns the
runtime record itself.

## Landed Slice: Julia Artifact Ownership

The next bounded plugin-first move retires the remaining host-local Julia
deployment-artifact implementation under `link_graph/runtime_config/artifacts.rs`
and makes `xiuxian-wendao-julia` own the Julia-specific artifact payload
resolution, transport diagnostics, and artifact rendering helpers.

`xiuxian-wendao-julia` now keeps:

1. the Julia deployment-artifact payload resolver for
   `LinkGraphJuliaRerankRuntimeConfig`
2. the transport-diagnostics attachment logic for Julia artifact payloads
3. the Julia deployment-artifact TOML rendering helper and focused
   compatibility coverage for that surface

`xiuxian-wendao` now keeps only:

1. a thin selector-dispatch surface in `runtime_config/artifacts.rs`
2. the generic runtime-config public exports consumed by gateway and router
   code
3. the end-to-end runtime-config tests that prove the host-facing behavior
   still resolves and renders Julia deployment artifacts correctly

This slice matters because artifact payloads and transport diagnostics are
plugin-specific implementation, not long-term host runtime-config ownership.

## Landed Slice: Julia Artifact Wrapper Retirement

The next bounded plugin-first move retires the remaining test-only Julia
artifact wrappers in `link_graph/runtime_config.rs` and updates the host
runtime-config tests to consume the generic artifact-dispatch surface plus
plugin-owned Julia types directly.

`xiuxian-wendao` no longer keeps:

1. `resolve_link_graph_julia_deployment_artifact`
2. `export_link_graph_julia_deployment_artifact_toml`
3. the extra test-only Julia imports that only existed to support those
   wrappers

Host runtime-config tests now use:

1. `resolve_link_graph_plugin_artifact_for_selector`
2. `render_link_graph_plugin_artifact_toml_for_selector`
3. `LinkGraphJuliaDeploymentArtifact` directly for Julia-specific shaping

This slice matters because test-only Julia wrapper functions were still extra
host-local ownership, even after the real artifact implementation moved into
the plugin crate.

## Landed Slice: Julia Compatibility Proof Extraction

The next bounded plugin-first move extracts the remaining pure
Julia-compatibility proofs from `xiuxian-wendao/src/link_graph/runtime_config/tests.rs`
into `xiuxian-wendao-julia/src/compatibility/link_graph/tests.rs`, leaving the
host runtime-config tests focused on end-to-end host behavior only.

`xiuxian-wendao-julia` now keeps:

1. the deployment-artifact TOML file-write proof
2. the deployment-artifact JSON file-write proof
3. the Julia rerank runtime to generic binding conversion proof

`xiuxian-wendao` now keeps only:

1. host-owned runtime-config resolution proofs
2. host-owned generic artifact-dispatch and rendering proofs
3. seven end-to-end runtime-config tests instead of mixing in pure plugin
   compatibility checks

This slice matters because test ownership should follow implementation
ownership; pure plugin compatibility proofs should not stay in the host crate.

## Landed Slice: Julia Runtime Config Proof Thinning

The next bounded plugin-first move thins the remaining Julia-heavy host
runtime-config integration test by extracting the pure plugin-local descriptor,
launch-manifest, deployment-artifact, and direct binding proofs into the Julia
compatibility test module.

`xiuxian-wendao-julia` now keeps:

1. the pure `LinkGraphJuliaRerankRuntimeConfig` proof for descriptor shaping
2. the pure runtime-launch manifest and plugin launch-spec proof
3. the pure deployment-artifact payload and direct binding shaping proof

`xiuxian-wendao` now keeps only:

1. config resolution proof for the Julia rerank settings
2. host-owned score-weight and schema-version resolution proof
3. generic rerank-binding and generic artifact-dispatch proof

This slice matters because the last thick Julia-heavy host test no longer
carries proof that belongs with the plugin-owned runtime record.

## Landed Slice: Julia Host Runtime Integration Split

The next bounded plugin-first move split the remaining host Julia runtime-config
integration proof into narrower host-owned tests, so the host test surface now
tracks config resolution and host helper projection explicitly instead of
bundling them into one long block.

`xiuxian-wendao` now keeps:

1. one host test for raw Julia rerank config resolution from Wendao config
2. one host test for host-owned helper projection such as score weights,
   schema version, flight settings, and generic rerank binding
3. dedicated host artifact tests as the owner of generic artifact-dispatch and
   render proof, without duplicate assertions in the main config-resolution test

This slice also introduced a shared local fixture helper in the host test
module and removed the touched-scope unused imports that were left behind by
the split.

This slice matters because the host test surface should be explicit about what
is host integration proof versus already-covered artifact or plugin-local proof.

## Julia OpenAPI Artifact Example Ownership

This slice moved the Julia deployment-artifact OpenAPI example source of truth
into `xiuxian-wendao-julia`, so the host gateway package no longer owns
plugin-specific example payload shaping.

The landed changes are:

1. `xiuxian-wendao-julia` now owns the generic plugin-artifact and legacy Julia
   deployment-artifact OpenAPI example helpers in
   `compatibility/link_graph/openapi_examples.rs`
2. `xiuxian-wendao` keeps the bundled OpenAPI document but validates it against
   plugin-owned JSON and TOML example helpers in
   `gateway/openapi/document.rs`
3. the checked-in OpenAPI resource now matches the plugin-owned contract for
   both the Julia launcher path
   `.data/WendaoAnalyzer.jl/scripts/run_analyzer_service.sh` and the TOML
   serializer's multiline `args` array layout

This slice matters because plugin-specific documentation examples should follow
plugin ownership just like runtime config, launch metadata, and artifact
payload shaping.

## Julia Gateway Artifact Fixture Ownership

This slice moved the Julia-specific Studio gateway artifact test fixtures into
`xiuxian-wendao-julia`, so the host gateway tests no longer own repeated Julia
runtime-config snippets, selector literals, or stable TOML fragment shaping.

The landed changes are:

1. `xiuxian-wendao-julia` now owns focused Studio gateway fixture helpers in
   `integration_support/gateway_artifact.rs`, including the runtime-config TOML
   fixture, selector path, stable schema/base-url helpers, and expected TOML
   fragments
2. `xiuxian-wendao` gateway tests in
   `router/handlers/capabilities/deployment.rs` and `router/tests/config.rs`
   now consume those plugin-owned helpers instead of repeating Julia-specific
   fixture strings inline
3. a minimal compile-unblocker landed in
   `gateway/studio/search/queries/graphql/document.rs` to split the borrow
   lifetime and AST lifetime for `graphql_parser::Field`, so focused host
   gateway tests can compile under the current worktree

This slice matters because host gateway tests should validate transport and UI
adapter behavior, not re-own language-specific artifact fixture construction.

## Julia UI Artifact Fixture Ownership

This slice moved the Julia-specific `PluginArtifactPayload` fixture for Studio
UI artifact mapping tests into `xiuxian-wendao-julia`, so the host
`UiPluginArtifact` test no longer owns inline Julia deployment-payload
construction.

The landed changes are:

1. `xiuxian-wendao-julia` now owns the stable UI artifact payload fixture in
   `integration_support/gateway_artifact.rs`
2. `xiuxian-wendao-julia::integration_support` now re-exports that fixture so
   host tests can consume it through the plugin-owned support surface
3. `xiuxian-wendao` `gateway/studio/types/config.rs` now validates generic
   `UiPluginArtifact` mapping against the plugin-owned fixture instead of
   constructing Julia payload fields inline

This slice matters because host UI mapping tests should validate generic shape
projection, not re-own plugin-specific payload construction.

## Julia Zhenfa Router Artifact Fixture Ownership

This slice moved the Julia-specific generic plugin-artifact fixture support
used by `tests/unit/zhenfa_router/native/deployment.rs` and
`tests/unit/zhenfa_router/rpc.rs` into `xiuxian-wendao-julia`, so host
`zhenfa_router` tests now consume plugin-owned config, request/selector, and
stable output-fragment helpers instead of repeating Julia deployment details
inline.

The landed changes are:

1. `xiuxian-wendao-julia::integration_support` now owns the extra
   `zhenfa_router` fixture helpers in `integration_support/gateway_artifact.rs`,
   including the default strategy helper, stable JSON output fragments, and a
   JSON-RPC params fixture for generic plugin-artifact export tests
2. `xiuxian-wendao` `tests/unit/zhenfa_router/native/deployment.rs` now
   consumes plugin-owned path, config, and expected-output helpers instead of
   importing Julia-specific ids, route, or launcher constants directly
3. `xiuxian-wendao` `tests/unit/zhenfa_router/rpc.rs` now consumes
   plugin-owned runtime-config, request, and expected-output helpers so the
   host RPC tests validate generic export behavior rather than re-owning Julia
   payload construction

This slice matters because the host `zhenfa_router` tests should validate the
generic export surfaces, while plugin-specific deployment artifact fixtures
stay with the Julia plugin crate.

## Julia Planned-Search Config Fixture Ownership

This slice moved the remaining Julia-specific runtime-config TOML fixtures used
by `planned_search_julia_rerank.rs` and
`planned_search_julia_rerank_vector_store.rs` into
`xiuxian-wendao-julia::integration_support`, so the host integration tests no
longer own inline Julia rerank config strings for the custom `WendaoArrow`
service lane.

The landed changes are:

1. `xiuxian-wendao-julia::integration_support` now owns focused planned-search
   runtime-config helpers in `integration_support/planned_search.rs` for the
   `openai-compatible` and `vector-store` semantic-ignition variants used by
   the custom Julia rerank tests
2. `xiuxian-wendao-julia::integration_support` now re-exports those helpers
   through `integration_support/mod.rs`, so the plugin crate owns the stable
   fixture surface directly
3. `xiuxian-wendao` host integration tests in
   `tests/integration/planned_search_julia_rerank.rs` and
   `tests/integration/planned_search_julia_rerank_vector_store.rs` now consume
   those plugin-owned helpers and keep only the real planned-search behavior
   assertions

This slice matters because the host planned-search tests should validate
retrieval behavior and rerank outcomes, while Julia-specific runtime-config
fixture construction stays with the Julia plugin crate.

## Queued Slice: Julia Planned-Search Official Example Fixture Reuse

This slice will rewire the remaining official-example and analyzer-example
planned-search tests that still inline the generic `vector-store` Julia rerank
runtime-config TOML, so those host tests consume the existing
`xiuxian-wendao-julia::integration_support::planned_search` helper instead of
re-owning the same fixture string.

The intended changes are:

1. `planned_search_julia_rerank_official_example.rs`,
   `planned_search_julia_rerank_metadata_example.rs`, and
   `planned_search_wendaoanalyzer_linear_blend.rs` should all consume the
   existing plugin-owned `vector-store` planned-search runtime-config helper
2. the slice should stay bounded to those three host tests plus the package
   roadmap, GTD, and ExecPlan tracking surfaces
3. focused validation should cover the plugin-owned integration-support tests
   and the three host official-example/analyzer-example planned-search tests

## Landed Slice: Search Query SQL Extraction From Gateway

This slice will realign the Studio search query surfaces so the main SQL and
`FlightSQL` implementations no longer live under `gateway/`. The gateway layer
should stay a thin protocol adapter, while the real query construction,
registration, and execution logic moves into a src-owned `search/queries/`
namespace.

The intended changes are:

1. the real SQL and `FlightSQL` query implementations should move out of
   `gateway/studio/search/queries/` into a src-owned `search/queries/`
   namespace
2. `gateway/studio/search/*` should keep only thin adapter responsibilities:
   request decoding, response encoding, and dispatch into the src-owned query
   implementation surface
3. the first bounded extraction slice should define the ownership map and move
   one coherent query subtree without widening gateway responsibilities further

The first bounded move landed on exactly that seam:

1. shared SQL execution and registration have moved under
   `src/search/queries/sql/`
2. `gateway/studio/search/queries/sql/` is being reduced to a thin facade plus
   the Flight provider
3. gateway SQL tests now consume shared registration constants from the
   src-owned `search/queries/sql/registration` surface instead of a gateway
   wrapper path
4. touched-scope warning closure is complete, and the related `FlightSQL` plus
   planned-search regression checks reran cleanly

## Landed Slice: FlightSQL Adapter Extraction From Gateway

This slice continues the same boundary correction for the `FlightSQL` adapter.
The adapter implementation now lives under `src/search/queries/flightsql/`,
which brings the code layout back into alignment with the `search/queries/`
architecture note.

The landed changes are:

1. moved the coherent `FlightSQL` implementation subtree into
   `src/search/queries/flightsql/`
2. reduced `gateway/studio/search/queries/flightsql/` to a thin facade and
   compatibility re-export
3. aligned both `FlightSQL` and `GraphQL` with the src-owned shared SQL seam
   under `search/queries/sql/`
4. reran the focused `FlightSQL` adapter regression suite plus the dedicated
   `wendao_search_flightsql_server` binary check

## Landed Slice: GraphQL Adapter Extraction From Gateway

This slice continues the same boundary correction for the `GraphQL` adapter.
The implementation now lives under `src/search/queries/graphql/`, which
realigns the code layout with the `search/queries/` architecture note and
leaves the gateway surface thin.

The landed changes are:

1. moved the coherent `GraphQL` implementation subtree into
   `src/search/queries/graphql/`
2. moved feature-local `GraphQL` tests with it so the adapter remains
   self-contained under the src-owned feature folder
3. reduced `gateway/studio/search/queries/graphql/` to a thin facade that
   re-exports the src-owned adapter entrypoints
4. reran the focused `GraphQL` regression suite from the new
   `search::queries::graphql::tests::` owner path plus the `wendao` binary
   check

## Landed Slice: Search Adapter Search Plane Naming Clarification

This slice keeps the actual `search_plane` subsystem untouched and only
clarifies adapter-surface naming now that SQL, `FlightSQL`, and GraphQL
ownership already lives in `src/search/queries/`.

The landed changes are:

1. renamed thin `FlightSQL` adapter builders to the Studio-owned
   `build_studio_flightsql_service` surface
2. renamed the gateway native Flight facade module to `repo_search.rs` and
   updated the public builders to `build_repo_search_flight_service*` and
   `build_studio_flight_service*`
3. updated touched bins, tests, and re-export seams without adding compatibility
   wrappers
4. reran focused native Flight, `FlightSQL`, and binary verification on the
   renamed adapter surfaces

## Landed Slice: Repo Search Flight Provider Naming Clarification

This slice keeps the actual `search_plane` subsystem untouched and only
clarifies the remaining public concrete adapter type name on the repo-search
Flight surface.

The landed changes are:

1. renamed `SearchPlaneRepoSearchFlightRouteProvider` to the Studio-owned
   concrete adapter `StudioRepoSearchFlightRouteProvider`
2. update touched outward-facing error strings and adapter wording so they no
   longer present the gateway adapter as `search_plane` owned
3. updated touched call sites and focused tests without adding compatibility
   aliases
4. reran focused native Flight tests, repo-search unit tests, and the related
   binary `cargo check` using the default Cargo target

## Landed Slice: Repo Search Flight Adapter Wording Cleanup

This slice keeps the actual `search_plane` subsystem untouched and only removes
the remaining adapter-facing `search-plane` wording inside the repo-search
Flight handler.

The landed changes are:

1. replaced the remaining Studio adapter comments in `repo_search.rs` that still
   describe the provider as `search-plane` owned
2. renamed the focused repo-search Flight test names and test-only keyspace
   labels so they match the Studio or repo-search adapter surface
3. keep the real `SearchPlaneService` dependency names and runtime behavior
   unchanged
4. reran focused repo-search Flight validation on the default Cargo target

## Landed Slice: Gateway SQL Facade Warning Closure

This slice keeps the shared SQL ownership under `src/search/queries/sql/` and
only closes the adjacent default-build warnings in the thin gateway SQL facade.

The landed changes are:

1. gated the gateway SQL facade re-exports in `gateway/studio/search/queries/sql/mod.rs`
   so test-only symbols are only compiled for tests
2. kept gateway SQL tests compiling through the same facade surface
3. left the shared SQL implementation and ownership boundary unchanged
4. reran focused gateway SQL tests plus the related binary `cargo check`

## Landed Slice: Shared Query Owner Consumption Cutover

This slice keeps the shared query implementation under `src/search/queries/*`
and only cuts the remaining local consumers over from `gateway::studio`
re-exports to the real owner surface.

The landed changes are:

1. exposed the minimal `search::queries/*` surface needed by the CLI and local
   server bins
2. repointed `wendao` SQL and GraphQL query execution plus
   `wendao_search_flightsql_server` to that shared owner surface
3. removed the matching `gateway::studio` query re-exports that those bins no
   longer need
4. reran focused query-bin and FlightSQL server validation without touched-scope
   warnings

## Landed Slice: Gateway Query Facade Test-Only Reduction

This slice keeps the shared query owner under `search::queries/*` and only
removes the last non-test dependency on the gateway query facade tree.

The landed changes are:

1. repointed the internal Flight service builder to
   `search::queries::sql::provider::StudioSqlFlightRouteProvider`
2. gated `gateway::studio::search::queries` to tests in `search/mod.rs`
3. kept the gateway SQL facade tests compiling as the remaining compatibility
   surface
4. reran focused gateway Flight and SQL validation plus the related binary
   `cargo check` without touched-scope warnings

## Landed Slice: Julia Non-Feature Knowledge Flight Ungating

This slice advances plugin independence by removing an incorrect Julia feature
gate from a plugin-agnostic gateway Flight surface.

The landed changes are:

1. kept `knowledge/intent/flight.rs` compiled for non-Julia host builds because
   the touched code path only depends on generic Studio search response shaping
2. preserved the Julia-specific ownership boundary by avoiding any new Julia
   semantics or fallback wrappers in the host crate
3. reran the non-Julia host compile probe with
   `--no-default-features --features zhenfa-router`, which now finishes
   successfully past the previous `knowledge::intent::flight` blocker
4. reran focused default-feature gateway Flight validation and confirmed the
   touched seam still behaves the same
5. clarified that this seam was a host build-gating mistake rather than a real
   Julia plugin dependency

## Landed Slice: Julia Optional Host Dependency Cutover

This slice advances plugin independence by turning `xiuxian-wendao-julia` into
an optional host dependency and by gating the touched host runtime-config seams
that still import Julia-specific types unconditionally.

The landed changes are:

1. make the `xiuxian-wendao-julia` Cargo dependency optional and bind it to the
   existing `julia` host feature
2. keep `link_graph/runtime_config/*` compiling without the Julia plugin crate
   when `feature = "julia"` is disabled
3. preserve the current Julia-enabled behavior for rerank binding, schema
   version, score weights, and artifact rendering
4. keep `xiuxian-wendao-runtime` transport support enabled for the host so the
   generic Flight/DataFusion transport line stays available even when the
   Julia plugin crate is absent
5. tighten the touched SQL and gateway facade modules so non-Julia host builds
   do not pick up new unused-import warnings from Julia- or test-only helper
   re-exports
6. rerun both a non-Julia host compile probe and a Julia-enabled runtime-config
   validation lane so the cutover proves real host and plugin independence

This slice proves the host/plugin split concretely:

1. `direnv exec . cargo tree -p xiuxian-wendao --no-default-features --features zhenfa-router | rg 'xiuxian-wendao-julia'`
   returns no match, so the non-Julia host feature graph no longer includes the
   Julia plugin crate
2. `direnv exec . cargo check -p xiuxian-wendao --no-default-features --features zhenfa-router`
   passes on the cut-over host build
3. `direnv exec . cargo test -p xiuxian-wendao --lib link_graph::runtime_config`
   still passes with `8 passed`, so the touched Julia-enabled runtime-config
   surface remains intact

## Landed Slice: Runtime Transport Feature Realignment

This slice kept the Julia plugin-first boundary but removed the misleading
runtime feature name `julia` from `xiuxian-wendao-runtime`. The generic
Flight/DataFusion transport seam belongs to runtime ownership, so the feature
gate should read as transport infrastructure rather than plugin semantics.

The landed changes are:

1. rename the `xiuxian-wendao-runtime` generic transport feature from `julia`
   to `transport`
2. switch the bounded runtime transport cfg gates, test gates, and README
   verification commands to `feature = "transport"`
3. update `xiuxian-wendao` and `xiuxian-wendao-julia` to consume the renamed
   runtime feature while keeping the host/plugin feature name `julia`
4. rerun focused runtime transport tests plus both the non-Julia host compile
   probe and the Julia-enabled host runtime-config lane
5. accept one bounded compile-probe deviation in the `wendao` CLI by restoring
   the missing `RestQueryArgs` re-export that the non-Julia host probe exposed,
   without widening the slice into broader query CLI refactors

This slice proves the runtime/host ownership split more explicitly:

1. `xiuxian-wendao-runtime` now exposes the generic Flight/DataFusion seam
   under a transport-owned feature name instead of a Julia-plugin name
2. the bounded runtime sources no longer use `feature = "julia"` for generic
   transport cfg gates or README verification commands
3. the searched Cargo/runtime scope no longer contains runtime feature
   references to `julia`; the only remaining matches are the intentional
   `xiuxian-ast` language feature edges in host/plugin Cargo manifests
4. `direnv exec . cargo test -p xiuxian-wendao-runtime --features transport`
   passes with `183 passed`
5. `direnv exec . cargo check -p xiuxian-wendao --no-default-features --features zhenfa-router`
   now passes again after the bounded CLI re-export fix, while still surfacing
   broad ambient warnings outside this slice
6. `direnv exec . cargo test -p xiuxian-wendao --lib link_graph::runtime_config`
   still passes with `8 passed`

## Landed Slice: Studio Flight Julia Gate Retirement

This slice continues the same plugin-independence lane by retiring false
`feature = "julia"` gates around the generic Studio Flight builder surface.
The bounded Flight adapter, SQL Flight provider, and direct server consumers
depend on runtime transport plus generic Studio state, not on Julia plugin
types.

The landed changes are:

1. ungated the bounded `build_studio_flight_service*`,
   `build_studio_search_flight_service_with_repo_provider`, and their direct
   `gateway::studio` plus binary consumers from Julia feature naming, so the
   generic Studio Flight service now compiles under the non-Julia
   `zhenfa-router` host probe
2. ungated the shared SQL Flight provider plus the immediate generic Studio
   Flight route-provider re-exports in the `analysis`, `graph`, `repo`,
   `search`, and `vfs` owner modules, then repointed the Studio Flight service
   builder to those owner-level surfaces instead of private submodule paths
3. kept the bounded slice out of Julia-owned analyzer and rerank semantics;
   only generic adapter and provider seams moved
4. reran focused Flight handler tests plus the non-Julia host compile probes
   for `xiuxian-wendao`, `wendao_search_flight_server`, and `wendao`

## Landed Slice: Studio Flight Test Julia Gate Retirement

This slice removed the last test-only false Julia gates around the generic
Studio Flight test surface. The top-level `flight` test module and the
internal `repo_search` test module are now owned by test scope alone, not by
the Julia plugin feature.

The landed changes are:

1. replaced the remaining `#[cfg(all(test, feature = "julia"))]` gates on the
   generic Studio Flight test modules with `#[cfg(test)]`
2. kept the slice out of Julia-owned runtime-config, analyzer, and planned
   search semantics
3. reran the focused Studio Flight test suites in both the default and
   non-Julia `zhenfa-router` lanes after the later blocker slices cleared the
   surrounding host-test baseline

## Landed Slice: Julia-Dependent Host Lib-Test Gate Alignment

This slice aligned bounded host lib-test surfaces with the now-optional Julia
plugin dependency. Julia-specific host proofs now stay behind
`feature = "julia"`, while the generic host lanes remain available in the
non-Julia `zhenfa-router` probe.

The landed changes are:

1. gated Julia-dependent host test imports and test cases in the bounded
   blocker files under `link_graph/runtime_config`, `gateway/openapi`,
   `gateway/studio/types`, `gateway/studio/router`, and `zhenfa_router`
2. kept the slice out of runtime implementation and generic adapter logic
3. reran the non-Julia host test probe far enough to expose the next real
   blocker as a bounded Flight-analysis and agentic-test partition problem

## Landed Slice: Studio Flight Julia-Dependent Analysis Test Partition

This slice handled the next real blocker after host lib-test alignment. The
non-Julia Studio Flight lane proved that a bounded set of analysis-route tests
really do require the Julia plugin to be registered at runtime, so those tests
now stay in the Julia-enabled lane while the genuinely generic Flight tests
remain available without Julia.

The landed changes are:

1. gated the Julia-dependent Flight analysis tests behind `feature = "julia"`
   in the bounded `analysis.rs`, `headers.rs`, and `repo_search.rs` test
   surfaces
2. kept the generic Flight route tests, provider tests, and repo-search tests
   available in both the default and non-Julia `zhenfa-router` lanes
3. reran the focused non-Julia Studio Flight test probes and confirmed the
   previous `MISSING_PLUGIN` Julia-registration failures are gone

## Landed Slice: Link-Graph Agentic Julia Test Partition

This slice handled the next real blocker exposed by the broader non-Julia host
`cargo check --tests` probe. The bounded failures in
`tests/unit/link_graph_agentic/expansion.rs` came from two proof cases that
depend on Julia graph-structural transport helpers and a Julia plugin-backed
repository fixture. Those proofs now stay in the Julia-enabled lane, while the
generic agentic expansion plan tests remain available without Julia.

The landed changes are:

1. gated the Julia-dependent graph-structural imports, helpers, and two proof
   cases in `tests/unit/link_graph_agentic/expansion.rs` behind
   `feature = "julia"`
2. kept the generic agentic plan-budget and candidate-narrowing proofs visible
   in the non-Julia host test lane
3. reran the Julia-focused `link_graph_agentic::expansion` proof lane plus the
   broader non-Julia host `cargo check --tests` probe, which now passes beyond
   this file

## Landed Slice: Process-Managed WendaoSearch Live Service

This slice added one formal repo-level live smoke surface for the Julia Search
lane without turning the product crate or the process manager into the owner
of Search service semantics.

The landed changes are:

1. added one package-owned TOML live-service descriptor under `WendaoSearch.jl`
   for the standard `solver_demo` multi-route service
2. kept the repository `devenv` process layer as a thin launcher over that
   package-owned config instead of hard-coding route and port semantics inside
   `nix/modules/process.nix`
3. kept Rust plugin and host tests on self-spawned Julia services so those
   bounded live proofs still use isolated ports and deterministic cleanup

## Landed Slice: Plugin-Owned Modelica Parser-Summary Transport Reuse

This slice kept another runtime concern on the plugin side instead of letting
gateway or host wrappers absorb parser-summary execution policy. The remaining
live Modelica code-AST timeout came from the Modelica parser-summary owner
seam: blocking file-summary fetches created a fresh tokio runtime per request,
and the transport layer rebuilt a fresh negotiated Flight client for every
call.

The landed changes are:

1. `xiuxian-wendao-julia` now reuses one process-local tokio runtime for
   blocking Modelica parser-summary fetches instead of spinning up a new
   runtime on every request
2. the Modelica parser-summary transport now caches one negotiated Flight
   client per transport identity inside the plugin crate, so the existing lazy
   Arrow Flight connection survives across repo-owned file-analysis calls
3. focused plugin proofs cover transport-slot reuse and shared-runtime blocking
   fetch behavior, and the live frontend gateway proof now passes again on the
   unchanged same-origin code-AST contract

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[01_core/103_package_layering|Wendao Package Layering]], [[06_roadmap/412_core_runtime_plugin_program|Wendao Core Runtime Plugin Program]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/417_wendao_package_boundary_matrix|Wendao Package Boundary Matrix]]
:END:

---

:FOOTER:
:AUDITOR: codex
:END:
